//! Account management
//!
//! Core account operations: register, login, phone binding,
//! phone change, account recovery, and identity verification.

use chrono::Utc;
use uuid::Uuid;

use jiaodai_core::{Account, AccountStatus, IdentityInfo, JiaodaiError, PhoneBind, Result};

use crate::event::{AccountEvent, EventBus};
use crate::identity::{
    id_number_hash, IdentityProvider, IdentityVerificationResult, MockIdentityProvider,
};
use crate::jwt::{JwtConfig, JwtManager, TokenPair};
use crate::phone::{derive_phone_key, phone_encrypt, phone_hash, validate_phone_format};
use crate::sms::{MockSmsProvider, SmsProvider, VerificationCodeManager};

/// Account service configuration
#[derive(Debug, Clone)]
pub struct AccountConfig {
    /// Master secret for phone number encryption
    pub phone_encryption_secret: Vec<u8>,
    /// JWT configuration
    pub jwt_config: JwtConfig,
}

impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            phone_encryption_secret: b"jiaodai-phone-secret-change-me".to_vec(),
            jwt_config: JwtConfig::default(),
        }
    }
}

/// Account service — the main interface for account operations
pub struct AccountService {
    accounts: std::sync::Mutex<Vec<Account>>,
    sms_provider: Box<dyn SmsProvider>,
    code_manager: VerificationCodeManager,
    identity_provider: Box<dyn IdentityProvider>,
    jwt_manager: JwtManager,
    event_bus: EventBus,
    phone_key: [u8; 32],
}

impl AccountService {
    /// Create a new account service with default providers
    pub fn new(config: AccountConfig) -> Self {
        let phone_key = derive_phone_key(&config.phone_encryption_secret);
        Self {
            jwt_manager: JwtManager::new(config.jwt_config.clone()),
            phone_key,
            accounts: std::sync::Mutex::new(Vec::new()),
            sms_provider: Box::new(MockSmsProvider),
            code_manager: VerificationCodeManager::new(),
            identity_provider: Box::new(MockIdentityProvider::new_pass()),
            event_bus: EventBus::new(),
        }
    }

    /// Create account service with custom providers
    pub fn with_providers(
        config: AccountConfig,
        sms: Box<dyn SmsProvider>,
        identity: Box<dyn IdentityProvider>,
    ) -> Self {
        let phone_key = derive_phone_key(&config.phone_encryption_secret);
        Self {
            jwt_manager: JwtManager::new(config.jwt_config.clone()),
            phone_key,
            accounts: std::sync::Mutex::new(Vec::new()),
            sms_provider: sms,
            code_manager: VerificationCodeManager::new(),
            identity_provider: identity,
            event_bus: EventBus::new(),
        }
    }

    // ─── Registration ─────────────────────────────────────────

    /// Send a verification code to a phone number for registration
    pub async fn send_register_code(&self, phone: &str) -> Result<()> {
        if !validate_phone_format(phone) {
            return Err(JiaodaiError::SerializationError(
                "Invalid phone number format".to_string(),
            ));
        }

        let hash = phone_hash(phone);
        let accounts = self.accounts.lock().unwrap();
        if accounts
            .iter()
            .any(|a| a.phone_numbers.iter().any(|p| p.phone_hash == hash))
        {
            return Err(JiaodaiError::SerializationError(
                "Phone number already registered".to_string(),
            ));
        }

        let code = self.code_manager.generate_code(phone);
        let result = self.sms_provider.send_verification_code(phone, &code).await;
        if !result.success {
            return Err(JiaodaiError::SerializationError(format!(
                "SMS send failed: {}",
                result.message
            )));
        }

        Ok(())
    }

