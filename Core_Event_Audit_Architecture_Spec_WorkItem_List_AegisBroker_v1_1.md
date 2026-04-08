# AegisBroker — Core Event & Audit Architecture Spec & WorkItem List
Version: v1.1  
Status: Blueprint-ready / Repo-ready design baseline  
Target deployment: ZEABUR  
System stack baseline: Python Frontend + Rust Backend + PostgreSQL + Object Storage  
Addendum in this revision: Stripe-backed Payment Service + internal points ledger

This document is an integrated, unsegmented planning file that formally merges the control framework into the main README direction, the Architecture Spec direction, and the software development worklist. This revision also adds the payment domain and the internal add-point / deduct-point model so payment, wallet value, and auditability fit the same event-driven control architecture.

---

## 0. Document Purpose

This document has four purposes at the same time:

1. Formally merge the control framework into the main system direction of AegisBroker.
2. Upgrade the overall design from module-level planning into a development blueprint layer.
3. Define the complete blueprint deliverable:
   Core Event & Audit Architecture Spec & WorkItem List.
4. Provide a directly actionable work item structure so engineering can start implementation without rethinking the system from first principles.

This document is intentionally written as one integrated planning file. It can later be split into:
- README sections
- architecture specification sections
- engineering work breakdown sections
- repository bootstrap documents

---

## 1. Formal Positioning Update

AegisBroker is formally defined as:

A broker-centered, event-driven, decision and execution operating system for commodity brokerage, built to preserve human broker judgment, trace every material action, support failure reconstruction, enable loss attribution, provide payment and value ledger integrity, and provide the foundation for expert trader behavior replication.

The platform is not designed merely to store records. It is designed to answer, after the fact and with evidentiary quality:

- What happened?
- Why did it happen?
- Who or what triggered it?
- What state changed?
- Was the transition valid?
- What was lost, delayed, missed, charged, credited, or gained?
- Can the system reconstruct the full causal chain?
- Can the system later approximate or replicate how a top trader would have acted?
- Can payment and point-balance changes be verified end to end?

---

## 2. Main README Direction

### 2.1 First-class system principle

Every commercially material operation in AegisBroker must be represented through the combination of:

- event flow
- state machine control
- distributed tracing
- standard reason codes
- tamper-evident audit records

This now includes:
- Stripe payment initiation and confirmation
- internal point credits
- internal point debits
- payment-to-points reconciliation
- refund / reversal compensation paths

### 2.2 Why this principle exists

Without this control framework, the platform can store data but cannot reliably support:

- failure backtracking
- loss attribution
- causal replay
- compliance-grade reconstruction
- expert behavior comparison
- AI explanation grounded in actual history
- payment and points reconciliation

### 2.3 System outcome expected from this principle

After this framework is implemented, AegisBroker should be able to support:

- transaction failure reverse verification during integrated simulation testing
- loss attribution and root-cause reconstruction during integrated simulation testing
- expert trader behavior replay and system comparison during operational simulation testing
- evidence-backed workflow validation in production-like runs
- tamper-evident audit review for important commercial, operational, payment, and value-ledger steps

---

## 3. Updated Architecture Positioning

The architecture should now be understood in the following layered form:

- L0 Identity and Access
- L1 Core Business Records
- L2 Decision Layer
- L3 Event Flow and State Machine Layer
- L4 Trace and Audit Layer
- L5 Market, Supply Chain, Pricing, Workflow, Payment Services
- L6 Replay, Attribution, Simulation, and AI Comparison Layer

### 3.1 Added payment domain role

The Payment Service is part of L5, but all payment-related state transitions must still conform to L3 and L4. Payment is not an external black box. It is an externally connected domain wrapped by internal state, event, audit, and points-ledger controls.

---

## 4. Scope Expansion Formally Added to Architecture Spec

The following items are part of the architecture specification:

1. Event-first modeling for every material business transition
2. Explicit state machine design for core aggregates
3. Trace propagation across all backend operations
4. Mandatory use of standard reason codes for decision and exception semantics
5. Tamper-evident audit chain for important events and overrides
6. Replayability as a design requirement
7. Attribution readiness as a design requirement
8. Expert replication readiness as a future testability requirement
9. Payment state capture and reconciliation as a design requirement
10. Points-ledger integrity as a design requirement

