# AegisBroker Development Plan

> Last Updated: 2026-04-12
> Version: v0.1.5-plan

## Purpose

This plan keeps AegisBroker aligned with its control-oriented architecture while expanding from the core broker operating system into a content-service, commodity-service, and BrokerAI-assisted operating platform.

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
- Google OAuth login flow with callback redirect back to the console
- event, audit, and ledger baseline schema
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

### BrokerAI interaction principle

- the interface should evolve toward a ChatGPT-style conversation-first operating shell
- BrokerAI becomes the primary interaction layer for content queries, document discovery, and service guidance
- structured outputs must appear as expandable result modules inside or beside the main conversation flow
- future broker operations must be attachable without replacing the core conversation layout

## Current Scope

### Platform scope

- backend health, readiness, and deployment safety
- workspace and session baseline
- Google OAuth integration
- diagnostic-oriented frontend console
- content-service architecture preservation
- traditional form-library integration

### Frontend scope

- chat-style operating shell with BrokerAI as the main interaction point
- compact status indicators for backend, database, and login
- left navigation modeled on a conversation-first workflow
- connection/session diagnostics
- content-service and form-service navigation integration

### Backend scope

- auth and session flows
- workspace validation and default fallback
- health and readiness verification
- DB diagnostics with trace-aware error logging
- additive content and document service rollout preparation

## 2026-04-12 UI and Navigation Baseline

The frontend information architecture is now planned as a ChatGPT-style navigation and conversation system.

### Left sidebar: primary navigation

- New Conversation
- Commodity Categories
- Recent Conversations
- Saved Queries

### Left sidebar: work navigation

- Forms Service
- Process Documents
- Operation Logs
- Currently Available Services

### Left sidebar: user operations

- hidden overflow menu at the bottom-left
- account profile
- workspace
- session
- login/logout
- future preference controls

### Top-right indicators

The only persistent system chrome retained from the current console is:

- backend status indicator
- database status indicator
- login status indicator

These remain compact light indicators with tooltips.

## Main Conversation Area Design Principle

The main area must no longer behave like a fixed dashboard.
It should behave like a conversation-first control surface with highly extensible structured result blocks.

### Core structure

- central BrokerAI conversation stream
- optional right-side structured result panel
- input area as the single operational entry point

### Required extensibility

The conversation area must remain ready to support future attachments such as:

- price matching
- commodity advertising placement
- current market commodity search results
- order-entry or market execution panels
- document previews
- workflow tracking
- audit and operation result cards

### Suggested result module types

- `market_search_result`
- `pricing_feed_snapshot`
- `order_entry_panel`
- `price_match_panel`
- `ad_campaign_panel`
- `document_preview_panel`
- `workflow_status_panel`
- `audit_event_panel`
- `service_availability_matrix`

These modules should be additive and independently renderable inside the conversation response area.

## Currently Available Services Module

A new module named `Currently Available Services` must be introduced in the left work navigation.

### Purpose

This module provides a unified service availability matrix for all current and planned AegisBroker capabilities.

It must help users understand:

- which services are currently supported
- which services are planned
- which system roles may access each service
- what permission or operational restrictions exist
- when future services are expected to be supported

### Query behavior

When the user clicks `Currently Available Services`, the main conversation area should generate a single BrokerAI result response that summarizes all planned and available services in one query result.

### Minimum data columns

- service name
- service category
- supported roles
- service status
- permission restrictions
- access entry point
- planned availability time
- implementation note

### Recommended status values

- `online`
- `limited`
- `planned`
- `internal_only`
- `deprecated`

### Recommended role values

- `guest`
- `workspace_member`
- `workspace_admin`
- `operator`

### Recommended display behavior

The module should follow a ChatGPT-style interaction pattern:

1. BrokerAI posts a summary response in the main conversation area
2. the response includes a service-availability summary card
3. the summary card expands into a structured matrix
4. clicking a service opens a detailed result block in the conversation area or the right result panel

### Recommended summary layer

The first visible layer should summarize:

- count of online services
- count of planned services
- count of role-restricted services
- nearest upcoming service milestones

### Recommended detail layer

The detail layer should show the service matrix and highlight intersections between:

- service role support
- current availability
- permission restriction

This is the minimum readable way to explain what is usable, by whom, and under which constraints.

## 2026-04-11 Content Service Architecture

- add a brokerage-style public content layer modeled on broker information architecture
- integrate the existing `library/` traditional form libraries as a first-class document-service domain
- treat guest public-document browsing as default-workspace read-only access
- keep controlled publication, restricted download, and form-request flows inside event and audit boundaries
- align frontend navigation to conversation-first browsing and document-service entry points
- preserve the design in `Content_Service_Architecture.md` before deeper schema and API rollout

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
- CME and ICE-aligned pricing feed normalization

## Delivery Roadmap

### Stage 1: Stabilize design and repository paths

- enforce `docs/design` as the only design-document root
- enforce `library` as the only business document and form-library root
- preserve design decisions in Markdown before schema rollout

### Stage 2: Conversation-first frontend shell

- replace dashboard-first layout with conversation-first BrokerAI layout
- implement chat-style left sidebar navigation
- implement bottom-left hidden user operation menu
- preserve compact top-right system indicators

### Stage 3: Service availability and content discovery

- implement the `Currently Available Services` query module
- expose service availability summary and matrix rendering
- add queryable content and document browsing summaries for BrokerAI

### Stage 4: Content and document metadata services

- ingest `library/` assets into metadata registries
- expose read-only public content and document browsing APIs
- add taxonomy links for commodity, market, and document collections

### Stage 5: Commodity service modules

- implement commodity ontology tables and API contracts
- register broker coverage by product and market
- add instrument, venue, and pricing-source metadata
- normalize CME and ICE pricing snapshots

### Stage 6: Controlled workflow integration

- controlled document request flows
- publication and operator release flows
- event and audit-backed restricted distribution

### Stage 7: Trading-domain expansion

- RFQ, Quote, and Deal alignment by commodity family
- feed-aware pricing references
- exchange and OTC coverage modeling
- future reconciliation to payment and ledger flows where applicable
- attachable modules for price matching, advertising placement, and order-entry surfaces

## Success Criteria

- all design documents resolve from `docs/design`
- all traditional forms and business assets resolve from `library`
- commodity rollout phases are explicitly represented in both plan and library structure
- BrokerAI becomes the primary interaction shell without weakening event, audit, or workspace rules
- the `Currently Available Services` module can explain service status, role support, restrictions, and planning in one query result
- broker content, forms, and commodity modules share one coherent control-oriented architecture
- future API and schema work can be implemented without bypassing event, audit, or workspace rules
