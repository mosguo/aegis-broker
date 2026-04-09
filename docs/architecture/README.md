# Architecture Source Index

以下為目前版本化的架構基線（v0.9.0）：

1. `../../Core_Event_Audit_Architecture_Spec_WorkItem_List_AegisBroker.md`
2. `../../AegisBroker_PostgreSQL_Schema_Migration_Spec.md`
3. `../../AegisBroker_Rust_API_Contract_Spec.md`
4. `../../AegisBroker_ZEABUR_Deployment_Repo_Blueprintr.md`
5. `../../AegisBroker_Financial_Control_Reconciliation_Spec.md`
6. `../../AegisBroker_Double_Entry_Ledger_Accounting_Spec.md`
7. `../../AegisBroker_role_permission_api_db_ledger_mapping.md`

> 註：AGENTS.md 的通用檔名為語意名稱；目前主檔名不含版本後綴，未來若需版控後綴統一使用 `_v1.1` 規則。

## 計劃文件確認狀態（2026-04-09）

| 文件代號 | 文件名稱 | 檔案路徑 | 狀態 |
|---|---|---|---|
| PLAN-CEA | Core Event & Audit Architecture Spec & WorkItem List | `../../Core_Event_Audit_Architecture_Spec_WorkItem_List_AegisBroker.md` | ✅ 已確認 |
| PLAN-DB | PostgreSQL Schema & Migration Spec | `../../AegisBroker_PostgreSQL_Schema_Migration_Spec.md` | ✅ 已確認 |
| PLAN-API | Rust API Contract Spec | `../../AegisBroker_Rust_API_Contract_Spec.md` | ✅ 已確認 |
| PLAN-DEPLOY | ZEABUR Deployment & Repo Blueprint | `../../AegisBroker_ZEABUR_Deployment_Repo_Blueprintr.md` | ✅ 已確認 |
| PLAN-FIN | Financial Control Reconciliation Spec | `../../AegisBroker_Financial_Control_Reconciliation_Spec.md` | ✅ 已確認 |
| PLAN-LEDGER | Double Entry Ledger Accounting Spec | `../../AegisBroker_Double_Entry_Ledger_Accounting_Spec.md` | ✅ 已確認 |
| PLAN-RBAC | Role Permission API/DB/Ledger Mapping Spec | `../../AegisBroker_role_permission_api_db_ledger_mapping.md` | ✅ 已確認 |

## 目前發行版本

- Repository Version: `0.0.9`
- Spec Baseline Filename Rule: `no suffix (future: _v1.1)`

## 開發階段代號與名稱（對齊 AGENTS.md Build Order）

| 階段代號 | 階段名稱 |
|---|---|
| PH-01 | Repository Skeleton |
| PH-02 | Migrations |
| PH-03 | Backend Health/Readiness Endpoints |
| PH-04 | Config Loading & Env Contract |
| PH-05 | Auth/Session Skeleton |
| PH-06 | Event Store / Audit Chain / State-machine Framework |
| PH-07 | RFQ / Quote / Deal Core Flow |
| PH-08 | PaymentOrder + Stripe Integration |
| PH-09 | Points Ledger and Debit/Credit/Reversal Flows |
| PH-10 | Replay / Audit Verify / Reconciliation Support |
| PH-11 | ZEABUR Deployment Hardening |

## v0.1.0 目標（Next Milestone）

v0.1.0 將聚焦可實際上線能力，包含：

1. 多國語言（i18n）
   - 繁體中文 (`zh-TW`)
   - 簡體中文 (`zh-CN`)
   - 英文 (`en`)
   - 西班牙文 (`es`)
   - 土耳其語 (`tr`)
2. Zeabur 可運作的前端 / 後端 / PostgreSQL 部署組態（見 `infra/zeabur/`）
3. Google OAuth 實際登入流程（含 callback、session 交換、審計與事件紀錄）
4. User Profile 與角色 / 權限對應與資料庫映射（見 `../../AegisBroker_role_permission_api_db_ledger_mapping.md`）
5. 角色設定可由授權用戶調整（含事件、審計、理由碼、權限校驗）
