# AegisBroker — Financial Control & Reconciliation Spec
Version: v1.1  
Status: Next-layer Financial Control Blueprint  
Target deployment: ZEABUR  
Related documents:
- Core Event & Audit Architecture Spec & WorkItem List
- PostgreSQL Schema & Migration Spec
- Rust API Contract Spec
- ZEABUR Deployment & Repo Blueprint
- Double-entry Ledger & Accounting Spec

---

## 0. Document Purpose

This document upgrades AegisBroker from:
- payment-enabled system

into:
- financial control system

It defines:
- reconciliation engine
- compensation engine
- financial (cash + liability) control layer
- broker settlement layer
- payment risk layer

This layer ensures:
- monetary correctness
- ledger consistency
- operational recoverability
- audit-grade traceability

---

## 1. Financial System Principles

### 1.1 Money vs Points Separation

External money and internal points must NEVER be treated as the same entity.

- Money = real asset
- Points = platform liability / stored value

---

### 1.2 Ledger-first Design

All financial value changes must be represented as:
- immutable ledger entries

No destructive overwrite allowed.

---

### 1.3 Reconciliation is mandatory

System must continuously verify:

Stripe ↔ PaymentOrder ↔ Internal Ledger ↔ Points Ledger

---

### 1.4 Compensation over mutation

Failures must be corrected using:
- reversal entries
- compensation flows

NOT:
- deleting records
- rewriting balances

---

### 1.5 Audit completeness

Every financial transition must be:
- evented
- traceable
- auditable

---

### 1.6 Accounting implementation dependency

Financial ledger implementation MUST follow Double-entry Accounting Spec.

This means:
- Financial Control & Reconciliation Spec defines the control, reconciliation, compensation, and operational integrity layer.
- Double-entry Ledger & Accounting Spec defines the accounting language, journal structure, chart of accounts, debit/credit rules, and recognition model.
- When this document refers to financial ledger behavior, journal posting logic, liability reduction, revenue recognition, refund reversal, settlement clearing, broker commission recognition, or escrow-like holding behavior, the concrete accounting implementation must conform to the Double-entry Ledger & Accounting Spec.
- If a conflict appears between a control flow in this document and journal-posting detail in the accounting spec, the correct resolution is to preserve this document’s control requirements while implementing the underlying journal entries according to the accounting spec.

---

## 2. Reconciliation Engine

### 2.1 Purpose

Ensure consistency across:
- Stripe
- PaymentOrder
- Financial ledger
- Points ledger

---

### 2.2 Tables

#### payment_reconciliation_runs
- id
- started_at
- completed_at
- status
- summary_json

#### payment_reconciliation_items
- id
- run_id
- payment_order_id
- stripe_payment_intent_id
- mismatch_type
- severity
- resolved
- details_json

---

### 2.3 Mismatch Types

- MISSING_PAYMENT_ORDER
- MISSING_LEDGER_ENTRY
- DUPLICATE_POINTS_CREDIT
- PAYMENT_STATE_MISMATCH
- REFUND_NOT_REVERSED
- WEBHOOK_NOT_PROCESSED

---

### 2.4 Output Actions

- flag anomaly
- trigger compensation
- require manual review

---

## 3. Compensation Engine

### 3.1 Purpose

Handle partial-failure scenarios.

---

### 3.2 States

- COMPENSATION_PENDING
- COMPENSATED
- COMPENSATION_FAILED

---

### 3.3 Flows

Example:

payment success → points credited → downstream failure

→ system must:
- reverse points OR
- create pending compensation

---

### 3.4 Events

- compensation.requested
- compensation.applied
- compensation.failed

---

## 4. Financial Control Ledger Layer

### 4.1 Purpose

Track the control-layer mapping between:
- real money
- platform obligations
- service consumption
- refunds
- settlement allocations

This document defines the control intent, reconciliation expectations, and mutation policies.
The underlying accounting entries must be implemented according to the Double-entry Ledger & Accounting Spec.

---

### 4.2 Control-facing Table Concept

#### financial_ledger_entries
- id
- workspace_id
- ledger_type (cash / liability / revenue / refund)
- amount_minor
- currency
- direction (debit / credit)
- source_type
- source_id
- trace_id
- occurred_at

Note:
This structure may remain as a control-facing projection, convenience view, or compatibility layer, but canonical accounting posting should be backed by the double-entry journal model defined in the Double-entry Ledger & Accounting Spec.

---

### 4.3 Flows

#### Payment success
- cash ↑
- liability ↑

#### Points usage
- liability ↓
- revenue ↑

#### Refund
- cash ↓
- liability ↓

The journal-level posting details for these flows are defined in the Double-entry Ledger & Accounting Spec.

---

## 5. Broker Settlement Layer

### 5.1 Purpose

Support brokerage flows:
- commissions
- split payments
- payout preparation

---

### 5.2 Tables

#### settlement_accounts
#### settlement_instructions
#### payout_requests
#### commission_rules

---

### 5.3 Example Flow

Buyer payment:
- portion → supplier
- portion → broker
- portion → platform

The financial allocation and recognition entries for this flow must be implemented using the double-entry accounting model.

---

## 6. Payment Risk Layer

### 6.1 Purpose

Detect abnormal behavior.

---

### 6.2 Tables

#### payment_risk_flags
#### risk_rule_hits

---

### 6.3 Basic Rules

- duplicate payments
- abnormal frequency
- refund abuse
- repeated webhook anomalies

---

## 7. System Flows

### 7.1 Top-up Flow

1. Payment success
2. financial ledger control record or journal posting
3. liability record or journal posting
4. points credit
5. reconciliation

---

### 7.2 Service Consumption

1. points debit
2. liability decrease
3. revenue increase

---

### 7.3 Refund

1. refund success
2. liability reverse
3. points reversal

All concrete postings must follow the Double-entry Ledger & Accounting Spec.

---

## 8. Integration Requirements

Every module must:
- emit events
- attach reason codes
- write audit records
- support trace_id

Financially relevant modules must also:
- produce reconcilable source references
- preserve journal-posting linkage
- preserve compensation linkage
- preserve settlement allocation linkage

---

## 9. Work Items

### FC-01 Reconciliation Engine
### FC-02 Compensation Engine
### FC-03 Financial Control Ledger Integration
### FC-04 Settlement Layer
### FC-05 Risk Layer
### FC-06 Double-entry Integration Mapping
- map payment flows to journal templates
- map points flows to liability/revenue templates
- map refund flows to reversal templates
- map broker settlement flows to allocation templates
- define control-to-journal reconciliation checks

---

## 10. Final Statement

This document upgrades AegisBroker into a financial control system.

After implementation, the system becomes:
- verifiable
- auditable
- financially consistent
- brokerage-ready

This document is not replaced by the Double-entry Ledger & Accounting Spec.
Instead:
- this document governs control integrity, reconciliation, and compensation
- the accounting spec governs journal structure and accounting correctness
