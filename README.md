# Orcrustrator

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.1-orange.svg)](https://crates.io/crates/orcrustrator)
[![Documentation](https://docs.rs/orcrustrator/badge.svg)](https://docs.rs/orcrustrator)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A modular, high-performance **API aggregator, relational service orchestrator, and client tool** written in pure Rust.

Orcrustrator is dedicated entirely to **runtime data orchestration**—with zero forced databases, zero server bloat, and pure in-memory execution. It enables developers to pull data across diverse authentication protocols (OAuth2, OAuth1, Basic, Bearer, API Key, No-Auth), inspect raw data schemas (JSON, XML, text), establish complex relational links (`HasOne`, `HasMany`, independent sibling joins) across multiple endpoints using Object-Oriented principles, and execute requests concurrently using a Tokio-powered Directed Acyclic Graph (DAG) engine.

---

## Features

- **Multi-Auth Engine**: First-class support for Basic Auth, Bearer Token, API Key (Header/Query), OAuth 1.0a (HMAC-SHA1), and in-memory volatile OAuth 2.0 with automatic token caching and renewal.
- **Dynamic Format & Schema Inspection**: Sniffs `Content-Type` and raw payloads for JSON, XML, or plain text, normalizing them into a unified AST (`StructuredValue`) with live schema type inference.
- **Relational Endpoint Orchestration**:
  - `HasOne` (1:1): Dynamic dependent fetching where parent attributes parameterize child requests.
  - `HasMany` (1:N): Resolves child lists or multiple entity references into nested child collections.
  - `IndependentJoin`: Concurrent fan-out retrieval stitched in RAM based on matching keys.
- **Fearless Concurrency & DAG Execution**: Independent root queries and child fan-out requests execute concurrently across Tokio tasks, bounding latency to the slowest single endpoint.
- **Full HTTP Lifecycle (GET, POST, PUT, PATCH, DELETE)**:
  - Sequential and parallel mutation pipelines with input payload mapping and parameter forwarding between steps.
  - Compensating rollback actions (Saga pattern) for automatic cleanup on failure.
- **Pluggable Storage Extensibility**: Includes the `DataSink` trait and universal Serde `Serialize`/`Deserialize` support with `.into_typed::<T>()` so users can easily persist outputs to PostgreSQL, SQLite, Redis, S3, or MongoDB.
- **Crate-Friendly & Lightweight**: Pure library-first design with modular Cargo feature flags.

---

## Installation

Add Orcrustrator to your `Cargo.toml`:

```toml
[dependencies]
orcrustrator = "0.1.1"
```

### Feature Flags

- `xml` *(default)*: Enables XML format parsing via `quick-xml`.
- `oauth` *(default)*: Enables OAuth 1.0a and OAuth 2.0 client credentials flows.
- `cli`: Enables the `orcrustrator` command-line executable.

---

## Quickstart

### 1. Relational Read Orchestration (GET)

```rust
use orcrustrator::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Build a relational endpoint hierarchy
    let user_relational = RelationalEndpointBuilder::new(
        "/users/{id}/full",
        "user",
        Endpoint::get("/users/{id}")
    )
    // 1:1 relation: fetch company using user.company_id and embed as "company"
    .has_one("company", Endpoint::get("/companies/{company_id}"), "company_id", "company_id", "company")
    // 1:N relation: fetch orders using user.id and embed as "orders"
    .has_many("orders", Endpoint::get("/orders?userId={id}"), "id", "id", "orders")
    .build();

    // 2. Configure the Service mesh
    let service = Service::builder("https://api.example.com")
        .auth(AuthStrategy::bearer("my-secret-token"))
        .register_endpoint(user_relational)
        .build();

    // 3. Execute orchestration in memory
    let unified_result = service.handle_request(HttpMethod::Get, "/users/42/full", None).await?;

    println!("Unified JSON:\n{}", unified_result.to_json_string_pretty()?);
    Ok(())
}
```

### 2. Strongly-Typed Extraction

```rust
#[derive(serde::Deserialize)]
struct FullUserProfile {
    id: u64,
    name: String,
    company: CompanyInfo,
    orders: Vec<OrderInfo>,
}

let profile: FullUserProfile = unified_result.into_typed()?;
```

### 3. Mutation Pipelines with Rollback (POST / PUT / PATCH / DELETE)

```rust
use orcrustrator::prelude::*;

let onboard_pipeline = MutationEndpointBuilder::post("/api/v1/onboard")
    // Step 1: Create Organization
    .step(
        MutationStep::new("create_org", Endpoint::post("/orgs"))
            .map_body("name", "$.company_name")
    )
    // Compensating rollback for Step 1 if Step 2 fails
    .rollback(
        "create_org",
        Endpoint::delete("/orgs/{id}").resolve_url(...)
    )
    // Step 2: Create User linked to Organization
    .step(
        MutationStep::new("create_user", Endpoint::post("/users"))
            .map_body("name", "$.user_name")
            .map_body("org_id", "$steps.create_org.id")
    )
    .map_response("user", "$steps.create_user")
    .map_response("organization", "$steps.create_org")
    .build();
```

### 4. Pluggable Storage Hooks (`DataSink`)

```rust
use orcrustrator::prelude::*;

struct DatabaseAuditSink;

#[async_trait::async_trait]
impl DataSink for DatabaseAuditSink {
    async fn on_data(
        &self,
        endpoint_path: &str,
        method: HttpMethod,
        params: &HashMap<String, String>,
        data: &StructuredValue,
    ) -> Result<()> {
        println!("Persisting payload for {} {} to database...", method, endpoint_path);
        // Save to PostgreSQL / MongoDB / Redis / S3
        Ok(())
    }
}

let service = Service::builder("https://api.example.com")
    .sink(DatabaseAuditSink)
    .build();
```

---

## CLI Usage

When installed with the `cli` feature:

```bash
# Inspect an upstream API and infer its schema structure
orcrustrator inspect --url https://jsonplaceholder.typicode.com --endpoint /users/1

# Run an in-memory orchestration pipeline directly
orcrustrator orchestrate --upstream https://jsonplaceholder.typicode.com --id 1
```

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
