use crate::types::{ProcessFacts, RuleHit, RunLocation};
use std::collections::HashMap;

const SAFE_PUBLISHERS: &[&str] = &[
    "Microsoft Corporation",
    "Google LLC",
    "Discord Inc.",
    "Mozilla Corporation",
];

fn hit(rule_id: &'static str, weight: i32) -> Option<RuleHit> {
    Some(RuleHit {
        rule_id,
        weight,
        context: HashMap::new(),
    })
}

fn hit_with(rule_id: &'static str, weight: i32, context: HashMap<String, String>) -> Option<RuleHit> {
    Some(RuleHit { rule_id, weight, context })
}

pub fn unsigned_binary(facts: &ProcessFacts) -> Option<RuleHit> {
    if !facts.is_signed {
        hit("unsigned_binary", 15)
    } else {
        None
    }
}

pub fn unknown_publisher(facts: &ProcessFacts) -> Option<RuleHit> {
    if facts.publisher.is_none() {
        hit("unknown_publisher", 10)
    } else {
        None
    }
}

pub fn recent_file(facts: &ProcessFacts) -> Option<RuleHit> {
    match facts.file_age_days {
        Some(days) if days <= 2 => {
            let mut ctx = HashMap::new();
            ctx.insert("days_old".to_string(), days.to_string());
            hit_with("recent_file", 15, ctx)
        }
        _ => None,
    }
}

pub fn runs_from_temp_or_downloads(facts: &ProcessFacts) -> Option<RuleHit> {
    match facts.run_location {
        RunLocation::Temp | RunLocation::Downloads => {
            let mut ctx = HashMap::new();
            let location = match facts.run_location {
                RunLocation::Temp => "Temp",
                RunLocation::Downloads => "Downloads",
                _ => unreachable!(),
            };
            ctx.insert("location".to_string(), location.to_string());
            hit_with("runs_from_temp_or_downloads", 20, ctx)
        }
        _ => None,
    }
}

pub fn autostart_enabled(facts: &ProcessFacts) -> Option<RuleHit> {
    if facts.is_autostart {
        hit("autostart_enabled", 15)
    } else {
        None
    }
}

pub fn network_no_publisher(facts: &ProcessFacts) -> Option<RuleHit> {
    if facts.has_network_activity && facts.publisher.is_none() {
        hit("network_no_publisher", 25)
    } else {
        None
    }
}

pub fn high_cpu_unknown(facts: &ProcessFacts) -> Option<RuleHit> {
    if facts.cpu_usage > 40.0 && facts.publisher.is_none() {
        hit("high_cpu_unknown", 10)
    } else {
        None
    }
}

pub fn known_safe_publisher(facts: &ProcessFacts) -> Option<RuleHit> {
    match &facts.publisher {
        Some(p) if SAFE_PUBLISHERS.contains(&p.as_str()) => hit("known_safe_publisher", -40),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RunLocation;

    fn base_facts() -> ProcessFacts {
        ProcessFacts {
            pid: 1234,
            name: "test.exe".to_string(),
            exe_path: "C:\\Windows\\System32\\test.exe".to_string(),
            publisher: Some("Microsoft Corporation".to_string()),
            is_signed: true,
            file_age_days: Some(400),
            cpu_usage: 1.0,
            memory_bytes: 1024,
            has_network_activity: false,
            is_autostart: false,
            run_location: RunLocation::System32,
        }
    }

    #[test]
    fn clean_signed_process_triggers_no_bad_rules() {
        let facts = base_facts();
        assert!(unsigned_binary(&facts).is_none());
        assert!(unknown_publisher(&facts).is_none());
        assert!(recent_file(&facts).is_none());
        assert!(runs_from_temp_or_downloads(&facts).is_none());
        assert!(autostart_enabled(&facts).is_none());
    }

    #[test]
    fn known_safe_publisher_gives_negative_weight() {
        let facts = base_facts();
        let result = known_safe_publisher(&facts).expect("should hit");
        assert_eq!(result.weight, -40);
    }

    #[test]
    fn suspicious_pattern_stacks_multiple_rules() {
        let mut facts = base_facts();
        facts.publisher = None;
        facts.is_signed = false;
        facts.file_age_days = Some(0);
        facts.run_location = RunLocation::Downloads;
        facts.is_autostart = true;
        facts.has_network_activity = true;

        assert!(unsigned_binary(&facts).is_some());
        assert!(unknown_publisher(&facts).is_some());
        assert!(recent_file(&facts).is_some());
        assert!(runs_from_temp_or_downloads(&facts).is_some());
        assert!(autostart_enabled(&facts).is_some());
        assert!(network_no_publisher(&facts).is_some());
    }

    #[test]
    fn old_file_does_not_trigger_recent_file_rule() {
        let facts = base_facts();
        assert!(recent_file(&facts).is_none());
    }
}
