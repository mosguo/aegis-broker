# AegisBroker — Double-entry Ledger & Accounting Spec
Version: v0.9.0  
Status: Accounting Layer Blueprint  
Target deployment: ZEABUR  
Related documents:
- Core Event & Audit Architecture Spec & WorkItem List
- PostgreSQL Schema & Migration Spec
- Rust API Contract Spec
- ZEABUR Deployment & Repo Blueprint
- Financial Control & Reconciliation Spec

---

## 0. Document Purpose

This document adds the accounting layer required to turn AegisBroker from:
- payment-capable
- ledger-aware
- financially controlled

into:
- accounting-structured
- journal-balanced
- broker-settlement-ready
- audit-friendly at accounting depth

This document defines:
- chart of accounts strategy
- double-entry journal model
- posting templates
- payment / points / refund / settlement journal mappings
- revenue recognition timing
- commission recognition direction
- accounting-safe reversal policy
- accounting linkage to events, audit, and trace

This document does NOT replace Financial Control & Reconciliation Spec.
Instead:
- Financial Control & Reconciliation Spec governs control flow, reconciliation, compensation, and anomaly handling
- this document governs accounting correctness and journal structure

---

## 1. Accounting Principles

### 1.1 Double-entry is mandatory for financial ledger truth

Every financially material value movement must be representable as balanced journal entries.

Each journal must satisfy:

total debits = total credits

No canonical financial posting may exist as a single-sided mutable record.

---

### 1.2 Control layer and accounting layer are distinct

The system has:
- control layer
- accounting layer

Control layer answers:
- did the correct flow happen
- was the payment verified
- was reconciliation complete
- was compensation required

Accounting layer answers:
- what accounts changed
- were debits and credits balanced
- when was liability created or released
- when was revenue recognized
- how was broker commission accounted

---

### 1.3 Points are not cash

Points are not cash assets.
Points represent stored value, deferred service rights, platform liability, promotional value, or other internal entitlement classes depending on source and policy.

Accordingly:
- payment success does not mean immediate revenue recognition in all cases
- point grant may increase liability or promotional expense depending on issuance type
- point consumption may reduce liability and increase revenue, or consume promotional reserve, depending on source class

---

### 1.4 Reversal over destructive mutation

Accounting corrections must be implemented as:
- reversing journals
- adjusting journals
- compensating journals

Never by deleting posted accounting history.

---

### 1.5 Journal entries must remain linkable

Every journal entry must preserve linkage to:
- workspace_id
- source_type
- source_id
- trace_id
- event_id optional
- audit_id optional
- payment_order_id optional
- ledger entry / points linkage where relevant

---

## 2. Accounting Layer Scope

This document governs accounting treatment for at least:

- payment top-up
- payment failure and cancellation
- refund
- point credit after payment
- point debit during service use
- promotional point grant
- manual point adjustment
- broker commission accrual
- broker or supplier settlement allocation
- pending payout obligations
- escrow-like or clearing-style holding if introduced later

---

## 3. Chart of Accounts Strategy

### 3.1 Account structure

Use a flexible chart of accounts table rather than hardcoded enums.

Suggested top-level account classes:
- asset
- liability
- equity
- revenue
- expense
- contra

Suggested account number ranges:

- 1000–1999 Assets
- 2000–2999 Liabilities
- 3000–3999 Equity
- 4000–4999 Revenue
- 5000–5999 Expenses
- 6000–6999 Contra / adjustments optional

---

### 3.2 Minimum starter accounts

#### Assets
- 1100 Cash - Stripe Clearing
- 1110 Cash - Bank / Treasury
- 1120 Accounts Receivable optional
- 1130 Settlement Receivable optional

#### Liabilities
- 2100 Customer Stored Value Liability
- 2110 Unearned Revenue
- 2120 Refund Payable
- 2130 Broker Commission Payable
- 2140 Supplier Payable
- 2150 Payment Exception Suspense
- 2160 Promotional Points Reserve optional

#### Revenue
- 4100 Service Revenue
- 4110 Platform Fee Revenue
- 4120 Broker Commission Revenue if platform earns direct commission
- 4130 Breakage Revenue optional and policy-controlled

#### Expenses
- 5100 Payment Gateway Fees
- 5110 Promotional Points Expense
- 5120 Refund Expense / adjustment optional
- 5130 Settlement Loss / write-off optional

#### Contra / technical
- 6100 Reconciliation Adjustment Suspense
- 6110 Manual Accounting Adjustment Suspense

---

### 3.3 Workspace and global account strategy

Recommended:
- global chart of accounts templates
- workspace-scoped usage where postings are workspace-specific

This allows a shared accounting model while preserving tenant boundaries.

---

## 4. Core Accounting Tables

### 4.1 accounts