---

## 5. The Control Framework as a System Contract

The following combination is now a formal system contract.

### 5.1 Event Flow
Every materially meaningful system action must emit a canonical event.

### 5.2 State Machine
Every core aggregate must have explicit states, valid transitions, invalid transition handling, and transition side effects.

### 5.3 Distributed Tracing
Every request, workflow action, asynchronous job, and event-driven internal operation must be correlated through trace context.

### 5.4 Standard Reason Codes
Every important decision, rejection, exception, loss signal, completion explanation, override, payment adjustment, and points-ledger adjustment must be attached to one or more standardized reason codes.

### 5.5 Tamper-evident Audit
Every important action and state-changing operation must be captured in an append-only tamper-evident audit chain.

If any future feature is implemented without these five concerns where applicable, it should be considered architecturally incomplete.

---

## 6. Business Motivation Behind the Five-part Control Framework

### 6.1 Reverse verification
The platform must reconstruct:
- market conditions
- incoming RFQ or broker initiative
- decision taken
- quote produced
- counteroffer progression
- deal confirmation or rejection
- execution events
- settlement status
- payment initiation, external gateway outcome, and internal point balance effects

### 6.2 Loss attribution
The platform must later identify whether loss originated from:
- wrong price threshold
- slow response
- low confidence quote source
- rejected terms
- delayed shipment
- failed inspection
- incomplete documents
- payment delay or non-payment
- external gateway failure
- duplicate callback handling failure
- point deduction logic error
- human override error
- unreliable counterparty

### 6.3 Expert replication
A future simulation phase must compare:
- top trader decision path
- system recommendation path
- actual human-selected path
- payment and balance effects on final commercial outcome

### 6.4 Defensibility
If the platform supports enterprise clients or monetized broker features, the ability to verify that payment records, point credits, point debits, and overrides were not silently modified becomes commercially critical.

---

## 7. Canonical Event Architecture

### 7.1 Event-first principle

An event is the durable system statement that something materially relevant happened. Events are not simple logs and are not optional diagnostics. They are part of business truth reconstruction.

### 7.2 Event categories

#### Market events
- market.price.updated
- market.bar.closed
- market.spread.changed
- market.alert.triggered
- market.source.failed

#### RFQ and quote events
- rfq.received
- rfq.qualified
- rfq.rejected
- quote.generated
- quote.sent
- quote.revised
- quote.expired

#### Negotiation events
- negotiation.offer.received
- negotiation.counter.sent
- negotiation.accepted
- negotiation.rejected
- negotiation.timeout

#### Deal events
- deal.pending_confirmation
- deal.confirmed
- deal.cancelled
- deal.failed
- deal.completed

#### Execution and supply chain events
- shipment.created
- shipment.booked
- shipment.departed
- shipment.delayed
- shipment.delivered
- inspection.created
- inspection.failed
- inspection.passed
- document.missing
- exception.opened
- exception.escalated
- exception.resolved

#### Settlement events
- settlement.due
- settlement.partial
- settlement.received
- settlement.overdue
- settlement.disputed
- settlement.closed

#### Payment and points events
- payment.intent.created
- payment.gateway.awaiting_confirmation
- payment.gateway.succeeded
- payment.gateway.failed
- payment.gateway.canceled
- payment.gateway.refund_requested
- payment.gateway.refunded
- payment.webhook.received
- payment.webhook.verified
- points.credit.requested
- points.credited
- points.debit.requested
- points.debited
- points.adjustment.applied
- points.reversal.applied
- payment.reconciliation.completed

#### Control and audit events
- decision.logged
- state.transition.applied
- state.transition.rejected
- trust.score.changed
- task.auto_created
- human.override.recorded
- audit.chain.sealed
- audit.verify.completed

### 7.3 Canonical event structure

