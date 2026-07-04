use crate::types::{PersistenceFacts, PersistenceType, RuleHit};
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

pub fn unsigned_binary(facts: &PersistenceFacts) -> Option<RuleHit> {
    if !facts.is_signed {
        hit("unsigned_binary", 15)
    } else {
        None
    }
}

pub fn unknown_publisher(facts: &PersistenceFacts) -> Option<RuleHit> {
    if facts.publisher.is_none() {
        hit("unknown_publisher", 10)
    } else {
        None
    }
}

pub fn recent_file(facts: &PersistenceFacts) -> Option<RuleHit> {
    match facts.file_age_days {
        Some(days) if days <= 2 => {
            let mut ctx = HashMap::new();
            ctx.insert("days_old".to_string(), days.to_string());
            hit_with("recent_file", 15, ctx)
        }
        _ => None,
    }
}

pub fn known_safe_publisher(facts: &PersistenceFacts) -> Option<RuleHit> {
    match &facts.publisher {
        Some(p) if SAFE_PUBLISHERS.contains(&p.as_str()) => hit("known_safe_publisher", -40),
        _ => None,
    }
}

/// New, persistence-specific rule (per your call to keep this independently
/// tunable from the process rules). Scheduled tasks are a less common
/// autostart mechanism than a Run key, so an unsigned scheduled task is
/// treated as a stronger signal on its own.
pub fn persistence_via_scheduled_task(facts: &PersistenceFacts) -> Option<RuleHit> {
    if facts.source == PersistenceType::ScheduledTask && !facts.is_signed {
        hit("persistence_via_scheduled_task", 20)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_facts() -> PersistenceFacts {
        PersistenceFacts {
            name: "OneDrive".to_string(),
            command: "C:\\Program Files\\Microsoft OneDrive\\OneDrive.exe".to_string(),
            source: PersistenceType::RegistryRun,
            publisher: Some("Microsoft Corporation".to_string()),
            is_signed: true,
            file_age_days: Some(400),
        }
    }

    #[test]
    fn signed_known_publisher_startup_entry_is_clean() {
        let facts = base_facts();
        assert!(unsigned_binary(&facts).is_none());
        assert!(unknown_publisher(&facts).is_none());
        assert!(known_safe_publisher(&facts).is_some());
    }

    #[test]
    fn scheduled_task_rule_only_fires_for_scheduled_tasks() {
        let mut facts = base_facts();
        facts.is_signed = false;
        facts.source = PersistenceType::RegistryRun;
        assert!(persistence_via_scheduled_task(&facts).is_none());

        facts.source = PersistenceType::ScheduledTask;
        assert!(persistence_via_scheduled_task(&facts).is_some());
    }

    #[test]
    fn signed_scheduled_task_does_not_trigger_task_rule() {
        let mut facts = base_facts();
        facts.source = PersistenceType::ScheduledTask;
        facts.is_signed = true;
        assert!(persistence_via_scheduled_task(&facts).is_none());
    }
}

