# AegisBroker Config Management Spec

Version: v0.1.1  
Last Updated: 2026-04-09

## 1) ENV 分層策略（dev / staging / prod）

AegisBroker 採用「同一套程式碼、分環境配置」原則，避免在程式碼中硬編環境差異。

### 1.1 Environment 定義

- `dev`：開發者本機、功能驗證用；允許較寬鬆日志與測試憑證。
- `staging`：預上線驗證；設定需盡量貼近 production（外部服務、權限、網路拓樸）。
- `prod`：正式環境；僅允許經審核變更，禁止使用測試憑證。

### 1.2 命名與檔案

- 參考模板：
  - `infra/env/backend.env.example`
  - `infra/env/frontend.env.example`
- 不提交實際 secrets（`.env`, `.env.local`, `.env.prod`）。
- 建議命名：
  - `APP_ENV=dev|staging|prod`
  - `SERVICE_NAME=aegis-broker-backend`

### 1.3 環境差異最小化

必須一致：
- DB schema（透過 migration）
- API contract / error_code / reason_code
- event / audit 寫入規則

允許差異：
- log level
- OAuth redirect URI
- service endpoint
- feature flag（需有 default）


### 1.4 v0.1.1 Backend 最低必要環境變數

以下鍵值為 v0.1.1 最低要求（名稱需一致）：

```env
DATABASE_URL=
MAX_DB_CONNECTIONS=5
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
GOOGLE_REDIRECT_URI=
GOOGLE_CONNECTOR_REDIRECT_URI=
```

---

## 2) Secrets 管理規範

### 2.1 分類

- Public Config（可公開）：
  - `APP_BIND_ADDR`, `PORT`, `DEFAULT_LOCALE`
- Sensitive Secrets（不可公開）：
  - `DATABASE_URL`
  - `GOOGLE_CLIENT_SECRET`
  - `POSTGRES_PASSWORD`
  - 第三方 API keys / webhook signing secrets

### 2.2 基本規則

1. secrets 僅存於部署平台密鑰管理（Zeabur Environment / Secret 管理）。
2. 不可寫入 Git repo、PR 描述、日誌明文。
3. rotation 必須可追蹤，並記錄：執行人、時間、影響範圍、回滾策略。
4. staging/prod 使用不同 credentials。
5. 應最小權限（least privilege）與定期輪替（至少每 90~180 天）。

### 2.3 事故應對

若疑似洩漏：
1. 立即撤銷/輪替。
2. 封鎖可疑 session/token。
3. 審核 event_store / audit_chain 追蹤影響。
4. 補充 postmortem 與預防改善。

---

## 3) Zeabur 對應策略

本專案採三服務部署：
- `postgres`
- `backend-rust`
- `frontend-python`

對應檔案：
- `infra/zeabur/services/postgres.service.yaml`
- `infra/zeabur/services/backend-rust.service.yaml`
- `infra/zeabur/services/frontend-python.service.yaml`

### 3.1 變數映射建議

Backend:
- `DATABASE_URL` ← `${POSTGRES_DATABASE_URL}`
- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`
- `GOOGLE_REDIRECT_URI`
- `GOOGLE_CONNECTOR_REDIRECT_URI`
- `SESSION_TTL_HOURS`
- `GOOGLE_OAUTH_SCOPE`

Frontend:
- `BACKEND_API_BASE_URL`
- `DEFAULT_LOCALE`
- `ENABLED_LOCALES=zh-TW,zh-CN,en,es,tr`

Postgres:
- `POSTGRES_DB`
- `POSTGRES_USER`
- `POSTGRES_PASSWORD`

### 3.2 依賴順序

1. 先起 Postgres
2. 跑 migration
3. 起 Backend，確認 `/health/ready`
4. 起 Frontend
5. 再切流量

---

## 4) Rust Config Loader 規格（含 fallback）

### 4.1 設計原則

- 強型別載入（typed config）
- required 與 optional 明確分離
- fallback 僅允許低風險欄位
- 對外錯誤為 machine-usable code，不暴露 secrets

### 4.2 載入順序（建議）

1. Process env（最高優先）
2. 其次採預設值（僅非敏感欄位）
3. 無預設的敏感欄位必須報錯（fail fast）

### 4.3 Fallback 規範

可 fallback：
- `APP_BIND_ADDR` 預設 `0.0.0.0:8080`
- `SERVICE_NAME` 預設 `aegis-broker-backend`
- `GOOGLE_OAUTH_SCOPE` 預設 `openid email profile`
- `SESSION_TTL_HOURS` 預設 `24`

不可 fallback（缺失即啟動失敗）：
- `DATABASE_URL`
- 在啟用 OAuth 時的 `GOOGLE_CLIENT_ID / SECRET / REDIRECT_URI`

### 4.4 啟動檢查

- Config 載入失敗 → 直接退出。
- `/health/ready` 必須驗證：
  - DB connectivity
  - 必要 config 已載入

---

## 5) Deployment 流程（標準作業）

### 5.1 上線前

1. 確認 migration SQL 已審核並可回滾（或補償）
2. 確認 staging 完整演練（OAuth、session、role update、event/audit）
3. 確認 Zeabur env/secrets 已設定
4. 確認 healthcheck path 與 port 對齊

### 5.2 發佈步驟

1. 部署 Postgres 變更
2. 執行 migration
3. 部署 backend-rust
4. 驗證 `/health/live`、`/health/ready`
5. 部署 frontend-python
6. 進行 smoke test（登入、profile、角色更新）
7. 放量（可灰度）

### 5.3 發佈後

- 監控 error_rate / latency / auth failure rate
- 抽樣比對 event_store 與 audit_chain 一致性
- 記錄 release note 與異常處理

---

## 6) 與多語策略整合

本規格與 `docs/design/Development_Plan.md` 一致：
- 任何 user-facing 訊息（含非圖形介面）都需支援
  `zh-TW`, `zh-CN`, `en`, `es`, `tr`。
- config 中 `DEFAULT_LOCALE` 與 `ENABLED_LOCALES` 必須維持環境可配置。
