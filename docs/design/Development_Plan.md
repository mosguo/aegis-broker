# AegisBroker Development Plan

> Last Updated: 2026-04-11
> Version: v0.1.4-plan

## Purpose

This plan keeps AegisBroker aligned with its control-oriented architecture while expanding from the core broker operating system into a content-service and commodity-service platform.

The plan preserves these non-negotiable properties:

- event-driven material operations
- explicit state validation
- append-only audit and ledger records
- structured reason codes and trace propagation
- server-side workspace enforcement
- Zeabur-safe deployment and readiness behavior

## Current Foundation

The current repository already includes:

- Rust backend with health and readiness endpoints
- Python frontend aligned to Zeabur deployment
- workspace-aware auth/session skeleton
- event/audit/ledger baseline schema
- guest default-workspace fallback behavior
- content-service design preservation in Markdown
- library-based traditional business document inventory

## Active Design Principles

### Workspace fallback principle

- Guest users may browse public, read-only documents under the seeded default workspace
- when `workspace_id` is omitted, the system falls back to `00000000-0000-0000-0000-000000000001`
- login success and login failure paths must both log `workspace_id`
- guest flows must not require users to manually input a UUID before login

### Content-service principle

- public broker content may be exposed as read-only guest content
- controlled publication and restricted downloads must remain event- and audit-backed
- traditional forms remain file-backed assets in `library/`
- architecture and preserved design decisions live only in `docs/design/`

## Current Scope

### Platform scope

- backend health/readiness and deployment safety
- workspace and session baseline
- Google OAuth integration
- diagnostic-oriented frontend console
- content-service architecture preservation
- traditional form-library integration

### Frontend scope

- Zeabur-style operational shell
- compact status indicators for backend/database/login
- reserved future-content area
- connection/session diagnostics
- content-service and form-service navigation integration

### Backend scope

- auth/session flows
- workspace validation and default fallback
- health and readiness verification
- DB diagnostics with trace-aware error logging
- additive content/document service rollout preparation

## 2026-04-11 Content Service Architecture

- Add a brokerage-style public content layer modeled on broker information architecture
- Integrate the existing `library/` traditional form libraries as a first-class document-service domain
- Treat guest public-document browsing as default-workspace readonly access
- Keep controlled publication, restricted download, and form-request flows inside event and audit boundaries
- Align frontend navigation to Overview, Content Services, Forms Service, Connection/Session, and Event Output
- Preserve the design in `Content_Service_Architecture.md` before deeper schema and API rollout

## Commodity Rollout Phases

### Phase 1

- 原油
- 黃金
- 天然氣
- 小麥
- 咖啡

### Phase 2

- 銅
- 大豆
- 糖
- LNG

### Phase 3

- 鋰（新能源）
- 碳權（Carbon credits）
- 電力

The corresponding rollout placeholders are maintained under:

- `library/10_商品服務路線圖`

## New Design Baseline: Global Commodity Broker Ontology

This plan now includes the engineering-level commodity-market module design in:

- `Commodity_Market_Ontology_and_Module_Design.md`

That design expands the platform with:

- global commodity classification ontology
- broker vs trader vs exchange role mapping
- normalized commodity and market module boundaries
- API schema for instrument, venue, and feed metadata
- CME / ICE-aligned pricing feed normalization

## Delivery Roadmap

### Stage 1: Stabilize design and repository paths

- enforce `docs/design` as the only design-document root
- enforce `library` as the only business document and form-library root
- preserve design decisions in Markdown before schema rollout

### Stage 2: Content and document metadata services

- ingest `library/` assets into metadata registries
- expose read-only public content/document browsing APIs
- add taxonomy links for commodity, market, and document collections

### Stage 3: Commodity service modules

- implement commodity ontology tables and API contracts
- register broker coverage by product and market
- add instrument, venue, and pricing-source metadata
- normalize CME / ICE pricing snapshots

### Stage 4: Controlled workflow integration

- controlled document request flows
- publication and operator release flows
- event/audit-backed restricted distribution

### Stage 5: Trading-domain expansion

- RFQ / Quote / Deal alignment by commodity family
- feed-aware pricing references
- exchange and OTC coverage modeling
- future reconciliation to payment and ledger flows where applicable

## Success Criteria

- all design documents resolve from `docs/design`
- all traditional forms and business assets resolve from `library`
- commodity rollout phases are explicitly represented in both plan and library structure
- broker content, forms, and commodity modules share one coherent control-oriented architecture
- future API and schema work can be implemented without bypassing event, audit, or workspace rules
