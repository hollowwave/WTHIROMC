use wthiromc_lib::explain::explain;
use wthiromc_lib::rules::evaluate;
use wthiromc_lib::types::{ProcessFacts, RiskLevel, RunLocation};

fn clean_process() -> ProcessFacts {
    ProcessFacts {
        pid: 100,
        name: "explorer.exe".to_string(),
        exe_path: "C:\\Windows\\System32\\explorer.exe".to_string(),
        publisher: Some("Microsoft Corporation".to_string()),
        is_signed: true,
        signature_detail: None,
        file_age_days: Some(1000),
        cpu_usage: 2.0,
        memory_bytes: 50_000_000,
        has_network_activity: false,
        is_autostart: false,
        run_location: RunLocation::System32,
    }
}

fn simulated_malware() -> ProcessFacts {
    ProcessFacts {
        pid: 4242,
        name: "Minecraft_Free_Premium.exe".to_string(),
        exe_path: "C:\\Users\\test\\Downloads\\Minecraft_Free_Premium.exe".to_string(),
        publisher: None,
        is_signed: false,
        signature_detail: Some("no signature was found on the file".to_string()),
        file_age_days: Some(0),
        cpu_usage: 5.0,
        memory_bytes: 10_000_000,
        has_network_activity: true,
        is_autostart: true,
        run_location: RunLocation::Downloads,
    }
}

#[test]
fn clean_system_process_scores_green_with_reassuring_summary() {
    let facts = clean_process();
    let risk = evaluate(&facts);
    let explained = explain(&facts, &risk);

    assert_eq!(explained.risk_level, RiskLevel::Green);
    assert!(explained.explanations.is_empty());
    assert!(explained.summary.contains("normal"));
}

#[test]
fn simulated_malware_scores_high_and_explains_why() {
    let facts = simulated_malware();
    let risk = evaluate(&facts);
    let explained = explain(&facts, &risk);

    assert!(
        matches!(explained.risk_level, RiskLevel::Red | RiskLevel::Black),
        "expected Red or Black, got {:?}",
        explained.risk_level
    );

    // Every triggered warning rule should have produced a readable sentence.
    assert!(!explained.explanations.is_empty());
    assert!(explained
        .explanations
        .iter()
        .any(|e| e.contains("Downloads")));
    assert!(explained
        .explanations
        .iter()
        .any(|e| e.contains("automatically")));

    // The combined-pattern summary should fire since recent + autostart + unsigned all hit.
    assert!(explained.summary.contains("installed very recently"));
}

