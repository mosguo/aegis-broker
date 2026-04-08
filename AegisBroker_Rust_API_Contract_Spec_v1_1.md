# AegisBroker — Rust API Contract Spec
Version: v1.1  
Status: Blueprint-ready / Repo-ready API baseline  
Target deployment: ZEABUR  
Primary backend stack: Rust + Axum + SQLx + Tokio + OpenTelemetry  
Related documents:
- Core Event & Audit Architecture Spec & WorkItem List
- PostgreSQL Schema & Migration Spec
Addendum in this revision: Stripe-backed Payment Service + internal points service

This document defines the Rust backend API contract for AegisBroker. It now includes the Payment Service interface for Stripe-backed payment flows and the internal add-point / deduct-point service, while preserving the event-driven, state-controlled, audit-first control framework.

---

## 0. Document Purpose

This document exists to define the Rust API surface and behavior rules required to implement AegisBroker as an event-driven, state-controlled, trace-correlated, reason-coded, tamper-evident broker operating system with monetization-safe payment and points support.

Its purposes are:

1. Define the API boundary between frontend and Rust backend.
2. Establish endpoint groups, DTO direction, validation rules, and mutation behavior.
3. Make event emission, state transition checks, audit writing, and trace propagation mandatory parts of write paths.
4. Specify idempotency, error semantics, payment callback behavior, and control-framework obligations.
5. Keep the API design directly compatible with ZEABUR deployment and the PostgreSQL schema baseline.

---

## 1. API Design Principles

Every write endpoint that creates, changes, validates, rejects, escalates, completes, charges, credits, debits, refunds, or reconciles a materially important business action must:

- validate identity and authorization
- validate input shape and semantic conditions
- load current aggregate state where applicable
- enforce state machine transition rules where applicable
- validate reason code requirements where applicable
- write business record changes and canonical event records atomically or via approved durable outbox pattern
- write audit records where required
- preserve trace continuity
- return structured error codes on failure

Payment and points endpoints are not exceptions. They are first-class controlled write paths.

---

## 2. API Style

Recommended:
- RESTful JSON API
- versioned under `/api/v1`
- resource-oriented reads
- command-oriented writes where state transitions matter

Examples:
- `POST /api/v1/payment-orders`
- `POST /api/v1/payment-orders/{payment_order_id}/create-intent`
- `POST /api/v1/payments/webhooks/stripe`
- `POST /api/v1/points/debit`
- `POST /api/v1/control/override`

---

## 3. Route Groups

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
- `/api/v1/payment-orders`
- `/api/v1/payments`
- `/api/v1/points`
- `/api/v1/tasks`
- `/api/v1/reasons`
- `/api/v1/audit`
- `/api/v1/control`
- `/api/v1/replay`

Admin/internal optional:
- `/api/v1/admin/...`

---

## 4. Common Request Context Requirements

Every authenticated request should be associated with:
- workspace_id
- user_id
- role set
- trace_id
- optional span_id
- request timestamp
- request id or correlation id optional

Trace propagation headers:
- `traceparent`
- `tracestate`
- `x-request-id`
- `x-idempotency-key`

Workspace scope:
- all business resource routes must be tenant-scoped through authenticated workspace context

Payment webhook route exception:
- webhook verification may precede workspace resolution, then workspace is resolved from payment metadata or internal lookup after signature verification

---

## 5. Common Response Contract

Success envelope:

```json
{
  "data": {},
  "meta": {
    "trace_id": "uuid",
    "request_id": "optional"
  }
}
```

Error envelope:

```json
{
  "error": {
    "code": "STATE_TRANSITION_INVALID",
    "message": "PaymentOrder cannot transition from FAILED to SUCCEEDED directly.",
    "details": {
      "aggregate_type": "payment_order",
      "aggregate_id": "uuid",
      "from_state": "FAILED",
      "to_state": "SUCCEEDED"
    }
  },
  "meta": {
    "trace_id": "uuid"
  }
}
```

---

## 6. Common Error Code Families

Auth and access:
- AUTH_REQUIRED
- AUTH_INVALID
- SESSION_EXPIRED
- FORBIDDEN
- ROLE_REQUIRED