Fields:
- id
- account_code
- account_name
- account_class
- normal_side (debit / credit)
- parent_account_id optional
- is_active
- created_at
- updated_at

---

### 4.2 journals

Fields:
- id
- workspace_id
- journal_number
- journal_type
- journal_status (draft / posted / reversed / superseded)
- source_type
- source_id
- trace_id
- event_id optional
- audit_id optional
- description
- occurred_at
- posted_at
- reversal_of_journal_id optional
- created_at
- updated_at

Rules:
- only posted journals affect accounting truth
- draft journals must not affect account balances
- posted journals must be immutable except status transitions to reversed through controlled flow

---

### 4.3 journal_lines

Fields:
- id
- journal_id
- line_no
- account_id
- direction (debit / credit)
- amount_minor
- currency
- description optional
- linked_payment_order_id optional
- linked_point_ledger_entry_id optional
- linked_settlement_id optional
- metadata_json
- created_at

Rules:
- each journal must have at least 2 lines
- per currency, total debits must equal total credits
- amount_minor must be positive
- direction carries sign semantics, not negative amount values

---

### 4.4 accounting_periods optional but recommended

Fields:
- id
- period_code
- starts_at
- ends_at
- status (open / soft_closed / closed)
- created_at
- updated_at

Used later for close controls.

---

### 4.5 posting_templates

Fields:
- id
- template_code
- template_name
- source_type
- description
- template_json
- is_active
- created_at
- updated_at

This allows deterministic posting logic from operational events.

---

## 5. Journal Posting Rules

### 5.1 General rule

Every materially financial event should map to:
- 1 operational event
- 1 or more accounting journal postings where financially relevant

Not every event becomes a journal, but every financial truth change must.

---

### 5.2 Posting timing rule

A journal should be posted when the financial meaning becomes sufficiently certain by policy.

Examples:
- payment intent creation does not necessarily justify full accounting posting
- payment confirmed / webhook verified may justify posting cash and liability
- point usage may justify posting liability release and revenue recognition
- refund completion may justify refund payable release and cash reduction

---

### 5.3 Atomicity expectation

Where feasible:
- operational state update
- event write
- audit write
- journal post marker

should remain linked in one transactional boundary or a durable compensatable sequence.

---

## 6. Payment Journal Templates

### 6.1 Customer pays for stored value / point top-up

When webhook-verified payment succeeds and points represent stored value:

Journal:
- Dr 1100 Cash - Stripe Clearing
- Cr 2100 Customer Stored Value Liability

This means:
- money entered platform control
- platform now owes stored service value

Point credit itself does not create another revenue journal if the payment was already accounted as stored-value liability.

---

### 6.2 Payment gateway fee recognition

When Stripe fee becomes known and recognized:

Journal:
- Dr 5100 Payment Gateway Fees
- Cr 1100 Cash - Stripe Clearing

Alternative implementations may use a payable/clearing subaccount if fee timing differs.

---

### 6.3 Payment failure

If payment fails before recognized success:
- no final success journal should be posted

If a suspense/pre-authorization journal was used:
- reverse according to policy

---

### 6.4 Payment cancellation

If canceled before success recognition:
- no cash/liability posting should remain outstanding

---

## 7. Points Journal Templates

### 7.1 Point credit after successful paid top-up

Operationally:
- points ledger increases

Accounting:
- no additional journal may be needed if liability was already recognized at payment success and point credit is just entitlement issuance under the same stored-value policy

But if accounting policy distinguishes payment receipt from entitlement release, then a controlled mapping may be introduced. The default recommended path is:
- payment success posts liability
- point ledger credit remains operational, not separate financial posting

---

### 7.2 Promotional point grant

If points are granted without customer payment:

Possible journal:
- Dr 5110 Promotional Points Expense
- Cr 2160 Promotional Points Reserve
or
- Cr 2100 Customer Stored Value Liability

Final choice depends on policy. The important rule is:
- free/promotional value must not be mistaken for paid cash receipt

---

### 7.3 Point consumption for paid service

When user spends stored-value points for platform service:

Journal:
- Dr 2100 Customer Stored Value Liability
- Cr 4100 Service Revenue

This means:
- obligation to deliver value decreases
- revenue is recognized

---

### 7.4 Point debit for non-revenue internal consumption

If debit relates to non-revenue internal offsets or penalties, use different accounts according to policy rather than forcing all debits into service revenue.

---

### 7.5 Point reversal

If a debit must be reversed due to failed downstream service:

Journal:
- reverse the original revenue/liability movement
or
- post compensating adjustment entries if already period-controlled

Operationally:
- create reversal point ledger entry
Accounting:
- create reversing journal

---

## 8. Refund Journal Templates

### 8.1 Refund before stored value consumed

If payment is refunded and the related stored value remains unused:

