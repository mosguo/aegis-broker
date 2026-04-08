# AegisBroker — PostgreSQL Schema & Migration Spec
Version: v1.1  
Status: Blueprint-ready / Repo-ready schema baseline  
Target deployment: ZEABUR  
System stack baseline: Python Frontend + Rust Backend + PostgreSQL + Object Storage  
Predecessor document: Core Event & Audit Architecture Spec & WorkItem List  
Addendum in this revision: Stripe-backed Payment Service + internal points ledger

This document defines the PostgreSQL schema and migration baseline required to implement the AegisBroker control framework, business records, payment service, and internal add-point / deduct-point service in a ZEABUR-compatible deployment.

---

## 0. Document Purpose

This document exists to turn the agreed architecture into concrete PostgreSQL implementation guidance.

Its objectives are:

1. Define the database design required for the event-first, traceable, auditable architecture.
2. Establish migration sequencing and dependency rules so the repo can bootstrap reliably on ZEABUR.
3. Define append-only, integrity-preserving data structures for event flow, audit chain, payment, and points ledger use cases.
4. Provide DDL-ready table specifications, indexes, keys, constraints, and migration order.
5. Preserve compatibility with the Rust API Contract Spec and ZEABUR Deployment & Repo Blueprint.

---

## 1. Schema Design Principles

The PostgreSQL design must satisfy:
- business records remain queryable by aggregate and timeline
- event and audit records remain append-only
- every material state change remains reconstructable
- trace_id correlates request, event, audit, and workflow records
- reason codes remain normalized and enforceable
- writes support idempotency for retry-safe APIs and webhooks
- points balance is ledger-backed, not mutable by destructive overwrite
- migrations are deterministic and startup-safe on ZEABUR
- schema supports future service decomposition without redesigning the core data model

---

## 2. Extension Baseline

Recommended minimum:
- pgcrypto
- citext
- pg_trgm

Optional later:
- btree_gin
- pgvector

---

## 3. Identifier and Timestamp Policy

Use UUID primary keys for domain tables with `gen_random_uuid()` defaults.  
Use `timestamptz` consistently.

Mutable tables:
- created_at
- updated_at

Append-only tables:
- created_at only, plus occurred_at / ingested_at as needed

---

## 4. Utility Functions

### 4.1 `set_updated_at()`
Shared trigger function for mutable tables.

### 4.2 `prevent_update_delete()`
Shared trigger function to reject UPDATE and DELETE against append-only tables.

Hash generation remains application-driven, but columns must exist to store:
- payload_hash
- event_hash
- prev_event_hash
- prev_audit_hash
- record_hash
- seal_hash

---

## 5. Core Identity and Workspace Tables

### 5.1 workspaces
- id uuid pk
- workspace_code text unique not null
- name text not null
- status text not null check in ('active','inactive','suspended')
- owner_user_id uuid null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

### 5.2 users
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- email citext not null
- display_name text not null
- status text not null check in ('active','inactive','invited','disabled')
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, email)

### 5.3 roles
- id uuid pk
- role_code text unique not null
- role_name text not null
- is_system boolean not null default false
- created_at timestamptz not null default now()

### 5.4 user_roles
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- role_id uuid not null references roles(id)
- created_at timestamptz not null default now()
- unique(workspace_id, user_id, role_id)

### 5.5 oauth_accounts
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- provider text not null
- provider_subject text not null
- email citext null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(provider, provider_subject)

### 5.6 sessions
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- session_token_hash text not null
- status text not null check in ('active','expired','revoked')
- expires_at timestamptz not null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

---

## 6. Core Business Master Tables

### 6.1 counterparties
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- counterparty_code text not null
- name text not null
- counterparty_type text not null check in ('buyer','supplier','broker_partner','logistics','warehouse','inspector','financial','other')
- region text null
- country_code text null
- status text not null check in ('active','inactive','blocked')
- notes text null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, counterparty_code)

### 6.2 contacts
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- counterparty_id uuid not null references counterparties(id)
- full_name text not null
- email citext null
- phone text null
- title text null
- is_primary boolean not null default false
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

### 6.3 broker_profiles
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- display_alias text null
- regions_json jsonb not null default '[]'::jsonb
- commodities_json jsonb not null default '[]'::jsonb
- preferences_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, user_id)

---

## 7. Core Transactional Business Tables

