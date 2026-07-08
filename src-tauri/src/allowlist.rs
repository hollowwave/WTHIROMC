//! User-controlled allowlist ("mark as safe"). This is deliberately kept
//! separate from `rules`/`explain` - it's a user preference, not a
//! detection heuristic, so those modules stay pure and testable without
//! needing a database. This module applies as a final layer on top of
//! their output, in `commands.rs`.

use crate::types::{ExplainedPersistence, ExplainedProcess, RiskLevel};
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::PathBuf;

fn db_path() -> PathBuf {
    let appdata = std::env::var_os("APPDATA").unwrap_or_default();
    let dir = std::path::Path::new(&appdata).join("WTHIROMC");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("wthiromc.db")
}

fn connect() -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS allowlist (
            identifier TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            added_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;
    Ok(conn)
}

/// Marks something as safe. `identifier` should be stable across scans
/// (an exe path for processes, `source:name` for persistence entries -
/// see `ExplainedPersistence`'s frontend-mirrored key). `label` is just for
/// display in a future "manage allowlist" screen.
pub fn mark_safe(identifier: &str, label: &str) -> Result<(), String> {
    let conn = connect().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO allowlist (identifier, label, added_at) VALUES (?1, ?2, datetime('now'))",
        params![identifier, label],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn unmark_safe(identifier: &str) -> Result<(), String> {
    let conn = connect().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM allowlist WHERE identifier = ?1", params![identifier])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Full allowlist as (identifier, label) pairs - for a future "manage your
/// allowlist" settings screen. Not wired into the UI yet (Phase 3 roadmap).
pub fn list_safe() -> Vec<(String, String)> {
    let Ok(conn) = connect() else { return Vec::new() };
    let Ok(mut stmt) = conn.prepare("SELECT identifier, label FROM allowlist ORDER BY added_at DESC") else {
        return Vec::new();
    };
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)));
    match rows {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

fn safe_set() -> HashSet<String> {
    list_safe().into_iter().map(|(identifier, _)| identifier).collect()
}

/// The identifier used for a persistence entry - kept in sync with the
/// frontend's `entryKey()` in `StartupList.tsx` so a mark-safe action from
/// either side of the IPC boundary refers to the same string.
pub fn persistence_identifier(entry: &ExplainedPersistence) -> String {
    format!("{:?}:{}", entry.source, entry.name)
}

/// Applies the allowlist on top of a fresh process scan. Called once per
/// scan from `commands::scan_processes` rather than per-process, to avoid
/// opening a database connection for every single running process.
pub fn apply_to_processes(mut processes: Vec<ExplainedProcess>) -> Vec<ExplainedProcess> {
    let safe = safe_set();
    for p in &mut processes {
        if safe.contains(&p.exe_path) {
            override_as_safe(&mut p.risk_level, &mut p.score, &mut p.summary, &mut p.explanations);
            p.user_marked_safe = true;
        }
    }
    processes
}

pub fn apply_to_persistence(mut entries: Vec<ExplainedPersistence>) -> Vec<ExplainedPersistence> {
    let safe = safe_set();
    for e in &mut entries {
        if safe.contains(&persistence_identifier(e)) {
            override_as_safe(&mut e.risk_level, &mut e.score, &mut e.summary, &mut e.explanations);
            e.user_marked_safe = true;
        }
    }
    entries
}

fn override_as_safe(
    risk_level: &mut RiskLevel,
    score: &mut i32,
    summary: &mut String,
    explanations: &mut Vec<String>,
) {
    *risk_level = RiskLevel::Green;
    *score = 0;
    *summary = "You marked this as safe, so WTHIROMC won't flag it going forward.".to_string();
    explanations.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises the same schema/queries as the real `connect()`/`mark_safe()`/
    /// `unmark_safe()` functions, but against an in-memory DB instead of the
    /// real file under %APPDATA% - keeps the test hermetic and fast.
    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS allowlist (
                identifier TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                added_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn mark_then_unmark_round_trips() {
        let conn = test_conn();
        conn.execute(
            "INSERT OR REPLACE INTO allowlist (identifier, label, added_at) VALUES (?1, ?2, datetime('now'))",
            params!["C:\\tools\\cargo.exe", "cargo"],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM allowlist", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        conn.execute(
            "DELETE FROM allowlist WHERE identifier = ?1",
            params!["C:\\tools\\cargo.exe"],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM allowlist", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn override_as_safe_forces_green_and_clears_explanations() {
        let mut level = RiskLevel::Red;
        let mut score = 90;
        let mut summary = "This looks dangerous.".to_string();
        let mut explanations = vec!["unsigned".to_string(), "recent".to_string()];

        override_as_safe(&mut level, &mut score, &mut summary, &mut explanations);

        assert_eq!(level, RiskLevel::Green);
        assert_eq!(score, 0);
        assert!(explanations.is_empty());
        assert!(summary.contains("marked this as safe"));
    }

    #[test]
    fn persistence_identifier_matches_frontend_key_format() {
        use crate::types::PersistenceType;

        let entry = ExplainedPersistence {
            name: "GoogleUpdateTaskMachineCore".to_string(),
            command: "C:\\Program Files\\Google\\Update\\GoogleUpdate.exe".to_string(),
            source: PersistenceType::ScheduledTask,
            publisher: Some("Google LLC".to_string()),
            risk_level: RiskLevel::Green,
            score: 0,
            summary: String::new(),
            explanations: Vec::new(),
            user_marked_safe: false,
        };

        // Must match the frontend's `entryKey()` in StartupList.tsx exactly:
        // `${e.source}:${e.name}` - since PersistenceType's Debug output and
        // its serde wire format are both the plain variant name.
        assert_eq!(
            persistence_identifier(&entry),
            "ScheduledTask:GoogleUpdateTaskMachineCore"
        );
    }
}

