# AegisBroker — Rust API Contract Spec
Version: v1.0  
Status: Blueprint-ready / Repo-ready API baseline  
Target deployment: ZEABUR  
Primary backend stack: Rust + Axum + SQLx + Tokio + OpenTelemetry  
Related documents:
- Core Event & Audit Architecture Spec & WorkItem List
- PostgreSQL Schema & Migration Spec

This document defines the Rust backend API contract for AegisBroker. It converts the architecture and database design into an implementation-facing API blueprint that is specific enough for repository construction, endpoint development, validation policy, error handling, trace propagation, idempotent writes, and control-framework compliance.

---

## 0. Document Purpose

This document exists to define the Rust API surface and behavior rules required to implement AegisBroker as an event-driven, state-controlled, trace-correlated, reason-coded, tamper-evident broker operating system.

Its purposes are:

1. Define the API boundary between frontend and Rust backend.
2. Establish endpoint groups, DTO direction, validation rules, and mutation behavior.
3. Make event emission, state transition checks, audit writing, and trace propagation mandatory parts of write paths.
4. Specify idempotency, error semantics, and control-framework obligations.
5. Keep the API design directly compatible with ZEABUR deployment and the PostgreSQL schema baseline.

This is not sample API guidance. It is the working contract blueprint for backend implementation.

---

## 1. API Design Principles

### 1.1 Core principle

Every write endpoint that creates, changes, validates, rejects, escalates, or completes a materially important business action must:

- validate identity and authorization
- validate input shape and semantic conditions
- load current aggregate state where applicable
- enforce state machine transition rules where applicable
- validate reason code requirements where applicable
- write business record changes and canonical event records atomically or via approved durable outbox pattern
- write audit records where required
- preserve trace continuity
- return structured error codes on failure

### 1.2 API style

Recommended API style:
- RESTful JSON API
- versioned under `/api/v1`
- resource-oriented reads
- command-oriented writes where state transitions matter

Examples:
- `POST /api/v1/rfqs`
- `POST /api/v1/quotes/{quote_id}/send`
- `POST /api/v1/deals/{deal_id}/confirm`
- `POST /api/v1/audit/verify`

### 1.3 Data ownership principle

The Rust backend is the source of truth for:
- authorization decisions
- business validation
- state transitions
- event emission
- audit emission
- trace context
- idempotency enforcement

The frontend must not be relied on for enforcing control integrity.

---

## 2. Recommended Rust Service Structure

Recommended service layout for the backend repository:

```text
backend-rust/
├─ src/
│  ├─ main.rs
│  ├─ app/
│  ├─ config/
│  ├─ auth/
│  ├─ middleware/
│  ├─ api/
│  │  ├─ health.rs
│  │  ├─ auth.rs
│  │  ├─ counterparties.rs
│  │  ├─ rfqs.rs
│  │  ├─ quotes.rs
│  │  ├─ negotiations.rs
│  │  ├─ deals.rs
│  │  ├─ shipments.rs
│  │  ├─ inspections.rs
│  │  ├─ exceptions.rs
│  │  ├─ settlements.rs
│  │  ├─ tasks.rs
│  │  ├─ reasons.rs
│  │  ├─ audit.rs
│  │  └─ replay.rs
│  ├─ domain/
│  │  ├─ events/
│  │  ├─ state_machine/
│  │  ├─ audit/
│  │  ├─ reason_codes/
│  │  └─ tracing/
│  ├─ models/
│  ├─ dto/
│  ├─ services/
│  ├─ repositories/
│  ├─ errors/
│  └─ telemetry/
```

This document assumes a modular monolith baseline that is service-split-ready later.

---

## 3. API Versioning and Route Structure

### 3.1 Versioning

All stable endpoints should be prefixed with:
- `/api/v1`

### 3.2 Route groups

Recommended route groups:

- `/health`
- `/api/v1/auth`
- `/api/v1/me`
- `/api/v1/workspaces`
- `/api/v1/counterparties`
- `/api/v1/contacts`
- `/api/v1/rfqs`
- `/api/v1/quotes`
- `/api/v1/negotiations`
- `/api/v1/deals`
- `/api/v1/shipments`
- `/api/v1/inspections`
- `/api/v1/exceptions`
- `/api/v1/settlements`
- `/api/v1/tasks`
- `/api/v1/reasons`
- `/api/v1/audit`
- `/api/v1/control`
- `/api/v1/replay`

