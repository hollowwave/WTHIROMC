![banner](assets/banner.png)

# WTHIROMC
**What The Hell Is Running On My Computer**

An open-source desktop assistant that translates what's running on your computer and what's set to run automatically into plain English, so you can tell what's normal and what's worth worrying about, without needing to know what a registry key is.

> Cybersecurity tools tell you what happened. WTHIROMC tells you what it means.

## What it does

- **Running Processes**: lists everything currently running, scores each against a set of heuristics (unsigned binaries, suspicious install locations, unknown publishers reaching the network, etc.), and explains *why* something is flagged in plain English instead of raw technical output.

- **Startup Items**: scans registry Run keys, the Startup folder, and (non-Microsoft) scheduled tasks for anything set to launch automatically, with the same scoring and explanation treatment.

- **Digital signature verification**: checks real Windows Authenticode signatures rather than guessing, and shows the actual publisher name where available.

- **"Mark as safe" allowlist**: if you've confirmed something is fine (a niche tool, something you built yourself), mark it safe once and WTHIROMC stops flagging it on future scans. Stored locally in a SQLite database under `%APPDATA%\WTHIROMC\`, never uploaded anywhere.

- **"New since last time" detection**: WTHIROMC remembers what it's seen running/autostarting across previous launches and flags anything genuinely new with a small "New" tag, a real signal for catching recent compromise, independent of the risk score. (Nothing gets flagged as new on your very first-ever launch, since there's no prior baseline to compare against yet.)

- **Export results**: export the current process or startup list as JSON or CSV, e.g. to share with someone helping you investigate.

- **"What to do next" guidance**: Red/Black findings show concrete next steps (check VirusTotal, run a full antivirus scan, how to disable a startup item yourself). WTHIROMC never takes these actions itself, see [`CONTRIBUTING.md`](./CONTRIBUTING.md)'s ethics section.

It is **not** an antivirus replacement. It doesn't remove anything or claim certainty, it surfaces signals and explains them, and leaves the judgment call to you.

## Architecture

```
Collector (Rust) -> Rule Engine (pure functions) -> Explanation Engine (templates) -> React UI
```

Two parallel pipelines share this shape: one for running processes, one for startup/persistence entries with independently tunable rule sets (see `rules::process_rules` vs `rules::persistence_rules`) but shared infrastructure for signature checking and template rendering.

Three layers, deliberately decoupled:
- **`collector`** gathers raw facts. No judgment calls.
- **`rules`** scores facts against pure, unit-testable heuristics. No knowledge of where facts came from or how they'll be displayed.
- **`explain`** turns a score into a sentence a non-expert can read. This is the one seam designed for an LLM later; see `explain::explain()` / `explain::explain_persistence()`.

## Keyboard shortcuts

- `Tab`: switch between Running Processes and Startup Items
- `Escape`: clear the current selection

## Install (for non-developers)
Grab the latest installer from the [Releases page](../../releases) no Rust, Node, or build tools required.

**Verify your download** before running it: every release includes a `SHA256SUMS.txt`. See [`SECURITY.md`](./SECURITY.md#verifying-release-downloads) for verification steps. The installer is currently unsigned, so Windows SmartScreen may warn about it, checksum verification confirms the file matches what CI built from the public source.

# For developers
## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, MSVC target on Windows)
- [Node.js](https://nodejs.org/) 18+
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) (WebView2 on Windows, comes with most modern Windows installs)
- Windows is the primary/only supported platform, this project reads Windows-specific system state (registry, Authenticode signatures, scheduled tasks)

## Setup

```bash
npm install
npm run tauri dev
```

This launches the app in dev mode with hot reload on both the React frontend and Rust backend.

**Note:** avoid running this from inside a OneDrive-synced folder, real-time sync can lock Rust's build output files and cause `EBUSY` errors during compilation.

## Running tests

```bash
cd src-tauri
cargo test
```

All rule engine and explanation engine tests run against synthetic facts (`ProcessFacts` / `PersistenceFacts`), no real system access or actual malware required.

## Known limitations

- **Scheduled task parsing depends on English-language `schtasks` output** (`TaskName:`, `Task To Run:` labels). On a non-English Windows display language, the Startup Items tab will silently show no scheduled tasks. Switching to the Task Scheduler COM API would remove this limitation but is a larger lift than shelling out to `schtasks`.
- **Startup folder shortcuts (`.lnk` files) aren't resolved to their target.** Signature checks run against the shortcut file itself, not the program it points to, so a shortcut to a legitimate signed app may still show as unsigned. Resolving `.lnk` targets needs Windows' `IShellLink` COM API.
- **Network activity detection isn't implemented yet**, `has_network_activity` is currently always `false`. Real network monitoring would likely use ETW (Event Tracing for Windows).
- **CPU-usage-based heuristics are sensitive to hardware** — the `high_cpu_unknown` rule's threshold is a blunt instrument; on lower-end/single-core-constrained machines it can flag legitimate CPU-intensive work. A more correct fix would normalize against core count and sustained (not instantaneous) usage.
- **WTHIROMC flags itself as high-risk when self-built**, since a locally compiled, unsigned binary genuinely matches the "unsigned + unknown publisher" pattern. This is correct behavior for an unsigned build, not a bug, but a properly code-signed release build would score differently.

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for setup, code style, and PR guidelines. Found a security issue in WTHIROMC itself? See [`SECURITY.md`](./SECURITY.md) instead of opening a public issue.

## License

MIT — see [`LICENSE`](./LICENSE).
