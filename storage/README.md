# Storage / Database Overview (v0.1.1)

## Database Overview

AegisBroker PostgreSQL schema is organized into three layers:

1. **Control history (append-only where required)**
   - `event_store`
   - `audit_chain`
   - `payment_webhook_events`
   - `point_ledger_entries`
2. **Operational aggregates**
   - `payment_orders`
   - `point_accounts`
   - `users`, `user_profiles`, `user_sessions`, `oauth_identities`, `oauth_state_tokens`
3. **RBAC mapping**
   - `role_definitions`
   - `permission_definitions`
   - `role_permissions`
   - `user_roles`

## Initial Setup

1. Prepare `DATABASE_URL`.
2. Apply migrations in order:
   - `0001_init_control_tables.sql`
   - `0002_auth_session_rbac.sql`
3. Verify readiness endpoint: `GET /health/ready`.
4. Verify seeded roles/permissions exist:
   - `workspace_admin`
   - `workspace_member`

## Zeabur Deployment Notes

- Deploy `postgres` first.
- Run migrations before routing write traffic to backend.
- Keep `MAX_DB_CONNECTIONS` aligned with Zeabur plan capacity (default `5`).