Every event record must support:
- event_id
- event_type
- event_version
- aggregate_type
- aggregate_id
- workspace_id
- trace_id
- span_id
- parent_span_id optional
- correlation_id optional
- causation_event_id optional
- actor_type
- actor_id
- source_service
- occurred_at
- ingested_at
- state_before optional
- state_after optional
- reason_code_primary optional
- reason_codes_secondary optional
- payload_json
- payload_schema_version
- prev_event_hash optional
- event_hash
- signature optional
- is_material flag
- audit_required flag

### 7.4 Event writing rules

1. Events must be immutable after write.
2. Event ordering must be preserved per aggregate.
3. Idempotency must be enforced for retry-prone operations.
4. Event write must happen within a controlled transaction boundary with the state change or through a durable outbox pattern.
5. Material events must be eligible for audit chain inclusion.
6. Payment webhook processing must be idempotent.
7. Points balance mutation must never occur without a corresponding immutable ledger event.

---

## 8. Aggregate Design and Event Ownership

The system must treat the following as first-class aggregates:

- RFQ
- Quote
- Negotiation
- Deal
- Shipment
- Inspection
- Exception
- Settlement
- PaymentOrder
- PointAccount
- PointLedgerEntry
- Counterparty trust profile
- Task or workflow instance when operationally relevant

Every event must belong to one primary aggregate_type and aggregate_id.

---

## 9. State Machine Architecture

### 9.1 Core aggregates requiring state machines

At minimum:
- RFQ
- Quote
- Deal
- Shipment
- Inspection
- Exception
- Settlement
- PaymentOrder

### 9.2 PaymentOrder state machine baseline

States:
- DRAFT
- INTENT_CREATED
- AWAITING_CUSTOMER_ACTION
- PROCESSING
- SUCCEEDED
- FAILED
- CANCELED
- REFUND_PENDING
- REFUNDED
- RECONCILED

Valid examples:
- DRAFT -> INTENT_CREATED
- INTENT_CREATED -> AWAITING_CUSTOMER_ACTION
- AWAITING_CUSTOMER_ACTION -> PROCESSING
- PROCESSING -> SUCCEEDED
- PROCESSING -> FAILED
- AWAITING_CUSTOMER_ACTION -> CANCELED
- SUCCEEDED -> REFUND_PENDING
- REFUND_PENDING -> REFUNDED
- SUCCEEDED -> RECONCILED
- REFUNDED -> RECONCILED

### 9.3 PointAccount model

A PointAccount itself is balance-bearing, but the authoritative history comes from immutable PointLedgerEntry records. Balance is a derived or checkpointed state, while ledger entries are the canonical value trail.

### 9.4 Failed transition handling

Failed transitions must return explicit error codes and preserve trace context. For payment and point mutations, failed transitions should also record control outcomes where materially relevant.

---

## 10. Distributed Tracing Architecture

Every incoming request that can produce or mutate business records must receive a trace_id. This includes:
- RFQ and quote actions
- shipment updates
- payment creation
- payment webhook handling
- point credits and debits
- refund flows
- reconciliation jobs

The following records must be queryable by trace_id:
- API request log
- event_store
- audit_chain
- failed transition records
- payment webhook records
- workflow task creation
- exception records
- settlement checks
- points ledger actions

---

## 11. Standard Reason Code Architecture

Suggested reason code families:
- Q = Quote and pricing
- N = Negotiation
- D = Deal outcome
- E = Execution and logistics
- S = Settlement
- T = Trust and counterparty behavior
- R = Risk and controls
- A = Audit and override
- M = Market condition
- W = Workflow and operations
- P = Payment
- L = Ledger / points

### 11.1 Payment and points starter dictionary

Payment:
- P-INIT-001 Payment intent created
- P-GATE-001 Gateway customer action required
- P-GATE-002 Gateway payment succeeded
- P-GATE-003 Gateway payment failed
- P-GATE-004 Gateway payment canceled
- P-WEBH-001 Webhook verified
- P-WEBH-002 Webhook rejected as invalid
- P-REFD-001 Refund initiated
- P-REFD-002 Refund completed
- P-RECO-001 Payment reconciliation completed

