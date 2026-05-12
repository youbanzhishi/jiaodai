# Changelog

## [0.1.0] - 2026-05-11

### Added — Phase 1-12: 完整时间封存平台

**Phase 1: 项目骨架+数据模型+核心trait**
- Rust项目骨架（Axum + SQLite + migrations）
- 核心trait定义：`Sealable`、`TriggerCondition`、`Viewer`
- 数据模型：Tape / SealCondition / Viewer / TriggerEvent
- 基础API：创建封存/查询封存/更新状态

**Phase 2: 账号体系**
- 手机号+验证码注册/登录
- 账号→手机号绑定（一对多）
- 换号找回 + 实名认证兜底
- JWT token管理 + 刷新机制

**Phase 3: 封存核心**
- 端到端加密：客户端加密→服务端存密文，平台零知识
- 内容hash计算（SHA-256）
- 封存凭证生成（hash + 封存时间 + 解封条件摘要）
- 凭证可分享（短链/二维码）

**Phase 4: 解封引擎**
- 心跳失联触发 + 双向匹配触发 + 指定日期触发 + 多人确认触发
- 触发事件记录（TriggerEvent）
- 解封后内容解密+通知查看人

**Phase 5: 暗恋表白场景**
- 手机号搜索 + 被动注册 + 双向hash匹配
- 隐私保护：搜索不留痕，单方永远静默

**Phase 6: 遗嘱交代场景**
- 实名认证 + 查看人验证 + 心跳失联
- 宽限期机制防误触发

**Phase 7: 时间胶囊场景**
- 短链邀请 + 指定日期解封 + 倒计时展示

**Phase 8: 区块链时间戳集成**
- Merkle树批量上链 + MockChain + 时间戳验证API
- 批量上链调度

**Phase 9: OpenLink集成**
- 封存凭证 = Identity Card + 短链分享
- 凭证验证端点 + Agent发现

**Phase 10: OpenVault集成**
- Shamir's Secret Sharing 完整M-of-N实现
- VaultConnector trait + 大文件加密存储引用

**Phase 11: Web前端API准备**
- CORS中间件 + JWT中间件 + WebSocket
- OpenAPI 3.0 spec完整定义

**Phase 12: Agent Action Protocol**
- `/.well-known/agent.json` 完整定义
- Action中间件 + Agent可创建/查询/触发封存
- 与OpenMind联动占位

### Technical
- 9 crate Rust workspace: jiaodai-core, jiaodai-seal, jiaodai-unseal, jiaodai-match, jiaodai-chain, jiaodai-auth, jiaodai-scene, jiaodai-api, jiaodai-cli
- ~9400行Rust代码
- 177 tests全绿
- CI: check → test → fmt + Release（cross-platform）
- Docker部署支持