    /// Register a new account with phone + verification code
    pub async fn register(&self, phone: &str, code: &str) -> Result<(Account, TokenPair)> {
        if !validate_phone_format(phone) {
            return Err(JiaodaiError::SerializationError(
                "Invalid phone number format".to_string(),
            ));
        }

        if !self.code_manager.verify_code(phone, code) {
            return Err(JiaodaiError::SerializationError(
                "Invalid or expired verification code".to_string(),
            ));
        }

        let hash = phone_hash(phone);
        let encrypted = phone_encrypt(phone, &self.phone_key);

        let now = Utc::now();
        let account_id = Uuid::new_v4().to_string();
        let phone_bind_id = Uuid::new_v4().to_string();

        let phone_bind = PhoneBind {
            id: phone_bind_id,
            account_id: account_id.clone(),
            phone_hash: hash.clone(),
            phone_encrypted: encrypted,
            is_primary: true,
            verified: true,
            created_at: now,
        };

        let account = Account {
            id: account_id.clone(),
            phone_numbers: vec![phone_bind],
            identity: None,
            status: AccountStatus::Active,
            created_at: now,
            updated_at: now,
        };

        let tokens = self.jwt_manager.generate_token_pair(&account_id);

        self.accounts.lock().unwrap().push(account.clone());

        self.event_bus.broadcast(&AccountEvent::AccountCreated {
            account_id: account_id.clone(),
            phone_hash: hash.clone(),
            at: now,
        });
        self.event_bus.broadcast(&AccountEvent::PhoneBound {
            account_id,
            phone_hash: hash,
            is_primary: true,
            at: now,
        });

        Ok((account, tokens))
    }

    // ─── Login ───────────────────────────────────────────────

    /// Send a verification code for login
    pub async fn send_login_code(&self, phone: &str) -> Result<()> {
        if !validate_phone_format(phone) {
            return Err(JiaodaiError::SerializationError(
                "Invalid phone number format".to_string(),
            ));
        }

        let hash = phone_hash(phone);
        let accounts = self.accounts.lock().unwrap();
        if !accounts
            .iter()
            .any(|a| a.phone_numbers.iter().any(|p| p.phone_hash == hash))
        {
            return Err(JiaodaiError::AccountNotFound(
                "Phone number not registered".to_string(),
            ));
        }

        let code = self.code_manager.generate_code(phone);
        let result = self.sms_provider.send_verification_code(phone, &code).await;
        if !result.success {
            return Err(JiaodaiError::SerializationError(format!(
                "SMS send failed: {}",
                result.message
            )));
        }

        Ok(())
    }

    /// Login with phone + verification code
    pub async fn login(&self, phone: &str, code: &str) -> Result<(Account, TokenPair)> {
        if !self.code_manager.verify_code(phone, code) {
            return Err(JiaodaiError::SerializationError(
                "Invalid or expired verification code".to_string(),
            ));
        }

        let hash = phone_hash(phone);
        let mut accounts = self.accounts.lock().unwrap();
        let account = accounts
            .iter_mut()
            .find(|a| a.phone_numbers.iter().any(|p| p.phone_hash == hash))
            .ok_or_else(|| JiaodaiError::AccountNotFound("Account not found".to_string()))?;

        if account.status != AccountStatus::Active {
            return Err(JiaodaiError::AccountNotFound(
                "Account is not active".to_string(),
            ));
        }

        account.updated_at = Utc::now();
        let tokens = self.jwt_manager.generate_token_pair(&account.id);

        self.event_bus.broadcast(&AccountEvent::LoggedIn {
            account_id: account.id.clone(),
            phone_hash: hash,
            at: Utc::now(),
        });

        Ok((account.clone(), tokens))
    }

    // ─── Token Refresh ────────────────────────────────────────

