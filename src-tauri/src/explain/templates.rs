use crate::types::RuleHit;

/// Renders one rule hit into a plain-English sentence, substituting any
/// context values (e.g. {days_old}) the rule provided.
pub fn render(hit: &RuleHit) -> String {
    let template = match hit.rule_id {
        "unsigned_binary" => {
            "This program is not digitally signed, so there's no way to verify who made it."
        }
        "unknown_publisher" => "This program doesn't identify a publisher.",
        "recent_file" => "This program appeared on your system {days_old} day(s) ago.",
        "runs_from_temp_or_downloads" => {
            "This program is running from your {location} folder, which is unusual for legitimate software."
        }
        "autostart_enabled" => {
            "This program is set to start automatically every time you turn on your computer."
        }
        "network_no_publisher" => {
            "This program is connecting to the internet, but it has no verified publisher — there's no way to know who's receiving that data."
        }
        "high_cpu_unknown" => {
            "This program is using a lot of your computer's processing power and has no verified publisher."
        }
        "persistence_via_scheduled_task" => {
            "This program uses a scheduled task to launch automatically, and it isn't signed by a known publisher."
        }
        other => return format!("Flagged by rule: {other}"),
    };

    let mut rendered = template.to_string();
    for (key, value) in &hit.context {
        rendered = rendered.replace(&format!("{{{key}}}"), value);
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn substitutes_context_placeholders() {
        let mut ctx = HashMap::new();
        ctx.insert("days_old".to_string(), "1".to_string());
        let h = RuleHit { rule_id: "recent_file", weight: 15, context: ctx };
        assert_eq!(render(&h), "This program appeared on your system 1 day(s) ago.");
    }

    #[test]
    fn unknown_rule_id_still_produces_readable_text() {
        let h = RuleHit { rule_id: "made_up_rule", weight: 5, context: HashMap::new() };
        assert!(render(&h).contains("made_up_rule"));
    }
}