Validation:
- VALIDATION_ERROR
- FIELD_REQUIRED
- FIELD_INVALID
- REASON_CODE_INVALID
- REASON_CODE_REQUIRED
- IDEMPOTENCY_KEY_REQUIRED

Not found / conflict:
- RESOURCE_NOT_FOUND
- RESOURCE_CONFLICT
- DUPLICATE_RESOURCE
- IDEMPOTENT_REPLAY_CONFLICT

State and control:
- STATE_TRANSITION_INVALID
- STATE_GUARD_FAILED
- AGGREGATE_SNAPSHOT_MISSING
- OVERRIDE_JUSTIFICATION_REQUIRED
- CONTROL_POLICY_VIOLATION

Payment and ledger:
- PAYMENT_GATEWAY_ERROR
- PAYMENT_WEBHOOK_INVALID
- PAYMENT_WEBHOOK_DUPLICATE
- PAYMENT_RECONCILIATION_FAILED
- POINTS_INSUFFICIENT_BALANCE
- POINTS_LEDGER_CONFLICT
- POINTS_REVERSAL_REQUIRED

Audit and verification:
- AUDIT_WRITE_FAILED
- AUDIT_VERIFICATION_FAILED
- AUDIT_CHAIN_BROKEN

Infrastructure:
- DATABASE_ERROR
- STORAGE_ERROR
- UPSTREAM_ERROR
- INTERNAL_ERROR

---

## 7. Idempotency Contract

Write actions that must support idempotency:
- create RFQ
- create quote
- send quote
- confirm deal
- create shipment
- resolve exception
- mark settlement received
- create payment order
- create Stripe payment intent
- process Stripe webhook
- credit points
- debit points
- manual override commands
- audit verify commands where replaying is expensive or material

Transport:
- `X-Idempotency-Key`

Behavior:
- replay with same payload returns original logical result
- do not duplicate material events
- do not duplicate audit writes
- do not duplicate point ledger entries
- same key with different payload returns `IDEMPOTENT_REPLAY_CONFLICT`

---

## 8. DTO Strategy

Use separate Rust types for:
- request DTOs
- response DTOs
- domain commands
- domain events
- repository models

Do not expose DB models directly.

Use `serde` with explicit field names and unknown field rejection where command safety matters.

---

## 9. Core Middleware Contract

Include middleware for:
- request tracing extraction/injection
- session/auth resolution
- workspace resolution
- request logging
- idempotency header capture
- panic/error normalization
- optional role-based access check hooks

Trace middleware obligation:
- every request gets a trace context

Webhook middleware obligation:
- Stripe webhook handler must preserve raw request body for signature verification before JSON mutation

---

## 10. Health and Meta Endpoints

### GET /health/live
Basic liveness.

### GET /health/ready
Checks:
- database reachable
- critical migrations applied
- required seed baseline present
- Stripe env vars present when payment module enabled

### GET /api/v1/me
Returns:
- user_id
- workspace_id
- email
- display_name
- roles
- point_account_summary optional

---

## 11. Auth API Contract

### GET /api/v1/auth/google/start
Start OAuth flow.

### GET /api/v1/auth/google/callback
Handle OAuth callback, resolve/create user, create session.

### POST /api/v1/auth/logout
Invalidate session.

---

## 12. Counterparty and Contact API Contract

### POST /api/v1/counterparties
Creates counterparty.

### GET /api/v1/counterparties
List with filters.

### GET /api/v1/counterparties/{id}
Return profile, contacts, trust summary.

### POST /api/v1/contacts
Create contact.

---

## 13. RFQ API Contract

### POST /api/v1/rfqs
Create RFQ, initial state RECEIVED, emits `rfq.received`.

### POST /api/v1/rfqs/{rfq_id}/qualify
Transition to QUALIFIED.

### POST /api/v1/rfqs/{rfq_id}/reject
Transition to REJECTED, requires reason code.

### GET /api/v1/rfqs/{rfq_id}/timeline
Returns event timeline.

---

## 14. Quote API Contract

### POST /api/v1/quotes
Create quote, emits `quote.generated`.

### POST /api/v1/quotes/{quote_id}/send
Transition to SENT, audit required.

### POST /api/v1/quotes/{quote_id}/revise
Emit `quote.revised`.

### POST /api/v1/quotes/{quote_id}/expire
Marks quote expired.

### GET /api/v1/quotes/{quote_id}
Returns detail.