    /// Refresh tokens using a valid refresh token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair> {
        self.jwt_manager.refresh(refresh_token)
    }

    // ─── Phone Binding ────────────────────────────────────────

    /// Bind an additional phone number to an account
    pub async fn bind_phone(&self, account_id: &str, phone: &str, code: &str) -> Result<Account> {
        if !validate_phone_format(phone) {
            return Err(JiaodaiError::SerializationError(
                "Invalid phone number format".to_string(),
            ));
        }

        if !self.code_manager.verify_code(phone, code) {
            return Err(JiaodaiError::SerializationError(
                "Invalid or expired verification code".to_string(),
            ));
        }

        let hash = phone_hash(phone);
        let encrypted = phone_encrypt(phone, &self.phone_key);

        // Check if phone is already bound to another account
        {
            let accounts = self.accounts.lock().unwrap();
            if accounts
                .iter()
                .any(|a| a.phone_numbers.iter().any(|p| p.phone_hash == hash))
            {
                return Err(JiaodaiError::SerializationError(
                    "Phone number already bound to another account".to_string(),
                ));
            }
        }

        let mut accounts = self.accounts.lock().unwrap();
        let account = accounts
            .iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| JiaodaiError::AccountNotFound(account_id.to_string()))?;

        let phone_bind = PhoneBind {
            id: Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            phone_hash: hash.clone(),
            phone_encrypted: encrypted,
            is_primary: account.phone_numbers.is_empty(),
            verified: true,
            created_at: Utc::now(),
        };

        account.phone_numbers.push(phone_bind);
        account.updated_at = Utc::now();

        self.event_bus.broadcast(&AccountEvent::PhoneBound {
            account_id: account_id.to_string(),
            phone_hash: hash,
            is_primary: account.phone_numbers.len() == 1,
            at: Utc::now(),
        });

        Ok(account.clone())
    }

    // ─── Phone Change ─────────────────────────────────────────

    /// Change phone number: verify old phone, then bind new phone
    pub async fn change_phone(
        &self,
        account_id: &str,
        old_phone: &str,
        old_code: &str,
        new_phone: &str,
        new_code: &str,
    ) -> Result<Account> {
        // Verify old phone
        if !self.code_manager.verify_code(old_phone, old_code) {
            return Err(JiaodaiError::SerializationError(
                "Old phone verification failed".to_string(),
            ));
        }

        let old_hash = phone_hash(old_phone);

        // Verify old phone belongs to this account and remove it
        {
            let mut accounts = self.accounts.lock().unwrap();
            let account = accounts
                .iter_mut()
                .find(|a| a.id == account_id)
                .ok_or_else(|| JiaodaiError::AccountNotFound(account_id.to_string()))?;

            if !account
                .phone_numbers
                .iter()
                .any(|p| p.phone_hash == old_hash)
            {
                return Err(JiaodaiError::SerializationError(
                    "Old phone not bound to this account".to_string(),
                ));
            }

            account.phone_numbers.retain(|p| p.phone_hash != old_hash);
        }

        // Verify new phone code
        if !self.code_manager.verify_code(new_phone, new_code) {
            return Err(JiaodaiError::SerializationError(
                "New phone verification failed".to_string(),
            ));
        }

        // Check new phone not bound elsewhere, then bind
        let new_hash = phone_hash(new_phone);
        let encrypted = phone_encrypt(new_phone, &self.phone_key);

        let mut accounts = self.accounts.lock().unwrap();
        if accounts
            .iter()
            .any(|a| a.id != account_id && a.phone_numbers.iter().any(|p| p.phone_hash == new_hash))
        {
            return Err(JiaodaiError::SerializationError(
                "New phone already bound to another account".to_string(),
            ));
        }

        let account = accounts
            .iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| JiaodaiError::AccountNotFound(account_id.to_string()))?;

        let is_primary = account.phone_numbers.is_empty();
        let phone_bind = PhoneBind {
            id: Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            phone_hash: new_hash.clone(),
            phone_encrypted: encrypted,
            is_primary,
            verified: true,
            created_at: Utc::now(),
        };

        account.phone_numbers.push(phone_bind);
        account.updated_at = Utc::now();

        self.event_bus.broadcast(&AccountEvent::PhoneChanged {
            account_id: account_id.to_string(),
            old_phone_hash: old_hash,
            new_phone_hash: new_hash,
            at: Utc::now(),
        });

        Ok(account.clone())
    }

    // ─── Account Recovery ─────────────────────────────────────

    /// Verify identity for account recovery
    pub async fn verify_identity_for_recovery(
        &self,
        name: &str,
        id_number: &str,
        face_image: &[u8],
    ) -> Result<IdentityVerificationResult> {
        let result = self
            .identity_provider
            .verify_identity(name, id_number, face_image)
            .await?;
        Ok(result)
    }

    /// Recover account using identity verification
    /// After identity is verified, bind a new phone number
    pub async fn recover_account(
        &self,
        id_number_hash_val: &str,
        new_phone: &str,
        new_code: &str,
    ) -> Result<(Account, TokenPair)> {
        if !self.code_manager.verify_code(new_phone, new_code) {
            return Err(JiaodaiError::SerializationError(
                "Invalid verification code".to_string(),
            ));
        }

        let mut accounts = self.accounts.lock().unwrap();
        let account = accounts
            .iter_mut()
            .find(|a| {
                a.identity
                    .as_ref()
                    .map_or(false, |id| id.id_number_hash == id_number_hash_val)
            })
            .ok_or_else(|| {
                JiaodaiError::AccountNotFound("No account found with this identity".to_string())
            })?;

        // Bind new phone
        let new_hash = phone_hash(new_phone);
        let encrypted = phone_encrypt(new_phone, &self.phone_key);
        let phone_bind = PhoneBind {
            id: Uuid::new_v4().to_string(),
            account_id: account.id.clone(),
            phone_hash: new_hash.clone(),
            phone_encrypted: encrypted,
            is_primary: true,
            verified: true,
            created_at: Utc::now(),
        };

        account.phone_numbers.clear();
        account.phone_numbers.push(phone_bind);
        account.status = AccountStatus::Active;
        account.updated_at = Utc::now();

        let tokens = self.jwt_manager.generate_token_pair(&account.id);

        self.event_bus.broadcast(&AccountEvent::AccountRecovered {
            account_id: account.id.clone(),
            new_phone_hash: new_hash,
            at: Utc::now(),
        });

        Ok((account.clone(), tokens))
    }

    // ─── Identity Verification ────────────────────────────────

    /// Complete identity verification (OCR + liveness + face match)
    pub async fn complete_identity_verification(
        &self,
        account_id: &str,
        id_card_image: &[u8],
        face_image: &[u8],
    ) -> Result<Account> {
        let ocr = self.identity_provider.ocr_scan(id_card_image).await?;
        let liveness = self.identity_provider.liveness_check(face_image).await?;

        if !liveness.is_live {
            return Err(JiaodaiError::ViewerVerificationFailed(
                "Liveness check failed".to_string(),
            ));
        }

        let verify = self
            .identity_provider
            .verify_identity(&ocr.name, &ocr.id_number, face_image)
            .await?;

        if !verify.verified {
            return Err(JiaodaiError::ViewerVerificationFailed(
                "Identity verification failed".to_string(),
            ));
        }

        let now = Utc::now();
        let identity = IdentityInfo {
            id: Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            id_number_hash: id_number_hash(&ocr.id_number),
            name_encrypted: crate::phone::phone_encrypt(&ocr.name, &self.phone_key),
            id_number_encrypted: crate::phone::phone_encrypt(&ocr.id_number, &self.phone_key),
            verified: true,
            verified_at: Some(now),
            created_at: now,
        };

        let mut accounts = self.accounts.lock().unwrap();
        let account = accounts
            .iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| JiaodaiError::AccountNotFound(account_id.to_string()))?;

        account.identity = Some(identity);
        account.updated_at = now;

        self.event_bus.broadcast(&AccountEvent::IdentityVerified {
            account_id: account_id.to_string(),
            at: now,
        });

        Ok(account.clone())
    }

    // ─── Query ────────────────────────────────────────────────

    /// Get an account by ID
    pub fn get_account(&self, account_id: &str) -> Result<Account> {
        let accounts = self.accounts.lock().unwrap();
        accounts
            .iter()
            .find(|a| a.id == account_id)
            .cloned()
            .ok_or_else(|| JiaodaiError::AccountNotFound(account_id.to_string()))
    }

    /// Find an account by phone number hash
    pub fn find_by_phone_hash(&self, phone_hash_val: &str) -> Result<Account> {
        let accounts = self.accounts.lock().unwrap();
        accounts
            .iter()
            .find(|a| {
                a.phone_numbers
                    .iter()
                    .any(|p| p.phone_hash == phone_hash_val)
            })
            .cloned()
            .ok_or_else(|| {
                JiaodaiError::AccountNotFound(format!(
                    "No account with phone hash {}",
                    phone_hash_val
                ))
            })
    }

    /// Check if a phone number is already registered
    pub fn is_phone_registered(&self, phone: &str) -> bool {
        let hash = phone_hash(phone);
        let accounts = self.accounts.lock().unwrap();
        accounts
            .iter()
            .any(|a| a.phone_numbers.iter().any(|p| p.phone_hash == hash))
    }

    /// Send verification code (for any purpose: register, login, bind, etc.)
    pub async fn send_code(&self, phone: &str) -> Result<String> {
        if !validate_phone_format(phone) {
            return Err(JiaodaiError::SerializationError(
                "Invalid phone number format".to_string(),
            ));
        }
        let code = self.code_manager.generate_code(phone);
        let result = self.sms_provider.send_verification_code(phone, &code).await;
        if !result.success {
            return Err(JiaodaiError::SerializationError(format!(
                "SMS send failed: {}",
                result.message
            )));
        }
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_service() -> AccountService {
        AccountService::new(AccountConfig::default())
    }

    #[tokio::test]
    async fn test_register_and_login() {
        let service = create_service().await;
        let phone = "13800138000";
        let code = service.send_code(phone).await.unwrap();

        let (account, tokens) = service.register(phone, &code).await.unwrap();
        assert_eq!(account.status, AccountStatus::Active);
        assert!(account.phone_numbers.len() == 1);
        assert!(account.phone_numbers[0].is_primary);

        // Login
        let code = service.send_code(phone).await.unwrap();
        let (logged_in, login_tokens) = service.login(phone, &code).await.unwrap();
        assert_eq!(logged_in.id, account.id);
        assert_ne!(login_tokens.access_token, tokens.access_token);
    }

    #[tokio::test]
    async fn test_register_invalid_phone() {
        let service = create_service().await;
        let result = service.send_register_code("123").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_duplicate_phone() {
        let service = create_service().await;
        let phone = "13800138001";
        let code = service.send_code(phone).await.unwrap();
        service.register(phone, &code).await.unwrap();

        // Try to register again
        let result = service.send_register_code(phone).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_wrong_code() {
        let service = create_service().await;
        let phone = "13800138002";
        let code = service.send_code(phone).await.unwrap();
        service.register(phone, &code).await.unwrap();

        let _code = service.send_code(phone).await.unwrap();
        let result = service.login(phone, "000000").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bind_additional_phone() {
        let service = create_service().await;
        let phone1 = "13800138003";
        let code1 = service.send_code(phone1).await.unwrap();
        let (account, _) = service.register(phone1, &code1).await.unwrap();

        let phone2 = "13800138004";
        let code2 = service.send_code(phone2).await.unwrap();
        let updated = service
            .bind_phone(&account.id, phone2, &code2)
            .await
            .unwrap();
        assert_eq!(updated.phone_numbers.len(), 2);
    }

    #[tokio::test]
    async fn test_change_phone() {
        let service = create_service().await;
        let old_phone = "13800138005";
        let code = service.send_code(old_phone).await.unwrap();
        let (account, _) = service.register(old_phone, &code).await.unwrap();

        let new_phone = "13800138006";
        let old_code = service.send_code(old_phone).await.unwrap();
        let new_code = service.send_code(new_phone).await.unwrap();

        let updated = service
            .change_phone(&account.id, old_phone, &old_code, new_phone, &new_code)
            .await
            .unwrap();
        assert_eq!(updated.phone_numbers.len(), 1);
        assert_eq!(updated.phone_numbers[0].phone_hash, phone_hash(new_phone));
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let service = create_service().await;
        let phone = "13800138007";
        let code = service.send_code(phone).await.unwrap();
        let (_, tokens) = service.register(phone, &code).await.unwrap();

        let new_tokens = service.refresh_token(&tokens.refresh_token).unwrap();
        let claims = service
            .jwt_manager
            .verify_access_token(&new_tokens.access_token)
            .unwrap();
        // Verify the account exists
        assert!(service.get_account(&claims.sub).is_ok());
    }

    #[tokio::test]
    async fn test_identity_verification() {
        let service = create_service().await;
        let phone = "13800138008";
        let code = service.send_code(phone).await.unwrap();
        let (account, _) = service.register(phone, &code).await.unwrap();

        let updated = service
            .complete_identity_verification(&account.id, &[], &[])
            .await
            .unwrap();
        assert!(updated.identity.is_some());
        assert!(updated.identity.unwrap().verified);
    }

    #[tokio::test]
    async fn test_is_phone_registered() {
        let service = create_service().await;
        let phone = "13800138009";
        assert!(!service.is_phone_registered(phone));

        let code = service.send_code(phone).await.unwrap();
        service.register(phone, &code).await.unwrap();
        assert!(service.is_phone_registered(phone));
    }

    #[tokio::test]
    async fn test_find_by_phone_hash() {
        let service = create_service().await;
        let phone = "13800138010";
        let code = service.send_code(phone).await.unwrap();
        let (account, _) = service.register(phone, &code).await.unwrap();

        let found = service.find_by_phone_hash(&phone_hash(phone)).unwrap();
        assert_eq!(found.id, account.id);
    }
}
