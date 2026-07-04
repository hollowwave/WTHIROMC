# Contributing to WTHIROMC

Thanks for considering contributing. WTHIROMC's whole point is helping ordinary people understand what's happening on their computer — contributions that keep it honest, understandable, and trustworthy are the most valuable kind.

## Before you start

- Check open [Issues](../../issues) and [Discussions](../../discussions) — someone may already be working on it, or there may be context worth reading first.
- For anything nontrivial (a new rule, a new data source, a UI redesign), open an issue to discuss the approach before writing code. Saves everyone time.
- Read [`docs/plan.md`](./docs/plan.md) for the architecture rationale — the three-layer split (collector / rules / explain) is deliberate, and PRs that blur that boundary will likely get asked to restructure.

## Development setup

See the [README's Setup section](./README.md#setup) — you'll need Rust (MSVC toolchain), Node 18+, and Tauri's prerequisites.

```bash
npm install
npm run tauri dev
```

Run tests before opening a PR:

```bash
cd src-tauri
cargo test
```

## Where things live

- **New detection heuristic?** → `src-tauri/src/rules/process_rules.rs` or `persistence_rules.rs`. Rules are pure functions (`facts -> Option<RuleHit>`) — write a unit test with fabricated facts, no real system access needed.
- **New explanation sentence?** → `src-tauri/src/explain/templates.rs`. Plain English, no jargon. If you're not sure whether a sentence is clear enough, imagine explaining it to a relative who's never heard the word "registry."
- **New data source (e.g. resolving `.lnk` targets, network activity)?** → `src-tauri/src/collector/`. Collectors only gather facts — they don't score or explain anything.
- **UI changes?** → `src/components/`. Keep the "Task Manager, not a dashboard" philosophy — utilitarian over flashy, nothing hidden from users who want to see everything.

## Code style

- Rust: run `cargo fmt` and `cargo clippy` before pushing. CI checks both (currently advisory, will become blocking once the existing codebase is fully compliant).
- TypeScript: keep components small and typed; avoid `any`.
- Comments should explain *why*, not *what* — the code already says what it does.

## Pull requests

- Keep PRs focused — one rule, one bug fix, one feature. Large PRs mixing unrelated changes are harder to review and more likely to get stuck.
- Include or update tests for anything in `rules/` or `explain/`.
- If you're fixing a known limitation from the README, update the README to remove it once it's actually fixed.
- Describe what you tested it on (Windows version, whether you simulated a scenario or tested against your real system).

## Reporting bugs / false positives

Open an issue with:
- What WTHIROMC flagged and at what risk level
- Whether you believe it's a false positive or a real finding
- Your Windows version (matters for the `schtasks` locale issue and similar platform quirks)

False-positive reports are genuinely valuable — they're how the rule weights get tuned over time.

## Security-sensitive issues

Don't open a public issue for a security vulnerability in WTHIROMC itself (e.g. something that could be exploited to make WTHIROMC hide a real threat, or a privilege escalation issue). See [`SECURITY.md`](./SECURITY.md) instead.

## Ethics / scope boundaries for contributions

- WTHIROMC does not remove, quarantine, or modify anything on the user's system — it's a visibility and explanation tool, not a remediation tool. PRs that add destructive actions (deleting files, killing processes, editing the registry) are out of scope; keep changes read-only.
- No telemetry or remote data upload without an explicit, documented, opt-in mechanism. A security tool silently phoning home defeats the point of the project.
- Be conservative about privilege requirements — if a feature needs administrator rights, document why and make sure the app degrades gracefully without them.
