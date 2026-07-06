use crate::types::{ProcessFacts, RunLocation};
use rayon::prelude::*;
use std::path::Path;
use sysinfo::System;

use super::signature;

/// Classifies where an executable lives. This is intentionally simple string
/// matching for v1 — good enough for the rule engine, no Windows-specific
/// APIs required.
fn classify_run_location(exe_path: &str) -> RunLocation {
    let lower = exe_path.to_lowercase();
    if lower.contains("\\windows\\system32") {
        RunLocation::System32
    } else if lower.contains("\\program files") {
        RunLocation::ProgramFiles
    } else if lower.contains("\\appdata\\local\\temp") || lower.contains("\\temp\\") {
        RunLocation::Temp
    } else if lower.contains("\\downloads\\") {
        RunLocation::Downloads
    } else if lower.contains("\\appdata\\") {
        RunLocation::AppData
    } else {
        RunLocation::Other
    }
}

/// Best-effort file age in days from filesystem metadata.
/// Returns None if the file can't be stat'd (e.g. process already exited).
/// Prefers creation time, falls back to modified time if creation time
/// isn't available on this filesystem/permission level.
fn file_age_days(exe_path: &str) -> Option<i64> {
    let metadata = std::fs::metadata(Path::new(exe_path)).ok()?;
    let reference_time = metadata.created().or_else(|_| metadata.modified()).ok()?;
    let age = reference_time.elapsed().ok()?;
    Some(age.as_secs() as i64 / 86400)
}

/// Collects facts for every currently running process.
///
/// NOTE (v1 scope, see plan section 3): real network-activity detection and
/// autostart cross-referencing still require further work (see milestones
/// M3+/M6). Digital signature verification (is_signed/publisher) is wired
/// up via the `signature` module.
///
/// Signature checks are the expensive part of this pipeline (real disk I/O
/// + crypto verification per process), so we run them across a Vec with
/// rayon's `par_iter` rather than sysinfo's HashMap iterator directly
/// (std HashMap doesn't implement rayon's parallel iterator, so we collect
/// into a Vec first — cheap, since it's just pointers/refs).
pub fn collect_processes() -> Vec<ProcessFacts> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let entries: Vec<_> = sys.processes().iter().collect();

    entries
        .par_iter()
        .map(|(pid, proc)| {
            let exe_path = proc
                .exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let sig = signature::check_signature(&exe_path);

            ProcessFacts {
                pid: pid.as_u32(),
                name: proc.name().to_string_lossy().to_string(),
                exe_path: exe_path.clone(),
                publisher: sig.publisher,
                is_signed: sig.is_signed,
                signature_detail: sig.detail,
                file_age_days: file_age_days(&exe_path),
                cpu_usage: proc.cpu_usage(),
                memory_bytes: proc.memory(),
                has_network_activity: false, // TODO(M3+): real network data
                is_autostart: false,         // TODO(M6): cross-reference persistence scan
                run_location: classify_run_location(&exe_path),
            }
        })
        .collect()
}