### 7.1 rfqs
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- rfq_number text not null
- requester_counterparty_id uuid not null references counterparties(id)
- commodity text not null
- spec_json jsonb not null default '{}'::jsonb
- requested_volume numeric(20,6) null
- requested_unit text null
- target_price numeric(20,6) null
- target_currency text null
- urgency text null check in ('low','normal','high','critical')
- source_channel text null check in ('chat','phone','email','manual','api','market_signal')
- current_state text not null
- received_at timestamptz not null
- closed_at timestamptz null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, rfq_number)

### 7.2 quotes
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- quote_number text not null
- rfq_id uuid null references rfqs(id)
- broker_user_id uuid null references users(id)
- buyer_counterparty_id uuid not null references counterparties(id)
- supplier_counterparty_id uuid null references counterparties(id)
- buy_side_price numeric(20,6) null
- sell_side_price numeric(20,6) null
- spread_amount numeric(20,6) null
- spread_currency text null
- confidence_score numeric(5,2) null check (confidence_score between 0 and 100)
- validity_expires_at timestamptz null
- current_state text not null
- pricing_snapshot_json jsonb not null default '{}'::jsonb
- reason_summary text null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, quote_number)

### 7.3 quote_sources
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- quote_id uuid not null references quotes(id)
- source_counterparty_id uuid null references counterparties(id)
- source_type text not null check in ('firm','indicative','internal_model','manual','other')
- price_value numeric(20,6) null
- price_currency text null
- reliability_score numeric(5,2) null check (reliability_score between 0 and 100)
- observed_at timestamptz null
- payload_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()

### 7.4 negotiations
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- quote_id uuid null references quotes(id)
- deal_id uuid null
- current_state text not null
- started_at timestamptz not null default now()
- ended_at timestamptz null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

### 7.5 negotiation_events
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- negotiation_id uuid not null references negotiations(id)
- related_quote_id uuid null references quotes(id)
- related_deal_id uuid null
- actor_type text not null check in ('user','counterparty','system')
- actor_user_id uuid null references users(id)
- actor_counterparty_id uuid null references counterparties(id)
- action_type text not null check in ('offer','counter','accept','reject','timeout','note')
- price_value numeric(20,6) null
- price_currency text null
- terms_json jsonb not null default '{}'::jsonb
- sentiment text null
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

### 7.6 deals
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- deal_number text not null
- quote_id uuid null references quotes(id)
- buyer_counterparty_id uuid not null references counterparties(id)
- supplier_counterparty_id uuid null references counterparties(id)
- commodity text not null
- agreed_price numeric(20,6) null
- price_currency text null
- agreed_volume numeric(20,6) null
- volume_unit text null
- incoterm text null
- current_state text not null
- confirmed_at timestamptz null
- completed_at timestamptz null
- failed_at timestamptz null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, deal_number)

### 7.7 deal_outcomes
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- deal_id uuid not null references deals(id)
- outcome_type text not null check in ('win','loss','cancel','complete','failure')
- win_reason_code text null
- loss_reason_code text null
- key_factor text null
- summary_text text null
- created_at timestamptz not null default now()
- unique(deal_id)

---

## 8. Execution, Supply Chain, and Settlement Tables

### 8.1 shipments
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- shipment_number text not null
- deal_id uuid null references deals(id)
- origin_counterparty_id uuid null references counterparties(id)
- destination_counterparty_id uuid null references counterparties(id)
- current_state text not null
- etd timestamptz null
- eta timestamptz null
- actual_departed_at timestamptz null
- actual_arrived_at timestamptz null
- shipment_context_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, shipment_number)

### 8.2 inspections
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- shipment_id uuid null references shipments(id)
- deal_id uuid null references deals(id)
- inspector_counterparty_id uuid null references counterparties(id)
- current_state text not null
- scheduled_at timestamptz null
- completed_at timestamptz null
- result_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

### 8.3 exceptions
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- exception_number text not null
- aggregate_type text not null
- aggregate_id uuid not null
- exception_type text not null
- severity text not null check in ('low','medium','high','critical')
- current_state text not null
- responsible_counterparty_id uuid null references counterparties(id)
- opened_at timestamptz not null
- resolved_at timestamptz null
- summary_text text null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, exception_number)

### 8.4 settlements
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- deal_id uuid not null references deals(id)
- current_state text not null
- amount_due numeric(20,6) null check (amount_due is null or amount_due >= 0)
- amount_paid numeric(20,6) null check (amount_paid is null or amount_paid >= 0)
- currency text null
- due_at timestamptz null
- paid_at timestamptz null
- disputed_at timestamptz null
- written_off_at timestamptz null
- settlement_context_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(deal_id)

