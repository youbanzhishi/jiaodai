# Web Frontend Architecture (Phase 11)

> Status: API Ready, Frontend Code TBD

## Technology Selection

### Option A: WeChat Mini Program (微信小程序)
- **Pros**: Native WeChat ecosystem, push notifications, phone number quick-login
- **Cons**: Limited to WeChat, review process required
- **Tech Stack**: Taro / UniApp + Vue3 + TypeScript

### Option B: H5 (Mobile Web)
- **Pros**: Cross-platform, no app installation needed
- **Cons**: Limited push notification capability
- **Tech Stack**: Vue3 + Vite + TypeScript + Vant/NutUI

### Recommended: Hybrid (小程序 + H5)
- 小程序: Core features (create seal, heartbeat, search)
- H5: Share pages (viewers open short links to view content)

## CORS Configuration

The API server (Axum) is configured with `tower-http` CORS middleware:

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)  // Production: restrict to specific origins
    .allow_methods(Any)
    .allow_headers(Any);
```

Production CORS should restrict origins to:
- `https://jiaod.ai`
- `https://servicewechat.com` (WeChat Mini Program)

## Authentication Flow

1. **Login/Register**: Phone + SMS verification code
2. **Token Pair**: Access token (1h) + Refresh token (30d)
3. **Auto Refresh**: Frontend refreshes access token when 401 received
4. **Storage**: Access token in memory, Refresh token in secure storage

## API Endpoints for Frontend

All endpoints are under `/api/v1/`:

| Endpoint | Method | Description | Auth Required |
|----------|--------|-------------|---------------|
| `/account/register` | POST | Register with phone + code | No |
| `/account/login` | POST | Login with phone + code | No |
| `/account/refresh` | POST | Refresh tokens | Refresh token |
| `/account/bind-phone` | POST | Bind additional phone | Access token |
| `/account/change-phone` | POST | Change phone number | Access token |
| `/account/identity-verify` | POST | Real-name verification | Access token |
| `/seal` | POST | Create sealed tape | Access token |
| `/tape/{id}/status` | GET | Get tape status | Access token |
| `/tape/{id}/verify` | GET | Verify tape integrity | No |
| `/tape/{id}/certificate` | GET | Get seal certificate | Access token |
| `/tape/{id}/share` | POST | Generate share link | Access token |
| `/unseal/{id}` | POST | Attempt to unseal | Access token |
| `/heartbeat/confirm` | POST | Confirm heartbeat | Access token |
| `/match/check` | GET | Check mutual match | Access token |
| `/crush/search` | POST | Search phone for crush | Access token |
| `/crush/create` | POST | Create crush seal | Access token |
| `/will/create` | POST | Create will | Access token |
| `/will/heartbeat` | POST | Send will heartbeat | Access token |
| `/capsule/create` | POST | Create time capsule | Access token |
| `/capsule/{id}/countdown` | GET | Get capsule countdown | No |

## Client-Side Encryption

End-to-end encryption MUST happen on the client side:

1. **Key Generation**: Client generates a random 256-bit AES key
2. **Encryption**: Content encrypted with AES-256-GCM before upload
3. **Key Storage**: Key stored locally + optionally split via Shamir SSS
4. **Upload**: Only ciphertext is sent to server
5. **Decryption**: Only on unseal, key provided to verified viewers

### Libraries
- **Mini Program**: `crypto-js` or `jsencrypt`
- **H5**: Web Crypto API (native browser support)

## Page Structure

### Mini Program Pages
1. Home (我的胶带 - My Tapes)
2. Create Seal (创建封存)
3. Tape Detail (胶带详情)
4. Heartbeat (心跳确认)
5. Crush Search (搜索手机号)
6. Will Management (遗嘱管理)
7. Capsule List (胶囊列表)
8. Profile (个人中心)

### H5 Share Pages
1. Certificate View (封存凭证)
2. Capsule Countdown (倒计时)
3. Unsealed Content View (解封内容)
