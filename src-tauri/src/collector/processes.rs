use crate::types::{ProcessFacts, RunLocation};
use rayon::prelude::*;
use std::path::Path;
use sysinfo::System;

use super::signature;

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

fn file_age_days(exe_path: &str) -> Option<i64> {
    let metadata = std::fs::metadata(Path::new(exe_path)).ok()?;
    let created = metadata.created().ok()?;
    let age = created.elapsed().ok()?;
    Some(age.as_secs() as i64 / 86400)
}

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
                file_age_days: file_age_days(&exe_path),
                cpu_usage: proc.cpu_usage(),
                memory_bytes: proc.memory(),
                has_network_activity: false,
                is_autostart: false,
                run_location: classify_run_location(&exe_path),
            }
        })
        .collect()
}

