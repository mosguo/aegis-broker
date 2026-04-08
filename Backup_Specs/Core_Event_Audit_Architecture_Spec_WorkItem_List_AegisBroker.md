# AegisBroker — Core Event & Audit Architecture Spec & WorkItem List
Version: v1.0  
Status: Blueprint-ready / Repo-ready design baseline  
Target deployment: ZEABUR  
System stack baseline: Python Frontend + Rust Backend + PostgreSQL + Object Storage  
This document is an integrated, unsegmented planning file that formally merges today's decisions into the main README direction, the Architecture Spec direction, and the software development worklist. It also establishes the first complete blueprint document on the 100% path: Core Event & Audit Architecture Spec & WorkItem List.

---

## 0. Document Purpose

This document has four purposes at the same time:

1. Formally merge the newly agreed control framework into the main system direction of AegisBroker.
2. Upgrade the overall design from module-level planning into a development blueprint layer.
3. Define the first complete blueprint deliverable on the 100% completion path:
   Core Event & Audit Architecture Spec & WorkItem List.
4. Provide a directly actionable work item structure so engineering can start implementation without rethinking the system from first principles.

This document is intentionally written as one integrated, unsegmented planning file. It should be treated as a master planning text that can later be split into:
- README sections
- architecture specification sections
- engineering work breakdown sections
- repository bootstrap documents

---

## 1. Formal Positioning Update

AegisBroker is no longer defined only as a broker enablement platform.

AegisBroker is now formally defined as:

A broker-centered, event-driven, decision and execution operating system for commodity brokerage, built to preserve human broker judgment, trace every material action, support failure reconstruction, enable loss attribution, and provide the foundation for expert trader behavior replication.

This definition supersedes earlier narrower positioning that emphasized CRM, market data, supply chain, or workflow as separate module clusters.

Those modules remain necessary, but they now sit under a stronger architectural purpose:

Market context + broker memory + structured decisions + controlled execution + replayable evidence + verifiable auditability.

In practical terms, this means the system is not designed merely to store records. It is designed to answer, after the fact and with evidentiary quality:

- What happened?
- Why did it happen?
- Who or what triggered it?
- What state changed?
- Was the transition valid?
- What was lost, delayed, missed, or gained?
- Can the system reconstruct the full causal chain?
- Can the system later approximate or replicate how a top trader would have acted?

---

## 2. Core Design Direction Added to Main README

The following design direction is now considered part of the main README-level system definition.

### 2.1 New first-class system principle

Every commercially material operation in AegisBroker must be represented through the combination of:

- event flow
- state machine control
- distributed tracing
- standard reason codes
- tamper-evident audit records

This combination is not optional and is not a secondary observability feature. It is part of the core business logic architecture.

### 2.2 Why this principle exists

Without this control framework, the platform can store data but cannot reliably support:

- failure backtracking
- loss attribution
- causal replay
- compliance-grade reconstruction
- expert behavior comparison
- AI explanation grounded in actual history

### 2.3 System outcome expected from this principle

After this framework is implemented, AegisBroker should be able to support:

- transaction failure reverse verification during integrated simulation testing
- loss attribution and root-cause reconstruction during integrated simulation testing
- expert trader behavior replay and system comparison during operational simulation testing
- evidence-backed workflow validation in production-like runs
- tamper-evident audit review for important commercial and operational steps

---

## 3. Updated Architecture Positioning

The architecture should now be understood in the following layered form:

- L0 Identity and Access
- L1 Core Business Records
- L2 Decision Layer
- L3 Event Flow and State Machine Layer
- L4 Trace and Audit Layer
- L5 Market, Supply Chain, Pricing, Workflow Services
- L6 Replay, Attribution, Simulation, and AI Comparison Layer

### 3.1 Layer roles

#### L0 Identity and Access
Responsible for authentication, authorization, workspace ownership, actor identification, session records, and role-based operation eligibility.

#### L1 Core Business Records
Responsible for master data and transactional business records such as counterparties, RFQs, quotes, negotiations, deals, shipments, settlement records, and documents.

#### L2 Decision Layer
Responsible for human judgment capture and machine-assistable decisions. This includes decision logs, recommendation context, reason selection, human override records, confidence markers, and outcome-linked decision memory.

