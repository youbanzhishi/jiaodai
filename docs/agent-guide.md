# 胶带 Agent 指南

## Agent Action Protocol

胶带作为生态节点暴露以下能力供AI Agent调用：

### 可用 Actions

#### 封存 (seal)

```
POST /api/v1/seal
Content-Type: application/json

{
  "content_type": "text",
  "trigger_condition": { "type": "date_trigger", "open_at": "2027-01-01T00:00:00Z" },
  "viewers": [{ "type": "anyone" }]
}
```

#### 解封 (unseal)

```
POST /api/v1/unseal/{tape_id}
```

#### 验证 (verify)

```
GET /api/v1/tape/{tape_id}/verify
```

验证封存的真实性（链上hash比对）。

#### 匹配 (match)

```
GET /api/v1/match/check
```

检查双向匹配状态。

### Agent 发现

```
GET /.well-known/agent.json
```

返回Agent能力描述和端点列表。

## 使用场景

1. **AI助手创建封存**：用户说"帮我把这段话封存到明年"，AI调用seal API
2. **验证封存真实性**：任何Agent可通过verify API验证封存是否被篡改
3. **被动心跳**：Agent定期提交心跳信号，证明用户仍然活跃
4. **状态查询**：Agent查询封存状态，不做解封操作
