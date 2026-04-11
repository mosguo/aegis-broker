# AGENTS.md

## Project Identity

AegisBroker is an event-driven broker operating system for commodity brokerage.

Target stack:
- Rust backend
- Python frontend
- PostgreSQL
- Stripe-backed Payment Service
- immutable internal points ledger
- ZEABUR deployment target

This repository is not a generic CRUD app. It is a control-oriented system whose architecture is built around:
- event flow
- explicit state machines
- distributed tracing
- standardized reason codes
- tamper-evident audit records

All implementation decisions must preserve those properties.

---

## Source of Truth

Before changing architecture, database structure, API behavior, payment flow, or deployment assumptions, read these documents in `docs/design/`:

1. `Core_Event_Audit_Architecture_Spec_WorkItem_List.md`
2. `PostgreSQL_Schema_Migration_Spec.md`
3. `Rust_API_Contract_Spec.md`
4. `ZEABUR_Deployment_Repo_Blueprint.md`

These four files are the authoritative design baseline.

If code or existing files conflict with these documents, treat the documents as the source of truth unless the repository has a newer, explicitly versioned replacement approved by the project owner.

---

## Mission for Agents

When working in this repo, your job is to build or modify software that remains faithful to the documented control architecture.

Always optimize for:
- correctness over speed
- explicitness over hidden behavior
- additive, version-safe changes over destructive rewrites
- replayability and auditability over convenience
- deployability on ZEABUR
- production-oriented code structure

Do not simplify the system into a plain request -> DB write pattern when the specs require:
- state validation
- event emission
- audit write
- reason-code enforcement
- trace propagation
- idempotency

---

## Non-Negotiable System Rules

### 1. Material writes must not bypass the event layer
Any materially relevant create/update/transition must emit canonical events and persist them to `event_store` or the designated equivalent defined by the specs.

### 2. Material state transitions must not bypass state-machine validation
If an aggregate has defined states, the transition must be validated against declared transition rules before mutation is committed.

### 3. Auditable actions must not bypass the audit chain
Any operation marked materially important in the architecture must write a tamper-evident audit record.

### 4. Points must never mutate by direct balance overwrite alone
Internal points are ledger-backed. Every credit, debit, reversal, or adjustment must create immutable `point_ledger_entries`. A balance snapshot/checkpoint is allowed, but it is not the sole source of truth.

### 5. Stripe webhook handling must be verified and idempotent
Stripe webhook requests must:
- preserve the raw request body for signature verification
- verify the Stripe signature
- be idempotent by provider event identity
- avoid duplicate payment or ledger mutation

### 6. Prefer compensating entries over destructive edits
For financial, ledger, event, and audit records:
- do not delete history
- do not reinterpret old states in place
- prefer reversal, adjustment, additive migration, or compensating event patterns

### 7. Workspace scope must be enforced server-side
Do not trust user-supplied workspace identifiers in write flows. Resolve scope from authenticated session/context wherever the specs require.

### 8. Trace context must be preserved across material operations
All material operations should preserve or create `trace_id`, and propagate it across services, commands, events, audits, and background jobs when applicable.

---

## Build Order

Unless the user explicitly asks for a different sequence, follow this implementation order:

1. repository skeleton
2. migrations
3. backend health/readiness endpoints
4. config loading and env contract
5. auth/session skeleton
6. event store / audit chain / state-machine framework
7. RFQ / Quote / Deal core flow
8. PaymentOrder + Stripe integration
9. points ledger and debit/credit/reversal flows
10. replay / audit verify / reconciliation support
11. ZEABUR deployment hardening

Do not jump to advanced UI or convenience features before the control foundation exists.

---

## Required Repository Structure

Keep the repository aligned with this shape unless there is a documented reason to evolve it:

```text
aegisbroker/
├─ AGENTS.md
├─ README.md
├─ docs/
│  └─ architecture/
├─ infra/
├─ backend-rust/
├─ frontend-python/
└─ storage/
```

Expected ownership:
- `backend-rust/` owns business logic, state transitions, events, audits, payment integration, and points ledger logic
- `frontend-python/` owns UI and frontend orchestration
- `infra/` owns deployment, Docker, env templates, and migration orchestration
- `docs/` owns architectural truth

---

## Backend Implementation Rules

### Rust backend
Use the documented backend approach:
- Axum
- SQLx
- Tokio
- tracing / OpenTelemetry-compatible instrumentation
- structured error handling
- service-layer command pattern

### Handler pattern
Prefer:
handler -> request DTO -> service command -> validation -> DB transaction -> event write -> audit write -> response DTO

Do not place substantial business logic directly in HTTP handlers.

### DTO separation
Keep separate types for:
- request DTOs
- response DTOs
- domain commands
- domain events
- DB/repository models

