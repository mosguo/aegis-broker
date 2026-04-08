-- AegisBroker control-plane bootstrap migration
-- additive-only baseline tables for event/audit/payment/points flows

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

CREATE TABLE IF NOT EXISTS payment_orders (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    payment_state TEXT NOT NULL,
    amount_minor BIGINT NOT NULL,
    currency TEXT NOT NULL,
    stripe_payment_intent_id TEXT,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment_webhook_events (
    id UUID PRIMARY KEY,
    provider TEXT NOT NULL,
    provider_event_id TEXT NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    raw_payload JSONB NOT NULL,
    signature_header TEXT,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS point_accounts (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    account_code TEXT NOT NULL,
    current_balance BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, account_code)
);

CREATE TABLE IF NOT EXISTS point_ledger_entries (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    account_id UUID NOT NULL REFERENCES point_accounts(id),
    entry_type TEXT NOT NULL,
    amount BIGINT NOT NULL,
    reason_code TEXT NOT NULL,
    related_entry_id UUID,
    trace_id UUID NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_point_ledger_entries_account
ON point_ledger_entries (workspace_id, account_id, created_at);