### 3.3 Internal/admin split

Optional:
- `/api/v1/admin/...`

Use only for privileged actions such as:
- seeding reason codes
- managing transition rules
- running audit verification
- replay operations
- diagnostics

---

## 4. Common Request Context Requirements

Every authenticated request should be associated with:

- `workspace_id`
- `user_id`
- role set
- `trace_id`
- optional `span_id`
- request timestamp
- request id or correlation id optional

### 4.1 Trace propagation headers

Recommended support:
- `traceparent`
- `tracestate`
- `x-request-id`
- `x-idempotency-key`

### 4.2 Auth propagation

Recommended auth transport:
- HTTP-only session cookie or bearer token after OAuth login completion
- backend resolves session to workspace and user identity

### 4.3 Workspace scope

All business resource routes must be tenant-scoped through authenticated workspace context, not by trusting caller-provided workspace identifiers in body payloads.

---

## 5. Common Response Contract

### 5.1 Success response style

Recommended JSON response envelope for most endpoints:

```json
{
  "data": {},
  "meta": {
    "trace_id": "uuid",
    "request_id": "optional"
  }
}
```

For list endpoints:

```json
{
  "data": [],
  "meta": {
    "page": 1,
    "page_size": 50,
    "next_cursor": null,
    "trace_id": "uuid"
  }
}
```

### 5.2 Error response style

All errors should be structured.

Recommended envelope:

```json
{
  "error": {
    "code": "STATE_TRANSITION_INVALID",
    "message": "Quote cannot transition from GENERATED to ACCEPTED directly.",
    "details": {
      "aggregate_type": "quote",
      "aggregate_id": "uuid",
      "from_state": "GENERATED",
      "to_state": "ACCEPTED"
    }
  },
  "meta": {
    "trace_id": "uuid"
  }
}
```

### 5.3 Error transparency policy

The API must:
- expose stable machine-readable `code`
- expose human-readable `message`
- optionally expose safe diagnostic details
- never leak secrets or internal credentials
- keep trace_id visible for support and replay linkage

---

## 6. Common Error Code Families

Recommended error families:

### 6.1 Auth and access
- AUTH_REQUIRED
- AUTH_INVALID
- SESSION_EXPIRED
- FORBIDDEN
- ROLE_REQUIRED

### 6.2 Validation
- VALIDATION_ERROR
- FIELD_REQUIRED
- FIELD_INVALID
- REASON_CODE_INVALID
- REASON_CODE_REQUIRED
- IDEMPOTENCY_KEY_REQUIRED

### 6.3 Not found / conflict
- RESOURCE_NOT_FOUND
- RESOURCE_CONFLICT
- DUPLICATE_RESOURCE
- IDEMPOTENT_REPLAY_CONFLICT

### 6.4 State and control
- STATE_TRANSITION_INVALID
- STATE_GUARD_FAILED
- AGGREGATE_SNAPSHOT_MISSING
- OVERRIDE_JUSTIFICATION_REQUIRED
- CONTROL_POLICY_VIOLATION

### 6.5 Audit and verification
- AUDIT_WRITE_FAILED
- AUDIT_VERIFICATION_FAILED
- AUDIT_CHAIN_BROKEN

### 6.6 Infrastructure
- DATABASE_ERROR
- STORAGE_ERROR
- UPSTREAM_ERROR
- INTERNAL_ERROR

---

## 7. Idempotency Contract

### 7.1 Scope

The following write actions should support idempotency:
- create RFQ
- create quote
- send quote
- confirm deal
- create shipment
- resolve exception
- mark settlement received
- manual override commands
- audit verify commands where replaying is expensive or material

### 7.2 Transport

Recommended header:
- `X-Idempotency-Key`

### 7.3 Behavior

If an identical idempotent command is replayed:
- return the original logical result where possible
- do not duplicate material events
- do not duplicate audit writes
- preserve the same aggregate outcome

If the same idempotency key is reused with a meaningfully different payload:
- return `IDEMPOTENT_REPLAY_CONFLICT`

---

## 8. DTO Strategy

### 8.1 Separation

Use separate Rust types for:
- request DTOs
- response DTOs
- domain commands
- domain events
- repository models

Do not reuse DB models directly as public API DTOs.

### 8.2 Serialization