#### L3 Event Flow and State Machine Layer
Responsible for turning business actions into canonical events and enforcing valid state transitions across aggregate types such as RFQ, Quote, Deal, Shipment, Inspection, Exception, and Settlement.

#### L4 Trace and Audit Layer
Responsible for trace propagation, cross-service correlation, tamper-evident append-only audit recording, verification capability, and replay support.

#### L5 Market, Supply Chain, Pricing, Workflow Services
Responsible for domain functions such as market price ingestion, quote pricing, lot tracking, shipment milestones, exception handling, tasks, alerts, and reminders.

#### L6 Replay, Attribution, Simulation, and AI Comparison Layer
Responsible for event replay, root-cause reconstruction, loss attribution, expert-vs-system comparison, simulation consoles, and future AI explainability.

---

## 4. Scope Expansion Formally Added to Architecture Spec

The following items are now part of the architecture specification, not just implementation preferences:

1. Event-first modeling for every material business transition
2. Explicit state machine design for core aggregates
3. Trace propagation across all backend operations
4. Mandatory use of standard reason codes for decision and exception semantics
5. Tamper-evident audit chain for important events and overrides
6. Replayability as a design requirement
7. Attribution readiness as a design requirement
8. Expert replication readiness as a future testability requirement

This means all future modules must be evaluated not only by whether they provide business functionality, but also by whether they emit the right events, respect the right state transitions, preserve trace continuity, carry reason semantics, and write auditable records where required.

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
Every important decision, rejection, exception, loss signal, completion explanation, and override must be attached to one or more standardized reason codes.

### 5.5 Tamper-evident Audit
Every important action and state-changing operation must be captured in an append-only tamper-evident audit chain.

If any future feature is implemented without these five concerns where applicable, it should be considered architecturally incomplete.

---

## 6. Business Motivation Behind the Five-part Control Framework

This section explains why the control framework is not merely a technical architecture preference.

### 6.1 For reverse verification
When a transaction fails, the platform must reconstruct the chain from:
- market conditions
- incoming RFQ or broker initiative
- decision taken
- quote produced
- counteroffer progression
- deal confirmation or rejection
- execution events
- settlement status

### 6.2 For loss attribution
The platform must later identify whether loss originated from:
- wrong price threshold
- slow response
- low confidence quote source
- rejected terms
- delayed shipment
- failed inspection
- incomplete documents
- payment delay or non-payment
- human override error
- unreliable counterparty

### 6.3 For expert replication
A future simulation phase must compare:
- top trader decision path
- system recommendation path
- actual human-selected path
- outcome differences

This requires event-level and reason-level evidence, not broad summaries only.

### 6.4 For defensibility
If the platform ever supports enterprise clients, teams, or regulated workflows, the ability to verify that historical records were not silently modified becomes commercially and operationally important.

---

## 7. Core Architecture Deliverable Defined in This Document

This document formally defines Deliverable 1 on the path to 100% design completion:

Core Event & Audit Architecture Spec & WorkItem List

This deliverable must provide enough detail so that a competent engineering implementation can begin in a Rust + PostgreSQL + Python + ZEABUR target environment without requiring conceptual reinterpretation.

This deliverable covers:
- canonical event model
- aggregate state machine strategy
- distributed tracing baseline
- reason code system baseline
- tamper-evident audit chain design
- development work item structure
- integration obligations for other modules

This deliverable does not fully replace later documents, but it establishes the control skeleton that later documents must conform to.

---

## 8. Canonical Event Architecture

### 8.1 Event-first principle

An event is the durable system statement that something materially relevant happened. Events are not simple logs and are not optional diagnostics. They are part of business truth reconstruction.

### 8.2 Event categories

The platform must support at least the following canonical event families.

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

#### Control and audit events
- decision.logged
- state.transition.applied
- state.transition.rejected
- trust.score.changed
- task.auto_created
- human.override.recorded
- audit.chain.sealed
- audit.verify.completed

### 8.3 Canonical event structure

Every event record must support at least the following fields:

- event_id
- event_type
- event_version
- aggregate_type
- aggregate_id
- tenant_id or workspace_id
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

### 8.4 Event writing rules