Ledger / points:
- L-CRDT-001 Points credited after successful payment
- L-DEBT-001 Points deducted for service usage
- L-ADJ-001 Manual points adjustment approved
- L-RVSL-001 Points reversal applied
- L-LOCK-001 Insufficient points for debit
- L-DUPL-001 Duplicate ledger mutation prevented

Reason codes must be applied to:
- decision logs
- material state transitions
- payment outcomes
- webhook verification outcomes
- point credits and debits
- refunds and reversals
- exceptions
- loss attribution outputs
- human overrides
- audit verification summaries

---

## 12. Tamper-evident Audit Chain Architecture

The audit chain must include, at minimum:
- decision.logged events for material choices
- quote generation and send events
- deal confirmation and cancellation events
- material state transition applications
- important failed transition attempts
- exception escalation or resolution events
- settlement overdue or disputed events
- payment intent creation
- payment success / failure / refund confirmation
- webhook verification outcome
- points credit / debit / reversal / manual adjustment
- any human override
- audit verification runs
- checkpoint seals

The audit chain remains append-only and hash-linked.

---

## 13. Replay and Reconstruction Requirements

Every payment and points operation must preserve enough information so later components can:
- rebuild payment lifecycle
- tie external gateway outcome to internal aggregate state
- verify point credit/debit lineage
- explain whether a service access decision was caused by insufficient points, failed payment, or delayed reconciliation
- compare user path and system path in simulation

---

## 14. Integration Obligations for Other Modules

### 14.1 RFQ module obligations
- emit RFQ events
- maintain RFQ state machine
- attach reason codes
- write audit records for material decisions

### 14.2 Quote module obligations
- emit quote generation, revision, send, expiry events
- enforce quote state machine
- preserve pricing inputs
- attach pricing and eligibility reason codes
- audit material quote operations

### 14.3 Negotiation module obligations
- record every material offer/counter/reject/accept action
- preserve actor and trace context
- attach reason codes to close/loss outcomes

### 14.4 Deal module obligations
- enforce confirmation and completion transitions
- emit deal status events
- trigger audit records
- support attribution hooks

### 14.5 Supply chain module obligations
- emit shipment and inspection events
- enforce shipment and inspection state machines
- attach execution and logistics reason codes
- write auditable exception records

### 14.6 Settlement module obligations
- enforce settlement states
- emit overdue, dispute, receipt, and closure events
- attach payment and dispute reason codes
- support trust score updates

### 14.7 Payment Service obligations
- wrap external Stripe gateway operations with internal PaymentOrder aggregate state
- emit canonical payment events for create / action-required / success / failure / cancel / refund / reconcile
- verify webhook signatures before material state mutation
- enforce idempotent webhook handling
- create points credit events only after internal confirmation rules pass
- write audit records for material payment outcomes

### 14.8 Internal points service obligations
- maintain immutable point ledger entries
- support service add-point and deduct-point operations
- prevent negative balance unless a future overdraft mode is explicitly introduced
- attach ledger reason codes for every mutation
- emit auditable events for every material balance change
- support reversal entries instead of destructive edits

---

## 15. Work Item List Added to Main Software Development Worklist

### 15.1 Track CEA-01 — Canonical Event Architecture
- define canonical event model
- define event naming conventions
- define event versioning rules
- define material vs non-material event policy
- define payload schema version handling
- design event_store table
- implement Rust event model structs
- implement event serialization and validation
- implement event write service
- implement idempotent event write strategy
- implement aggregate event timeline query
- implement event hash generation
- implement outbox or atomic write policy

### 15.2 Track CEA-02 — Aggregate State Machine Framework
- define aggregate list requiring state control
- define baseline states per aggregate
- define transition matrix format
- define guard condition interface
- define side effect interface
- define invalid transition error model
- design state_transition_rules table
- implement Rust state machine engine
- implement transition validator middleware/service
- implement failed transition recorder
- implement state snapshot update policy
- implement state reconstruction from events fallback

### 15.3 Track CEA-03 — Distributed Tracing Foundation
- choose OpenTelemetry baseline libraries for Rust
- define trace context middleware
- define trace field policy in logs
- define trace correlation fields in event_store
- define trace correlation fields in audit_chain
- define span naming conventions
- instrument core write paths
- instrument state transition paths
- instrument workflow-triggered paths
- instrument audit verification paths
- define local and ZEABUR-ready trace configuration modes

