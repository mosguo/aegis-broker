CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- v0.1.5 workspace bootstrap

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