1. Events must be immutable after write.
2. Event ordering must be preserved per aggregate.
3. Idempotency must be enforced for retry-prone operations.
4. Event write must happen within a controlled transaction boundary with the state change or through a durable outbox pattern.
5. Material events must be eligible for audit chain inclusion.
6. Event payloads must carry enough context to support replay, but avoid uncontrolled duplication of large documents.
7. Sensitive payload sections must support redaction strategy without invalidating structural integrity.

### 8.5 Event transport strategy

The implementation path should support two stages:

#### Stage A: single-service or modular monolith baseline
- write events into PostgreSQL event_store
- use internal event dispatcher within Rust backend
- replay and verification operate from DB

#### Stage B: service-split evolution
- maintain PostgreSQL event source of truth
- add asynchronous internal messaging if needed
- keep trace and reason propagation mandatory across service boundaries

---

## 9. Aggregate Design and Event Ownership

The system must treat the following as first-class aggregates for event and state purposes:

- RFQ
- Quote
- Negotiation
- Deal
- Shipment
- Inspection
- Exception
- Settlement
- Counterparty trust profile
- Task or workflow instance when operationally relevant

### 9.1 Aggregate ownership rule

Every event must belong to one primary aggregate_type and aggregate_id, even if it references related aggregates. This is necessary for reconstruction and state consistency.

### 9.2 Cross-aggregate linking

If an event impacts multiple business records, payload_json must include linked entity references such as:
- rfq_id
- quote_id
- deal_id
- shipment_id
- exception_id
- settlement_id
- counterparty_id

---

## 10. State Machine Architecture

### 10.1 Purpose

State machines exist to ensure business process integrity. They prevent silent or invalid transitions and provide a clean basis for replay, reasoning, and error detection.

### 10.2 Core aggregates requiring state machines

At minimum, state machines must be defined for:

- RFQ
- Quote
- Deal
- Shipment
- Inspection
- Exception
- Settlement

Negotiation can either be modeled as an event-led subflow or have a simplified state model, but its transitions must still be explicit enough for replay.

### 10.3 RFQ state machine baseline

States:
- RECEIVED
- QUALIFIED
- REJECTED
- QUOTED
- CLOSED

Examples of valid transitions:
- RECEIVED -> QUALIFIED
- RECEIVED -> REJECTED
- QUALIFIED -> QUOTED
- QUALIFIED -> CLOSED
- QUOTED -> CLOSED

Examples of invalid transitions:
- RECEIVED -> QUOTED without qualification or explicit override
- REJECTED -> QUOTED unless re-opened through a controlled event
- CLOSED -> QUALIFIED

### 10.4 Quote state machine baseline

States:
- DRAFT
- GENERATED
- SENT
- NEGOTIATING
- ACCEPTED
- REJECTED
- EXPIRED
- CANCELLED

Valid examples:
- DRAFT -> GENERATED
- GENERATED -> SENT
- SENT -> NEGOTIATING
- NEGOTIATING -> ACCEPTED
- NEGOTIATING -> REJECTED
- SENT -> EXPIRED
- GENERATED -> CANCELLED

### 10.5 Deal state machine baseline

States:
- PENDING_CONFIRMATION
- CONFIRMED
- EXECUTING
- COMPLETED
- FAILED
- CANCELLED

Valid examples:
- PENDING_CONFIRMATION -> CONFIRMED
- CONFIRMED -> EXECUTING
- EXECUTING -> COMPLETED
- EXECUTING -> FAILED
- CONFIRMED -> CANCELLED under explicit rule

### 10.6 Shipment state machine baseline

States:
- CREATED
- BOOKED
- IN_TRANSIT
- DELAYED
- DELIVERED
- EXCEPTION
- CLOSED

### 10.7 Inspection state machine baseline

States:
- CREATED
- SCHEDULED
- IN_PROGRESS
- PASSED
- FAILED
- WAIVED
- CLOSED

### 10.8 Exception state machine baseline

States:
- OPEN
- TRIAGED
- ESCALATED
- RESOLVED
- CLOSED
- CANCELLED

### 10.9 Settlement state machine baseline

States:
- NOT_DUE
- DUE
- PARTIAL
- PAID
- OVERDUE
- DISPUTED
- WRITTEN_OFF
- CLOSED

### 10.10 State transition rule contract

