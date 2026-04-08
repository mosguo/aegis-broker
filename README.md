# AegisBroker

AegisBroker 是面向大宗商品經紀業務的事件驅動控制型系統。

## Repository Layout

```text
aegis-broker/
├─ AGENTS.md
├─ README.md
├─ docs/
│  └─ architecture/
├─ infra/
├─ backend-rust/
├─ frontend-python/
└─ storage/
```

## Current Bootstrap Scope

目前完成的基礎建設（依 AGENTS.md build order）：

1. repository skeleton
2. migration skeleton
3. backend health/readiness endpoints
4. config loading and env contract

## Quick Start (Backend)

```bash
cd backend-rust
cp ../infra/env/backend.env.example .env
cargo run
```

Health endpoints:

- `GET /health/live`
- `GET /health/ready`

`/health/ready` 會檢查：

- 必填環境變數是否存在
- PostgreSQL 是否可連線（`SELECT 1`）

## Architecture Sources

請先閱讀 `docs/architecture/README.md` 對應的 v1.1 規格文件後再進行核心流程開發。