---

## 9. Documents and File Metadata Tables

### 9.1 documents
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- document_number text null
- document_type text not null
- storage_key text not null
- original_filename text not null
- mime_type text null
- file_size_bytes bigint null
- uploaded_by_user_id uuid null references users(id)
- aggregate_type text null
- aggregate_id uuid null
- checksum_sha256 text null
- created_at timestamptz not null default now()
- unique(workspace_id, storage_key)

### 9.2 document_versions
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- document_id uuid not null references documents(id)
- version_no integer not null
- storage_key text not null
- checksum_sha256 text null
- uploaded_by_user_id uuid null references users(id)
- created_at timestamptz not null default now()
- unique(document_id, version_no)

### 9.3 document_links
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- document_id uuid not null references documents(id)
- linked_aggregate_type text not null
- linked_aggregate_id uuid not null
- created_at timestamptz not null default now()

---

## 10. Decision and Trust Tables

### 10.1 decision_logs
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- aggregate_type text not null
- aggregate_id uuid not null
- decision_context_type text not null
- actor_type text not null check in ('user','system')
- actor_user_id uuid null references users(id)
- decision_action text not null
- confidence_score numeric(5,2) null check (confidence_score is null or confidence_score between 0 and 100)
- primary_reason_code text null
- referenced_data_json jsonb not null default '{}'::jsonb
- explanation_text text null
- trace_id uuid null
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

### 10.2 trust_profiles
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- counterparty_id uuid not null references counterparties(id)
- trust_score numeric(5,2) not null default 0 check (trust_score between 0 and 100)
- risk_band text null
- summary_json jsonb not null default '{}'::jsonb
- last_scored_at timestamptz null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, counterparty_id)

### 10.3 trust_score_history
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- trust_profile_id uuid not null references trust_profiles(id)
- prior_score numeric(5,2) null
- new_score numeric(5,2) not null
- reason_code text null
- trace_id uuid null
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

---

## 11. Payment and Points Tables

### 11.1 payment_orders

Purpose:
Internal payment aggregate that wraps an external Stripe payment lifecycle.

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- payment_order_number text not null
- payer_user_id uuid null references users(id)
- deal_id uuid null references deals(id)
- settlement_id uuid null references settlements(id)
- purpose_type text not null check in ('subscription','wallet_topup','deal_payment','service_fee','manual')
- current_state text not null
- currency text not null
- amount_minor bigint not null check (amount_minor >= 0)
- stripe_payment_intent_id text null
- stripe_customer_id text null
- stripe_checkout_session_id text null
- client_reference_id text null
- idempotency_key text null
- points_credit_amount bigint null check (points_credit_amount is null or points_credit_amount >= 0)
- metadata_json jsonb not null default '{}'::jsonb
- succeeded_at timestamptz null
- failed_at timestamptz null
- canceled_at timestamptz null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Constraints:
- unique(workspace_id, payment_order_number)
- unique(stripe_payment_intent_id) where stripe_payment_intent_id is not null
- unique(workspace_id, idempotency_key) where idempotency_key is not null

Indexes:
- index(workspace_id, current_state)
- index(payer_user_id)
- index(deal_id)
- index(settlement_id)

### 11.2 payment_webhook_events

Purpose:
Idempotent capture of Stripe webhook deliveries and verification outcome.

Columns:
- id uuid pk
- workspace_id uuid null references workspaces(id)
- provider text not null check in ('stripe')
- provider_event_id text not null
- provider_event_type text not null
- signature_verified boolean not null default false
- processing_state text not null check in ('received','verified','processed','ignored','failed')
- trace_id uuid null
- payment_order_id uuid null references payment_orders(id)
- payload_json jsonb not null
- received_at timestamptz not null
- processed_at timestamptz null
- created_at timestamptz not null default now()

Constraints:
- unique(provider, provider_event_id)

Indexes:
- index(provider_event_type)
- index(processing_state)
- index(trace_id)
- index(payment_order_id)

### 11.3 point_accounts

Purpose:
Current checkpoint balance per user/workspace. Canonical truth still comes from point ledger entries.

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- current_balance bigint not null default 0
- reserved_balance bigint not null default 0
- status text not null check in ('active','locked','suspended')
- last_ledger_seq bigint not null default 0
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Constraints:
- unique(workspace_id, user_id)

Indexes:
- index(workspace_id, status)