Use `serde` with:
- explicit field names
- optional fields carefully controlled
- unknown field rejection where command safety matters

### 8.3 Validation

Use:
- structural validation at DTO level
- semantic validation at service layer
- state validation at control layer

---

## 9. Core Middleware Contract

The backend should include middleware for:

- request tracing extraction/injection
- session/auth resolution
- workspace resolution
- request logging
- idempotency header capture
- panic/error normalization
- role-based access check hooks where needed

### 9.1 Trace middleware obligation

Every request must get a trace context, even if the caller did not provide one.

### 9.2 Error normalization middleware

All errors from handlers/services must be normalized into the standard error envelope.

---

## 10. Health and Meta Endpoints

### 10.1 GET /health/live

Purpose:
basic liveness

Response:
```json
{
  "data": {
    "status": "ok"
  },
  "meta": {
    "trace_id": "uuid"
  }
}
```

### 10.2 GET /health/ready

Purpose:
readiness including DB connectivity and migration readiness

Checks:
- database reachable
- critical migrations applied
- startup seed baseline present where required

### 10.3 GET /api/v1/me

Purpose:
return current user/session/workspace info

Response fields:
- user_id
- workspace_id
- email
- display_name
- roles

---

## 11. Auth API Contract

### 11.1 GET /api/v1/auth/google/start

Purpose:
start OAuth flow

Behavior:
- returns redirect URL or performs redirect
- generates anti-CSRF state and PKCE if applicable

### 11.2 GET /api/v1/auth/google/callback

Purpose:
handle OAuth callback

Behavior:
- validate state
- exchange code
- resolve/create user
- create session
- attach workspace
- redirect to frontend or return session result

### 11.3 POST /api/v1/auth/logout

Behavior:
- invalidate session
- emit session logout audit if desired
- return success envelope

---

## 12. Counterparty and Contact API Contract

### 12.1 POST /api/v1/counterparties

Purpose:
create counterparty

Request:
```json
{
  "counterparty_code": "CP-001",
  "name": "Example Buyer Ltd",
  "counterparty_type": "buyer",
  "region": "APAC",
  "country_code": "TW",
  "notes": "Strategic buyer"
}
```

Behavior:
- validate unique code per workspace
- create record
- may emit non-material creation event or material event based on policy
- return created resource

### 12.2 GET /api/v1/counterparties

Supports:
- type filter
- status filter
- search query
- cursor or offset pagination

### 12.3 GET /api/v1/counterparties/{id}

Returns:
- base profile
- contacts summary
- trust profile summary optional
- open tasks optional

### 12.4 POST /api/v1/contacts

Creates contact under counterparty.

---

## 13. RFQ API Contract

### 13.1 POST /api/v1/rfqs

Purpose:
create new RFQ

Request:
```json
{
  "rfq_number": "RFQ-2026-0001",
  "requester_counterparty_id": "uuid",
  "commodity": "coffee",
  "spec": {},
  "requested_volume": 100.0,
  "requested_unit": "bags",
  "target_price": 180.5,
  "target_currency": "USD",
  "urgency": "high",
  "source_channel": "email",
  "received_at": "2026-04-08T10:00:00Z"
}
```

Behavior:
- requires idempotency support
- initial state = RECEIVED
- emits `rfq.received`
- may write audit if policy treats RFQ receipt as material
- creates aggregate snapshot or state row

Response returns:
- RFQ resource
- current state
- latest event id optional

### 13.2 POST /api/v1/rfqs/{rfq_id}/qualify

Purpose:
transition RFQ from RECEIVED to QUALIFIED

Request:
```json
{
  "primary_reason_code": "Q-ELIG-001",
  "explanation_text": "Qualified for pricing review",
  "override": false
}
```

Behavior:
- load current state
- validate transition RECEIVED -> QUALIFIED
- validate reason code policy if required
- emit `rfq.qualified`
- write audit if material

### 13.3 POST /api/v1/rfqs/{rfq_id}/reject

Purpose:
transition RFQ to REJECTED

Requires:
- primary_reason_code
- explanation_text optional but recommended

### 13.4 GET /api/v1/rfqs/{rfq_id}/timeline

Purpose:
return aggregate timeline

Response includes:
- key business fields
- current state
- event timeline summary
- audit summary optional

---

## 14. Quote API Contract

### 14.1 POST /api/v1/quotes

