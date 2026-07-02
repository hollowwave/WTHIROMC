# WTHIROMC — v1 Technical Plan

**Scope of this plan:** Process Explorer + Rule-Based Risk Scoring + Explanation Engine + Startup Persistence Scanner. Everything else in the original vision doc is deferred to a Roadmap section at the end.

---

## 1. Architecture

```
┌─────────────────────────────────────────┐
│  React + TypeScript UI (Tauri webview)   │
│  - Process list / detail view            │
│  - Startup persistence view              │
│  - Risk badges, "why flagged" panel      │
└───────────────┬───────────────────────────┘
                │ Tauri IPC (invoke/emit)
┌───────────────▼───────────────────────────┐
│  Rust Core (Tauri backend)                │
│                                            │
│  ┌──────────────────────────────────┐     │
│  │ Collector                        │     │
│  │  - process list (sysinfo)        │     │
│  │  - startup entries (registry,    │     │
│  │    startup folder, sched tasks)  │     │
│  └───────────────┬──────────────────┘     │
│                  │ raw facts               │
│  ┌───────────────▼──────────────────┐     │
│  │ Rule Engine                      │     │
│  │  - evaluates facts against rules │     │
│  │  - produces triggered rules +    │     │
│  │    weighted score                │     │
│  └───────────────┬──────────────────┘     │
│                  │ RiskResult              │
│  ┌───────────────▼──────────────────┐     │
│  │ Explanation Engine               │     │
│  │  - maps triggered rules → plain- │     │
│  │    English template strings      │     │
│  │  - (later) optional LLM polish   │     │
│  └───────────────┬──────────────────┘     │
│                  │ Explained result        │
│  ┌───────────────▼──────────────────┐     │
│  │ SQLite (scan history / timeline) │     │
│  └────────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

**Key design decision:** Collector, Rule Engine, and Explanation Engine are three separate modules with no knowledge of each other's internals — they only pass typed data structures. This means:
- You can unit-test the rule engine with fake process data, no real system access needed.
- Swapping template-based explanations for an LLM later touches *only* the Explanation Engine.
- It mirrors the doc's own philosophy ("AI never makes security decisions") — the Rule Engine is the only thing allowed to produce a risk score.

---

## 2. Data Model

### Rust (core)

```rust
// A raw observed fact about a process, collected once per scan
struct ProcessFacts {
    pid: u32,
    name: String,
    exe_path: String,
    publisher: Option<String>,      // from digital signature, if present
    is_signed: bool,
    created_at: Option<DateTime>,   // file creation time
    parent_pid: Option<u32>,
    cpu_usage: f32,
    memory_bytes: u64,
    has_network_activity: bool,     // v1: simple bool; v2: connection details
    is_autostart: bool,
    run_location: RunLocation,      // enum: System32, ProgramFiles, Temp, Downloads, AppData, Other
}

enum RunLocation { System32, ProgramFiles, Temp, Downloads, AppData, Other(String) }

// Output of a single rule evaluation
struct RuleHit {
    rule_id: &'static str,
    weight: i32,
    // facts substituted into the explanation template, e.g. {"days_old": "0"}
    context: HashMap<String, String>,
}

// Output of the Rule Engine for one process
struct RiskResult {
    pid: u32,
    score: i32,
    level: RiskLevel,           // Green/Yellow/Orange/Red/Black
    hits: Vec<RuleHit>,
}

enum RiskLevel { Green, Yellow, Orange, Red, Black }

