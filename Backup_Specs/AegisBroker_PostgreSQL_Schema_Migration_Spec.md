# AegisBroker — PostgreSQL Schema & Migration Spec
Version: v1.0  
Status: Blueprint-ready / Repo-ready schema baseline  
Target deployment: ZEABUR  
System stack baseline: Python Frontend + Rust Backend + PostgreSQL + Object Storage  
Predecessor document: Core Event & Audit Architecture Spec & WorkItem List  
This document defines the PostgreSQL schema and migration baseline required to implement the AegisBroker control framework and core business records in a ZEABUR-compatible deployment. It is written to move the project from architecture-level clarity into database-level implementation readiness.

---

## 0. Document Purpose

This document exists to turn the previously agreed architecture into concrete PostgreSQL implementation guidance.

Its objectives are:

1. Define the database design required for the event-first, traceable, auditable architecture.
2. Establish migration sequencing and dependency rules so the repo can bootstrap reliably on ZEABUR.
3. Define append-only, integrity-preserving data structures for event flow and audit chain use cases.
4. Provide DDL-ready table specifications, indexes, keys, enums, constraints, and migration order.
5. Preserve compatibility with later Rust API Contract Spec and ZEABUR Deployment & Repo Blueprint.

This document is intended to be sufficiently explicit that a competent engineer can implement SQL migrations without having to redesign the schema model.

---

## 1. Schema Design Principles

### 1.1 Primary principles

The PostgreSQL design must satisfy the following principles:

- business records must remain queryable by aggregate and timeline
- event and audit records must be append-only
- every material state change must be reconstructable
- trace_id must correlate request, event, audit, and workflow records
- reason codes must be normalized and enforceable
- writes must support idempotency for retry-safe APIs
- migrations must be deterministic and startup-safe on ZEABUR
- schema must support future service decomposition without redesigning the core data model

### 1.2 Storage model categories

The schema is divided into these categories:

- identity and workspace
- master and transactional business records
- decision and control records
- event store
- state machine support
- reason code registry
- tamper-evident audit chain
- workflow / tasks
- observability-supporting records
- file/object metadata

### 1.3 Schema namespace recommendation

Use a single PostgreSQL database and initially keep all tables in the `public` schema unless operational complexity later requires separation.

Optional future namespaces:
- control
- market
- workflow
- audit

For initial ZEABUR deployment, `public` is acceptable and simpler.

---

## 2. Extension Baseline

The following PostgreSQL extensions should be enabled in early migrations where available:

- pgcrypto
- uuid-ossp (optional if pgcrypto/gen_random_uuid is sufficient)
- citext
- btree_gin (optional but useful)
- pg_trgm (recommended for search-like needs)
- pgvector (optional and deferred; not required for this phase)

Recommended minimum:
- pgcrypto
- citext
- pg_trgm

---

## 3. Identifier and Timestamp Policy

### 3.1 Primary keys

Use UUID primary keys for almost all domain tables.

Recommendation:
- `uuid` with `gen_random_uuid()` default

### 3.2 Timestamps

All important tables should include:
- `created_at timestamptz not null default now()`
- `updated_at timestamptz not null default now()` where mutable
- `occurred_at timestamptz` for event/audit/business timing
- `ingested_at timestamptz` where write time differs from source occurrence time

### 3.3 Updated_at trigger

Use a shared trigger function for mutable tables:
- `set_updated_at()`

Do not use this on append-only tables such as event_store or audit_chain.

---

## 4. Enum Strategy

Avoid excessive PostgreSQL ENUM lock-in for business states that may evolve often. Prefer constrained text + check constraints or dedicated registry tables where flexibility matters.

Recommended approach:
- use text columns with check constraints for stable domains
- use registry/reference tables for evolving code systems
- use native enum only for highly stable actor/source classes if desired

For this schema baseline, text + check constraints is preferred.

---

## 5. Core Shared Utility Functions

### 5.1 set_updated_at function

A generic trigger function should update `updated_at`.

### 5.2 append-only guard function

Create a reusable trigger function to reject UPDATE and DELETE against append-only tables.

Suggested function:
- `prevent_update_delete()`

Apply to:
- event_store
- audit_chain
- audit_seals
- failed_transition_records if designated immutable

### 5.3 hash helper policy

Hash generation will typically happen in the application layer for consistency with Rust code.  
However, the schema must provide text/binary columns to store:
- payload_hash
- event_hash
- prev_event_hash
- prev_audit_hash
- record_hash
- seal_hash

