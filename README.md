# Orcrustrator

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.1-orange.svg)](https://crates.io/crates/orcrustrator)
[![Documentation](https://docs.rs/orcrustrator/badge.svg)](https://docs.rs/orcrustrator)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A modular, high-performance API aggregator, relational service orchestrator, and client tool written in pure Rust.

Orcrustrator is dedicated entirely to runtime data orchestration: zero forced databases, zero server bloat, and pure in-memory execution. It enables developers to pull data across diverse authentication protocols (OAuth2, OAuth1, Basic, Bearer, API Key, No-Auth), inspect raw data schemas (JSON, XML, text), establish complex relational links (`HasOne`, `HasMany`, independent sibling joins) across multiple endpoints using Object-Oriented principles, and execute requests concurrently using a Tokio-powered Directed Acyclic Graph (DAG) engine.

---

## Visual Dashboard and GUI Notice

Launch the companion visual studio instantly in any project directory with:

```bash
npx orcrustrator-gui@latest
```

Alternatively, clone the repository to run or customize the studio locally:

```bash
git clone https://github.com/BenziDarwin/Orcrustrator.git
cd Orcrustrator/ui
npm install
npm run dev
```

Orcrustrator maintains a strict library-first architecture: the core crate never spins up background HTTP listening servers or runs heavy daemon web servers. The visual dashboard interacts with Orcrustrator through decoupled file manifests (`orcrustrator/overrides.toml`) and binary relational event streams.

---

## Features

- **Multi-Auth Engine**: First-class support for Basic Auth, Bearer Token, API Key (Header/Query), OAuth 1.0a (HMAC-SHA1), and in-memory volatile OAuth 2.0 with automatic token caching and renewal.
- **Dynamic Format and Schema Inspection**: Inspects `Content-Type` headers and raw payloads for JSON, XML, or plain text, normalizing them into a unified AST (`StructuredValue`) with live schema type inference.
- **Relational Endpoint Orchestration**:
  - `HasOne` (1:1): Dynamic dependent fetching where parent attributes parameterize child requests.
  - `HasMany` (1:N): Resolves child lists or multiple entity references into nested child collections.
  - `IndependentJoin`: Concurrent fan-out retrieval stitched in RAM based on matching keys.
- **Fearless Concurrency and DAG Execution**: Independent root queries and child fan-out requests execute concurrently across Tokio tasks, bounding latency to the slowest single endpoint.
- **Full HTTP Lifecycle (GET, POST, PUT, PATCH, DELETE)**:
  - Sequential and parallel mutation pipelines with input payload mapping and parameter forwarding between steps.
  - Compensating rollback actions (Saga pattern) for automatic cleanup on failure.
- **Code-Driven Relationship Compiler (`orcrustrator compile`)**:
  - Static AST discovery of `relation!` declarations across Rust source trees using `syn`.
  - Kahn's algorithm cycle detection guaranteeing Directed Acyclic Graphs (DAG).
  - Clean separation: code-defined relations live in `definitions.toml`, operator GUI tweaks live in `overrides.toml`.
- **CNCF CloudEvents v1.0 Binary Event Streaming**:
  - Outbound relational event publication encoded in compact MessagePack.
  - RFC-standard 4-byte Big-Endian length-prefixed framing over pluggable transports (in-memory Tokio broadcast channels, OS IPC, or external brokers like NATS).
- **Pluggable Read-Through Caching (`CacheProvider`)**:
  - High-performance thread-safe `MemoryCache` with automatic TTL eviction.
  - Transparent read-through cache checks on GET requests and automated cache invalidation on mutations.
- **Pluggable Storage Extensibility**: Includes the `DataSink` trait and universal Serde `Serialize`/`Deserialize` support with `.into_typed::<T>()` for easy persistence to PostgreSQL, SQLite, Redis, S3, or MongoDB.

---

## Installation

Add Orcrustrator to your `Cargo.toml`:

```toml
[dependencies]
orcrustrator = { version = "0.1.1", features = ["compiler", "events"] }
```

### Feature Flags

- `xml` (default): Enables XML format parsing via `quick-xml`.
- `oauth` (default): Enables OAuth 1.0a and OAuth 2.0 client credentials flows.
- `compiler`: Enables source code relationship scanning, AST parsing, cycle validation, and manifest synchronization (`definitions.toml`).
- `events`: Enables CNCF CloudEvents v1.0 MessagePack binary event streaming and RFC length-prefixed framing.
- `cli`: Enables the `orcrustrator` command-line executable.

---

## Architecture: Code Manifest vs GUI Overrides

Orcrustrator enforces clean decoupling between developer-defined code relationships and operator overrides:

```
your-project/
├── src/
│   └── main.rs               # Rust source code declaring relation! macros
├── orcrustrator/
│   ├── definitions.toml      # Generated and owned by 'orcrustrator compile'
│   ├── overrides.toml        # Managed by GUI or field operators
│   └── config.toml           # Project settings
└── Cargo.toml
```

- The compiler owns `definitions.toml`. When you run `orcrustrator compile`, it scans your Rust files and generates canonical TOML declarations.
- The GUI owns `overrides.toml`. Operator modifications (such as rerouting an endpoint target or adjusting cache TTL) are preserved across recompilations.
- At runtime, Orcrustrator merges both files into an `EffectiveConfiguration`. Compilation never erases GUI changes.

### Declaring Relations in Rust

```rust
use orcrustrator::relation;

relation! {
    id: "patient_terminology_link",
    source: "patient_registry",
    target: "terminology_service",
    relation_type: "references",
    foreign_key: "concept_id",
    embed_as: "concept",
}
```

---

## Command-Line Usage

Install Orcrustrator CLI with:

```bash
cargo install --path . --features "cli,compiler,events"
```

### Compiler Commands

```bash
# Scan Rust source tree and update definitions.toml atomically
orcrustrator compile

# Check if definitions.toml is up to date without modifying files
orcrustrator compile --check

# Display additions, modifications, and removals against definitions.toml
orcrustrator compile --diff

# Inspect the effective runtime configuration after merging definitions and overrides
orcrustrator inspect --effective
```

---

## Pluggable Read-Through Caching

Wrap endpoints or services with a cache provider to accelerate reads and automatically invalidate stale records on mutation:

```rust
use orcrustrator::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // In-memory cache with a 5-minute default TTL
    let cache = Arc::new(MemoryCache::with_default_ttl(Duration::from_secs(300)));

    let service = Service::builder("https://api.example.com")
        .auth(AuthStrategy::bearer("secret-token"))
        .cache(cache)
        .build();

    // First call fetches from upstream and populates cache
    let result1 = service.handle_request(HttpMethod::Get, "/catalog/items", None).await?;

    // Second call resolves instantly from RAM cache in 111 nanoseconds
    let result2 = service.handle_request(HttpMethod::Get, "/catalog/items", None).await?;

    Ok(())
}
```

---

## CNCF CloudEvents Binary Streaming

Stream relational events across external processes or brokers without running an internal web server:

```rust
use orcrustrator::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let transport = Arc::new(ChannelTransport::new(1024));
    let mut rx = transport.subscribe();

    let service = Service::builder("https://api.example.com")
        .event_transport(transport.clone())
        .build();

    // Listen for CloudEvents in a background worker
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            println!("Received CloudEvent ID: {} | Operation: {}", event.id, event.operation);
            
            // Frame event into RFC 4-byte Big-Endian MessagePack bytes for network transport
            let wire_bytes = encode_frame(&event).unwrap();
            // Publish wire_bytes to NATS, Kafka, IPC socket, or file log
        }
    });

    Ok(())
}
```

---

## Empirical Benchmarks (Uganda MoH Live Dataset)

Performance measured on the official Uganda Ministry of Health (MoH) National Health Products dataset (6,857 records, 100 test runs, release build):

### 1. Read-Through Caching: Before vs After

| Metric | Before (Cold Re-evaluation) | After (MemoryCache Warm) | Acceleration |
|---|---|---|---|
| Mean Latency | 309 ns | 111 ns | 2.79x faster |
| Throughput | 3,226,847 req/sec | 9,002,520 req/sec | +179% throughput |
| Allocation | String formatting | Zero allocation RAM read | Pure lock-free cache |

### 2. Binary Streaming: MessagePack vs JSON

| Metric | Standard JSON | MessagePack + 4-Byte Framing | Improvement |
|---|---|---|---|
| Serialization Rate | 653,641 events/sec | 1,079,290 events/sec | 1.65x faster |
| Average Event Size | 605 bytes | 540 bytes | 10.74% smaller payload |
| Frame Decoding Rate | 412,000 events/sec | 1,050,425 frames/sec | 2.55x faster |

### 3. Core Relational Engine

| Pipeline Stage | Mean Latency | Operations per Second |
|---|---|---|
| URL Template Interpolation | 599.42 ns | 1,668,268 ops/sec |
| HasOne (1:1) In-Memory Join | 64.00 ns | 15,396,458 joins/sec |
| HasMany (1:N) Child Aggregation | 56.00 ns | 17,546,938 aggregations/sec |
| Schema Type Inference | 630.00 ns | 1,591,089 inferences/sec |
| DataSink Hook Dispatch | 25.73 ns | 38,871,180 dispatches/sec |
| Parallel Worker Fan-Out (8 threads) | 5.82 ms / 6,857 rows | 1,178,522 rows/sec |

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
