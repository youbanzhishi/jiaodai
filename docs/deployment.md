# 胶带 部署指南

## Docker 部署（推荐）

### 构建镜像

```bash
docker build -t jiaodai .
```

### 使用 docker-compose

```bash
docker compose up -d
```

服务将在 `http://localhost:3000` 启动。

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `JIAODAI_PORT` | 3000 | 服务端口 |
| `JIAODAI_DB_PATH` | ./data/jiaodai.db | SQLite数据库路径 |
| `JIAODAI_LOG_LEVEL` | debug | 日志级别 |
| `JIAODAI_CHAIN_RPC` | - | L2 RPC端点 |
| `JIAODAI_CHAIN_KEY` | - | L2 签名私钥 |

## 二进制部署

### 编译

```bash
cargo build --release
```

### 运行

```bash
JIAODAI_LOG_LEVEL=info ./target/release/jiaodai
```

## 健康检查

```bash
curl http://localhost:3000/api/v1/health
```

## 数据库

SQLite 数据文件存储在 `JIAODAI_DB_PATH` 指定的路径，默认为 `./data/jiaodai.db`。

首次运行会自动创建数据库和执行迁移。