Journal:
- Dr 2100 Customer Stored Value Liability
- Cr 1100 Cash - Stripe Clearing

This means:
- obligation removed
- cash returned

Operationally:
- points reversal should also occur if points were already credited

---

### 8.2 Refund after partial consumption

If customer already consumed part of the value:
- refund may be partial
- accounting must reflect only remaining liability being refunded

Possible journal:
- Dr 2100 Customer Stored Value Liability (remaining refundable portion)
- Cr 1100 Cash - Stripe Clearing

If any exceptional expense is recognized, that should be separate and policy-driven.

---

### 8.3 Refund payable intermediate model

If refund is approved before cash actually leaves:

Journal at approval:
- Dr 2100 Customer Stored Value Liability
- Cr 2120 Refund Payable

Journal at cash out:
- Dr 2120 Refund Payable
- Cr 1100 Cash - Stripe Clearing

This model is recommended when operational timing between approval and payout matters.

---

## 9. Broker Settlement and Commission Templates

### 9.1 Platform earns broker/platform fee

If platform fee becomes earned at settlement-ready stage:

Journal:
- Dr 1130 Settlement Receivable or 1100 Cash - Stripe Clearing
- Cr 4110 Platform Fee Revenue

Use receivable if cash is not yet actually received.

---

### 9.2 Broker commission payable

If the system must track broker-entitled payout:

At accrual:
- Dr appropriate clearing / settlement source
- Cr 2130 Broker Commission Payable

At payout:
- Dr 2130 Broker Commission Payable
- Cr 1110 Cash - Bank / Treasury

---

### 9.3 Supplier payable

If buyer payment includes supplier-bound funds:

At recognition:
- Dr 1100 Cash - Stripe Clearing or settlement clearing
- Cr 2140 Supplier Payable

At payout:
- Dr 2140 Supplier Payable
- Cr 1110 Cash - Bank / Treasury

---

### 9.4 Escrow-like or clearing-style holding

If later introduced, use explicit clearing / holding liability accounts rather than collapsing everything into revenue or cash.

---

## 10. Revenue Recognition Rules

### 10.1 Default rule for stored-value top-up

Payment receipt does not automatically equal revenue recognition.

Default:
- payment success creates liability
- service usage recognizes revenue

---

### 10.2 Immediate revenue cases

If a payment is for an immediately consumed one-off fee rather than stored value:

Journal:
- Dr 1100 Cash - Stripe Clearing
- Cr 4100 Service Revenue

Only use this when the purchased value is not future-stored entitlement.

---

### 10.3 Breakage revenue optional and policy-sensitive

Unspent points or expired stored value may eventually become breakage revenue, but this should be policy-controlled, jurisdiction-sensitive, and not auto-enabled by default.

If implemented:
- Dr 2100 Customer Stored Value Liability
- Cr 4130 Breakage Revenue

This must require explicit business and legal approval.

---

## 11. Reconciliation Relationship to Accounting

The Financial Control & Reconciliation Spec defines reconciliation as a control requirement.  
This accounting spec adds what must be checked at accounting depth.

Reconciliation should verify:
- payment success has correct posted journal
- posted journal is balanced
- related point ledger actions are consistent with accounting state
- refunds have reversing journals
- no duplicate financial recognition occurred for duplicate webhook delivery
- broker settlement accrual and payout remain consistent

---

## 12. Accounting-safe Relationship to Points Ledger

### 12.1 Operational vs accounting truth

Points ledger answers:
- how many points a user can use
- where those points came from
- what debits/credits/reversals happened operationally

Double-entry ledger answers:
- what obligations and revenues were recognized
- what cash moved
- what payables or receivables exist

These systems must be linked, not merged blindly.

---

### 12.2 Linkage expectations

A point ledger entry should be linkable to:
- payment_order_id if funded by payment
- journal_id optional or derived linkage
- source_type / source_id
- trace_id

A journal line may optionally reference:
- linked_point_ledger_entry_id
- linked_payment_order_id
- linked_settlement_id

---

## 13. Posting Templates by Source Type

Suggested posting templates:
- PAYMENT_TOPUP_SUCCESS
- PAYMENT_GATEWAY_FEE
- POINTS_CONSUMED_SERVICE_REVENUE
- PROMOTIONAL_POINTS_GRANTED
- PAYMENT_REFUND_APPROVED
- PAYMENT_REFUND_COMPLETED
- BROKER_COMMISSION_ACCRUED
- BROKER_COMMISSION_PAID
- SUPPLIER_PAYABLE_ACCRUED
- SUPPLIER_PAYABLE_PAID
- MANUAL_ACCOUNTING_ADJUSTMENT
- REVERSAL_GENERIC

Each template should define:
- source_type
- journal_type
- required inputs
- account mappings
- line generation rules
- reversal strategy

