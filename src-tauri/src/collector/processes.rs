use crate::types::{ProcessFacts, RunLocation};
use std::path::Path;
use sysinfo::System;

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
fn file_age_days(exe_path: &str) -> Option<i64> {
    let metadata = std::fs::metadata(Path::new(exe_path)).ok()?;
    let created = metadata.created().ok()?;
    let age = created.elapsed().ok()?;
    Some(age.as_secs() as i64 / 86400)
}

/// Collects facts for every currently running process.
///
/// NOTE (v1 scope, see plan section 3): digital signature verification and
/// real network-activity detection require Windows-specific APIs
/// (WinVerifyTrust / ETW) that aren't wired up yet. Both fields are stubbed
/// conservatively (is_signed: false, has_network_activity: false) so the
/// rule engine has real inputs to run against today; replacing these stubs
/// is the natural next step (see plan milestone M3).
pub fn collect_processes() -> Vec<ProcessFacts> {
    let mut sys = System::new_all();
    sys.refresh_all();

    sys.processes()
        .iter()
        .map(|(pid, proc)| {
            let exe_path = proc
                .exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            ProcessFacts {
                pid: pid.as_u32(),
                name: proc.name().to_string_lossy().to_string(),
                exe_path: exe_path.clone(),
                publisher: None, // TODO(M3): digital signature lookup
                is_signed: false, // TODO(M3): digital signature lookup
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
