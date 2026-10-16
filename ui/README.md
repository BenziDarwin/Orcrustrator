# orcrustrator-gui

Visual Studio companion for **Orcrustrator**, a high-performance relational API aggregator written in pure Rust.

## Quickstart

Run directly in any project folder without installing:

```bash
npx orcrustrator-gui@latest
```

Or install globally:

```bash
npm install -g orcrustrator-gui
orcrustrator-gui
```

## Features

- **Interactive DAG Studio**: Drag and connect endpoints (`Root`, `HasOne`, `HasMany`, `IndependentJoin`) with dynamic cubic bezier connectors.
- **Manifest Synchronization**: Live side-by-side view of `definitions.toml` (code-generated) and `overrides.toml` (operator tweaks).
- **Workspace Bridge**: Directly reads and writes `orcrustrator/overrides.toml` in your project without modifying Rust source files.
- **CloudEvents Stream**: Live CNCF CloudEvents v1.0 relational telemetry feed.
- **Engine Benchmarks**: Live benchmark metrics from the Uganda MoH dataset.

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT license

at your option.
