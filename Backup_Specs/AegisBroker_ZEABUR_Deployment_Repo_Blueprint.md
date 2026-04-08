# AegisBroker — ZEABUR Deployment & Repo Blueprint
Version: v1.0  
Status: 100%-path final blueprint  
Target platform: ZEABUR  
Primary stack: Python Frontend + Rust Backend + PostgreSQL + Object Storage  
Related documents:
- Core Event & Audit Architecture Spec & WorkItem List
- PostgreSQL Schema & Migration Spec
- Rust API Contract Spec

This document defines the repository blueprint, deployment topology, build and startup order, environment variable contract, health checks, service boundaries, Stripe payment deployment requirements, internal points service runtime requirements, and Zeabur-specific operational guidance needed to implement a working repository that can be deployed on Zeabur.

---

## 0. Completion Standard

This document is written to close the remaining gap to the project’s 100% definition:

100% means that, with the complete document set handed over, a competent engineer can independently build a working ZEABUR-compatible software repository without needing to redesign core architecture, database, API, payment model, or deployment topology.

This blueprint therefore focuses on:
- repository layout
- service split
- configuration contract
- build and runtime expectations
- startup and migration order
- health and readiness behavior
- secret management expectations
- environment variable naming
- deployment path for Stripe-backed Payment Service and internal points service

---

## 1. Deployment Topology on Zeabur

Recommended initial topology uses three primary runtime services and one managed data service:

1. `frontend-python`
   - Python web frontend
   - serves broker UI
   - talks only to backend-rust API

2. `backend-rust`
   - main Axum API service
   - contains domain services for RFQ, quotes, deals, shipments, settlements, payment service, and points service
   - exposes health endpoints
   - receives Stripe webhook callback

3. `worker-rust` optional but recommended
   - asynchronous/background runner for reminder jobs, payment reconciliation jobs, quote expiry jobs, settlement overdue jobs, and seal generation
   - can initially be merged into backend-rust if simplicity is preferred

4. `postgres`
   - Zeabur PostgreSQL service

Optional later:
- object storage service or S3-compatible external bucket
- observability exporter sidecar if needed

### 1.1 Initial practical recommendation

For fastest first release on Zeabur:
- deploy `frontend-python`
- deploy `backend-rust`
- provision `postgres`
- keep worker logic inside backend-rust scheduled tasks if load is low
- split worker-rust later when background throughput grows

---

## 2. Repository Blueprint

Recommended repository layout:

```text
aegisbroker/
├─ README.md
├─ .gitignore
├─ docs/
│  ├─ architecture/
│  │  ├─ Core_Event_Audit_Architecture_Spec_WorkItem_List.md
│  │  ├─ PostgreSQL_Schema_Migration_Spec.md
│  │  ├─ Rust_API_Contract_Spec.md
│  │  └─ ZEABUR_Deployment_Repo_Blueprint.md
│  ├─ adr/
│  └─ api/
├─ infra/
│  ├─ env/
│  │  ├─ backend.example.env
│  │  ├─ frontend.example.env
│  │  └─ worker.example.env
│  ├─ docker/
│  │  ├─ backend.Dockerfile
│  │  ├─ frontend.Dockerfile
│  │  └─ worker.Dockerfile
│  ├─ migrations/
│  ├─ scripts/
│  │  ├─ wait_for_db.sh
│  │  ├─ run_migrations.sh
│  │  └─ bootstrap_seed.sh
│  └─ compose/
│     └─ docker-compose.local.yml
├─ backend-rust/
│  ├─ Cargo.toml
│  ├─ Cargo.lock
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ app/
│  │  ├─ config/
│  │  ├─ middleware/
│  │  ├─ telemetry/
│  │  ├─ auth/
│  │  ├─ api/
│  │  ├─ domain/
│  │  ├─ dto/
│  │  ├─ models/
│  │  ├─ services/
│  │  ├─ repositories/
│  │  └─ errors/
│  ├─ migrations/          # optional mirror or symlink strategy if SQLx migrate is local here
│  └─ tests/
├─ frontend-python/
│  ├─ requirements.txt
│  ├─ app/
│  ├─ pages/
│  ├─ components/
│  ├─ services/
│  └─ main.py
└─ storage/
   └─ .gitkeep
```

