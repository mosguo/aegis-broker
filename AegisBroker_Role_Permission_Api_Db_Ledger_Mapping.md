# aegis_broker_role_permission_api_db_ledger_mapping
Version: v0.9.0
Status: Baseline approved for v0.1.0 implementation

此文件定義使用者 Profile、Role、Permission 的 API / DB / Ledger / Event / Audit 對映，並規範「角色可修改」流程必須遵守控制框架。

---

## 1. Scope

本規格涵蓋：

- User Profile 基本欄位與身份綁定
- Workspace 內 RBAC（Role-Based Access Control）
- Role 變更、授權、撤銷的 API 契約
- DB table 對映與 migration 方向
- 事件（event_store）、審計（audit_chain）、理由碼（reason codes）
- 與點數帳務（ledger）及高風險操作授權的連動

---

## 2. Core Mapping

| Domain | API | DB | Event | Audit |
|---|---|---|---|---|
| User Profile | `/api/v1/me` `/api/v1/users/{id}` | `users`, `user_profiles` | `user.profile.updated` | required |
| Role Catalog | `/api/v1/roles` | `roles` | `role.created`, `role.updated` | required |
| Permission Catalog | `/api/v1/permissions` | `permissions` | `permission.created`, `permission.updated` | required |
| Role-Permission Mapping | `/api/v1/roles/{id}/permissions` | `role_permissions` | `role.permission.granted`, `role.permission.revoked` | required |
| User Role Assignment | `/api/v1/users/{id}/roles` | `user_roles` | `user.role.assigned`, `user.role.revoked` | required |
| High-risk ledger operation guard | (service-level) | `point_ledger_entries`, `audit_chain` | `ledger.manual_adjustment.requested/approved/rejected` | required |

---

## 3. Minimal API Contract (v0.1.0 target)

### 3.1 Google OAuth Login

- `GET /api/v1/auth/google/start`
  - 產生 state + nonce，重導至 Google OAuth consent page
- `GET /api/v1/auth/google/callback`
  - 驗證 state，交換 token，取得 Google profile
  - 依 email / subject 映射 internal user
  - 寫入 `auth.google.login.succeeded` or `auth.google.login.failed`
  - 建立 session / jwt（依部署策略）

### 3.2 Profile

- `GET /api/v1/me/profile`
- `PATCH /api/v1/me/profile`
  - 僅可改 mutable 欄位
  - 必寫 `user.profile.updated` 事件及 audit

### 3.3 Role Management (User-editable by authorized actors)

- `GET /api/v1/roles`
- `POST /api/v1/roles`
- `PATCH /api/v1/roles/{role_id}`
- `POST /api/v1/roles/{role_id}/permissions:grant`
- `POST /api/v1/roles/{role_id}/permissions:revoke`
- `POST /api/v1/users/{user_id}/roles:assign`
- `POST /api/v1/users/{user_id}/roles:revoke`

所有寫入 API 必須：

1. 驗證 workspace scope（不可相信外部傳入 workspace id）
2. 驗證操作者是否具管理權限
3. 要求 reason_code（高風險或敏感權限變更）
4. 寫入 canonical event
5. 寫入 tamper-evident audit
6. 支援 idempotency key

---

## 4. DB Mapping (migration direction)

建議核心資料表：

- `user_profiles`
  - `id`, `workspace_id`, `user_id`, `display_name`, `locale`, `avatar_url`, timestamps
- `permissions`
  - `id`, `permission_code`, `permission_name`, `resource`, `action`, `risk_level`, `is_system`
- `role_permissions`
  - `id`, `workspace_id`, `role_id`, `permission_id`, timestamps
- `google_identities`
  - `id`, `workspace_id`, `user_id`, `google_sub`, `email`, `email_verified`, timestamps
- `auth_sessions` (若採 DB session)
  - `id`, `workspace_id`, `user_id`, `session_token_hash`, `expires_at`, timestamps

不可破壞 append-only 規則之表：

- `event_store`
- `audit_chain`
- `audit_seals`
- `point_ledger_entries`

---

## 5. Reason Codes (starter set)

- `RBAC_ROLE_CREATE_REQUESTED`
- `RBAC_ROLE_UPDATE_REQUESTED`
- `RBAC_PERMISSION_GRANT_REQUESTED`
- `RBAC_PERMISSION_REVOKE_REQUESTED`
- `RBAC_USER_ROLE_ASSIGN_REQUESTED`
- `RBAC_USER_ROLE_REVOKE_REQUESTED`
- `AUTH_GOOGLE_LOGIN_SUCCEEDED`
- `AUTH_GOOGLE_LOGIN_FAILED`

---

## 6. Security and Audit Constraints

1. 管理角色與權限變更需具備二次確認策略（至少可配置）
2. 高風險 permission（如 ledger adjustment）變更應記錄變更前後快照
3. 所有 auth / rbac material write 皆需 trace_id
4. Role 修改不得直接更新歷史 ledger；僅影響後續授權行為

---

## 7. Release Plan Binding

- v0.0.9: 文件與映射定稿（本文件）
- v0.1.0: 完成 Google OAuth 實作、RBAC role editable 流程、審計與事件落地
