# AegisBroker

AegisBroker is an event-driven broker operating system for commodity brokerage.

- Repository Version: `0.1.1`
- Deployment Target: `Zeabur`
- Primary Stack: `Rust backend` + `Python frontend` + `PostgreSQL`

## Repository Layout

```text
aegis-broker/
├─ AGENTS.md
├─ README.md
├─ docs/
│  └─ design/
├─ infra/
├─ backend-rust/
├─ frontend-python/
├─ library/
└─ storage/
```

## Directory Roles

- `docs/design/`: architecture source of truth, design preservation, and rollout planning
- `library/`: business document assets, traditional forms, commodity files, and content-service materials
- `backend-rust/`: API, control logic, event flow, audit flow, and payment/ledger services
- `frontend-python/`: operational console, content-service UI, and frontend orchestration
- `storage/`: migrations, DB bootstrap scripts, and storage-related operational files

## Start Here

Before changing architecture, schema, API behavior, payment flow, or deployment assumptions, read:

- [Design Index](/C:/Users/mos/Documents/GitHub/aegis-broker/docs/design/README.md)
- [Development Plan](/C:/Users/mos/Documents/GitHub/aegis-broker/docs/design/Development_Plan.md)
- [Content Service Architecture](/C:/Users/mos/Documents/GitHub/aegis-broker/docs/design/Content_Service_Architecture.md)

## Current Focus

- Zeabur-compatible backend/frontend deployment
- auth/session hardening
- content-service architecture
- traditional business form integration from `library/`
- phased commodity service rollout

## Commodity Rollout

- Phase 1: 原油 / 黃金 / 天然氣 / 小麥 / 咖啡
- Phase 2: 銅 / 大豆 / 糖 / LNG
- Phase 3: 鋰（新能源） / 碳權（Carbon credits） / 電力
