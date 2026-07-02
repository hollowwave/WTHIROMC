use crate::types::{ProcessFacts, RiskLevel, RiskResult, RuleHit};

pub mod process_rules;

/// A rule is a pure function: given facts, optionally produce a hit.
/// Keeping this as `fn` pointers (not trait objects) keeps rules trivially
/// testable and easy to read as a flat list.
pub type Rule = fn(&ProcessFacts) -> Option<RuleHit>;

pub fn all_rules() -> Vec<Rule> {
    vec![
        process_rules::unsigned_binary,
        process_rules::unknown_publisher,
        process_rules::recent_file,
        process_rules::runs_from_temp_or_downloads,
        process_rules::autostart_enabled,
        process_rules::network_no_publisher,
        process_rules::high_cpu_unknown,
        process_rules::known_safe_publisher,
    ]
}

/// Run every registered rule against one process's facts and produce a score.
pub fn evaluate(facts: &ProcessFacts) -> RiskResult {
    let hits: Vec<RuleHit> = all_rules()
        .into_iter()
        .filter_map(|rule| rule(facts))
        .collect();

    let score: i32 = hits.iter().map(|h| h.weight).sum::<i32>().max(0);

    RiskResult {
        score,
        level: RiskLevel::from_score(score),
        hits,
    }
}
