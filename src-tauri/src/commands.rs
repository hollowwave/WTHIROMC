use crate::allowlist;
use crate::collector::persistence::collect_persistence_entries;
use crate::collector::processes::collect_processes;
use crate::explain::{explain, explain_persistence};
use crate::history;
use crate::rules::{evaluate, evaluate_persistence};
use crate::types::{ExplainedPersistence, ExplainedProcess};

/// The command the frontend calls to get a full process scan: collect
/// facts, score with the rule engine, translate with the explanation
/// engine, then apply the user's allowlist and "is this new" history check
/// on top (Phase 3) - both live outside rules/explain deliberately, see
/// their module docs.
#[tauri::command]
pub fn scan_processes() -> Vec<ExplainedProcess> {
    let explained: Vec<ExplainedProcess> = collect_processes()
        .iter()
        .map(|facts| {
            let risk = evaluate(facts);
            explain(facts, &risk)
        })
        .collect();

    let mut explained = allowlist::apply_to_processes(explained);

    let items: Vec<(String, String)> = explained
        .iter()
        .map(|p| (p.exe_path.clone(), p.name.clone()))
        .collect();
    let new_ones = history::check_new_and_record("process", &items);
    for p in &mut explained {
        p.is_new = new_ones.contains(&p.exe_path);
    }

    explained
}

/// Same pipeline, for startup/persistence entries (M6).
#[tauri::command]
pub fn scan_startup_items() -> Vec<ExplainedPersistence> {
    let explained: Vec<ExplainedPersistence> = collect_persistence_entries()
        .iter()
        .map(|facts| {
            let risk = evaluate_persistence(facts);
            explain_persistence(facts, &risk)
        })
        .collect();

    let mut explained = allowlist::apply_to_persistence(explained);

    let items: Vec<(String, String)> = explained
        .iter()
        .map(|e| (allowlist::persistence_identifier(e), e.name.clone()))
        .collect();
    let new_ones = history::check_new_and_record("persistence", &items);
    for e in &mut explained {
        e.is_new = new_ones.contains(&allowlist::persistence_identifier(e));
    }

    explained
}

/// Marks a process as safe by its exe path. The frontend re-fetches the
/// scan afterward, which is what actually reflects the change in the UI.
#[tauri::command]
pub fn mark_process_safe(exe_path: String, name: String) -> Result<(), String> {
    allowlist::mark_safe(&exe_path, &name)
}

#[tauri::command]
pub fn unmark_process_safe(exe_path: String) -> Result<(), String> {
    allowlist::unmark_safe(&exe_path)
}

/// `identifier` here should be the same `source:name` key the frontend
/// already computes in `StartupList.tsx`'s `entryKey()` - kept as a plain
/// string across the IPC boundary rather than re-deriving it server-side
/// from separate fields, to guarantee both sides agree on the same key.
#[tauri::command]
pub fn mark_startup_safe(identifier: String, name: String) -> Result<(), String> {
    allowlist::mark_safe(&identifier, &name)
}

#[tauri::command]
pub fn unmark_startup_safe(identifier: String) -> Result<(), String> {
    allowlist::unmark_safe(&identifier)
}