Do not expose DB row structs directly as public API responses unless intentionally wrapped and documented.

### Error contract
Return structured machine-usable error codes.  
Do not return unstructured internal errors to clients.

### Idempotency
For retry-prone material writes, support idempotency using the project’s documented strategy.

---

## Database Implementation Rules

### Migrations
Schema changes must be implemented through explicit migrations.  
Do not manually patch production schema outside the migration system.

### Append-only tables
Treat these as append-only where the specs require it:
- event_store
- audit_chain
- audit_seals
- payment_webhook_events
- point_ledger_entries
- other immutable control/history tables defined in docs

Do not implement update/delete repository operations for those tables.

### Schema safety
Prefer:
- additive columns
- additive tables
- additive indexes
- new reason codes
- new transition rules

Avoid:
- destructive renames of historical semantics
- deleting codes or states that may already exist in history
- silent data reinterpretation

---

## Payment Service Rules

Payment is part of the core system, not a detached helper.

Implement Stripe-backed payment flows using the internal `PaymentOrder` aggregate.  
The internal system must remain authoritative for:
- payment state
- event emission
- audit records
- ledger effects
- reconciliation status

Recommended payment lifecycle:
1. create internal PaymentOrder
2. create Stripe PaymentIntent
3. persist Stripe identifiers internally
4. return frontend-facing payment initiation data
5. process Stripe webhook
6. update internal payment state
7. apply point credit only when internally valid
8. reconcile and verify

Never grant points based only on a frontend callback or unverified client claim.

---

## Internal Points Rules

The points service is an internal value ledger.

Supported operations:
- credit
- debit
- reversal
- adjustment
- service consumption linkage

Rules:
- every mutation must produce immutable ledger history
- insufficient balance must be handled explicitly
- reversal is preferred over deleting or rewriting old entries
- manual adjustments must follow role and audit rules from the specs

---

## Testing Expectations

When implementing or changing code, add or update tests where it matters.

Prioritize tests for:
- state transition validation
- invalid transition rejection
- idempotent material writes
- Stripe webhook duplicate handling
- payment success -> point credit flow
- insufficient balance debit rejection
- reversal logic
- audit-chain-required write paths
- migration correctness where practical

Do not add superficial tests that ignore the control framework.

---

## Logging and Observability Expectations

Use structured logs.  
Important logs should include, where applicable:
- trace_id
- operation name
- aggregate_type
- aggregate_id
- status
- error_code

Material operations should be instrumented consistently so replay and debugging are possible.

---

## ZEABUR Deployment Expectations

Assume ZEABUR as the deployment target.

All changes should remain compatible with:
- Rust backend deployment on ZEABUR
- Python frontend deployment on ZEABUR
- PostgreSQL-backed runtime
- health-check based rollout
- environment-variable based secret/config management

Do not introduce assumptions that require a radically different platform unless explicitly requested.

Keep startup safe:
- migrations first
- readiness must fail if required dependencies/config are missing
- app should not accept write traffic before migration/seed prerequisites are met

---

## Documentation Expectations

If you make structural changes, update the relevant documents or clearly note the mismatch.

At minimum, document:
- new env vars
- new commands
- new migration dependencies
- new service responsibilities
- any deliberate deviation from the original architecture

Do not silently diverge from the blueprint.

---

## How to Handle Unclear Cases

If a requested change is ambiguous:
1. first look at the four architecture documents
2. prefer the option that preserves event/state/audit integrity
3. prefer additive changes over destructive ones
4. leave short explanatory notes in code or docs if a tradeoff was necessary

If the request conflicts with the control framework, do not silently implement a shortcut. Instead, preserve the architecture and surface the issue clearly.

---

## What Not to Do

Do not:
- turn material flows into plain CRUD
- bypass event_store for convenience
- bypass audit_chain for convenience
- mutate points without ledger entries
- trust Stripe success without verified webhook or approved reconciliation path
- hardcode workspace scope from user input
- introduce destructive schema rewrites when additive evolution works
- replace structured errors with generic strings
- place all logic in routes/controllers
- skip readiness checks for services that depend on DB/config

---

## Preferred First Task When Starting Fresh

If the repo is still skeletal, start with:
1. migration skeleton
2. backend-rust app skeleton
3. health/live and health/ready endpoints
4. config loading
5. event_store / audit_chain / payment_orders / point_accounts / point_ledger_entries migrations
6. Dockerfile and env example files
7. auth/session baseline
8. then core domain endpoints

---

## Expected Output Quality

Code produced in this repository should be:
- production-oriented
- explicit
- reasonably modular
- traceable to the architecture docs
- safe for incremental extension
- suitable for ZEABUR deployment

Where there is a tradeoff between quick implementation and architectural integrity, choose architectural integrity.