The document does not require DB-side hashing, but permits optional DB-side verification helpers later.

---

## 6. Core Identity and Workspace Tables

### 6.1 workspaces

Purpose:
Represents a tenant or broker workspace boundary.

Columns:
- id uuid pk
- workspace_code text unique not null
- name text not null
- status text not null check in ('active','inactive','suspended')
- owner_user_id uuid null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Indexes:
- unique(workspace_code)
- index(owner_user_id)

### 6.2 users

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- email citext not null
- display_name text not null
- status text not null check in ('active','inactive','invited','disabled')
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Constraints:
- unique(workspace_id, email)

Indexes:
- index(workspace_id)
- index(email)

### 6.3 roles

Columns:
- id uuid pk
- role_code text unique not null
- role_name text not null
- is_system boolean not null default false
- created_at timestamptz not null default now()

### 6.4 user_roles

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- role_id uuid not null references roles(id)
- created_at timestamptz not null default now()

Constraints:
- unique(workspace_id, user_id, role_id)

### 6.5 oauth_accounts

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- provider text not null
- provider_subject text not null
- email citext null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Constraints:
- unique(provider, provider_subject)

### 6.6 sessions

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- session_token_hash text not null
- status text not null check in ('active','expired','revoked')
- expires_at timestamptz not null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Indexes:
- index(workspace_id, user_id)
- index(expires_at)

---

## 7. Core Business Master Tables

### 7.1 counterparties

Purpose:
Stores clients, suppliers, buyers, logistics parties, inspection providers, warehouses, and others.

Columns:
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

Constraints:
- unique(workspace_id, counterparty_code)

Indexes:
- index(workspace_id, counterparty_type)
- index(workspace_id, name)
- gin index on name using pg_trgm optional

### 7.2 contacts

Columns:
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

Indexes:
- index(counterparty_id)
- index(email)

### 7.3 broker_profiles

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- user_id uuid not null references users(id)
- display_alias text null
- regions_json jsonb not null default '[]'::jsonb
- commodities_json jsonb not null default '[]'::jsonb
- preferences_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Constraints:
- unique(workspace_id, user_id)

---

## 8. Core Transactional Business Tables

### 8.1 rfqs

Columns:
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

Constraints:
- unique(workspace_id, rfq_number)

Indexes:
- index(workspace_id, current_state)
- index(requester_counterparty_id)
- index(received_at)

### 8.2 quotes

Columns:
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
- confidence_score numeric(5,2) null
- validity_expires_at timestamptz null
- current_state text not null
- pricing_snapshot_json jsonb not null default '{}'::jsonb
- reason_summary text null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Constraints:
- unique(workspace_id, quote_number)

Indexes:
- index(workspace_id, current_state)
- index(rfq_id)
- index(buyer_counterparty_id)
- index(validity_expires_at)

### 8.3 quote_sources

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- quote_id uuid not null references quotes(id)
- source_counterparty_id uuid null references counterparties(id)
- source_type text not null check in ('firm','indicative','internal_model','manual','other')
- price_value numeric(20,6) null
- price_currency text null
- reliability_score numeric(5,2) null
- observed_at timestamptz null
- payload_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()

Indexes:
- index(quote_id)
- index(source_counterparty_id)

### 8.4 negotiations

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- quote_id uuid null references quotes(id)
- deal_id uuid null
- current_state text not null
- started_at timestamptz not null default now()
- ended_at timestamptz null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Indexes:
- index(quote_id)
- index(current_state)

### 8.5 negotiation_events

Columns:
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

Indexes:
- index(negotiation_id, occurred_at)
- index(related_quote_id)
- index(related_deal_id)

### 8.6 deals

Columns:
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

Constraints:
- unique(workspace_id, deal_number)

Indexes:
- index(workspace_id, current_state)
- index(quote_id)
- index(buyer_counterparty_id)
- index(supplier_counterparty_id)

### 8.7 deal_outcomes

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- deal_id uuid not null references deals(id)
- outcome_type text not null check in ('win','loss','cancel','complete','failure')
- win_reason_code text null
- loss_reason_code text null
- key_factor text null
- summary_text text null
- created_at timestamptz not null default now()

Constraints:
- unique(deal_id)

Indexes:
- index(workspace_id, outcome_type)

---

## 9. Execution, Supply Chain, and Settlement Tables

### 9.1 shipments

Columns:
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