### 15.4 Track CEA-04 — Standard Reason Code System
- define reason code family taxonomy
- define naming and numbering policy
- design reason_code_registry table
- define lifecycle policy
- define reason code validation service
- define required reason code cases
- define multi-code attachment rules
- implement Rust reason registry model
- implement admin seed data for starter dictionary
- implement reason code lookup API
- implement reason code validation in write flows

### 15.5 Track CEA-05 — Tamper-evident Audit Chain
- define auditable action policy
- design audit_chain table
- design audit_seals table
- define record hash formula versioning
- define append-only enforcement strategy
- implement audit record writer
- implement audit hash chain linking
- implement seal creation service
- implement verify single chain routine
- implement verify seal batch routine
- implement verification result model
- implement override audit write path

### 15.6 Track CEA-06 — Human Override and Control Integrity
- define override-required scenarios
- define justification policy
- define override event types
- define override reason code requirements
- implement override API contract placeholders
- implement override audit integration
- implement override visibility in aggregate timelines

### 15.7 Track CEA-07 — Replay-readiness and Reconstruction Preparation
- define replay-readiness data completeness checklist
- define aggregate reconstruction query requirements
- define root-cause reconstruction metadata requirements
- define missing-link detection policy
- define event ordering policy
- define timeline export format for later simulation tools

### 15.8 Track CEA-08 — Payment & Points Control Foundation
- define PaymentOrder aggregate and state machine
- define PointAccount and PointLedgerEntry control model
- define payment event taxonomy
- define payment webhook verification policy
- define payment idempotency policy
- define payment-to-ledger reconciliation rules
- define refund and reversal compensation policy
- define point debit insufficient-balance rule
- define manual ledger adjustment approval policy
- define payment audit inclusion rules

### 15.9 Track CEA-09 — Documentation and Blueprint Consolidation
- merge control framework into README
- merge control framework into architecture spec
- merge work items into development roadmap
- prepare PostgreSQL schema follow-on spec dependencies
- prepare Rust API follow-on spec dependencies
- prepare ZEABUR deployment follow-on spec dependencies
- maintain versioned architecture decision record for this control framework

---

## 16. Immediate Merge Text for Main README

AegisBroker adopts an event-driven control framework in which all materially relevant trading, negotiation, execution, settlement, payment, and internal points-ledger actions are governed by event flow, explicit state machines, distributed trace continuity, standardized reason codes, and tamper-evident audit records. This framework exists to support reverse verification of failed transactions, loss attribution, replayable causal reconstruction, expert trader behavior comparison, payment-to-ledger reconciliation, and future AI explanation grounded in verifiable system history.

---

## 17. Immediate Merge Text for Architecture Spec

The architecture of AegisBroker includes a dedicated Event Flow and State Machine Layer and a dedicated Trace and Audit Layer. No core module, including Payment Service and internal points service, is considered architecturally complete unless its material operations emit canonical events, respect declared state transitions, preserve trace context, attach reason semantics where required, and write tamper-evident audit records when commercially or operationally significant.

---

## 18. Immediate Merge Text for Software Development Worklist

High-priority control-foundation work must be completed before advanced market, execution, simulation, AI, or monetization features proceed. This control-foundation work includes canonical event architecture, aggregate state machine framework, distributed tracing foundation, standard reason code registry and enforcement, tamper-evident audit chain, payment and points control foundation, override integrity handling, and replay-readiness support.

---

## 19. Final Statement

This document formally pushes AegisBroker from a strong module-oriented design into a blueprint-oriented control architecture that now also covers Stripe-backed payments and internal point credits/debits.

From this point forward, every important system feature must be designed not only in terms of business capability, but also in terms of:
- event emission
- state integrity
- trace continuity
- reason semantics
- tamper-evident evidence
- payment reconciliation
- points-ledger integrity

This is the required foundation for:
- integrated simulation reverse verification
- loss attribution
- causal replay
- expert trader replication testing
- future auditable and explainable decision support
- monetized service operations