Every state transition definition must include:
- aggregate_type
- from_state
- to_state
- trigger_event_type
- allowed_actor_types
- guard_conditions
- required_reason_codes if applicable
- side_effects
- audit_required yes or no
- invalid_transition_error_code

### 10.11 Side effects

Example side effects include:
- create reminder
- create workflow task
- update trust signal
- open exception
- write audit record
- seal event chain checkpoint candidate
- notify assigned role
- start settlement countdown
- request document completion

### 10.12 Failed transition handling

Failed transitions must not disappear into generic API errors. They must be captured as structured control outcomes.

At minimum, failed transition handling must:
- return an explicit API error code
- write a state.transition.rejected event or structured control record when appropriate
- preserve trace context
- optionally produce an audit record if the attempted change was material

---

## 11. Distributed Tracing Architecture

### 11.1 Purpose

Distributed tracing is required for causal reconstruction across services, workflows, async jobs, scheduled jobs, and future service separation.

### 11.2 Trace policy

Every incoming request that can produce or mutate business records must receive a trace_id. If upstream trace context exists, it must be propagated.

Every material operation must create spans for:
- decision evaluation
- pricing computation
- quote generation
- negotiation event recording
- state transition validation
- shipment update processing
- settlement check
- audit chain write
- replay operation
- verification operation

### 11.3 Minimum trace fields

- trace_id
- span_id
- parent_span_id
- operation_name
- service_name
- actor_id optional
- aggregate_type
- aggregate_id
- status
- started_at
- ended_at
- error_code optional

### 11.4 Baseline implementation for ZEABUR-ready architecture

Minimum viable:
- OpenTelemetry instrumentation in Rust backend
- middleware for trace extraction and injection
- trace_id and span_id persisted alongside event and audit records
- structured logs include trace_id
- internal API responses can optionally surface correlation identifiers for admin/debug use

Recommended stronger version:
- OpenTelemetry exporter to Jaeger or Grafana Tempo when deployment complexity allows
- dashboard correlation views in admin tooling

### 11.5 Trace correlation rules

The following records must be queryable by trace_id:
- API request log
- event_store
- audit_chain
- failed transition records
- job execution records
- workflow task creation
- exception records
- settlement checks

---

## 12. Standard Reason Code Architecture

### 12.1 Purpose

Reason codes convert raw records into analyzable business semantics.

Without reason codes, the platform can record actions but cannot consistently explain:
- why a quote was not sent
- why a deal was lost
- why an exception escalated
- why a payment risk changed
- why a human override occurred

### 12.2 Reason code families

Suggested families:
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

### 12.3 Reason code format

Recommended format:
DOMAIN-CATEGORY-### or DOMAIN-SUBDOMAIN-###

Examples:
- Q-ELIG-001
- Q-PRICE-002
- N-LOSS-001
- D-WIN-003
- E-FAIL-004
- S-RISK-001
- T-SCORE-002
- A-OVRD-001
- M-VOLA-001
- W-MISS-002

### 12.4 Example starter dictionary

Quote and pricing:
- Q-ELIG-001 Counterparty not eligible for quote
- Q-PRICE-001 Benchmark moved beyond allowed band
- Q-PRICE-002 Quality adjustment increased quote
- Q-MARGIN-001 Minimum spread threshold not met
- Q-SRC-001 Price source reliability below threshold

Negotiation:
- N-LOSS-001 Counterparty demanded unacceptable price
- N-LOSS-002 Counterparty demanded unacceptable payment terms
- N-LOSS-003 Timing mismatch ended negotiation
- N-WIN-001 Delayed shipment accepted to secure deal
- N-WIN-002 Relationship trust enabled close

Deal:
- D-WIN-001 Best executable quote accepted
- D-WIN-002 Speed advantage secured close
- D-LOSS-001 Competing offer undercut margin floor
- D-LOSS-002 Trust concerns blocked confirmation

Execution:
- E-FAIL-001 Shipment delayed
- E-FAIL-002 Required document missing
- E-FAIL-003 Inspection failed
- E-FAIL-004 Logistics partner fault
- E-RES-001 Exception resolved through substitute route

Settlement:
- S-RISK-001 Payment overdue
- S-RISK-002 Payment disputed
- S-CLOSE-001 Full payment received
- S-WOFF-001 Amount written off