Purpose:
create draft/generated quote from RFQ or directly

Request:
```json
{
  "quote_number": "Q-2026-0001",
  "rfq_id": "uuid",
  "buyer_counterparty_id": "uuid",
  "supplier_counterparty_id": "uuid",
  "buy_side_price": 175.0,
  "sell_side_price": 180.5,
  "spread_amount": 5.5,
  "spread_currency": "USD",
  "confidence_score": 84.5,
  "validity_expires_at": "2026-04-08T12:00:00Z",
  "pricing_snapshot": {},
  "primary_reason_code": "Q-PRICE-002"
}
```

Behavior:
- requires idempotency support
- create quote in state GENERATED or DRAFT depending on implementation mode
- write `quote.generated`
- if linked RFQ should update RFQ state under controlled transaction
- write audit if quote generation is material

### 14.2 POST /api/v1/quotes/{quote_id}/send

Purpose:
transition quote to SENT

Request:
```json
{
  "primary_reason_code": "Q-PRICE-001",
  "recipient_contact_ids": ["uuid"],
  "delivery_channel": "email",
  "override": false
}
```

Behavior:
- validate quote current state
- validate required fields and expiry
- transition GENERATED -> SENT
- emit `quote.sent`
- audit required
- create reminder workflow if expiry exists

### 14.3 POST /api/v1/quotes/{quote_id}/revise

Purpose:
create revised quote revision and event

Behavior:
- may retain same quote aggregate with revision event
- or create explicit version semantics inside quote
- must emit `quote.revised`
- audit based on policy

### 14.4 POST /api/v1/quotes/{quote_id}/expire

Purpose:
mark quote expired manually or via job

Must support:
- actor_type system or user
- event `quote.expired`

### 14.5 GET /api/v1/quotes/{quote_id}

Return:
- quote detail
- current state
- price snapshot summary
- linked RFQ/deal ids
- latest events summary
- active reason codes summary

### 14.6 GET /api/v1/quotes/{quote_id}/timeline

Return:
- event timeline
- negotiation links
- audit entries summary
- state changes

---

## 15. Negotiation API Contract

### 15.1 POST /api/v1/negotiations

Purpose:
open negotiation thread for quote

Request:
```json
{
  "quote_id": "uuid"
}
```

Behavior:
- creates negotiation record
- emits non-material or material start event per policy

### 15.2 POST /api/v1/negotiations/{negotiation_id}/events

Purpose:
append negotiation event

Request:
```json
{
  "action_type": "counter",
  "actor_type": "user",
  "price_value": 179.0,
  "price_currency": "USD",
  "terms": {
    "payment_terms": "LC at sight"
  },
  "sentiment": "firm",
  "primary_reason_code": "N-WIN-001"
}
```

Behavior:
- append negotiation event
- optionally update negotiation state
- emit canonical negotiation event
- if accept/reject action causes quote or deal transition, invoke state control
- audit if material

### 15.3 GET /api/v1/negotiations/{negotiation_id}

Return:
- negotiation summary
- current state
- related quote/deal
- ordered events

---

## 16. Deal API Contract

### 16.1 POST /api/v1/deals

Purpose:
create pending deal from quote/negotiation

Request:
```json
{
  "deal_number": "D-2026-0001",
  "quote_id": "uuid",
  "buyer_counterparty_id": "uuid",
  "supplier_counterparty_id": "uuid",
  "commodity": "coffee",
  "agreed_price": 179.0,
  "price_currency": "USD",
  "agreed_volume": 100.0,
  "volume_unit": "bags",
  "incoterm": "FOB"
}
```

Behavior:
- create in PENDING_CONFIRMATION
- emit `deal.pending_confirmation`
- may transition quote to ACCEPTED if tied to final negotiation action
- audit required

### 16.2 POST /api/v1/deals/{deal_id}/confirm

Purpose:
transition deal to CONFIRMED

Request:
```json
{
  "primary_reason_code": "D-WIN-001",
  "override": false
}
```

Behavior:
- validate state transition
- emit `deal.confirmed`
- audit required

### 16.3 POST /api/v1/deals/{deal_id}/fail

Purpose:
mark deal failed

Requires:
- primary_reason_code
- optional explanation
- state transition to FAILED
- outcome record creation/update
- audit required

### 16.4 POST /api/v1/deals/{deal_id}/complete

