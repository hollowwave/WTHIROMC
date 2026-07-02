# WTHIROMC

**What The Hell Is Running On My Computer**

An open-source desktop assistant that translates what's running on your computer into plain English, so you can tell what's normal and what's worth worrying about — without needing to know what a registry key is.

## Status: v1 in progress (M1–M2 scaffolded)

See [`docs/plan.md`](./docs/plan.md) for the full technical plan, architecture, rule set, and milestone breakdown.

## Architecture

```
Collector (Rust/sysinfo) -> Rule Engine (pure functions) -> Explanation Engine (templates) -> React UI
```

Three independent layers with no shared knowledge of each other's internals — see the plan doc for why.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Node.js](https://nodejs.org/) 18+
- Tauri v2 system dependencies for your OS: see [Tauri's prerequisites guide](https://v2.tauri.app/start/prerequisites/)
- Windows is the primary target platform (this project reads Windows-specific system state)

## Setup

```bash
npm install
npm run tauri dev
```

This launches the app in dev mode with hot reload on both the React frontend and Rust backend.

## Running tests

```bash
cd src-tauri
cargo test
```

Rule engine and explanation engine tests run against synthetic `ProcessFacts` — no real system access or actual malware required. See `tests/rule_engine_tests.rs` for an end-to-end example using a simulated suspicious process.

## Current limitations (v1 scaffold)

- Digital signature verification is stubbed (`is_signed: false` for everything) — real signature checking via Windows APIs is milestone M3.
- Network activity detection is stubbed (`has_network_activity: false`) — same milestone.
- Autostart detection is stubbed (`is_autostart: false`) — wired up in M6 alongside the Startup Persistence Scanner.

These are intentional sequencing choices (see plan doc, section 3) so the pipeline works end-to-end before tackling the harder Windows-API-specific data collection.

## Project structure

```
src/                  React + TypeScript frontend
src-tauri/src/
  collector/          Gathers raw facts about the system
  rules/              Pure functions: facts -> risk score
  explain/            Templates: risk score -> plain English
  types.rs            Shared data model
  commands.rs         Tauri IPC commands exposed to the frontend
src-tauri/tests/      Integration tests
```

## License

TBD
