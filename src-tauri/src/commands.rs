use crate::collector::persistence::collect_persistence_entries;
use crate::collector::processes::collect_processes;
use crate::explain::{explain, explain_persistence};
use crate::rules::{evaluate, evaluate_persistence};
use crate::types::{ExplainedPersistence, ExplainedProcess};

/// The command the frontend calls to get a full process scan: collect
/// facts, score with the rule engine, translate with the explanation engine.
#[tauri::command]
pub fn scan_processes() -> Vec<ExplainedProcess> {
    collect_processes()
        .iter()
        .map(|facts| {
            let risk = evaluate(facts);
            explain(facts, &risk)
        })
        .collect()
}

/// Same pipeline, for startup/persistence entries (M6).
#[tauri::command]
pub fn scan_startup_items() -> Vec<ExplainedPersistence> {
    collect_persistence_entries()
        .iter()
        .map(|facts| {
            let risk = evaluate_persistence(facts);
            explain_persistence(facts, &risk)
        })
        .collect()
}

