CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- v0.1.0 auth/session/rbac baseline (additive)

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

INSERT INTO role_definitions (role_code, role_name, is_system)
VALUES
    ('workspace_admin', 'Workspace Administrator', true),
    ('workspace_member', 'Workspace Member', true)
ON CONFLICT (role_code) DO NOTHING;

INSERT INTO permission_definitions (permission_code, permission_name)
VALUES
    ('user.role.update', 'Update workspace user roles'),
    ('user.profile.read', 'Read current user profile')
ON CONFLICT (permission_code) DO NOTHING;

INSERT INTO role_permissions (id, role_code, permission_code)
VALUES
    (gen_random_uuid(), 'workspace_admin', 'user.role.update'),
    (gen_random_uuid(), 'workspace_admin', 'user.profile.read'),
    (gen_random_uuid(), 'workspace_member', 'user.profile.read')
ON CONFLICT (role_code, permission_code) DO NOTHING;
