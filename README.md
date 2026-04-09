# AegisBroker

AegisBroker 是面向大宗商品經紀業務的事件驅動控制型系統。

- Repository Version: `0.0.9`
- Spec Baseline: `v0.9.0`
- Next Milestone: `v0.1.0`

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

請先閱讀 `docs/architecture/README.md` 對應的 v0.9.0 規格文件後再進行核心流程開發。

## v0.1.0 Goals

1. 多國語言：繁中、簡中、英文、西班牙文、土耳其語
2. Zeabur 前端/後端/資料庫部署組態檔完成
3. Google OAuth 實際登入流程
4. User Profile + Role/Permission 對應與可編輯角色設定
