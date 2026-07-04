use crate::types::{PersistenceFacts, PersistenceType};
use std::path::Path;
use std::process::Command;

use super::signature;

/// Best-effort file age in days from filesystem metadata (same logic as
/// the process collector's version — duplicated rather than shared to keep
/// this module self-contained; worth extracting to a shared util if a
/// third caller shows up).
fn file_age_days(path: &str) -> Option<i64> {
    let metadata = std::fs::metadata(Path::new(path)).ok()?;
    let created = metadata.created().ok()?;
    let age = created.elapsed().ok()?;
    Some(age.as_secs() as i64 / 86400)
}

/// Registry Run-key values and Startup-folder shortcuts often include
/// arguments or are wrapped in quotes, e.g. `"C:\Program Files\App\app.exe" --silent`.
/// This pulls out just the executable path so we can check its signature/age.
fn extract_exe_path(command: &str) -> String {
    let trimmed = command.trim();
    if let Some(rest) = trimmed.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    trimmed
        .split_whitespace()
        .next()
        .unwrap_or(trimmed)
        .to_string()
}

fn build_entry(name: String, command: String, source: PersistenceType) -> PersistenceFacts {
    let exe_path = extract_exe_path(&command);
    let sig = signature::check_signature(&exe_path);
    PersistenceFacts {
        name,
        command,
        source,
        publisher: sig.publisher,
        is_signed: sig.is_signed,
        file_age_days: file_age_days(&exe_path),
    }
}

/// Collects every startup/persistence entry we currently know how to find:
/// registry Run keys, the Startup folder, and (non-Microsoft) scheduled tasks.
pub fn collect_persistence_entries() -> Vec<PersistenceFacts> {
    let mut entries = Vec::new();
    entries.extend(collect_registry_run_keys());
    entries.extend(collect_startup_folder());
    entries.extend(collect_scheduled_tasks());
    entries
}

#[cfg(windows)]
fn collect_registry_run_keys() -> Vec<PersistenceFacts> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    const SUBKEYS: [&str; 2] = [
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
    ];

    let mut results = Vec::new();
    let roots = [(HKEY_LOCAL_MACHINE, "HKLM"), (HKEY_CURRENT_USER, "HKCU")];

    for (root_const, _root_name) in roots {
        let root = RegKey::predef(root_const);
        for subkey_path in SUBKEYS {
            let Ok(key) = root.open_subkey(subkey_path) else {
                continue;
            };
            let names: Vec<String> = key
                .enum_values()
                .filter_map(|v| v.ok())
                .map(|(name, _)| name)
                .collect();

            for name in names {
                if let Ok(command) = key.get_value::<String, _>(&name) {
                    results.push(build_entry(name, command, PersistenceType::RegistryRun));
                }
            }
        }
    }

    results
}

#[cfg(not(windows))]
fn collect_registry_run_keys() -> Vec<PersistenceFacts> {
    Vec::new()
}

fn scan_startup_dir(dir: &Path) -> Vec<PersistenceFacts> {
    let mut results = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return results;
    };

    for entry in read_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        // Skip desktop.ini and similar folder metadata files.
        if path.extension().and_then(|e| e.to_str()) == Some("ini") {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let command = path.to_string_lossy().to_string();
        results.push(build_entry(name, command, PersistenceType::StartupFolder));
    }

    results
}

fn collect_startup_folder() -> Vec<PersistenceFacts> {
    let mut results = Vec::new();

    if let Some(appdata) = std::env::var_os("APPDATA") {
        let path = Path::new(&appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        results.extend(scan_startup_dir(&path));
    }
    if let Some(programdata) = std::env::var_os("PROGRAMDATA") {
        let path = Path::new(&programdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        results.extend(scan_startup_dir(&path));
    }

    results
}

/// Uses `schtasks /query` rather than the Task Scheduler COM API — simpler
/// to get working for v1, at the cost of depending on English-locale field
/// labels ("TaskName:", "Task To Run:"). On a non-English Windows install
/// this will silently return nothing; switching to the COM API removes that
/// limitation but is a bigger lift, worth revisiting post-v1.
fn collect_scheduled_tasks() -> Vec<PersistenceFacts> {
    let output = Command::new("schtasks").args(["/query", "/fo", "LIST", "/v"]).output();

    let Ok(output) = output else {
        return Vec::new();
    };

    let text = String::from_utf8_lossy(&output.stdout);
    parse_schtasks_output(&text)
}

fn parse_schtasks_output(text: &str) -> Vec<PersistenceFacts> {
    let mut results = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_command: Option<String> = None;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("TaskName:") {
            if let (Some(name), Some(command)) = (current_name.take(), current_command.take()) {
                push_if_not_builtin(&mut results, name, command);
            }
            current_name = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Task To Run:") {
            current_command = Some(rest.trim().to_string());
        }
    }
    if let (Some(name), Some(command)) = (current_name, current_command) {
        push_if_not_builtin(&mut results, name, command);
    }

    results
}

/// Windows ships hundreds of its own scheduled tasks under `\Microsoft\Windows\...`.
/// Surfacing all of them would bury anything actually worth looking at, so
/// we filter those out and only report user/third-party-created tasks.
fn push_if_not_builtin(results: &mut Vec<PersistenceFacts>, name: String, command: String) {
    if name.starts_with(r"\Microsoft\Windows\") {
        return;
    }
    if command.trim().is_empty() || command.trim() == "N/A" {
        return;
    }
    results.push(build_entry(name, command, PersistenceType::ScheduledTask));
}

