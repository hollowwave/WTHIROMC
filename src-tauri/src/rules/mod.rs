use crate::types::{PersistenceFacts, ProcessFacts, RiskLevel, RiskResult, RuleHit};

pub mod persistence_rules;
pub mod process_rules;

/// A rule is a pure function: given facts, optionally produce a hit.
/// Keeping this as `fn` pointers (not trait objects) keeps rules trivially
/// testable and easy to read as a flat list.
pub type Rule = fn(&ProcessFacts) -> Option<RuleHit>;
pub type PersistenceRule = fn(&PersistenceFacts) -> Option<RuleHit>;

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

pub fn all_persistence_rules() -> Vec<PersistenceRule> {
    vec![
        persistence_rules::unsigned_binary,
        persistence_rules::unknown_publisher,
        persistence_rules::recent_file,
        persistence_rules::known_safe_publisher,
        persistence_rules::persistence_via_scheduled_task,
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

/// Same as `evaluate`, for persistence entries.
pub fn evaluate_persistence(facts: &PersistenceFacts) -> RiskResult {
    let hits: Vec<RuleHit> = all_persistence_rules()
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

