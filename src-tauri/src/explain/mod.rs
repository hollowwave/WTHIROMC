//! Turns a risk score into plain-English sentences. This is the module to
//! touch if/when an LLM is ever added — `explain()` and `explain_persistence()`
//! are the two functions with that seam; everything upstream (collection,
//! scoring) stays untouched either way.

use crate::types::{ExplainedPersistence, ExplainedProcess, PersistenceFacts, ProcessFacts, RiskLevel, RiskResult};

pub mod templates;

/// Turns raw facts + a computed risk result into the UI-facing explained process.
/// This is the ONLY function that should be swapped out if/when an LLM is added
/// later (e.g. to smooth phrasing or handle rule combinations templates.rs doesn't
/// cover) — everything upstream of this stays untouched.
pub fn explain(facts: &ProcessFacts, risk: &RiskResult) -> ExplainedProcess {
    let explanations: Vec<String> = risk
        .hits
        .iter()
        .filter(|h| h.weight > 0) // don't surface negative/"safe" rules as warnings
        .map(|h| templates::render(h))
        .collect();

    let summary = summarize(risk);

    ExplainedProcess {
        pid: facts.pid,
        name: facts.name.clone(),
        exe_path: facts.exe_path.clone(),
        publisher: facts.publisher.clone(),
        cpu_usage: facts.cpu_usage,
        memory_bytes: facts.memory_bytes,
        risk_level: risk.level,
        score: risk.score,
        summary,
        explanations,
    }
}

/// One-line synthesis of the overall verdict. v1: a small set of hand-written
/// rules based on risk level and which rule_ids fired together. Deliberately
/// simple and readable — this is the piece most worth polishing by hand before
/// reaching for an LLM.
fn summarize(risk: &RiskResult) -> String {
    let rule_ids: Vec<&str> = risk.hits.iter().map(|h| h.rule_id).collect();

    let has = |id: &str| rule_ids.contains(&id);

    match risk.level {
        RiskLevel::Green => "This looks like normal, expected behavior.".to_string(),
        RiskLevel::Yellow => {
            "A few things about this program are worth a second look, but nothing alarming yet.".to_string()
        }
        RiskLevel::Orange => {
            "This program shows a combination of traits that's worth investigating.".to_string()
        }
        RiskLevel::Red | RiskLevel::Black => {
            if has("recent_file") && has("autostart_enabled") && has("unsigned_binary") {
                "This program is unsigned, was installed very recently, and is set to run automatically — a pattern often seen in malware.".to_string()
            } else if has("network_no_publisher") {
                "This program is sending data to the internet with no way to verify who's receiving it.".to_string()
            } else {
                "This program shows several strong signs of suspicious behavior and deserves a closer look.".to_string()
            }
        }
    }
}

/// Persistence-entry counterpart to `explain`. Kept as a separate function
/// (rather than a generic over both fact types) since the two have
/// different summary logic and different UI-facing shapes.
pub fn explain_persistence(facts: &PersistenceFacts, risk: &RiskResult) -> ExplainedPersistence {
    let explanations: Vec<String> = risk
        .hits
        .iter()
        .filter(|h| h.weight > 0)
        .map(|h| templates::render(h))
        .collect();

    let summary = summarize_persistence(risk);

    ExplainedPersistence {
        name: facts.name.clone(),
        command: facts.command.clone(),
        source: facts.source,
        publisher: facts.publisher.clone(),
        risk_level: risk.level,
        score: risk.score,
        summary,
        explanations,
    }
}

fn summarize_persistence(risk: &RiskResult) -> String {
    let rule_ids: Vec<&str> = risk.hits.iter().map(|h| h.rule_id).collect();
    let has = |id: &str| rule_ids.contains(&id);

    match risk.level {
        RiskLevel::Green => "This looks like a normal startup entry.".to_string(),
        RiskLevel::Yellow => {
            "A few things about this startup entry are worth a second look.".to_string()
        }
        RiskLevel::Orange => {
            "This startup entry shows a combination of traits that's worth investigating.".to_string()
        }
        RiskLevel::Red | RiskLevel::Black => {
            if has("persistence_via_scheduled_task") {
                "This program uses a scheduled task to run automatically and isn't signed by a known publisher — a technique sometimes used to survive reboots without showing up in the usual startup list.".to_string()
            } else if has("recent_file") && has("unsigned_binary") {
                "This is an unsigned program that was added to your startup very recently.".to_string()
            } else {
                "This startup entry shows several strong signs of suspicious behavior and deserves a closer look.".to_string()
            }
        }
    }
}