### GET /api/v1/quotes/{quote_id}/timeline
Returns timeline.

---

## 15. Negotiation API Contract

### POST /api/v1/negotiations
Open negotiation thread.

### POST /api/v1/negotiations/{negotiation_id}/events
Append negotiation event.

### GET /api/v1/negotiations/{negotiation_id}
Return summary and ordered events.

---

## 16. Deal API Contract

### POST /api/v1/deals
Create pending deal.

### POST /api/v1/deals/{deal_id}/confirm
Confirm deal, audit required.

### POST /api/v1/deals/{deal_id}/fail
Fail deal, requires reason code.

### POST /api/v1/deals/{deal_id}/complete
Complete deal when policy allows.

### GET /api/v1/deals/{deal_id}
Return details.

### GET /api/v1/deals/{deal_id}/timeline
Return timeline.

---

## 17. Shipment, Inspection, Exception, Settlement API Contract

### POST /api/v1/shipments
Create shipment.

### POST /api/v1/shipments/{shipment_id}/delay
Mark delay, create exception/task as needed.

### GET /api/v1/shipments/{shipment_id}
Return shipment details.

### POST /api/v1/inspections
Create inspection.

### POST /api/v1/inspections/{inspection_id}/fail
Mark fail, requires reason code, audit required.

### POST /api/v1/exceptions
Create exception.

### POST /api/v1/exceptions/{exception_id}/resolve
Resolve exception.

### POST /api/v1/settlements
Create settlement.

### POST /api/v1/settlements/{settlement_id}/receive-payment
Mark settlement payment received.

### POST /api/v1/settlements/{settlement_id}/mark-overdue
Mark overdue.

### GET /api/v1/settlements/{settlement_id}
Return settlement details.

---

## 18. Payment Service API Contract

### 18.1 Purpose

Payment Service is a Rust backend service deployed on Zeabur. It integrates with Stripe and wraps the gateway lifecycle in AegisBroker’s own PaymentOrder aggregate so external payment success/failure, internal ledger crediting, and auditability remain under internal control.

### 18.2 Design baseline

Use Stripe PaymentIntent as the main payment lifecycle object for one payment session/order. Metadata should carry internal identifiers such as workspace_id and payment_order_id so webhook reconciliation can resolve the internal aggregate cleanly.

### 18.3 POST /api/v1/payment-orders

Purpose:
create internal payment order before gateway intent creation

Request:
```json
{
  "payment_order_number": "PO-2026-0001",
  "purpose_type": "wallet_topup",
  "amount_minor": 1000,
  "currency": "usd",
  "deal_id": null,
  "settlement_id": null,
  "points_credit_amount": 1000,
  "metadata": {
    "package_code": "points-1000"
  }
}
```

Behavior:
- requires idempotency support
- creates internal PaymentOrder in DRAFT
- emits payment order creation event
- does not yet mutate points

### 18.4 POST /api/v1/payment-orders/{payment_order_id}/create-intent

Purpose:
create Stripe PaymentIntent and move internal state forward

Behavior:
- validate PaymentOrder state DRAFT
- create PaymentIntent with amount and currency
- attach metadata with workspace_id and payment_order_id
- store stripe_payment_intent_id
- transition DRAFT -> INTENT_CREATED or AWAITING_CUSTOMER_ACTION depending on flow
- emit `payment.intent.created`
- audit if policy marks it material

Response:
```json
{
  "data": {
    "payment_order_id": "uuid",
    "current_state": "AWAITING_CUSTOMER_ACTION",
    "stripe_payment_intent_id": "pi_xxx",
    "client_secret": "pi_xxx_secret_xxx"
  },
  "meta": {
    "trace_id": "uuid"
  }
}
```

### 18.5 GET /api/v1/payment-orders/{payment_order_id}

Returns:
- internal payment order
- current state
- linked Stripe identifiers
- point credit amount
- latest audit summary

### 18.6 GET /api/v1/payment-orders/{payment_order_id}/timeline

Returns:
- payment lifecycle events
- webhook events summary
- audit entries
- ledger linkage summary

### 18.7 POST /api/v1/payment-orders/{payment_order_id}/cancel

Purpose:
cancel internal and gateway order where possible

### 18.8 POST /api/v1/payment-orders/{payment_order_id}/request-refund

