# AegisBroker Content Service Architecture

> Last Updated: 2026-04-11
> Status: Draft for implementation

## Purpose

This document defines the additive content-service architecture for AegisBroker.
It integrates:

- a brokerage-style public-facing content structure inspired by MedAgri Brokers
- the repository's existing traditional business form libraries under `Docs/`
- the current workspace, event, audit, and ZEABUR deployment constraints

The goal is to let guest users browse public, read-only brokerage content and document libraries under the default workspace scope, while preserving the control architecture for any managed publication, controlled download, or workflow escalation.

## Reference Style Baseline

The MedAgri Brokers website presents a classic broker-information structure centered on:

- company/about information
- broker and advisory services
- commodity categories
- team/contact details

Reference:

- [MedAgri Brokers](https://medagribrokers.com/)

This is a style and information-architecture reference only. AegisBroker should not copy exact wording or branding. Instead, it should reuse the structural pattern:

1. public introduction
2. services and capabilities
3. commodity or market coverage
4. forms and documents
5. contact / inquiry entry points

## Existing Internal Document Libraries

The repository already contains a substantial traditional-form inventory under [Docs](/C:/Users/mos/Documents/GitHub/aegis-broker/Docs):

- `00_總覽`: 2 files
- `01_咖啡豆市場`: 35 files
- `02_原油市場`: 35 files
- `03_金屬市場`: 35 files
- `99_共用主檔與制度文件`: 5 files

These files are the initial source set for the new content-service layer.

### Current market-based form groupings

Each major market currently follows a repeatable hierarchy:

- `01_核心文件`
- `02_擴充文件`
- `03_進階文件`

This makes the library suitable for a unified document-service abstraction rather than one-off file pages.

## Architectural Principles

The content-service architecture must remain compatible with the repository control model:

- public read-only browsing may use the default workspace scope
- controlled publication must not bypass event and audit requirements
- document metadata changes must remain additive and replayable
- server-side workspace scope must remain authoritative
- controlled download requests and form workflow escalations must be traceable

## User Modes

### 1. Guest mode

Guest users may:

- browse public brokerage pages
- browse public market content
- browse public document indexes
- download public document assets

Guest requests should resolve to the default workspace:

- `00000000-0000-0000-0000-000000000001`

### 2. Authenticated member mode

Authenticated users may additionally:

- view restricted document categories
- request controlled forms
- submit workflow-bound document requests
- load profile-bound service context

### 3. Operator / publisher mode

Privileged users may:

- publish or unpublish content items
- register new form templates
- revise document metadata
- approve controlled distribution or release

These operations must emit events and tamper-evident audit records.

## Frontend Information Architecture

The frontend menu should evolve into five integrated content-service groups:

1. Overview
2. Content Services
3. Forms Service
4. Connection / Session
5. Event Output

### Overview

Purpose:

- landing context
- broker identity
- high-level status indicators

### Content Services

Purpose:

- public broker profile
- service capability pages
- market and commodity coverage
- public content landing blocks

Suggested subsections:

- About AegisBroker
- Brokerage Services
- Commodity Markets
- Market Intelligence
- Contact / Inquiry

### Forms Service

Purpose:

- traditional business form catalog
- market-specific file browsing
- public document index
- controlled document request entry point

Suggested subsections:

- Coffee document library
- Crude oil document library
- Metals document library
- Shared governance library
- Public document downloads
- Controlled form request queue

### Connection / Session

Purpose:

- backend endpoint
- guest workspace resolution
- session token diagnostics
- operator troubleshooting only

### Event Output

Purpose:

- request diagnostics
- trace output
- content-service operation logs during rollout

## Content Domain Model

The new content-service layer should be implemented as additive modules.

### Public content aggregates

- `content_spaces`
- `content_items`
- `content_item_versions`
- `content_publications`
- `content_navigation_nodes`

### Document and form aggregates

- `document_collections`
- `document_assets`
- `document_asset_versions`
- `form_templates`
- `form_template_releases`
- `document_access_policies`
- `document_download_requests`

### Taxonomy and mapping

- `market_taxonomy`
- `commodity_taxonomy`
- `content_item_taxonomy_links`
- `document_collection_taxonomy_links`

## Proposed Storage Mapping

### Source-of-record file mapping

The current filesystem library in [Docs](/C:/Users/mos/Documents/GitHub/aegis-broker/Docs) should be ingested into metadata records with these concepts:

- market
- document family
- file category
- source filename
- version label
- publication visibility
- workspace scope

### Visibility model

- `public_readonly`
- `workspace_member`
- `operator_only`
- `approval_required`

## Backend Service Responsibilities

### Content query service

Read-only APIs for:

- public content landing pages
- service descriptions
- commodity listings
- document indexes
- file metadata

### Publication control service

Material write APIs for:

- publish
- unpublish
- revise metadata
- create new version
- change visibility

These writes must produce canonical events and audit entries.

### Form request service

Material write APIs for:

- request controlled form
- request restricted document download
- initiate broker-support workflow

These writes must preserve:

- trace_id
- workspace scope
- reason code
- requester identity or guest channel

## API Outline

Suggested additive API groups:

### Public content

- `GET /v1/content/navigation`
- `GET /v1/content/pages/:slug`
- `GET /v1/content/services`
- `GET /v1/content/markets`
- `GET /v1/content/commodities`

### Public document browsing

- `GET /v1/documents/collections`
- `GET /v1/documents/collections/:collection_id`
- `GET /v1/documents/assets/:asset_id`

### Controlled document interactions

- `POST /v1/documents/download-requests`
- `POST /v1/forms/request`
- `POST /v1/content/publications`
- `PUT /v1/content/publications/:publication_id`

## Event and Audit Requirements

The following operations are materially relevant and must not bypass event/audit controls:

- publishing a content item
- changing a document visibility rule
- releasing a new form template version
- approving a restricted document request
- revoking a previously published controlled asset

Suggested event families:

- `content.item.created`
- `content.item.published`
- `content.item.unpublished`
- `document.asset.registered`
- `document.asset.version_released`
- `document.download.requested`
- `form.template.requested`
- `form.template.approved`

## ZEABUR-Oriented Rollout Sequence

1. Add content-service architecture document
2. Align frontend menu and placeholders
3. Build document-library metadata ingestion
4. Add read-only public content/document APIs
5. Add public document browsing UI
6. Add controlled document request workflows
7. Add publication and audit-backed operator flows

## Initial Implementation Scope

The first implementation slice should deliver:

- guest-visible public content navigation
- market-based document-library index pages
- file metadata registry for current `Docs/` assets
- default-workspace guest access
- Markdown-preserved architecture and menu design

The first slice should not yet attempt:

- full OCR or document parsing
- destructive migration of the existing `Docs/` library
- bypassing event/audit for controlled publication

## Menu-to-Service Integration Mapping

| Menu Group | Content Service Responsibility | Initial Data Source |
|---|---|---|
| Overview | Broker summary, system status, platform introduction | frontend runtime config + curated content |
| Content Services | About, services, commodities, contact | managed content items |
| Forms Service | Traditional forms, market libraries, public documents | `Docs/` folder metadata |
| Connection / Session | backend, workspace, session diagnostics | runtime config + auth/session APIs |
| Event Output | trace and API diagnostics | frontend debug output |

## Design Preservation Note

This document is the Markdown preservation layer for the content-service design. Future schema, API, and UI changes should reference this file when integrating public content, traditional forms, and document service modules into the AegisBroker platform.