Purpose:
mark deal completed when execution and settlement are complete enough by policy

Must validate:
- required execution/settlement conditions
- or explicit override with audit

### 16.5 GET /api/v1/deals/{deal_id}

Return:
- deal details
- current state
- linked quote/shipment/settlement
- outcome summary
- reason summary

### 16.6 GET /api/v1/deals/{deal_id}/timeline

Return:
- aggregate event timeline
- linked execution and settlement highlights
- audit highlights

---

## 17. Shipment API Contract

### 17.1 POST /api/v1/shipments

Purpose:
create shipment linked to deal

Request:
```json
{
  "shipment_number": "SHP-2026-0001",
  "deal_id": "uuid",
  "origin_counterparty_id": "uuid",
  "destination_counterparty_id": "uuid",
  "etd": "2026-04-10T00:00:00Z",
  "eta": "2026-04-18T00:00:00Z",
  "shipment_context": {}
}
```

Behavior:
- create shipment in CREATED
- emit `shipment.created`
- audit if policy marks as material

### 17.2 POST /api/v1/shipments/{shipment_id}/book

Transition:
- CREATED -> BOOKED

### 17.3 POST /api/v1/shipments/{shipment_id}/depart

Transition:
- BOOKED -> IN_TRANSIT

### 17.4 POST /api/v1/shipments/{shipment_id}/delay

Purpose:
mark delay

Request:
```json
{
  "primary_reason_code": "E-FAIL-001",
  "new_eta": "2026-04-20T00:00:00Z",
  "summary_text": "Port congestion"
}
```

Behavior:
- shipment state update
- emit `shipment.delayed`
- create or link exception
- audit required
- create workflow task

### 17.5 POST /api/v1/shipments/{shipment_id}/deliver

Transition:
- IN_TRANSIT or DELAYED -> DELIVERED

### 17.6 GET /api/v1/shipments/{shipment_id}

Return:
- shipment state
- timestamps
- exceptions summary
- inspections summary

---

## 18. Inspection API Contract

### 18.1 POST /api/v1/inspections

Create inspection.

### 18.2 POST /api/v1/inspections/{inspection_id}/pass

Requires:
- result payload
- state validation
- event emission

### 18.3 POST /api/v1/inspections/{inspection_id}/fail

Requires:
- primary_reason_code
- result payload
- exception creation as needed
- audit required

---

## 19. Exception API Contract

### 19.1 POST /api/v1/exceptions

Create exception against aggregate.

Request:
```json
{
  "exception_number": "EX-2026-0001",
  "aggregate_type": "shipment",
  "aggregate_id": "uuid",
  "exception_type": "delay",
  "severity": "high",
  "summary_text": "Delay due to customs hold"
}
```

Behavior:
- create OPEN exception
- emit `exception.opened`
- audit required if material

### 19.2 POST /api/v1/exceptions/{exception_id}/triage

### 19.3 POST /api/v1/exceptions/{exception_id}/escalate

Requires:
- reason code or severity change
- event emission
- audit required

### 19.4 POST /api/v1/exceptions/{exception_id}/resolve

Requires:
- primary_reason_code
- resolution summary
- state transition
- emits `exception.resolved`

---

## 20. Settlement API Contract

### 20.1 POST /api/v1/settlements

Create settlement record for deal.

### 20.2 POST /api/v1/settlements/{settlement_id}/mark-due

Purpose:
transition to DUE

### 20.3 POST /api/v1/settlements/{settlement_id}/receive-payment

Request:
```json
{
  "amount_paid": 17900.0,
  "currency": "USD",
  "paid_at": "2026-04-25T00:00:00Z",
  "primary_reason_code": "S-CLOSE-001"
}
```

Behavior:
- update amount paid
- transition state PARTIAL or PAID depending on amounts
- emit settlement event
- update trust profile if policy requires
- audit required

### 20.4 POST /api/v1/settlements/{settlement_id}/mark-overdue

Can be called by scheduled job or user.

### 20.5 POST /api/v1/settlements/{settlement_id}/dispute

Requires:
- reason code
- summary text
- audit required

### 20.6 GET /api/v1/settlements/{settlement_id}

Return:
- settlement status
- due/paid/dispute info
- trust impact summary optional

---

## 21. Task API Contract

### 21.1 GET /api/v1/tasks

Filters:
- status
- assigned_user_id
- source_aggregate_type
- due_before