Constraints:
- unique(workspace_id, shipment_number)

Indexes:
- index(workspace_id, current_state)
- index(deal_id)

### 9.2 inspections

Columns:
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

Indexes:
- index(shipment_id)
- index(deal_id)
- index(current_state)

### 9.3 exceptions

Columns:
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

Constraints:
- unique(workspace_id, exception_number)

Indexes:
- index(workspace_id, aggregate_type, aggregate_id)
- index(current_state)
- index(severity)

### 9.4 settlements

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- deal_id uuid not null references deals(id)
- current_state text not null
- amount_due numeric(20,6) null
- amount_paid numeric(20,6) null
- currency text null
- due_at timestamptz null
- paid_at timestamptz null
- disputed_at timestamptz null
- written_off_at timestamptz null
- settlement_context_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Constraints:
- unique(deal_id)

Indexes:
- index(workspace_id, current_state)
- index(due_at)

---

## 10. Documents and File Metadata Tables

### 10.1 documents

Columns:
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

Indexes:
- index(workspace_id, document_type)
- index(workspace_id, aggregate_type, aggregate_id)
- unique(workspace_id, storage_key)

### 10.2 document_versions

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- document_id uuid not null references documents(id)
- version_no integer not null
- storage_key text not null
- checksum_sha256 text null
- uploaded_by_user_id uuid null references users(id)
- created_at timestamptz not null default now()

Constraints:
- unique(document_id, version_no)

### 10.3 document_links

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- document_id uuid not null references documents(id)
- linked_aggregate_type text not null
- linked_aggregate_id uuid not null
- created_at timestamptz not null default now()

Indexes:
- index(document_id)
- index(workspace_id, linked_aggregate_type, linked_aggregate_id)

---

## 11. Decision and Trust Tables

### 11.1 decision_logs

Purpose:
Captures the human or system decision layer required for replay and reasoning.

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- aggregate_type text not null
- aggregate_id uuid not null
- decision_context_type text not null
- actor_type text not null check in ('user','system')
- actor_user_id uuid null references users(id)
- decision_action text not null
- confidence_score numeric(5,2) null
- primary_reason_code text null
- referenced_data_json jsonb not null default '{}'::jsonb
- explanation_text text null
- trace_id uuid null
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

Indexes:
- index(workspace_id, aggregate_type, aggregate_id)
- index(trace_id)
- index(primary_reason_code)

### 11.2 trust_profiles

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- counterparty_id uuid not null references counterparties(id)
- trust_score numeric(5,2) not null default 0
- risk_band text null
- summary_json jsonb not null default '{}'::jsonb
- last_scored_at timestamptz null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Constraints:
- unique(workspace_id, counterparty_id)

### 11.3 trust_score_history

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- trust_profile_id uuid not null references trust_profiles(id)
- prior_score numeric(5,2) null
- new_score numeric(5,2) not null
- reason_code text null
- trace_id uuid null
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

Indexes:
- index(trust_profile_id, occurred_at)
- index(reason_code)

---

## 12. Workflow Tables

### 12.1 tasks

Columns:
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

Constraints:
- unique(workspace_id, task_number)

Indexes:
- index(workspace_id, status)
- index(assigned_user_id)
- index(trace_id)

### 12.2 task_events

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- task_id uuid not null references tasks(id)
- actor_user_id uuid null references users(id)
- event_type text not null
- payload_json jsonb not null default '{}'::jsonb
- occurred_at timestamptz not null
- created_at timestamptz not null default now()

Indexes:
- index(task_id, occurred_at)

---

## 13. Event Store Tables

### 13.1 event_store

This is the central immutable event table.

Columns:
- id bigserial primary key
- event_id uuid not null
- workspace_id uuid not null references workspaces(id)
- aggregate_type text not null
- aggregate_id uuid not null
- aggregate_seq bigint not null
- event_type text not null
- event_version integer not null
- payload_schema_version integer not null
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

Indexes:
- index(workspace_id, aggregate_type, aggregate_id, aggregate_seq)
- index(trace_id)
- index(event_type)
- index(occurred_at)
- index(workspace_id, is_material)
- gin index(payload_json)

Notes:
- `id` bigserial supports fast append locality
- `event_id uuid` remains external canonical identifier
- `aggregate_seq` must be monotonic per aggregate

### 13.2 aggregate_state_snapshots

Recommended but optional for performance.

Columns:
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

Constraints:
- unique(workspace_id, aggregate_type, aggregate_id)