### 11.4 point_ledger_entries

Purpose:
Immutable value ledger for every add-point, deduct-point, reversal, and manual adjustment.

Columns:
- id bigserial primary key
- ledger_entry_id uuid not null
- workspace_id uuid not null references workspaces(id)
- point_account_id uuid not null references point_accounts(id)
- ledger_seq bigint not null
- entry_type text not null check in ('credit','debit','reversal','adjustment','hold','release')
- source_type text not null check in ('payment','service_usage','manual','refund','reconciliation','system')
- source_id uuid null
- reason_code text not null
- amount bigint not null check (amount >= 0)
- signed_amount bigint not null
- balance_before bigint not null
- balance_after bigint not null
- trace_id uuid null
- payment_order_id uuid null references payment_orders(id)
- related_entry_id uuid null
- metadata_json jsonb not null default '{}'::jsonb
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

Constraints:
- unique(ledger_entry_id)
- unique(point_account_id, ledger_seq)

Indexes:
- index(workspace_id, point_account_id, ledger_seq)
- index(trace_id)
- index(payment_order_id)
- index(source_type, source_id)

### 11.5 point_service_consumptions

Purpose:
Optional business-facing record linking service actions to point debits.

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- service_code text not null
- quantity numeric(20,6) null
- points_debited bigint not null check (points_debited >= 0)
- point_ledger_entry_id uuid null
- trace_id uuid null
- metadata_json jsonb not null default '{}'::jsonb
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

Indexes:
- index(workspace_id, user_id, occurred_at)
- index(service_code)
- index(trace_id)

---

## 12. Workflow Tables

### 12.1 tasks
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- task_number text not null
- source_aggregate_type text null
- source_aggregate_id uuid null
- task_type text not null
- status text not null check in ('open','in_progress','blocked','done','cancelled')
- title text not null
- description text null
- assigned_user_id uuid null references users(id)
- due_at timestamptz null
- trace_id uuid null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, task_number)

### 12.2 task_events
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- task_id uuid not null references tasks(id)
- actor_user_id uuid null references users(id)
- event_type text not null
- payload_json jsonb not null default '{}'::jsonb
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

---

## 13. Event Store Tables

### 13.1 event_store
Columns:
- id bigserial primary key
- event_id uuid not null
- workspace_id uuid not null references workspaces(id)
- aggregate_type text not null
- aggregate_id uuid not null
- aggregate_seq bigint not null check (aggregate_seq > 0)
- event_type text not null
- event_version integer not null check (event_version > 0)
- payload_schema_version integer not null check (payload_schema_version > 0)
- trace_id uuid null
- span_id uuid null
- parent_span_id uuid null
- correlation_id uuid null
- causation_event_id uuid null
- actor_type text not null check in ('user','system','external')
- actor_id uuid null
- source_service text not null
- occurred_at timestamptz not null
- ingested_at timestamptz not null default now()
- state_before text null
- state_after text null
- reason_code_primary text null
- reason_codes_secondary jsonb not null default '[]'::jsonb
- payload_json jsonb not null
- is_material boolean not null default false
- audit_required boolean not null default false
- idempotency_key text null
- prev_event_hash text null
- event_hash text not null
- signature text null
- created_at timestamptz not null default now()

Constraints:
- unique(event_id)
- unique(workspace_id, aggregate_type, aggregate_id, aggregate_seq)
- unique(workspace_id, idempotency_key) where idempotency_key is not null

### 13.2 aggregate_state_snapshots
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- aggregate_type text not null
- aggregate_id uuid not null
- current_state text not null
- last_event_id uuid not null
- last_aggregate_seq bigint not null
- snapshot_json jsonb not null default '{}'::jsonb
- snapshot_taken_at timestamptz not null default now()
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(workspace_id, aggregate_type, aggregate_id)

---

## 14. State Machine Support Tables

### 14.1 state_transition_rules
- id uuid pk
- aggregate_type text not null
- from_state text not null
- to_state text not null
- trigger_event_type text not null
- allowed_actor_types jsonb not null default '[]'::jsonb
- required_reason_codes jsonb not null default '[]'::jsonb
- guard_expression text null
- side_effects_json jsonb not null default '[]'::jsonb
- invalid_transition_error_code text not null
- audit_required boolean not null default false
- is_active boolean not null default true
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()
- unique(aggregate_type, from_state, to_state, trigger_event_type)