### 21.2 POST /api/v1/tasks

Manual task creation.

### 21.3 POST /api/v1/tasks/{task_id}/start

### 21.4 POST /api/v1/tasks/{task_id}/complete

Task events should preserve trace_id where available.

---

## 22. Reason Code API Contract

### 22.1 GET /api/v1/reasons

Filters:
- domain
- category
- status

### 22.2 GET /api/v1/reasons/{code}

Return full reason metadata.

### 22.3 POST /api/v1/admin/reasons

Admin-only create reason code.

### 22.4 POST /api/v1/admin/reasons/{code}/deprecate

Admin-only update lifecycle.

---

## 23. Audit API Contract

### 23.1 GET /api/v1/audit/entities/{entity_type}/{entity_id}

Return:
- audit chain records for entity
- pagination
- trace correlation summary

### 23.2 POST /api/v1/audit/verify/entity

Request:
```json
{
  "entity_type": "quote",
  "entity_id": "uuid"
}
```

Behavior:
- verify contiguous audit chain for entity
- persist verification run
- return result summary

### 23.3 POST /api/v1/audit/verify/seal

Request:
```json
{
  "seal_batch_id": "uuid"
}
```

Returns:
- passed/failed/partial
- first broken row if failed
- trace_id

### 23.4 GET /api/v1/audit/seals

List recent seals.

---

## 24. Control and Override API Contract

### 24.1 POST /api/v1/control/override

Purpose:
perform explicit human override for guarded action

Request:
```json
{
  "aggregate_type": "quote",
  "aggregate_id": "uuid",
  "intended_action": "send",
  "intended_to_state": "SENT",
  "primary_reason_code": "A-OVRD-001",
  "justification_text": "Approved by senior broker due to strategic client commitment"
}
```

Behavior:
- requires privileged role
- justification mandatory
- reason code mandatory
- audit required
- emits `human.override.recorded`
- may then allow guarded action under explicit override flow

Error if missing justification:
- OVERRIDE_JUSTIFICATION_REQUIRED

### 24.2 GET /api/v1/control/transitions/{aggregate_type}

Admin or internal-only.
Returns transition rule matrix currently loaded.

---

## 25. Replay and Reconstruction API Contract

This is initial contract scaffolding, even if full replay UI comes later.

### 25.1 GET /api/v1/replay/aggregates/{aggregate_type}/{aggregate_id}

Returns:
- aggregate timeline
- reconstructed state chain
- event sequence
- audit summary
- linked aggregates summary

### 25.2 GET /api/v1/replay/traces/{trace_id}

Returns:
- cross-aggregate view of events, audit records, failed transitions, and tasks

### 25.3 POST /api/v1/replay/root-cause

Request:
```json
{
  "deal_id": "uuid"
}
```

Behavior:
- returns a structured skeleton for root-cause analysis
- initially may be deterministic rules-driven
- later can incorporate attribution engine

---

## 26. Event and Audit Obligations by Endpoint Category

### 26.1 Mandatory event emission endpoints

The following endpoint classes must emit canonical events:
- RFQ create/qualify/reject
- quote create/send/revise/expire
- negotiation material event append
- deal create/confirm/fail/complete/cancel
- shipment create/book/depart/delay/deliver
- inspection pass/fail/waive
- exception open/triage/escalate/resolve
- settlement due/receive/dispute/overdue/close
- trust score changes
- manual overrides

### 26.2 Mandatory audit endpoints/actions

The following actions must write audit records:
- quote send
- deal confirm
- deal fail
- shipment delay
- inspection fail
- exception escalate
- settlement dispute
- settlement overdue if material
- trust score change if policy says material
- any manual override
- audit verification runs
- significant rejected transition attempts where policy says auditable

---

## 27. Service-layer Command Pattern

Recommended pattern for write endpoints:

handler -> request DTO -> service command -> control validation -> repo transaction -> event write -> audit write -> response DTO

Command examples:
- `CreateRfqCommand`
- `QualifyRfqCommand`
- `CreateQuoteCommand`
- `SendQuoteCommand`
- `AppendNegotiationEventCommand`
- `ConfirmDealCommand`
- `DelayShipmentCommand`
- `ReceiveSettlementCommand`
- `OverrideActionCommand`

Commands should be explicit and narrower than generic patch endpoints.

---

## 28. Transaction Boundary Policy