// Output of the Explanation Engine — what the UI actually renders
struct ExplainedProcess {
    facts: ProcessFacts,
    risk: RiskResult,
    summary: String,            // one-line human summary
    explanations: Vec<String>,  // one sentence per triggered rule
}
```

### TypeScript (UI side — mirrors the Rust types via Tauri's generated bindings or manual types)

```ts
interface ExplainedProcess {
  pid: number;
  name: string;
  exePath: string;
  publisher: string | null;
  riskLevel: "Green" | "Yellow" | "Orange" | "Red" | "Black";
  score: number;
  summary: string;
  explanations: string[];
}
```

---

## 3. Rule Engine Design

Each rule is a pure function: `ProcessFacts -> Option<RuleHit>`. Rules are registered in a list and all run against every process every scan. Simple, testable, no clever abstraction needed for v1.

**Initial rule set (v1 — 8 rules, tune weights after testing):**

| Rule ID | Condition | Weight | Rationale |
|---|---|---|---|
| `unsigned_binary` | `is_signed == false` | +15 | Legit software is usually signed |
| `unknown_publisher` | `publisher.is_none()` | +10 | Stacks with unsigned |
| `recent_file` | created within last 48h | +15 | New + unfamiliar = worth a look |
| `runs_from_temp_or_downloads` | `run_location in [Temp, Downloads]` | +20 | Malware rarely installs properly |
| `autostart_enabled` | `is_autostart == true` | +15 | Persistence is a red flag |
| `network_no_publisher` | `has_network_activity && publisher.is_none()` | +25 | Compound rule — talking to the internet with no accountability |
| `high_cpu_unknown` | `cpu_usage > 40% && publisher.is_none()` | +10 | Possible miner/exfil, weak signal alone |
| `known_safe_publisher` | publisher in an allowlist (Microsoft, Google, Discord Inc., etc.) | −40 | Strong negative signal to reduce false positives |

**Scoring → Risk Level thresholds (tune empirically):**
```
score < 10        → Green
10 <= score < 30   → Yellow
30 <= score < 50   → Orange
50 <= score < 75   → Red
score >= 75         → Black
```

**Important scoping note:** some "facts" (digital signature verification, accurate file creation time, real network activity) require real Windows API work, not just `sysinfo`. Suggested sequencing:
1. Build rules against whatever `sysinfo` gives you for free (name, path, PID, CPU, memory, parent) first.
2. Add signature-checking and network activity as a second pass once the pipeline works end to end — this is exactly the point where you'd start touching `windows-rs`.

This keeps you from being blocked on Windows API research before you have *anything* running.

---

## 4. Explanation Engine Design

Template map, keyed by `rule_id`, with `{placeholders}` filled from the rule's `context`:

```rust
"unsigned_binary" => "This program is not digitally signed, so there's no way to verify who made it.",
"unknown_publisher" => "This program doesn't identify a publisher.",
"recent_file" => "This program appeared on your system {days_old} day(s) ago.",
"runs_from_temp_or_downloads" => "This program is running from your {location} folder, which is unusual for legitimate software.",
"autostart_enabled" => "This program is set to start automatically every time you turn on your computer.",
"network_no_publisher" => "This program is connecting to the internet, but it has no verified publisher — there's no way to know who's receiving that data.",
"high_cpu_unknown" => "This program is using a lot of your computer's processing power and has no verified publisher.",
```

**Summary line** = a short synthesis rule based on risk level + top 1-2 hits, e.g.:
> "This program is unsigned, was installed today, and is set to run automatically — a pattern often seen in malware."

v1: hand-write the summary synthesis as a few more template rules (e.g. "if hits include recent_file + autostart_enabled + unsigned → use this stronger combined sentence"). This is genuinely most of the product's soul — worth writing thoughtfully rather than rushing.

**LLM hook (deferred, but designed for now):** the Explanation Engine exposes one function, `explain(facts, risk) -> ExplainedProcess`. When/if you add an LLM, it becomes a second implementation of that same function signature (e.g. take the template output and ask a model to smooth phrasing, or generate the summary line) — swapped behind a config flag. No other code changes.

---

## 5. Startup Persistence Scanner

Same Facts → Rules → Explanation pipeline, different collector:
- Registry `Run` / `RunOnce` keys (`HKCU` and `HKLM`)
- Startup folder contents (`shell:startup`)
- Scheduled tasks (via `schtasks` or `windows-rs` task scheduler API — start with shelling out to `schtasks /query` for v1, it's much simpler than the COM API)

Each entry becomes a `ProcessFacts`-like `PersistenceEntry` and reuses the same rule engine where applicable (unsigned, unknown publisher, recent) plus one new rule: `persistence_via_scheduled_task` (scheduled tasks are a less common but higher-suspicion persistence method than a simple Run key).

---

## 6. Project Structure

```
wthiromc/
├── src/                      # React/TS frontend
│   ├── components/
│   │   ├── ProcessList.tsx
│   │   ├── ProcessDetail.tsx
│   │   ├── RiskBadge.tsx
│   │   └── PersistenceView.tsx
│   ├── types/
│   │   └── explained.ts
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── collector/
│   │   │   ├── processes.rs
│   │   │   └── persistence.rs
│   │   ├── rules/
│   │   │   ├── mod.rs        # Rule trait + registry
│   │   │   └── process_rules.rs
│   │   ├── explain/
│   │   │   ├── mod.rs
│   │   │   └── templates.rs
│   │   ├── db.rs             # SQLite scan history
│   │   ├── commands.rs       # #[tauri::command] handlers exposed to UI
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/
│   └── rule_engine_tests.rs  # fake ProcessFacts fixtures, no real system needed
└── README.md
```

---

## 7. Milestones (with acceptance criteria)

**M1 — Skeleton pipeline**
- Tauri app opens, calls a Rust command, gets back a live process list via `sysinfo`.
- UI renders an unstyled table: PID, name, path, CPU%, memory.
- ✅ Done when: you can see your actual running processes in the app window.

**M2 — Rule engine v1 (offline-testable)**
- Implement the 8 rules above as pure functions with unit tests using fabricated `ProcessFacts` (no real system data needed yet).
- ✅ Done when: `cargo test` passes a suite proving each rule fires/doesn't fire correctly on synthetic inputs.

**M3 — Wire rule engine to real data**
- Feed real `sysinfo` output into the rule engine; add whatever real-signal collection is feasible at this stage (recent file creation via file metadata is doable immediately; digital signature checking may need `windows-rs` — treat as a stretch goal for this milestone, stub with `is_signed: false` if needed).
- ✅ Done when: the process list in the UI shows a risk badge per process, computed from real data.

**M4 — Explanation engine**
- Wire templates to rule hits; add the summary-line synthesis logic.
- Detail view: click a process → see "Why this is flagged" with full sentence list.
- ✅ Done when: a plausible-looking suspicious process (you can fake one, e.g. copy `notepad.exe` into `Downloads` and rename it) gets a coherent, correct-reading explanation.

**M5 — UI design pass**
- This is where "here are the 3 things you should care about" becomes real: sort by risk, collapse Green processes by default, make Red/Black impossible to miss.
- ✅ Done when: someone with zero security background can open the app and immediately understand what needs their attention.

**M6 — Startup Persistence Scanner**
- Registry + startup folder + scheduled tasks collector, reusing the rule/explanation pipeline.
- ✅ Done when: a new startup entry you add for testing shows up, flagged, with a sensible explanation.

**M7 — Portfolio polish**
- README with screenshots/GIF, architecture diagram (this doc's diagram, cleaned up), a "Roadmap" section listing deferred features, install instructions.
- ✅ Done when: someone who's never seen the project can read the README and understand what it does and why it's interesting in under 2 minutes.

---

## 8. Testing / Validation Strategy

You can't (and shouldn't) test against real malware. Instead:
- **Unit tests** for the rule engine against synthetic `ProcessFacts` fixtures (safe process, borderline process, clearly-bad process).
- **Manual "safe test malware" simulation**: rename a harmless executable (e.g. a copy of `notepad.exe`), drop it in `Downloads`, set it to autostart via a registry key you add yourself, and confirm the app flags it correctly and explains why. This gives you a realistic demo scenario for your portfolio without touching anything actually dangerous.
- **False-positive check**: run the tool on your own clean system and confirm common legitimate software (browser, Discord, IDE) scores Green.

---

## 9. Open Decisions (revisit as you build, not blocking to start)

- Exact risk-score thresholds — tune after seeing real scores on your own machine.
- Whether digital signature verification is in scope for v1 or v1.1 (it's the highest-value rule but also the most Windows-API-heavy).
- SQLite schema for scan history — can be deferred until after M4, since the timeline feature isn't in this scope.

---

## 10. Deferred (Roadmap section for your README, not v1 work)

Browser extension analysis, network geo-IP analysis, scam link analyzer, community threat intelligence network, cloud backend, multi-browser support, LLM-powered explanation polish, infection timeline reconstruction.