Indexes:
- index(workspace_id, aggregate_type, current_state)

---

## 14. State Machine Support Tables

### 14.1 state_transition_rules

Purpose:
Stores machine-readable rules for valid transitions.

Columns:
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

Constraints:
- unique(aggregate_type, from_state, to_state, trigger_event_type)

Indexes:
- index(aggregate_type, from_state)
- index(aggregate_type, to_state)
- index(is_active)

### 14.2 failed_transition_records

Columns:
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

Indexes:
- index(workspace_id, aggregate_type, aggregate_id)
- index(trace_id)
- index(error_code)
- index(occurred_at)

---

## 15. Reason Code Registry Tables

### 15.1 reason_code_registry

Columns:
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

Indexes:
- index(domain, category)
- index(status)

### 15.2 aggregate_reason_links

Optional normalization layer if multiple codes must attach to one entity outside event payloads.

Columns:
- id uuid pk
- workspace_id uuid not null references workspaces(id)
- aggregate_type text not null
- aggregate_id uuid not null
- reason_code text not null references reason_code_registry(code)
- source_kind text not null check in ('decision','event','audit','outcome','override')
- trace_id uuid null
- created_at timestamptz not null default now()

Indexes:
- index(workspace_id, aggregate_type, aggregate_id)
- index(reason_code)
- index(trace_id)

---

## 16. Audit Chain Tables

### 16.1 audit_chain

Immutable, append-only, tamper-evident audit records.

Columns:
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

Constraints:
- unique(audit_id)
- unique(record_hash)

Indexes:
- index(workspace_id, entity_type, entity_id, id)
- index(trace_id)
- index(source_event_id)
- index(occurred_at)

### 16.2 audit_seals

Columns:
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

Constraints:
- unique(seal_batch_id)
- unique(workspace_id, first_audit_row_id, last_audit_row_id)

Indexes:
- index(workspace_id, sealed_at desc)

### 16.3 audit_verification_runs

Columns:
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

Indexes:
- index(workspace_id, executed_at desc)
- index(trace_id)
- index(result_status)

---

## 17. Observability-supporting Tables

### 17.1 trace_operation_logs

Optional if not fully externalized to OTEL backend.

Columns:
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

Indexes:
- index(trace_id)
- index(service_name, started_at desc)
- index(workspace_id, aggregate_type, aggregate_id)

### 17.2 job_execution_logs

Columns:
- id uuid pk
- workspace_id uuid null references workspaces(id)
- job_name text not null
- trace_id uuid null
- status text not null
- payload_json jsonb not null default '{}'::jsonb
- started_at timestamptz not null
- ended_at timestamptz null
- created_at timestamptz not null default now()

Indexes:
- index(job_name, started_at desc)
- index(trace_id)

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
- job_execution_logs optional
- trace_operation_logs optional

Application controls:
- never expose update/delete repository methods for append-only tables

Database controls:
- attach `prevent_update_delete()` trigger to append-only tables
- revoke update/delete permissions from application role where feasible

---

## 19. Referential Integrity Policy

### 19.1 Hard FK usage

Use hard foreign keys for:
- workspaces to users/counterparties/core records
- deals to quotes
- shipments to deals where direct link exists
- settlements to deals
- decision_logs to workspace
- trust history to trust profile

### 19.2 Soft link usage

Use soft links for polymorphic aggregates:
- aggregate_type + aggregate_id
- entity_type + entity_id
- linked_aggregate_type + linked_aggregate_id

Reason:
PostgreSQL cannot directly enforce polymorphic FK integrity without added complexity.  
Application-layer validation is acceptable here.

---

## 20. Partitioning Strategy

### 20.1 Initial ZEABUR baseline

Do not partition initially unless data volume is expected to be extremely high from day one.

### 20.2 Partition candidates for later

Tables likely to benefit later:
- event_store by month or workspace+time
- audit_chain by month
- trace_operation_logs by month
- market_ticks when introduced in full scale

### 20.3 Design readiness

Even without initial partitioning:
- include `occurred_at` and `workspace_id` indexes
- keep migration plan open for later partitioning without renaming business columns

---

## 21. Idempotency Strategy

The schema must support safe retries for important write operations.

Recommended handling:
- material write APIs provide idempotency_key
- store idempotency_key in event_store for material event writes
- optionally add dedicated api_idempotency_keys table later if needed

Current baseline:
- event_store unique(workspace_id, idempotency_key) where idempotency_key is not null

