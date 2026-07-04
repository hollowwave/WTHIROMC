# Roadmap

Tracking where WTHIROMC is headed now that it's moving from "portfolio piece" to "tool meant for real use." Items are grouped by phase, roughly in priority order within each phase. Check items off as they land, and move anything newly discovered into the right phase rather than letting it float in an issue thread.

---

## Phase 1 — Trust & professionalism (in progress)

The goal: someone downloading the `.exe` should have a reasonable way to trust it, and someone wanting to contribute should have a clear path in.

- [x] GitHub Actions CI (`ci.yml`) — build + test on every push/PR
- [x] GitHub Actions Release workflow (`release.yml`) — builds installer, generates SHA256 checksums, publishes to GitHub Releases on tag push
- [x] `CONTRIBUTING.md`
- [x] `SECURITY.md` — vulnerability reporting + checksum verification instructions
- [x] Update README's install section to link the checksum verification steps
- [x] Turn on `cargo fmt --check` and `cargo clippy -D warnings` as **blocking** in CI (currently advisory — flip once the existing codebase is fully compliant)
- [x] Issue templates (`.github/ISSUE_TEMPLATE/`) — bug report, false-positive report, feature request

## Phase 2 — Fix known limitations

The stuff currently documented as "known limitations" in the README. These are real gaps, not nice-to-haves.

- [ ] **Locale-robust scheduled task parsing** — replace `schtasks /query` text parsing (English-only labels) with the Task Scheduler COM API via `windows-rs`, or another approach that doesn't depend on display language. Add tests covering the parsing logic.
- [ ] **Resolve `.lnk` shortcut targets** — Startup folder entries are often shortcuts; currently signature checks run against the `.lnk` file itself instead of what it points to. Use the Shell Link API (`IShellLink`) to resolve the real target before checking its signature. Update `collector/persistence.rs` and reuse `signature.rs` against the resolved path.
- [ ] **Real network activity detection** — `has_network_activity` is currently always `false`, which means `network_no_publisher` never actually fires on real data. Implement a conservative "has any active connection to a non-localhost address" check (ETW or a Windows networking API), wire it into the process collector, and add tests.
- [ ] **File age fallback** — `file_age_days()` uses `metadata.created()`, which can fail or be unavailable on some filesystems/permission levels. Add a fallback to `metadata.modified()` and document the difference in what each signal means.
- [ ] **Signature check error detail** — right now "unsigned" and "signature check failed for another reason" are collapsed into the same result. Worth distinguishing (e.g. revoked certificate vs. genuinely no signature) so explanations can be more precise.

## Phase 3 — User workflows

Features aimed at making the tool actually useful in a real "am I compromised?" scenario, not just informative.

- [ ] **"Mark as safe" allowlist** — let users tag a process/publisher/path as known-safe; persist it (likely SQLite, per the original plan) and factor it into scoring so confirmed-safe items stop generating noise on every scan.
- [ ] **Scan history / comparison** — store past scan results and let users ask "is this new since last week?" — a strong signal for catching recent compromise.
- [ ] **Export results** — JSON/CSV export of a scan, so a user can hand results to an IT person or a more experienced friend without screen-sharing.
- [ ] **Basic incident-response guidance** — when something scores Red/Black, surface next steps (the Recovery Assistant concept from the original vision doc), not just the explanation of what was found.

## Phase 4 — Bigger features (from the original vision doc)

Not started, no firm timeline. Revisit after Phases 1–3 are solid.

- [ ] Browser extension analysis (Chrome/Edge/Firefox) — permissions review, suspicious extension detection
- [ ] Network behavior analysis with geo-IP context
- [ ] Scam/phishing link analyzer
- [ ] Community threat intelligence — user-submitted hashes/URLs, shared reputation database
- [ ] Infection timeline reconstruction
- [ ] LLM-assisted explanation polish for cases the templates don't cover well (see `explain::explain()` — this is the designed seam for it)

## Explicitly out of scope (for now)

- Any destructive action (deleting files, killing processes, editing the registry) — WTHIROMC is read-only by design. See `CONTRIBUTING.md`'s ethics section.
- Telemetry or remote data upload without explicit, documented opt-in.
- Replacing a real antivirus/EDR product — WTHIROMC is a visibility and explanation layer, not a detection engine with removal capability.