### 14.2 failed_transition_records
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- aggregate_type text not null
- aggregate_id uuid not null
- attempted_from_state text null
- attempted_to_state text null
- trigger_event_type text not null
- error_code text not null
- reason_code text null
- trace_id uuid null
- actor_type text not null
- actor_id uuid null
- payload_json jsonb not null default '{}'::jsonb
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

---

## 15. Reason Code Registry Tables

### 15.1 reason_code_registry
- code text primary key
- domain text not null
- category text not null
- title text not null
- description text null
- severity text null
- attribution_class text null
- status text not null check in ('active','deprecated','disabled')
- effective_from timestamptz not null default now()
- effective_to timestamptz null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

### 15.2 aggregate_reason_links
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- aggregate_type text not null
- aggregate_id uuid not null
- reason_code text not null references reason_code_registry(code)
- source_kind text not null check in ('decision','event','audit','outcome','override')
- trace_id uuid null
- created_at timestamptz not null default now()

---

## 16. Audit Chain Tables

### 16.1 audit_chain
- id bigserial primary key
- audit_id uuid not null
- workspace_id uuid not null references workspaces(id)
- entity_type text not null
- entity_id uuid not null
- action_type text not null
- source_event_id uuid null
- trace_id uuid null
- actor_type text not null check in ('user','system','external')
- actor_id uuid null
- occurred_at timestamptz not null
- payload_hash text not null
- prev_audit_hash text null
- record_hash text not null
- signature text null
- seal_batch_id uuid null
- verification_status text null
- metadata_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()
- unique(audit_id)
- unique(record_hash)

### 16.2 audit_seals
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- seal_batch_id uuid not null
- first_audit_row_id bigint not null
- last_audit_row_id bigint not null
- record_count integer not null
- seal_hash text not null
- verification_method text not null
- sealed_at timestamptz not null
- created_at timestamptz not null default now()
- unique(seal_batch_id)
- unique(workspace_id, first_audit_row_id, last_audit_row_id)

### 16.3 audit_verification_runs
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- verification_scope text not null check in ('single_chain','seal_batch','range')
- target_identifier text not null
- result_status text not null check in ('passed','failed','partial')
- first_broken_row_id bigint null
- details_json jsonb not null default '{}'::jsonb
- trace_id uuid null
- executed_at timestamptz not null
- created_at timestamptz not null default now()

---

## 17. Observability-supporting Tables

### 17.1 trace_operation_logs
- id uuid pk
- workspace_id uuid null references workspaces(id)
- trace_id uuid not null
- span_id uuid not null
- parent_span_id uuid null
- operation_name text not null
- service_name text not null
- aggregate_type text null
- aggregate_id uuid null
- status text not null
- error_code text null
- started_at timestamptz not null
- ended_at timestamptz null
- created_at timestamptz not null default now()

### 17.2 job_execution_logs
- id uuid pk
- workspace_id uuid null references workspaces(id)
- job_name text not null
- trace_id uuid null
- status text not null
- payload_json jsonb not null default '{}'::jsonb
- started_at timestamptz not null
- ended_at timestamptz null
- created_at timestamptz not null default now()

---

## 18. Append-only Enforcement Policy

The following tables must be append-only:
- event_store
- audit_chain
- audit_seals
- audit_verification_runs
- trust_score_history
- negotiation_events
- task_events
- failed_transition_records
- payment_webhook_events
- point_ledger_entries
- point_service_consumptions
- job_execution_logs optional
- trace_operation_logs optional

Application controls:
- no update/delete repository methods

Database controls:
- attach `prevent_update_delete()` trigger
- revoke update/delete permissions from app role where feasible

---

## 19. Referential Integrity Policy

Hard FKs:
- workspaces to users/core records
- deals to quotes
- shipments to deals
- settlements to deals
- payment_orders to deals/settlements/users
- point_accounts to users
- point_ledger_entries to point_accounts and payment_orders when linked

Soft links:
- aggregate_type + aggregate_id
- entity_type + entity_id
- linked_aggregate_type + linked_aggregate_id

---

## 20. Partitioning Strategy

Initial ZEABUR baseline:
- no partitioning required at first release

Likely later partition candidates:
- event_store
- audit_chain
- payment_webhook_events
- point_ledger_entries
- trace_operation_logs

Design readiness:
- keep workspace_id and occurred_at indexed
- allow later monthly partitioning

---

## 21. Idempotency Strategy

The schema must support safe retries for:
- create RFQ
- create quote
- send quote
- confirm deal
- create shipment
- resolve exception
- mark settlement received
- create payment order
- process Stripe webhook
- credit points
- debit points
- manual override commands