### 28.1 Strong consistency for material writes

For material mutations, the API should prefer a single transaction that writes:
- business record change
- event_store row
- aggregate snapshot/state update
- audit_chain row if required

If the exact pattern is not feasible in one transaction, use a durable outbox design with strict replay guarantees.  
For initial modular monolith ZEABUR baseline, a single PostgreSQL transaction is preferred.

### 28.2 Transaction failure behavior

If event or audit write required by policy fails:
- reject the request
- roll back business mutation
- return structured error

Material state changes must not commit without required control records.

---

## 29. Authorization Policy

### 29.1 Role examples

Roles:
- owner
- admin
- broker
- ops
- viewer
- risk_reviewer

### 29.2 Example permissions

- broker: create RFQ, create quote, send quote, append negotiation events, create deals
- ops: manage shipments, inspections, documents, settlement updates
- risk_reviewer: approve guarded actions, perform override, review exceptions
- viewer: read-only
- admin/owner: manage reasons, transitions, audit verify, override

### 29.3 Authorization enforcement

Authorization must happen server-side and should be separable from handlers via guards or service checks.

---

## 30. Validation Policy

### 30.1 Structural validation

Examples:
- required fields present
- numeric ranges valid
- timestamp formats valid
- enum-like strings in allowed set

### 30.2 Semantic validation

Examples:
- quote expiry must be after now or business-valid horizon
- settlement amount_paid cannot exceed policy-defined tolerance without override
- buyer and supplier cannot be invalid same-party configuration where forbidden
- reason code must exist and be active
- linked aggregate must belong to same workspace

### 30.3 State validation

Examples:
- cannot send quote from ACCEPTED state
- cannot confirm deal already FAILED
- cannot mark shipment delayed if already DELIVERED unless explicit exceptional flow exists

---

## 31. Pagination and Query Policy

### 31.1 Read endpoints

Support either cursor or offset pagination.  
Recommended for timeline-heavy resources:
- cursor pagination

### 31.2 Filtering

Read endpoints should support filtering fields aligned to indexed columns:
- state
- counterparty_id
- due range
- created_at/occurred_at range
- event type
- trace_id

### 31.3 Sorting

Default:
- most recent first for lists
- aggregate_seq ascending for timelines
- occurred_at ascending when reconstructing flows

---

## 32. OpenTelemetry and Trace Contract in Rust

### 32.1 Instrumentation requirement

All material handlers and service commands should be instrumented with tracing spans.

Examples:
- `rfq.create`
- `rfq.qualify`
- `quote.send`
- `deal.confirm`
- `shipment.delay`
- `settlement.receive_payment`
- `audit.verify_entity`

### 32.2 Trace persistence

Where required by schema:
- persist trace_id
- persist span_id where relevant
- preserve parent_span_id where available

### 32.3 Log structure

Logs should include:
- trace_id
- operation_name
- aggregate_type
- aggregate_id
- status
- error_code if any

---

## 33. Example Rust Type Direction

Below are illustrative shape directions, not full code.

### 33.1 Request DTO example

```rust
#[derive(serde::Deserialize)]
pub struct SendQuoteRequest {
    pub primary_reason_code: String,
    pub recipient_contact_ids: Vec<uuid::Uuid>,
    pub delivery_channel: String,
    pub override_flag: bool,
}
```

### 33.2 Response DTO example