For quote/deal direct creation endpoints, application may also maintain unique natural numbers such as quote_number and deal_number to reduce accidental duplication.

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
- trust_profiles
- tasks
- aggregate_state_snapshots
- state_transition_rules
- reason_code_registry

Do not attach to append-only tables.

---

## 23. Migration Sequencing

### 23.1 Migration order overview

Recommended migration sequence:

1. extensions and utility functions
2. core identity and workspace tables
3. master business tables
4. transactional business tables
5. documents and file metadata
6. decision and trust tables
7. workflow tables
8. reason code registry
9. event store
10. aggregate state snapshot and state transition support
11. audit chain and verification tables
12. observability-supporting tables
13. triggers and append-only guards
14. seed data for roles and starter reason codes

### 23.2 Why this order matters

- workspaces must exist before tenant-scoped records
- domain records should exist before event and audit records reference them
- reason code registry should exist before full validation enforcement
- append-only triggers should be attached after initial schema creation, not before
- seed data should run after tables and constraints exist

---

## 24. Suggested Migration File List

Recommended file naming style using timestamp or sequence.

Example:

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
012_create_tasks.sql  
013_create_reason_code_registry.sql  
014_create_event_store.sql  
015_create_state_machine_support.sql  
016_create_audit_chain.sql  
017_create_observability_logs.sql  
018_attach_updated_at_triggers.sql  
019_attach_append_only_guards.sql  
020_seed_roles.sql  
021_seed_reason_codes.sql

---

## 25. Seed Data Policy

### 25.1 roles seed

Initial roles:
- owner
- admin
- broker
- ops
- viewer
- risk_reviewer

### 25.2 reason codes seed

Seed the starter dictionary defined in the architecture document, including at minimum:
- Q-ELIG-001
- Q-PRICE-001
- Q-PRICE-002
- Q-MARGIN-001
- Q-SRC-001
- N-LOSS-001
- N-LOSS-002
- N-WIN-001
- N-WIN-002
- D-WIN-001
- D-LOSS-001
- E-FAIL-001
- E-FAIL-002
- E-FAIL-003
- E-FAIL-004
- S-RISK-001
- S-RISK-002
- S-CLOSE-001
- T-SCORE-001
- T-SCORE-002
- R-CNTRL-001
- R-CNTRL-002
- A-OVRD-001
- A-OVRD-002
- A-VERI-001
- A-VERI-002

---

## 26. Integrity Constraints and Check Policies

### 26.1 Numeric check examples

For scores:
- confidence_score between 0 and 100
- trust_score between 0 and 100
- reliability_score between 0 and 100

For money/volume:
- amounts and volumes should be >= 0 where appropriate

### 26.2 Event integrity checks

Require:
- event_hash not null
- aggregate_seq > 0
- event_version > 0
- payload_schema_version > 0

### 26.3 Audit integrity checks

Require:
- payload_hash not null
- record_hash not null
- for non-first chain records prev_audit_hash should usually be present at application layer
- seal range consistency handled in application plus query verification

---

## 27. Query Patterns the Schema Must Support

The schema must efficiently support these reads:

1. Aggregate timeline reconstruction
   - event_store by workspace_id + aggregate_type + aggregate_id ordered by aggregate_seq

2. Full trace reconstruction
   - event_store + audit_chain + failed_transition_records + tasks by trace_id

3. Audit verification history
   - audit_seals and audit_verification_runs by workspace and recent time

4. Current state reads
   - aggregate_state_snapshots
   - fall back to latest event if snapshot absent

5. Loss attribution analysis
   - deals + deal_outcomes + exceptions + settlements + decision_logs + reason code links

6. Counterparty trust explanation
   - trust_profiles + trust_score_history + settlements + exceptions

7. Quote/deal progression
   - rfqs + quotes + negotiations + negotiation_events + deals

---

## 28. Example DDL Sketches

The following are abbreviated examples, not the full final migration code.

### 28.1 Utility function sketch

```sql
create extension if not exists pgcrypto;
create extension if not exists citext;
create extension if not exists pg_trgm;

create or replace function set_updated_at()
returns trigger as $$
begin
  new.updated_at = now();
  return new;
end;
$$ language plpgsql;

create or replace function prevent_update_delete()
returns trigger as $$
begin
  raise exception 'append-only table: update/delete is not allowed';
end;
$$ language plpgsql;
```

### 28.2 event_store sketch