Trust and risk:
- T-SCORE-001 Timely payment improved trust
- T-SCORE-002 Repeated delay reduced trust
- R-CNTRL-001 Guard condition blocked action
- R-CNTRL-002 Manual override required

Audit:
- A-OVRD-001 Human override with justification
- A-OVRD-002 Human override missing required justification
- A-VERI-001 Audit chain verification passed
- A-VERI-002 Audit chain verification failed

### 12.5 Reason code application rules

Reason codes must be applied to:
- decision logs
- material state transitions
- rejections
- exceptions
- loss attribution outputs
- trust changes
- settlement abnormal outcomes
- human overrides
- audit verification summaries

### 12.6 Reason code governance

The system must include:
- a reason_code_registry table
- effective_date and status fields
- category ownership
- deprecation rules
- optional severity and attribution class
- documentation field describing when to use each code

No free-text-only explanation should replace reason code usage for material operations. Free text is additive, not substitutive.

---

## 13. Tamper-evident Audit Chain Architecture

### 13.1 Purpose

The audit chain provides verifiable evidence that important records were appended in sequence and have not been silently rewritten.

This is not a blockchain requirement. It is an integrity requirement.

### 13.2 What must enter the audit chain

At minimum:
- decision.logged events for material choices
- quote generation and revision events
- quote send events
- negotiation acceptance or rejection events
- deal confirmation and cancellation events
- material state transition applications
- important failed transition attempts
- exception escalation or resolution events
- settlement overdue or disputed events
- trust score changes if materially impactful
- any human override
- audit verification runs
- checkpoint seals

### 13.3 Audit chain record structure

Each audit record must include:
- audit_id
- workspace_id
- entity_type
- entity_id
- action_type
- source_event_id optional
- trace_id
- actor_type
- actor_id
- occurred_at
- payload_hash
- prev_audit_hash
- record_hash
- signature optional
- seal_batch_id optional
- verification_status optional
- metadata_json

### 13.4 Hashing rule

Recommended record_hash input:
hash(prev_audit_hash + payload_hash + entity_type + entity_id + action_type + occurred_at + trace_id)

This formula should be documented and kept versioned.

### 13.5 Append-only rule

The audit_chain table must be treated as append-only.
Updates and deletes should be prevented at the application layer and additionally restricted by DB policy and/or triggers.

### 13.6 Seal strategy

At periodic intervals or after defined material thresholds, the system should create a seal checkpoint over a contiguous sequence of audit records.

Seal checkpoints should include:
- seal_batch_id
- first_audit_id
- last_audit_id
- seal_hash
- sealed_at
- record_count
- verification_method

Later documents will refine external anchoring strategies, but the first implementation must already support internal verification and checkpoint sealing.

### 13.7 Verification capability

The system must expose the ability to:
- verify a single audit record chain
- verify a seal batch
- report first broken link if chain integrity fails
- return structured verification results

---

## 14. Replay and Reconstruction Requirements

Although deeper replay tooling belongs to later documents, this architecture spec must already define replay-readiness obligations.

Every event and audit write path must preserve enough information so that later components can:
- rebuild aggregate timelines
- reconstruct state transitions
- detect invalid or missing expected transitions
- explain loss attribution inputs
- compare human and system paths in simulation

This means the event model, state model, reason model, and trace model must not be under-specified.

---

## 15. Integration Obligations for Other Modules

Every future module must integrate with the control framework.

### 15.1 RFQ module obligations
- emit rfq.received and rfq qualification/rejection events
- maintain RFQ state machine
- attach reason codes for rejection or qualification decisions
- write audit records for material RFQ decisions

### 15.2 Quote module obligations
- emit quote generation, revision, send, expiry events
- enforce quote state machine
- preserve pricing inputs and confidence markers
- attach pricing and eligibility reason codes
- audit material quote operations

### 15.3 Negotiation module obligations
- record every material offer/counter/reject/accept action as event
- keep causal link to quote and deal
- preserve actor and trace context
- attach reason codes to close/loss outcomes when applicable

### 15.4 Deal module obligations
- enforce confirmation and completion transitions
- emit deal status events
- trigger audit records for confirmation/cancel/failure/completion
- support attribution hooks