Request:
```json
{
  "primary_reason_code": "P-REFD-001",
  "amount_minor": 1000,
  "justification_text": "Customer requested refund"
}
```

Behavior:
- create refund against Stripe when policy allows
- transition to REFUND_PENDING
- no destructive points deletion; use reversal path if points were already credited
- audit required

### 18.9 POST /api/v1/payments/webhooks/stripe

Purpose:
receive Stripe webhooks

Behavior:
- preserve raw body
- verify Stripe signature before processing
- insert payment_webhook_event idempotently
- map relevant event types to PaymentOrder transitions
- supported baseline event handling includes successful payment, payment failure, cancellation, and refund-related outcomes
- emit `payment.webhook.received` and verification outcome events
- if payment succeeded and points_credit_amount > 0, invoke points credit service inside controlled transaction boundary or approved compensation pattern
- return 2xx after successful durable processing decision

Invalid signature:
- return `PAYMENT_WEBHOOK_INVALID`

Duplicate provider_event_id:
- return success-compatible no-op with duplicate handling metadata, not a second mutation

### 18.10 POST /api/v1/payment-orders/{payment_order_id}/reconcile

Purpose:
run explicit internal reconciliation

Behavior:
- compare internal PaymentOrder state, Stripe status, and internal ledger effects
- emit `payment.reconciliation.completed`
- update state to RECONCILED where appropriate
- return reconciliation summary

---

## 19. Internal Points Service API Contract

### 19.1 Purpose

Internal points are the platform’s service value ledger. They support add-point and deduct-point actions for wallet top-up, package purchase, subscription credits, and consumption of broker tools or services.

### 19.2 Non-negotiable rule

Points are not updated by directly overwriting balance as the source of truth. Every material change must create an immutable point ledger entry. The current balance is derived/checkpointed from the ledger.

### 19.3 GET /api/v1/points/account

Returns current user point account:
```json
{
  "data": {
    "point_account_id": "uuid",
    "current_balance": 1000,
    "reserved_balance": 0,
    "status": "active",
    "last_ledger_seq": 15
  },
  "meta": {
    "trace_id": "uuid"
  }
}
```

### 19.4 GET /api/v1/points/ledger

List current user ledger history, paginated.

### 19.5 POST /api/v1/points/credit

Purpose:
credit points manually or from system-authorized non-payment flow

Request:
```json
{
  "user_id": "uuid",
  "amount": 100,
  "reason_code": "L-ADJ-001",
  "source_type": "manual",
  "source_id": null,
  "metadata": {
    "note": "Promotional grant"
  }
}
```

Behavior:
- admin/finance_admin only for manual credit
- create immutable ledger entry
- update point account checkpoint
- emit `points.credited`
- audit required for manual credit

### 19.6 POST /api/v1/points/debit

Purpose:
deduct points for internal service use

Request:
```json
{
  "user_id": "uuid",
  "amount": 20,
  "reason_code": "L-DEBT-001",
  "source_type": "service_usage",
  "source_id": "uuid",
  "service_code": "market_alert_premium",
  "metadata": {
    "units": 1
  }
}
```

Behavior:
- load point account
- reject if insufficient balance
- create immutable debit ledger entry
- update point account checkpoint
- record point_service_consumption if service-linked
- emit `points.debited`
- audit if policy marks service debit as material

Error on insufficient balance:
- `POINTS_INSUFFICIENT_BALANCE`

### 19.7 POST /api/v1/points/reverse

Purpose:
apply compensating reversal entry

Request:
```json
{
  "related_ledger_entry_id": "uuid",
  "reason_code": "L-RVSL-001",
  "metadata": {
    "note": "Compensation for failed downstream action"
  }
}
```

Behavior:
- create reversal entry
- no delete/update of original ledger entry
- audit required

### 19.8 POST /api/v1/points/consume-service

Purpose:
combined domain command for service use + ledger debit

Request:
```json
{
  "service_code": "ai_negotiation_summary",
  "amount": 50,
  "metadata": {
    "job_id": "uuid"
  }
}
```

Behavior:
- validates user balance
- debits points
- writes service consumption record
- returns updated balance and ledger linkage

---

## 20. Task API Contract

### GET /api/v1/tasks
Filters by status, assignee, source aggregate, due date.