Baseline handling:
- event_store unique(workspace_id, idempotency_key)
- payment_orders unique(workspace_id, idempotency_key)
- payment_webhook_events unique(provider, provider_event_id)

---

## 22. Update Trigger Policy

Attach `set_updated_at()` to mutable tables including:
- workspaces
- users
- oauth_accounts
- sessions
- counterparties
- contacts
- broker_profiles
- rfqs
- quotes
- negotiations
- deals
- shipments
- inspections
- exceptions
- settlements
- payment_orders
- point_accounts
- trust_profiles
- tasks
- aggregate_state_snapshots
- state_transition_rules
- reason_code_registry

Do not attach to append-only tables.

---

## 23. Migration Sequencing

Recommended migration sequence:

1. extensions and utility functions  
2. core identity and workspace tables  
3. master business tables  
4. transactional business tables  
5. documents and file metadata  
6. decision and trust tables  
7. payment_orders and payment_webhook_events  
8. point_accounts and point_ledger_entries  
9. workflow tables  
10. reason code registry  
11. event store  
12. aggregate state snapshot and state transition support  
13. audit chain and verification tables  
14. observability-supporting tables  
15. triggers and append-only guards  
16. seed data for roles and starter reason codes

---

## 24. Suggested Migration File List

001_enable_extensions.sql  
002_create_utility_functions.sql  
003_create_workspaces.sql  
004_create_users_roles_auth.sql  
005_create_counterparties_contacts.sql  
006_create_broker_profiles.sql  
007_create_rfqs_quotes.sql  
008_create_negotiations_deals.sql  
009_create_shipments_inspections_exceptions_settlements.sql  
010_create_documents.sql  
011_create_decision_and_trust.sql  
012_create_payment_orders.sql  
013_create_point_accounts_and_ledger.sql  
014_create_tasks.sql  
015_create_reason_code_registry.sql  
016_create_event_store.sql  
017_create_state_machine_support.sql  
018_create_audit_chain.sql  
019_create_observability_logs.sql  
020_attach_updated_at_triggers.sql  
021_attach_append_only_guards.sql  
022_seed_roles.sql  
023_seed_reason_codes.sql

---

## 25. Seed Data Policy

### 25.1 roles seed
- owner
- admin
- broker
- ops
- viewer
- risk_reviewer
- finance_admin

### 25.2 reason codes seed

Include starter dictionary from prior docs and add:
- P-INIT-001
- P-GATE-001
- P-GATE-002
- P-GATE-003
- P-GATE-004
- P-WEBH-001
- P-WEBH-002
- P-REFD-001
- P-REFD-002
- P-RECO-001
- L-CRDT-001
- L-DEBT-001
- L-ADJ-001
- L-RVSL-001
- L-LOCK-001
- L-DUPL-001

---

## 26. Query Patterns the Schema Must Support

The schema must efficiently support:

1. Aggregate timeline reconstruction  
2. Full trace reconstruction  
3. Audit verification history  
4. Current state reads  
5. Loss attribution analysis  
6. Counterparty trust explanation  
7. Quote/deal progression  
8. Payment order lifecycle by workspace or payer  
9. Webhook processing traceability  
10. Point balance explanation from immutable ledger  
11. Service usage cost attribution via point debits

---

## 27. ZEABUR Migration Safety

Backend startup must not accept write traffic before:
- migrations complete successfully
- required seed data exists
- append-only guards are attached
- required indexes exist

Future changes must prefer:
- additive columns
- additive tables
- new reason codes
- new transition rules
- new indexes
- non-destructive backfills
- reversal ledger entries instead of destructive balance rewrites

---

## 28. Design Completion Assessment After This Deliverable

This document closes most database-level ambiguity for business, control, payment, and point-ledger design. Remaining major gap to 100% is the ZEABUR Deployment & Repo Blueprint.

---

## 29. Final Statement

This PostgreSQL Schema & Migration Spec establishes the database backbone for AegisBroker’s event-driven, state-controlled, trace-correlated, reason-coded, tamper-evident operating model, including Stripe-backed payments and internal add-point / deduct-point service behavior.

From this point forward:
- all new domain features must map to the event store model
- all material transitions must respect state machine support tables
- all important explanations must align to the reason code registry
- all integrity-sensitive records must write through the audit chain
- all payment callbacks must be idempotent and verifiable
- all points must move through immutable ledger entries
