# 全球商品經紀市場全分類 + 系統商品模組設計（工程級）

> Last Updated: 2026-04-11
> Status: Design Baseline

## Purpose

This document defines the engineering-grade commodity-market design baseline for AegisBroker.

It extends the existing broker operating-system architecture with:

- global commodity market ontology
- broker / trader / exchange role mapping
- normalized product and market modules
- API schema for commodity and venue metadata
- CME / ICE pricing-feed normalization

This document is additive. It does not replace the event, audit, state-machine, or workspace rules already defined for AegisBroker.

## Design Goals

- represent global commodity markets using a consistent ontology
- separate physical brokerage, exchange-traded products, and market-data concerns
- support guest-readable market content and document services
- prepare future RFQ / Quote / Deal flows by commodity family
- preserve traceability and additive evolution

## Global Commodity Ontology

### Level 1: Commodity super-sectors

1. Energy
2. Metals
3. Agriculture
4. Environmental Products
5. Power

### Level 2: Primary sector groupings

#### Energy

- Crude Oil
- Refined Products
- Natural Gas
- LNG
- LPG / NGL
- Coal

#### Metals

- Precious Metals
- Base Metals
- Ferrous Inputs
- Battery / New Energy Materials

#### Agriculture

- Grains
- Oilseeds
- Soft Commodities
- Feed Commodities
- Specialty Agricultural Products

#### Environmental Products

- Carbon Credits
- Renewable Energy Certificates
- Emissions Allowances

#### Power

- Day-ahead Power
- Real-time Power
- Forward Power
- Capacity / Ancillary Products

## Initial AegisBroker Commodity Coverage

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

## Product Modeling Dimensions

Each commodity instrument must be modeled across these dimensions:

- `sector`: Energy / Metals / Agriculture / Environmental / Power
- `commodity_family`: e.g. Crude Oil, Coffee, Gold
- `instrument_type`: spot / physical forward / futures / options / swaps / indices
- `delivery_type`: physical / cash-settled / index-settled
- `venue_type`: OTC brokered / exchange-traded / bilateral / auction
- `pricing_source`: exchange settlement / broker assessed / index provider / negotiated
- `unit_of_measure`: bbl / mmbtu / mt / lb / bu / troy_oz / mwh / tonne_co2e
- `quote_currency`: USD / EUR / JPY / TWD / etc.

## Broker vs Trader vs Exchange Mapping

### Broker

Role:

- intermediates buyers and sellers
- manages RFQ, quote routing, negotiation, and document exchange
- may provide market color and document workflows
- does not become principal unless explicitly modeled

System responsibilities:

- broker coverage registry
- relationship and counterparty mapping
- RFQ routing and quote orchestration
- audit-backed document and workflow operations

### Trader

Role:

- principal participant taking risk or holding inventory
- may be buyer, seller, or internal desk

System responsibilities:

- counterparty identity
- trading permissions
- deal participation
- inventory and settlement linkage in later phases

### Exchange

Role:

- central venue for standardized contracts
- publishes contract definitions, settlements, and reference prices

System responsibilities:

- venue registry
- contract specification metadata
- exchange-linked pricing feeds
- settlement and expiry metadata

### Mapping Model

| Role | Market Type | Primary AegisBroker Concern |
|---|---|---|
| Broker | OTC / brokered physical market | workflow, RFQ, documents, approvals |
| Trader | OTC or listed participant | quotes, deals, counterparty participation |
| Exchange | listed market venue | contract metadata, settlements, prices |
| Clearing / CCP | listed market post-trade | future settlement integration |
| Index / Assessor | OTC reference pricing | reference curves and assessment snapshots |

## Commodity Classification Baseline

### Energy

#### Crude Oil

- WTI
- Brent
- Dubai / Oman
- regional physical grades

#### Natural Gas

- Henry Hub
- TTF
- JKM-linked references
- regional physical gas hubs

#### LNG

- DES / FOB LNG cargoes
- regional LNG index-linked structures

### Metals

#### Precious Metals

- Gold
- Silver
- Platinum
- Palladium

#### Base Metals

- Copper
- Aluminum
- Zinc
- Nickel
- Lead

#### Battery / New Energy Materials

- Lithium
- Cobalt
- Graphite

### Agriculture

#### Grains

- Wheat
- Corn
- Barley

#### Oilseeds

- Soybeans
- Soybean Meal
- Soybean Oil

#### Soft Commodities

- Coffee
- Sugar
- Cocoa
- Cotton

### Environmental Products

- compliance carbon allowances
- voluntary carbon credits
- renewable certificates

### Power

- node/zone power contracts
- peak / off-peak strips
- day-ahead hourly products

## System Module Design

### Core commodity-service modules

1. `commodity_taxonomy_service`
2. `instrument_registry_service`
3. `market_venue_registry_service`
4. `broker_coverage_service`
5. `pricing_source_registry_service`
6. `pricing_feed_ingestion_service`
7. `content_document_link_service`

### Module responsibilities

#### commodity_taxonomy_service

- owns commodity hierarchy
- maps commodity families to sectors and document collections

#### instrument_registry_service

- owns instrument definitions
- models spot / physical forward / futures / options / swaps

#### market_venue_registry_service

- owns exchange / OTC venue definitions
- links instruments to venues

#### broker_coverage_service

- records which broker desks or service lines cover which commodities
- links public content and forms to market coverage

