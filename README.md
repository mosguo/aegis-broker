# AegisBroker

AegisBroker 是面向大宗商品經紀業務的事件驅動控制型系統。

- Repository Version: `0.1.1`
- Spec Baseline: `v0.9.0`
- Milestone Status: `v0.1.1（Config Layering + Zeabur Deploy-Ready Env Baseline）`

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

## v0.1.1 Scope

目前新增：

1. backend env 分層樣板（dev / staging / prod）
2. v0.1.1 最低必要變數：
   - `DATABASE_URL`
   - `MAX_DB_CONNECTIONS=5`
   - `GOOGLE_CLIENT_ID`
   - `GOOGLE_CLIENT_SECRET`
   - `GOOGLE_REDIRECT_URI`
   - `GOOGLE_CONNECTOR_REDIRECT_URI`
3. Zeabur backend service env 映射升級
4. DB overview + initial setup 文件（`storage/README.md`）

## Root Env Bootstrap for Zeabur

新增三個根目錄 env 啟動檔（僅含安全佔位，不含 secrets 實值）：

- `.env`：共用基線
- `.env.local`：本機開發預設
- `.env.prod`：Zeabur production 佔位

> secrets 未來仍以 Zeabur Environment Variables 為主，以上檔案用於「最大集合鍵名準備」與部署前核對。

## Quick Start (Backend)

```bash
cd backend-rust
cp ../.env.local .env
cargo run
```

Health endpoints:

- `GET /health/live`
- `GET /health/ready`

## Architecture Sources

請先閱讀 `docs/architecture/README.md` 對應規格文件再進行核心流程擴充。