```sql
create table event_store (
  id bigserial primary key,
  event_id uuid not null unique,
  workspace_id uuid not null references workspaces(id),
  aggregate_type text not null,
  aggregate_id uuid not null,
  aggregate_seq bigint not null,
  event_type text not null,
  event_version integer not null check (event_version > 0),
  payload_schema_version integer not null check (payload_schema_version > 0),
  trace_id uuid null,
  span_id uuid null,
  parent_span_id uuid null,
  correlation_id uuid null,
  causation_event_id uuid null,
  actor_type text not null,
  actor_id uuid null,
  source_service text not null,
  occurred_at timestamptz not null,
  ingested_at timestamptz not null default now(),
  state_before text null,
  state_after text null,
  reason_code_primary text null,
  reason_codes_secondary jsonb not null default '[]'::jsonb,
  payload_json jsonb not null,
  is_material boolean not null default false,
  audit_required boolean not null default false,
  idempotency_key text null,
  prev_event_hash text null,
  event_hash text not null,
  signature text null,
  created_at timestamptz not null default now(),
  unique (workspace_id, aggregate_type, aggregate_id, aggregate_seq)
);

create unique index uq_event_store_idempotency
  on event_store (workspace_id, idempotency_key)
  where idempotency_key is not null;

create index ix_event_store_aggregate_timeline
  on event_store (workspace_id, aggregate_type, aggregate_id, aggregate_seq);

create index ix_event_store_trace
  on event_store (trace_id);
```

### 28.3 audit_chain sketch

```sql
create table audit_chain (
  id bigserial primary key,
  audit_id uuid not null unique,
  workspace_id uuid not null references workspaces(id),
  entity_type text not null,
  entity_id uuid not null,
  action_type text not null,
  source_event_id uuid null,
  trace_id uuid null,
  actor_type text not null,
  actor_id uuid null,
  occurred_at timestamptz not null,
  payload_hash text not null,
  prev_audit_hash text null,
  record_hash text not null unique,
  signature text null,
  seal_batch_id uuid null,
  verification_status text null,
  metadata_json jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now()
);

create index ix_audit_chain_entity_timeline
  on audit_chain (workspace_id, entity_type, entity_id, id);

create index ix_audit_chain_trace
  on audit_chain (trace_id);
```

---

## 29. Trigger Attachment Examples

### 29.1 updated_at example

```sql
create trigger trg_users_updated_at
before update on users
for each row execute function set_updated_at();
```

### 29.2 append-only example

```sql
create trigger trg_event_store_append_only
before update or delete on event_store
for each row execute function prevent_update_delete();

create trigger trg_audit_chain_append_only
before update or delete on audit_chain
for each row execute function prevent_update_delete();
```

---

## 30. Migration Safety for ZEABUR

### 30.1 Startup rule

Backend application startup must not accept write traffic before:
- migrations complete successfully
- seed data required by write validation exists
- append-only guards are attached
- required indexes exist

### 30.2 Practical deployment rule

Recommended service startup approach:
- migration job or release command runs first
- backend service starts only after migration success
- frontend should tolerate backend warming period

### 30.3 Backward-compatible change rule

Future schema changes must prefer:
- additive columns
- additive tables
- new reason codes
- new transition rules
- new indexes
- non-destructive backfills

Avoid:
- renaming old states with direct historical reinterpretation
- deleting old reason codes in use
- destructive modification of append-only tables

---

## 31. Design Completion Assessment After This Deliverable

Relative to the project standard where 100% means the documents alone enable fully independent implementation of a working ZEABUR-compatible repo:

Estimated completion after this document:
- approximately 93%

Reason:
This document closes most database-level ambiguity, defines table structure, keys, indexes, migration order, append-only rules, and implementation policies. Remaining major gaps toward 100% are now concentrated in:
- Rust API Contract Spec
- ZEABUR Deployment & Repo Blueprint

---

## 32. Final Statement

This PostgreSQL Schema & Migration Spec establishes the database backbone for AegisBroker’s event-driven, state-controlled, trace-correlated, reason-coded, tamper-evident operating model.

From this point forward:
- all new domain features must map to the event store model
- all material transitions must respect state machine support tables
- all important explanations must align to the reason code registry
- all integrity-sensitive records must write through the audit chain
- all ZEABUR deployment planning must assume this migration order and startup dependency structure

This document is therefore the database implementation blueprint that converts the control architecture into a buildable repository foundation.