### 15.5 Supply chain module obligations
- emit shipment and inspection events
- enforce shipment and inspection state machines
- attach execution and logistics reason codes
- write auditable exception records

### 15.6 Settlement module obligations
- enforce settlement states
- emit overdue, dispute, receipt, and closure events
- attach payment and dispute reason codes
- support trust score updates

### 15.7 Workflow engine obligations
- consume events to create tasks, reminders, escalations
- preserve trace context
- emit task.auto_created and related events where material

### 15.8 AI and replay obligations
- consume structured event, trace, reason, and audit data
- never rely solely on free-text artifacts when structured evidence exists
- support comparison outputs that point back to traceable records

---

## 16. PostgreSQL-facing Requirements Introduced by This Spec

This document is not the full PostgreSQL Schema & Migration Spec, but it imposes mandatory schema requirements that later documents must honor.

The database must include, at minimum:
- event_store
- aggregate_state_snapshots optional but recommended
- state_transition_rules
- failed_transition_records
- reason_code_registry
- audit_chain
- audit_seals
- decision_logs
- job_execution_logs optional
- trace_operation_logs optional if not fully externalized

Key design requirements:
- append-only event and audit tables
- immutable event hash columns
- reason registry referential integrity
- index strategy supporting aggregate timeline reads and trace reconstruction
- migration discipline compatible with ZEABUR deployment startup order

---

## 17. Rust API-facing Requirements Introduced by This Spec

This document is not the full Rust API Contract Spec, but it imposes mandatory API behavior requirements that later documents must honor.

The API layer must support:
- trace context extraction/injection middleware
- idempotent write endpoints for retry-prone material actions
- structured error codes for invalid transitions and guard failures
- optional correlation headers for admin/debug
- explicit reason code acceptance where user choice is required
- human override endpoints that require justification and produce audit records
- verification endpoints for audit chain checking
- replay query endpoints later in the roadmap

---

## 18. ZEABUR Deployment-facing Requirements Introduced by This Spec

This document is not the full ZEABUR Deployment & Repo Blueprint, but it imposes mandatory deployment requirements that later documents must honor.

Deployment baseline must support:
- Rust backend service
- Python frontend service
- PostgreSQL database
- optional worker service if event processing is separated
- environment variable support for trace mode, audit mode, and seal intervals
- startup ordering that prevents API writes before migrations complete
- object storage compatibility for later document artifacts and optional seal export

---

## 19. Engineering Work Item List Added to Main Software Development Worklist

The following work item clusters must be formally inserted into the software development worklist as top-priority architecture work.

### 19.1 Track CEA-01 — Canonical Event Architecture
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

### 19.2 Track CEA-02 — Aggregate State Machine Framework
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

### 19.3 Track CEA-03 — Distributed Tracing Foundation
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

### 19.4 Track CEA-04 — Standard Reason Code System
- define reason code family taxonomy
- define reason code naming and numbering policy
- design reason_code_registry table
- define active/deprecated lifecycle policy
- define reason code validation service
- define required reason code cases
- define multi-code attachment rules
- implement Rust reason registry model
- implement admin seed data for starter dictionary
- implement reason code lookup API
- implement reason code validation in write flows

### 19.5 Track CEA-05 — Tamper-evident Audit Chain
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

### 19.6 Track CEA-06 — Human Override and Control Integrity
- define override-required scenarios
- define justification policy
- define override event types
- define override reason code requirements
- implement override API contract placeholders
- implement override audit integration
- implement override visibility in aggregate timelines

### 19.7 Track CEA-07 — Replay-readiness and Reconstruction Preparation
- define replay-readiness data completeness checklist
- define aggregate reconstruction query requirements
- define root-cause reconstruction metadata requirements
- define missing-link detection policy
- define event ordering policy
- define timeline export format for later simulation tools

### 19.8 Track CEA-08 — Documentation and Blueprint Consolidation
- merge control framework into README
- merge control framework into architecture spec
- merge work items into development roadmap
- prepare PostgreSQL schema follow-on spec dependencies
- prepare Rust API follow-on spec dependencies
- prepare ZEABUR deployment follow-on spec dependencies
- maintain versioned architecture decision record for this control framework

---

## 20. Work Item Dependencies

