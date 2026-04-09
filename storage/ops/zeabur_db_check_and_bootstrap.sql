CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Zeabur bootstrap / repair script for the current auth + workspace baseline.
-- Safe to re-run: all DDL and seed writes are idempotent.

CREATE TABLE IF NOT EXISTS event_store (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_version INT NOT NULL,
    reason_code TEXT NOT NULL,
    trace_id UUID NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_event_store_aggregate
ON event_store (workspace_id, aggregate_type, aggregate_id, occurred_at);

CREATE TABLE IF NOT EXISTS audit_chain (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    actor_type TEXT NOT NULL,
    actor_id TEXT,
    operation_name TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    reason_code TEXT NOT NULL,
    trace_id UUID NOT NULL,
    event_id UUID,
    prev_hash BYTEA,
    entry_hash BYTEA NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    email TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, email)
);

CREATE TABLE IF NOT EXISTS user_profiles (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    display_name TEXT,
    avatar_url TEXT,
    locale TEXT NOT NULL DEFAULT 'zh-TW',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id)
);

CREATE TABLE IF NOT EXISTS oauth_identities (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    provider TEXT NOT NULL,
    provider_sub TEXT NOT NULL,
    email TEXT NOT NULL,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_sub)
);

CREATE TABLE IF NOT EXISTS oauth_state_tokens (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    state_token TEXT NOT NULL UNIQUE,
    nonce TEXT NOT NULL,
    trace_id UUID NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    workspace_id UUID NOT NULL,
    session_token TEXT NOT NULL UNIQUE,
    trace_id UUID NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_user_sessions_identity
ON user_sessions (workspace_id, user_id, expires_at)
WHERE revoked_at IS NULL;

CREATE TABLE IF NOT EXISTS role_definitions (
    role_code TEXT PRIMARY KEY,
    role_name TEXT NOT NULL,
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS permission_definitions (
    permission_code TEXT PRIMARY KEY,
    permission_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS role_permissions (
    id UUID PRIMARY KEY,
    role_code TEXT NOT NULL REFERENCES role_definitions(role_code),
    permission_code TEXT NOT NULL REFERENCES permission_definitions(permission_code),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (role_code, permission_code)
);

CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    role_code TEXT NOT NULL REFERENCES role_definitions(role_code),
    assigned_by UUID,
    reason_code TEXT NOT NULL,
    trace_id UUID NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    revoked_by UUID,
    revoke_reason_code TEXT
);

CREATE INDEX IF NOT EXISTS idx_user_roles_active
ON user_roles (workspace_id, user_id, role_code)
WHERE revoked_at IS NULL;

CREATE TABLE IF NOT EXISTS workspaces (
    id UUID PRIMARY KEY,
    workspace_code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    owner_user_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_workspaces_status
ON workspaces (status, created_at);

INSERT INTO workspaces (id, workspace_code, name, status)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'default',
    'Default Workspace',
    'active'
)
ON CONFLICT (workspace_code) DO NOTHING;

INSERT INTO role_definitions (role_code, role_name, is_system)
VALUES
    ('workspace_admin', 'Workspace Administrator', true),
    ('workspace_member', 'Workspace Member', true)
ON CONFLICT (role_code) DO NOTHING;

INSERT INTO permission_definitions (permission_code, permission_name)
VALUES
    ('user.role.update', 'Update workspace user roles'),
    ('user.profile.read', 'Read current user profile'),
    ('user.profile.update', 'Update current user profile')
ON CONFLICT (permission_code) DO NOTHING;

INSERT INTO role_permissions (id, role_code, permission_code)
VALUES
    (gen_random_uuid(), 'workspace_admin', 'user.role.update'),
    (gen_random_uuid(), 'workspace_admin', 'user.profile.read'),
    (gen_random_uuid(), 'workspace_admin', 'user.profile.update'),
    (gen_random_uuid(), 'workspace_member', 'user.profile.read'),
    (gen_random_uuid(), 'workspace_member', 'user.profile.update')
ON CONFLICT (role_code, permission_code) DO NOTHING;

SELECT
    table_name,
    CASE
        WHEN to_regclass(format('public.%s', table_name)) IS NULL THEN 'missing'
        ELSE 'ready'
    END AS status
FROM (
    VALUES
        ('workspaces'),
        ('oauth_state_tokens'),
        ('users'),
        ('oauth_identities'),
        ('user_profiles'),
        ('user_sessions'),
        ('role_definitions'),
        ('permission_definitions'),
        ('role_permissions'),
        ('user_roles'),
        ('event_store'),
        ('audit_chain')
) AS required(table_name)
ORDER BY table_name;