### 2.1 Repository ownership rule

- `backend-rust` owns business logic, payment logic, state transitions, events, audit writes, Stripe integration, and points ledger mutations.
- `frontend-python` owns UI orchestration only.
- `infra` owns deployment and migration artifacts.
- `docs` owns the authoritative written blueprint.

---

## 3. Service Boundary Blueprint

### 3.1 backend-rust responsibilities

- session-backed API
- OAuth callback handling
- domain commands
- SQLx database access
- event_store writes
- audit_chain writes
- state transition validation
- Stripe PaymentIntent creation
- Stripe webhook verification and processing
- point ledger mutation service
- replay and audit verification endpoints
- health and readiness endpoints

### 3.2 frontend-python responsibilities

- broker dashboard
- RFQ / quote / negotiation / deal UI
- shipment / exception / settlement UI
- payment-order creation UI
- payment checkout client flow using returned client secret or redirect token
- point account and ledger UI
- admin pages for reasons, transitions, and audit views later

### 3.3 worker-rust responsibilities if split out

- scheduled quote expiry
- scheduled settlement overdue checks
- payment reconciliation sweeps
- audit seal generation
- retry-safe background tasks
- notification fan-out later

### 3.4 Why Payment Service stays inside backend-rust first

The payment feature is internally coupled to:
- state machine validation
- event and audit writes
- point ledger credits
- webhook idempotency
- workspace resolution

For first release, keeping it inside backend-rust reduces failure modes and simplifies Zeabur deployment. It can be extracted later into a dedicated payment microservice if needed.

---

## 4. Build and Runtime Baseline

### 4.1 Rust backend

Recommended crates and runtime:
- axum
- tokio
- sqlx with postgres
- serde
- tracing
- tracing-subscriber
- opentelemetry and related bridge crates
- async-stripe
- tower and tower-http
- chrono
- uuid
- anyhow / thiserror or equivalent

### 4.2 Python frontend

Recommended baseline:
- NiceGUI or Reflex
- httpx for backend API calls
- pydantic optional for client-side models

### 4.3 Rust worker

If separate, use same base stack as backend-rust but without HTTP handlers except optional health endpoint.

---

## 5. Zeabur Deployment Model

### 5.1 Rust application deployment on Zeabur

Zeabur supports Rust applications and can automatically detect executables. If the project contains multiple executables, the executable name can be specified using service configuration such as `ZBPACK_RUST_ENTRY`. Zeabur also performs health checks before routing production traffic to new deployments. citeturn276165search4turn276165search1

### 5.2 Recommended Zeabur service setup

Create services:
- `backend-rust`
- `frontend-python`
- `postgres`

Optional:
- `worker-rust`

Set each service’s environment variables in Zeabur’s Configuration panel. Zeabur supports environment variables at the service level and also supports redeploy/rollback patterns around health-check success. citeturn276165search7turn276165search10

### 5.3 Health checks

Zeabur health checks should point to:
- backend-rust: `/health/ready`
- frontend-python: `/health` or lightweight frontend-ready endpoint
- worker-rust optional: `/health/ready` if HTTP enabled, otherwise no external route

A deployment should only become active once readiness passes. Zeabur’s health-check gating is important because failed deployments do not replace the prior healthy version. citeturn276165search1turn276165search10

---

## 6. Environment Variable Contract

### 6.1 Shared backend variables