### POST /api/v1/tasks
Manual task creation.

### POST /api/v1/tasks/{task_id}/complete
Task events preserve trace_id.

---

## 21. Reason Code API Contract

### GET /api/v1/reasons
Supports domain/category/status filters.

### GET /api/v1/reasons/{code}
Returns full reason metadata.

### POST /api/v1/admin/reasons
Admin-only create.

### POST /api/v1/admin/reasons/{code}/deprecate
Admin-only lifecycle change.

---

## 22. Audit API Contract

### GET /api/v1/audit/entities/{entity_type}/{entity_id}
Return audit chain records for entity.

### POST /api/v1/audit/verify/entity
Verify chain for entity.

### POST /api/v1/audit/verify/seal
Verify seal batch.

### GET /api/v1/audit/seals
List recent seals.

---

## 23. Control and Override API Contract

### POST /api/v1/control/override
Requires privileged role, reason code, and justification.
Applies to guarded business actions including payment and ledger adjustments.

### GET /api/v1/control/transitions/{aggregate_type}
Admin/internal-only transition matrix view.

---

## 24. Replay and Reconstruction API Contract

### GET /api/v1/replay/aggregates/{aggregate_type}/{aggregate_id}
Return timeline, reconstructed state chain, event sequence, audit summary.

### GET /api/v1/replay/traces/{trace_id}
Cross-aggregate view.

### POST /api/v1/replay/root-cause
Returns structured root-cause skeleton for a target deal or payment-linked outcome.

---

## 25. Event and Audit Obligations by Endpoint Category

Mandatory canonical event emission:
- RFQ create/qualify/reject
- quote create/send/revise/expire
- negotiation material append
- deal create/confirm/fail/complete/cancel
- shipment create/book/depart/delay/deliver
- inspection pass/fail/waive
- exception open/triage/escalate/resolve
- settlement due/receive/dispute/overdue/close
- payment order create / intent create / cancel / refund / reconcile
- Stripe webhook verified material outcomes
- point credit / debit / reverse / service consumption
- trust score changes
- manual overrides

Mandatory audit actions:
- quote send
- deal confirm/fail
- shipment delay
- inspection fail
- exception escalate
- settlement dispute
- payment success/failure/refund/reconcile if material
- manual point adjustments
- reversal entries
- any manual override
- audit verification runs

---

## 26. Service-layer Command Pattern

Recommended pattern:
handler -> request DTO -> service command -> control validation -> repo transaction -> event write -> audit write -> response DTO

New command examples:
- `CreatePaymentOrderCommand`
- `CreateStripePaymentIntentCommand`
- `ProcessStripeWebhookCommand`
- `ReconcilePaymentOrderCommand`
- `CreditPointsCommand`
- `DebitPointsCommand`
- `ReversePointsCommand`
- `ConsumeServiceWithPointsCommand`

---

## 27. Transaction Boundary Policy

For material mutations, prefer a single transaction that writes:
- business record change
- event_store row
- aggregate snapshot/state update
- audit_chain row if required
- point_ledger_entry where applicable

If not feasible, use a durable outbox/compensation pattern.  
For initial modular monolith ZEABUR baseline, a single PostgreSQL transaction is preferred whenever the external Stripe call is not inside the transaction window.

Recommended practical split:
- create external PaymentIntent first
- then open DB transaction to persist internal state mutation + event + audit
- webhook processing writes durable internal records idempotently
- point credit after successful webhook must be transactional with internal payment state update

---

## 28. Authorization Policy

Roles:
- owner
- admin
- broker
- ops
- viewer
- risk_reviewer
- finance_admin

Example permissions:
- broker: create RFQ, create/send quote, append negotiation events, create deals
- ops: manage shipments, inspections, documents, settlement updates
- risk_reviewer: perform overrides, review exceptions
- finance_admin: create payment orders for finance flows, process manual point credit/debit/reversal, verify reconciliation
- viewer: read-only
- admin/owner: manage reasons, transitions, audit verify, override

---

## 29. Validation Policy

Structural validation:
- required fields
- numeric ranges
- timestamp formats
- allowed string sets

Semantic validation:
- quote expiry valid
- settlement amount checks
- reason code exists and active
- linked aggregate in same workspace
- payment currency and amount consistent
- points credit amount non-negative
- payment success must not credit points twice
- point debit must not exceed available balance unless future overdraft mode is explicitly enabled

