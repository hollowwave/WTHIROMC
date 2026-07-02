use crate::types::{ExplainedProcess, ProcessFacts, RiskLevel, RiskResult};

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