Required:
- `APP_ENV`
- `APP_BASE_URL`
- `RUST_LOG`
- `PORT`
- `DATABASE_URL`
- `SESSION_SECRET`
- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`
- `GOOGLE_REDIRECT_URI`
- `BACKEND_PUBLIC_BASE_URL`
- `FRONTEND_PUBLIC_BASE_URL`
- `AUDIT_SEAL_ENABLED`
- `AUDIT_SEAL_INTERVAL_SECONDS`
- `TRACE_MODE`
- `TRACE_EXPORTER_OTLP_ENDPOINT` optional
- `OBJECT_STORAGE_MODE`
- `OBJECT_STORAGE_BUCKET`
- `OBJECT_STORAGE_ENDPOINT` optional
- `OBJECT_STORAGE_ACCESS_KEY` optional
- `OBJECT_STORAGE_SECRET_KEY` optional

### 6.2 Stripe variables

Required when payment enabled:
- `STRIPE_SECRET_KEY`
- `STRIPE_WEBHOOK_SECRET`
- `STRIPE_PUBLISHABLE_KEY`
- `STRIPE_CURRENCY_DEFAULT`
- `STRIPE_API_MODE` (`test` or `live`)
- `PAYMENT_SUCCESS_URL`
- `PAYMENT_CANCEL_URL`

Optional:
- `STRIPE_ACCOUNT_ID` if using connected-account patterns later
- `PAYMENT_RECONCILIATION_ENABLED`
- `PAYMENT_RECONCILIATION_INTERVAL_SECONDS`

### 6.3 Points and service variables

Optional but recommended:
- `POINTS_SERVICE_ENABLED`
- `POINTS_DEFAULT_ZERO_BALANCE_ALLOWED` (expected false)
- `POINTS_MANUAL_ADJUSTMENT_REQUIRES_FINANCE_ADMIN`
- `POINTS_DEBIT_STRICT_MODE`
- `SERVICE_USAGE_BILLING_ENABLED`

### 6.4 Frontend variables

- `FRONTEND_ENV`
- `BACKEND_API_BASE_URL`
- `GOOGLE_CLIENT_ID`
- `STRIPE_PUBLISHABLE_KEY`

### 6.5 Worker variables

If separate:
- all DB and trace variables
- payment reconciliation variables
- audit seal variables
- workflow scheduler variables

---

## 7. Migration and Startup Order

### 7.1 Required startup order

The system must follow this startup order:

1. PostgreSQL becomes available
2. migrations run successfully
3. seed roles and reason codes complete
4. backend-rust starts
5. backend-rust readiness endpoint passes
6. frontend-python starts or becomes routable
7. optional worker-rust starts
8. Stripe webhook endpoint is registered against backend public URL

### 7.2 Migration execution strategy

Preferred first-release strategy:
- backend-rust binary supports a `migrate` subcommand or startup mode
- Zeabur release command or pre-start command runs migrations
- app start is blocked until migration success

Alternative:
- dedicated migration job service

### 7.3 Write safety rule

backend-rust must not accept write traffic before:
- migrations complete
- seed data exists
- DB connection established
- control tables reachable
- readiness check passes

---

## 8. Backend Runtime Contract

### 8.1 backend-rust executable responsibilities

The executable should:
- load configuration
- initialize tracing
- connect to PostgreSQL
- optionally verify migration version
- initialize repositories and services
- build Axum router
- expose `/health/live` and `/health/ready`
- bind to `0.0.0.0:${PORT}`

### 8.2 Recommended command modes

Support command modes:
- `serve`
- `migrate`
- `seed-reasons`
- `seed-roles`
- `verify-audit`
- `reconcile-payments` optional

Examples:
```bash
backend-rust migrate
backend-rust serve
```

### 8.3 Readiness logic

`/health/ready` should fail if:
- DB unavailable
- schema version too old
- required env vars for enabled modules missing
- Stripe config missing while payment enabled
- seed baseline missing for roles/reasons

---

## 9. Stripe Payment Deployment Blueprint

### 9.1 Core gateway choice

Stripe’s PaymentIntents API is the correct baseline for this service because Stripe recommends one PaymentIntent for each order or payment session, and the PaymentIntent holds the lifecycle of customer payment attempts. Metadata can be used to store internal references, and webhook signature verification depends on the webhook secret. citeturn276165search0turn276165search3turn276165search6

### 9.2 Internal service design

The deployed Rust backend should expose:
- PaymentOrder creation endpoint
- PaymentIntent creation endpoint
- Stripe webhook endpoint
- payment reconciliation endpoint

Internal sequence:
1. create internal PaymentOrder
2. call Stripe to create PaymentIntent
3. persist external identifiers internally
4. return client secret or redirect data to frontend
5. receive Stripe webhook
6. verify webhook signature
7. mutate internal PaymentOrder state
8. if configured, credit points through immutable ledger
9. write canonical events and audit records
10. allow replay and verification later

### 9.3 Webhook routing on Zeabur

Webhook public endpoint:
- `POST /api/v1/payments/webhooks/stripe`

Requirements:
- backend public domain must be stable
- HTTPS required
- raw request body preserved
- webhook secret set in backend env vars

### 9.4 Failure policy

If Stripe reports success but internal point credit fails:
- internal state must capture reconciliation-needed condition
- do not silently acknowledge as fully completed
- queue or schedule reconciliation job
- preserve trace and audit evidence

### 9.5 Recommended Rust integration crate

`async-stripe` is a suitable Rust crate for the Stripe HTTP API and is actively kept current on docs.rs. citeturn276165search2

---

## 10. Internal Points Service Deployment Blueprint

### 10.1 Runtime location

For first release, the points service lives inside backend-rust because it is tightly coupled to:
- user/workspace auth
- event writing
- audit writing
- payment reconciliation
- service-usage authorization

### 10.2 Internal services to implement

Inside backend-rust:
- `PaymentService`
- `StripeGatewayClient`
- `PaymentWebhookService`
- `PaymentReconciliationService`
- `PointAccountService`
- `PointLedgerService`
- `ServiceUsageBillingService`

### 10.3 Add-point flow

Example:
- user purchases points package
- Stripe succeeds
- webhook verified
- PaymentOrder transitions to SUCCEEDED
- PointLedgerEntry created with credit
- PointAccount current balance checkpoint updated
- audit and event written

### 10.4 Deduct-point flow

Example:
- user invokes premium service
- backend checks available balance
- if sufficient, create debit ledger entry and update checkpoint
- if insufficient, reject request with structured error
- write event and audit if policy requires

### 10.5 Reversal flow

If downstream service fails after debit:
- create reversal ledger entry
- never delete original debit
- preserve event and audit linkage

---

## 11. Local Development and Zeabur Parity

### 11.1 Local parity goal

The repo should support local development close to Zeabur runtime behavior using Docker Compose.

### 11.2 Local compose services

Suggested local compose:
- postgres
- backend-rust
- frontend-python
- optional mail/mock service later

### 11.3 Local Stripe testing

Use Stripe test keys locally and on Zeabur staging.
Webhook testing can use Stripe CLI locally, but deployed Zeabur environments should use a real Stripe webhook endpoint configured in the Stripe dashboard.

---

## 12. Docker Blueprint

### 12.1 backend-rust Dockerfile direction

Use multi-stage build:
1. Rust builder image
2. slim runtime image

Requirements:
- compile release binary
- copy binary only
- optionally copy migrations if runtime migration mode used
- expose `${PORT}`

### 12.2 frontend-python Dockerfile direction

Requirements:
- install dependencies
- copy app
- bind to `0.0.0.0:${PORT}`

### 12.3 worker-rust Dockerfile direction

Same pattern as backend-rust but different command entrypoint.

---

## 13. Zeabur Service Configuration Blueprint

### 13.1 backend-rust

Build source:
- `backend-rust/`

Start command example:
```bash
./backend-rust serve
```

Health check path:
- `/health/ready`

Watch paths:
- `backend-rust/**`
- `infra/**`
- `docs/**` optional for awareness but not necessary for deploy trigger

### 13.2 frontend-python

Build source:
- `frontend-python/`

Start command example:
```bash
python main.py
```

Health check path:
- `/health`

### 13.3 worker-rust optional

Build source:
- `backend-rust/` or dedicated worker crate
- custom entry command:
```bash
./backend-rust worker
```

### 13.4 postgres

Use Zeabur managed database service.

---

## 14. Secret Management Blueprint

Store only in Zeabur configuration:
- Google OAuth secrets
- Stripe secret key
- Stripe webhook secret
- session secret
- DB URL if externally managed
- object storage credentials

Do not commit secrets to repo.  
Provide only `.example.env` files with placeholders.

---

## 15. Health Checks and Monitoring Blueprint

### 15.1 Health endpoints

backend-rust:
- `/health/live`
- `/health/ready`

frontend-python:
- `/health`

worker-rust optional:
- `/health/live`
- `/health/ready`

### 15.2 Logging

Use structured logs with:
- timestamp
- level
- service_name
- trace_id
- operation_name
- aggregate_type
- aggregate_id
- error_code when present

### 15.3 Metrics later

Optional future metrics:
- payment order success/failure count
- webhook duplicate count
- point debit rejection count
- audit verification failure count
- quote send latency
- reconciliation backlog count

---

## 16. Rollback and Deployment Safety

Zeabur health checks help prevent a bad deployment from replacing a healthy one. Rollback strategy should therefore rely on:
- immutable deploy artifacts
- health-check gating
- non-destructive schema changes
- additive migrations wherever possible

Do not deploy destructive schema changes together with application code expecting new runtime behavior unless backward compatibility is preserved. Zeabur rollbacks retain current environment-variable settings, so deployment docs should note that a code rollback does not automatically roll back secret changes. citeturn276165search1turn276165search10

---

## 17. Repository Bootstrap Sequence

A competent engineer should bootstrap in this order:

1. create repository structure
2. add docs from the four blueprint files
3. implement backend-rust config and app skeleton
4. implement PostgreSQL migrations from schema spec
5. implement health endpoints
6. implement auth/session baseline
7. implement event and audit infrastructure
8. implement RFQ/Quote/Deal base endpoints
9. implement PaymentOrder and Stripe integration
10. implement PointAccount and PointLedger service
11. wire frontend forms and dashboard pages
12. configure Zeabur services and env vars
13. run migrations in staging
14. register Stripe webhook
15. verify readiness and end-to-end payment + points flow

---

## 18. Suggested Implementation Milestones

### Milestone 1
backend-rust + postgres + migrations + health + auth

### Milestone 2
RFQ / quote / negotiation / deal core flow

### Milestone 3
event store + audit chain + replay baseline

### Milestone 4
shipment / exception / settlement flow

### Milestone 5
Stripe-backed Payment Service + PaymentOrder flow

### Milestone 6
internal points ledger + add/deduct service

### Milestone 7
reconciliation + verification + Zeabur production hardening

---

## 19. Files That Should Exist in the Final Repo

At minimum:

- `README.md`
- `backend-rust/Cargo.toml`
- `backend-rust/src/main.rs`
- `backend-rust/src/config/mod.rs`
- `backend-rust/src/api/health.rs`
- `backend-rust/src/api/auth.rs`
- `backend-rust/src/api/payment_orders.rs`
- `backend-rust/src/api/payments_webhook.rs`
- `backend-rust/src/api/points.rs`
- `backend-rust/src/services/payment_service.rs`
- `backend-rust/src/services/stripe_gateway.rs`
- `backend-rust/src/services/point_ledger_service.rs`
- `backend-rust/src/services/payment_reconciliation_service.rs`
- `backend-rust/src/domain/events/*`
- `backend-rust/src/domain/audit/*`
- `backend-rust/src/domain/state_machine/*`
- `backend-rust/src/repositories/*`
- `backend-rust/migrations/*.sql`
- `frontend-python/main.py`
- `frontend-python/services/backend_client.py`
- `infra/env/backend.example.env`
- `infra/env/frontend.example.env`
- `infra/docker/backend.Dockerfile`
- `infra/docker/frontend.Dockerfile`
- `docs/architecture/*.md`

---

## 20. Final Completion Assessment

With the prior three documents plus this Zeabur blueprint, the design package reaches the project’s 100% standard:

- control architecture defined
- schema and migration plan defined
- Rust API contract defined
- Zeabur repository and deployment blueprint defined
- payment service deployment path defined
- internal points add/deduct design defined

At this point, the remaining work is implementation, not architectural invention.

---

## 21. Final Statement

This ZEABUR Deployment & Repo Blueprint closes the final planning gap and turns AegisBroker into a fully specified repository-and-deployment design.

The resulting system is specified as:
- a Python frontend
- a Rust Axum backend
- a PostgreSQL-backed event/audit/state system
- a Stripe-backed Payment Service
- an internal immutable points ledger
- a Zeabur-ready deployment topology with health-checked rollout

This document therefore serves as the final deployment blueprint binding the other three documents into an implementable ZEABUR-compatible software repository.
