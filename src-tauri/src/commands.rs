use crate::collector::processes::collect_processes;
use crate::explain::explain;
use crate::rules::evaluate;
use crate::types::ExplainedProcess;

/// The single command the frontend calls to get a full scan: collect facts,
/// score with the rule engine, translate with the explanation engine.
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