---

## 14. Manual Adjustments

Manual accounting adjustments must:
- require privileged role
- require reason code
- require justification text
- write canonical events
- write audit records
- post a journal in a dedicated adjustment class
- avoid rewriting prior posted journals

Recommended account:
- 6110 Manual Accounting Adjustment Suspense until properly classified if immediate classification is not safe

---

## 15. Accounting-related Reason Code Families

This spec may introduce or reserve:
- AC-POST-001 Journal posted successfully
- AC-POST-002 Journal posting blocked by validation
- AC-REV-001 Reversing journal created
- AC-ADJ-001 Manual accounting adjustment approved
- AC-RECO-001 Accounting reconciliation matched
- AC-RECO-002 Accounting reconciliation mismatch
- AC-REVN-001 Revenue recognized on consumption
- AC-LIAB-001 Liability created on top-up
- AC-RFND-001 Refund payable created
- AC-RFND-002 Refund cash completed
- AC-COMM-001 Broker commission accrued
- AC-COMM-002 Broker commission paid

These can coexist with operational payment and ledger codes.

---

## 16. Suggested PostgreSQL Additions

The PostgreSQL Schema & Migration Spec should later be extended with:

### accounts
- id uuid pk
- account_code text unique not null
- account_name text not null
- account_class text not null
- normal_side text not null
- parent_account_id uuid null
- is_active boolean not null default true
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

### journals
- id uuid pk
- workspace_id uuid not null
- journal_number text not null
- journal_type text not null
- journal_status text not null
- source_type text not null
- source_id uuid null
- trace_id uuid null
- event_id uuid null
- audit_id uuid null
- description text null
- occurred_at timestamptz not null
- posted_at timestamptz null
- reversal_of_journal_id uuid null
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

### journal_lines
- id uuid pk
- journal_id uuid not null
- line_no integer not null
- account_id uuid not null
- direction text not null
- amount_minor bigint not null
- currency text not null
- description text null
- linked_payment_order_id uuid null
- linked_point_ledger_entry_id uuid null
- linked_settlement_id uuid null
- metadata_json jsonb not null default '{}'::jsonb
- created_at timestamptz not null default now()

### posting_templates
- id uuid pk
- template_code text unique not null
- template_name text not null
- source_type text not null
- template_json jsonb not null
- is_active boolean not null default true
- created_at timestamptz not null default now()
- updated_at timestamptz not null default now()

Optional:
- accounting_periods

---

## 17. Suggested Rust API / Service Implications

The Rust API Contract Spec should later be extended with:
- accounting posting service
- journal read endpoints for admin/finance roles
- manual accounting adjustment command
- accounting reconciliation view
- posting template registry service
- optional accounting close controls later

Recommended internal services:
- `AccountingPostingService`
- `JournalRepository`
- `PostingTemplateResolver`
- `RevenueRecognitionService`
- `SettlementAccountingService`

These services should remain internal at first unless admin accounting APIs are needed.

---

## 18. Suggested Work Items

### DE-01 Chart of Accounts Foundation
- define starter account catalog
- design accounts table
- seed accounts

### DE-02 Journal Infrastructure
- design journals and journal_lines tables
- implement posting invariants
- implement balanced-journal validation
- implement immutable posting rules

### DE-03 Posting Template Engine
- define template catalog
- implement source-to-template mapping
- implement journal generation logic

### DE-04 Payment Accounting
- map payment success to cash/liability
- map gateway fees
- map refunds
- map suspense / exception handling

### DE-05 Points Accounting
- map promotional grants
- map stored-value consumption
- map reversals
- map breakage policy placeholder

### DE-06 Broker Settlement Accounting
- map platform fee revenue
- map broker commission payable
- map supplier payable
- map payout release

### DE-07 Accounting Reconciliation Extension
- compare journals to payment orders and point ledger entries
- surface accounting mismatches
- define recovery / manual review flows

---

## 19. Relationship to Financial Control Spec

This document must be used together with Financial Control & Reconciliation Spec.

Correct interpretation:
- Financial Control Spec tells the system when a financial action is valid, reconciled, compensatable, and operationally complete.
- Double-entry Ledger & Accounting Spec tells the system how to express that action in balanced accounting language.

Therefore:
- this document does not replace Financial Control Spec
- Financial Control Spec does not replace this document
- both are required for fintech-grade maturity

---

## 20. Final Statement

This document upgrades AegisBroker from:
- control-correct financial behavior

to:
- accounting-structured financial behavior

After implementation, AegisBroker will have:
- operationally correct payment control
- balanced journal-based accounting
- liability-aware stored-value treatment
- revenue recognition tied to actual consumption or policy-defined earning events
- broker settlement and commission accounting readiness

This is the accounting layer required to move the system toward financial-institution-grade internal rigor.
