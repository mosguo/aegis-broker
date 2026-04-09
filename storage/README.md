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
   - `0003_profile_update_permission.sql`
   - `0004_workspaces.sql`
3. Verify readiness endpoint: `GET /health/ready`.
4. Verify seeded roles/permissions exist:
   - `workspace_admin`
   - `workspace_member`
5. Verify seeded workspace exists:
   - `00000000-0000-0000-0000-000000000001`
   - `workspace_code=default`

## Zeabur Bootstrap Repair

If a Zeabur PostgreSQL database was provisioned before the latest auth/workspace migrations,
run:

- `storage/ops/zeabur_db_check_and_bootstrap.sql`

This script is idempotent and will:

- create missing auth/workspace/control tables
- seed the default workspace
- seed baseline roles and permissions
- print a ready/missing status row for the current required tables

## Zeabur Deployment Notes

- Deploy `postgres` first.
- Run migrations before routing write traffic to backend.
- Keep `MAX_DB_CONNECTIONS` aligned with Zeabur plan capacity (default `5`).