State validation:
- cannot send quote from ACCEPTED
- cannot confirm FAILED deal
- cannot move FAILED payment directly to SUCCEEDED without explicit reconciliation or compensation flow
- cannot reverse same ledger entry twice unless policy allows chained adjustments

---

## 30. Pagination and Query Policy

Use cursor or offset pagination.  
Recommended for timelines and ledgers:
- cursor pagination

Filters should align with indexed columns:
- state
- counterparty_id
- due range
- occurred_at range
- event type
- trace_id
- payment state
- user_id
- reason code
- service_code

---

## 31. OpenTelemetry and Trace Contract in Rust

All material handlers and service commands should be instrumented with spans, including:
- rfq.create
- quote.send
- deal.confirm
- shipment.delay
- settlement.receive_payment
- payment_order.create
- stripe.payment_intent.create
- stripe.webhook.process
- points.credit
- points.debit
- points.reverse
- payment.reconcile
- audit.verify_entity

Persist trace_id where schema defines it.  
Structured logs should include trace_id, operation_name, aggregate_type, aggregate_id, status, and error_code if any.

---

## 32. Stripe Integration Notes for Implementation

Recommended Rust crate baseline:
- `async-stripe`

Implementation expectations:
- create one Stripe PaymentIntent per payment order/session where applicable
- store Stripe identifiers on internal PaymentOrder
- use metadata to preserve internal linkage
- verify webhook signatures using the configured webhook secret
- keep webhook processing idempotent by provider_event_id
- do not grant points before internal success handling completes

---

## 33. Minimal Endpoint Set for First Usable Release

Must-have set now includes:

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

- POST /api/v1/payment-orders
- POST /api/v1/payment-orders/{payment_order_id}/create-intent
- POST /api/v1/payment-orders/{payment_order_id}/request-refund
- GET /api/v1/payment-orders/{payment_order_id}
- GET /api/v1/payment-orders/{payment_order_id}/timeline
- POST /api/v1/payments/webhooks/stripe
- POST /api/v1/payment-orders/{payment_order_id}/reconcile

- GET /api/v1/points/account
- GET /api/v1/points/ledger
- POST /api/v1/points/debit
- POST /api/v1/points/credit
- POST /api/v1/points/reverse
- POST /api/v1/points/consume-service

- GET /api/v1/reasons
- GET /api/v1/reasons/{code}

- POST /api/v1/control/override
- POST /api/v1/audit/verify/entity
- GET /api/v1/replay/aggregates/{aggregate_type}/{aggregate_id}

This set is sufficient for a first meaningful control-compliant broker workflow with payment and points support.

---

## 34. Follow-on Work Items for Backend Implementation

API framework and middleware:
- set up Axum router tree
- implement request context extractor
- implement auth/session middleware
- implement trace middleware
- implement idempotency middleware/helper
- implement standard error mapping
- implement raw-body webhook verification path

DTO and command layer:
- create DTO modules
- create command structs per material write
- create validation layer per endpoint family

Domain service layer:
- RFQ service
- Quote service
- Negotiation service
- Deal service
- Shipment service
- Exception service
- Settlement service
- Payment service
- Point ledger service
- Audit verification service

Control framework integration:
- state transition validator
- reason code validator
- event writer
- audit writer
- aggregate timeline reader
- payment reconciliation coordinator

Observability:
- instrument material handlers with tracing
- wire trace ids into repository calls
- align logs with trace contract

---

## 35. Design Completion Assessment After This Deliverable

This document closes the main API-layer ambiguity for business flows, payment flows, and points-ledger behavior. The main remaining gap to 100% is the ZEABUR Deployment & Repo Blueprint.

---

## 36. Final Statement

This Rust API Contract Spec establishes the backend implementation contract that turns AegisBroker from a database-and-architecture plan into a repository-buildable application design, including a Stripe-backed Payment Service and internal add-point / deduct-point capabilities.

From this point forward, no write endpoint should be considered complete unless it:
- validates auth and workspace scope
- enforces state and control rules
- emits canonical events
- writes required audit records
- preserves trace context
- returns structured, machine-usable errors
- handles payment and ledger idempotency correctly where applicable