The following dependency structure should be observed.

### 20.1 Immediate dependency chain
1. canonical event model
2. reason code taxonomy baseline
3. aggregate state list and state machine matrix format
4. audit chain data model
5. trace context policy
6. event and audit table design
7. Rust model definitions
8. write-path middleware and services

### 20.2 Before full market or execution modules advance
The following should already exist in at least minimal form:
- event_store
- reason_code_registry
- state machine engine
- trace middleware
- audit_chain
- invalid transition handling

### 20.3 Before simulation and attribution begin
The following must already be stable:
- consistent material event coverage
- state transition evidence
- reason code usage discipline
- trace continuity
- audit verification capability

---

## 21. Recommended Development Sequence After This Document

This is the recommended immediate sequence to keep the project on the 100% path.

### Step 1
Finalize this Core Event & Audit Architecture Spec & WorkItem List as the control blueprint baseline.

### Step 2
Produce PostgreSQL Schema & Migration Spec aligned with this document.

### Step 3
Produce Rust API Contract Spec aligned with this document.

### Step 4
Produce ZEABUR Deployment & Repo Blueprint aligned with this document.

### Step 5
Update master integrated README / architecture / worklist bundle so all subsequent feature work must conform to the control framework.

---

## 22. Completion Standard for This Deliverable

This deliverable should be considered complete only if an engineer can answer the following questions directly from the document:

- What is a canonical event in AegisBroker?
- Which aggregates need state machines?
- What fields must every event carry?
- How are invalid transitions handled?
- Where do reason codes apply?
- What enters the audit chain?
- How is tamper evidence created?
- How is trace context preserved?
- What work items must be implemented before downstream modules?
- How must later PostgreSQL, Rust API, and ZEABUR specs conform?

If the answer to any of these is unclear, the architecture is still below blueprint quality.

---

## 23. Design Completion Assessment After This Deliverable

Relative to the user-defined standard where 100% means:
only by handing over the documents, fully independent implementation of a working ZEABUR-compatible software repo is possible,

the completion level after this deliverable is estimated as follows:

- Before this deliverable: approximately low 80s
- After this deliverable is formally adopted: approximately 88%

Reason:
This document closes the control-framework ambiguity and removes a major portion of architecture uncertainty. However, 100% still requires the remaining three follow-on documents:
- PostgreSQL Schema & Migration Spec
- Rust API Contract Spec
- ZEABUR Deployment & Repo Blueprint

---

## 24. Immediate Merge Text for Main README

The following text should be considered merged into the main README direction:

AegisBroker adopts an event-driven control framework in which all materially relevant trading, negotiation, execution, and settlement actions are governed by event flow, explicit state machines, distributed trace continuity, standardized reason codes, and tamper-evident audit records. This framework exists to support reverse verification of failed transactions, loss attribution, replayable causal reconstruction, expert trader behavior comparison, and future AI explanation grounded in verifiable system history.

---

## 25. Immediate Merge Text for Architecture Spec

The following text should be considered merged into the architecture specification:

The architecture of AegisBroker includes a dedicated Event Flow and State Machine Layer and a dedicated Trace and Audit Layer. No core module is considered architecturally complete unless its material operations emit canonical events, respect declared state transitions, preserve trace context, attach reason semantics where required, and write tamper-evident audit records when commercially or operationally significant.

---

## 26. Immediate Merge Text for Software Development Worklist

The following text should be considered merged into the software development worklist:

High-priority control-foundation work must be completed before advanced market, execution, simulation, and AI features proceed. This control-foundation work includes canonical event architecture, aggregate state machine framework, distributed tracing foundation, standard reason code registry and enforcement, tamper-evident audit chain, override integrity handling, and replay-readiness support.

---

## 27. Final Statement

This document formally pushes AegisBroker from a strong module-oriented design into a blueprint-oriented control architecture.

From this point forward, every important system feature must be designed not only in terms of business capability, but also in terms of:
- event emission
- state integrity
- trace continuity
- reason semantics
- tamper-evident evidence

This is the required foundation for:
- integrated simulation reverse verification
- loss attribution
- causal replay
- expert trader replication testing
- future auditable and explainable decision support

This document is therefore not an optional appendix. It is the governing control blueprint for all subsequent system design and implementation.