#### pricing_source_registry_service

- describes whether pricing is from CME, ICE, broker assessment, or other sources

#### pricing_feed_ingestion_service

- normalizes snapshot and time-series pricing payloads
- stores source metadata without bypassing event/audit rules when writes become material

## API Schema Baseline

### Commodity taxonomy

```json
{
  "commodity_id": "uuid",
  "sector_code": "energy",
  "family_code": "crude_oil",
  "display_name": "Crude Oil",
  "phase": "phase_1",
  "status": "active"
}
```

### Instrument registry

```json
{
  "instrument_id": "uuid",
  "commodity_id": "uuid",
  "symbol": "CL",
  "instrument_type": "futures",
  "delivery_type": "physical",
  "unit_of_measure": "bbl",
  "quote_currency": "USD",
  "venue_id": "uuid",
  "pricing_source_id": "uuid",
  "status": "active"
}
```

### Venue registry

```json
{
  "venue_id": "uuid",
  "venue_code": "cme_nymex",
  "venue_type": "exchange",
  "display_name": "CME NYMEX",
  "country_code": "US",
  "timezone": "America/Chicago",
  "status": "active"
}
```

### Broker coverage

```json
{
  "coverage_id": "uuid",
  "workspace_id": "uuid",
  "desk_code": "energy_brokerage",
  "commodity_id": "uuid",
  "instrument_scope": [
    "spot",
    "physical_forward",
    "futures_reference"
  ],
  "document_collection_id": "uuid",
  "status": "active"
}
```

### Pricing source registry

```json
{
  "pricing_source_id": "uuid",
  "source_code": "cme_settlement",
  "provider_type": "exchange",
  "venue_id": "uuid",
  "latency_class": "end_of_day",
  "license_scope": "internal_reference",
  "status": "active"
}
```

## Suggested Public APIs

### Read-only taxonomy APIs

- `GET /v1/commodities/sectors`
- `GET /v1/commodities/families`
- `GET /v1/commodities/families/:family_code`
- `GET /v1/instruments`
- `GET /v1/venues`
- `GET /v1/pricing-sources`

### Broker content linkage APIs

- `GET /v1/broker-coverage`
- `GET /v1/broker-coverage/:commodity_id`
- `GET /v1/content/commodities/:family_code`
- `GET /v1/documents/collections?commodity_family=:family_code`

### Future controlled APIs

- `POST /v1/rfqs`
- `POST /v1/quotes`
- `POST /v1/deals`
- `POST /v1/pricing-feed/imports`

These future write APIs must remain event- and audit-backed.

## CME / ICE Pricing Feed Structure

The platform should normalize exchange-linked pricing into a common internal structure regardless of provider.

### Common normalized pricing snapshot

```json
{
  "feed_event_id": "uuid",
  "trace_id": "uuid",
  "provider_code": "cme",
  "venue_code": "cme_nymex",
  "instrument_symbol": "CL",
  "instrument_month": "2026-06",
  "price_type": "settlement",
  "price": "78.25",
  "currency": "USD",
  "unit_of_measure": "bbl",
  "as_of_ts": "2026-04-11T16:00:00Z",
  "source_record_ref": "provider-native-id",
  "ingest_status": "accepted"
}
```

### CME-aligned fields

- exchange: CME / NYMEX / COMEX / CBOT
- contract symbol
- contract month/year
- settlement price
- open/high/low/last if licensed and available
- volume
- open interest
- trading day

### ICE-aligned fields

- exchange: ICE Futures Europe / ICE Futures U.S.
- contract code
- contract month/year
- settlement / official close
- volume
- open interest
- trading date

### Internal feed metadata fields

- `provider_code`
- `venue_code`
- `symbol`
- `contract_period`
- `price_type`
- `as_of_ts`
- `ingest_batch_id`
- `source_record_ref`
- `license_scope`
- `quality_flag`

## Data-Model Tables

Suggested additive schema groups:

- `commodity_sectors`
- `commodity_families`
- `commodity_instruments`
- `market_venues`
- `pricing_sources`
- `broker_coverages`
- `pricing_feed_batches`
- `pricing_feed_snapshots`
- `pricing_feed_quality_events`

## Event and Audit Implications

Read-only public browsing does not require material-write events.

The following actions do require canonical event and audit handling:

- registering a new commodity family
- registering a new instrument
- changing broker coverage
- changing pricing-source licensing scope
- importing authoritative feed snapshots into persistent internal records
- correcting or reversing feed-derived reference values if they influence downstream workflows

Suggested event families:

- `commodity.family.registered`
- `commodity.instrument.registered`
- `broker.coverage.updated`
- `pricing.source.registered`
- `pricing.feed.batch.ingested`
- `pricing.feed.snapshot.corrected`

## Integration with Content and Form Services

Commodity modules must link to:

- public commodity landing pages
- broker service descriptions
- market-specific document collections
- traditional form templates in `library/`

This keeps content, documents, and future transaction workflows aligned to one ontology instead of fragmented per-page logic.

## Out of Scope for This Slice

- live exchange connectivity
- direct execution routing
- clearing and settlement engines
- portfolio risk or margining engines
- destructive migration of existing file libraries

## Design Preservation Note

This file is the engineering-grade baseline for commodity market modeling in AegisBroker. Future schema, APIs, and frontend modules should reference this document before implementing commodity-specific services, exchange-linked data models, or pricing normalization.