```rust
#[derive(serde::Serialize)]
pub struct QuoteResponse {
    pub id: uuid::Uuid,
    pub quote_number: String,
    pub current_state: String,
    pub validity_expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

### 33.3 App error example direction

```rust
pub enum AppError {
    Validation { code: &'static str, message: String },
    Forbidden { code: &'static str, message: String },
    NotFound { code: &'static str, message: String },
    Conflict { code: &'static str, message: String },
    StateTransition { code: &'static str, message: String },
    Database { code: &'static str, message: String },
    Internal { code: &'static str, message: String },
}
```

### 33.4 Command example direction

```rust
pub struct ConfirmDealCommand {
    pub workspace_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub deal_id: uuid::Uuid,
    pub primary_reason_code: String,
    pub override_flag: bool,
    pub trace_id: uuid::Uuid,
}
```

---

## 34. Endpoint-to-Schema Alignment Rules

This document must align to the PostgreSQL spec as follows:

- every material command maps to event_store write
- every auditable material command maps to audit_chain write
- every stateful aggregate command uses state_transition_rules and/or snapshot validation
- every reason-bearing write validates against reason_code_registry
- every replay/timeline read uses aggregate timeline query patterns
- every trace-aware command persists trace_id where schema defines it

No endpoint should be designed in a way that bypasses those data-layer contracts.

---

## 35. Minimal Endpoint Set for First Usable Release

To keep the first deployable ZEABUR slice realistic, the initial must-have endpoint set is:

- GET /health/live
- GET /health/ready
- GET /api/v1/me
- GET /api/v1/auth/google/start
- GET /api/v1/auth/google/callback
- POST /api/v1/auth/logout

- POST /api/v1/counterparties
- GET /api/v1/counterparties
- GET /api/v1/counterparties/{id}

- POST /api/v1/rfqs
- POST /api/v1/rfqs/{rfq_id}/qualify
- POST /api/v1/rfqs/{rfq_id}/reject
- GET /api/v1/rfqs/{rfq_id}

- POST /api/v1/quotes
- POST /api/v1/quotes/{quote_id}/send
- POST /api/v1/quotes/{quote_id}/revise
- GET /api/v1/quotes/{quote_id}
- GET /api/v1/quotes/{quote_id}/timeline

- POST /api/v1/negotiations
- POST /api/v1/negotiations/{negotiation_id}/events
- GET /api/v1/negotiations/{negotiation_id}

- POST /api/v1/deals
- POST /api/v1/deals/{deal_id}/confirm
- POST /api/v1/deals/{deal_id}/fail
- GET /api/v1/deals/{deal_id}
- GET /api/v1/deals/{deal_id}/timeline

- POST /api/v1/shipments
- POST /api/v1/shipments/{shipment_id}/delay
- GET /api/v1/shipments/{shipment_id}

- POST /api/v1/exceptions
- POST /api/v1/exceptions/{exception_id}/resolve

- POST /api/v1/settlements
- POST /api/v1/settlements/{settlement_id}/receive-payment
- POST /api/v1/settlements/{settlement_id}/mark-overdue
- GET /api/v1/settlements/{settlement_id}

- GET /api/v1/reasons
- GET /api/v1/reasons/{code}

- POST /api/v1/control/override
- POST /api/v1/audit/verify/entity
- GET /api/v1/replay/aggregates/{aggregate_type}/{aggregate_id}

This set is sufficient to create the first meaningful control-compliant broker workflow.

---

## 36. Follow-on Work Items for Backend Implementation

### 36.1 API framework and middleware
- set up Axum router tree
- implement request context extractor
- implement auth/session middleware
- implement trace middleware
- implement idempotency middleware/helper
- implement standard error mapping

### 36.2 DTO and command layer
- create request/response DTO modules
- create command structs per material write
- create validation layer per endpoint family

### 36.3 Domain service layer
- implement RFQ service
- implement Quote service
- implement Negotiation service
- implement Deal service
- implement Shipment service
- implement Exception service
- implement Settlement service
- implement Audit verification service

### 36.4 Control framework integration
- implement state transition validator service
- implement reason code validator service
- implement event writer service
- implement audit writer service
- implement aggregate timeline reader service

### 36.5 Observability
- instrument material handlers with tracing
- wire trace ids into repository calls
- align logs with trace contract

---

## 37. Design Completion Assessment After This Deliverable

Relative to the project standard where 100% means the handed-over documents alone are enough for fully independent implementation of a working ZEABUR-compatible repo:

Estimated completion after this document:
- approximately 97%

Reason:
This document closes the main API-layer ambiguity:
- endpoint structure is defined
- write behavior and control obligations are explicit
- error semantics are defined
- idempotency policy is defined
- handler/service layering is clear
- schema alignment is explicit

The main remaining gap to 100% is now the ZEABUR Deployment & Repo Blueprint.

---

## 38. Final Statement

This Rust API Contract Spec establishes the backend implementation contract that turns AegisBroker from a database-and-architecture plan into a repository-buildable application design.

From this point forward, no write endpoint should be considered complete unless it:
- validates auth and workspace scope
- enforces state and control rules
- emits canonical events
- writes required audit records
- preserves trace context
- returns structured, machine-usable errors

This document therefore functions as the API blueprint binding the architecture spec and PostgreSQL schema into a concrete Rust backend implementation path.
