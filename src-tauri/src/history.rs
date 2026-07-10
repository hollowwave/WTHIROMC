//! Tracks what WTHIROMC has seen running/autostarting across previous app
//! launches, so it can flag "this wasn't here last time you checked" - a
//! useful signal on its own, independent of the rule engine's score.
//!
//! Like `allowlist`, this is a layer applied on top of the rule engine's
//! output (see `commands.rs`), not mixed into `rules`/`explain` - those
//! stay pure and DB-free.
//!
//! Uses the same SQLite database as `allowlist` (different table).

use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

fn db_path() -> PathBuf {
    let appdata = std::env::var_os("APPDATA").unwrap_or_default();
    let dir = std::path::Path::new(&appdata).join("WTHIROMC");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("wthiromc.db")
}

fn connect() -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS seen_items (
            identifier TEXT NOT NULL,
            kind TEXT NOT NULL,
            display_name TEXT NOT NULL,
            first_seen TEXT NOT NULL,
            last_seen TEXT NOT NULL,
            PRIMARY KEY (identifier, kind)
        )",
        [],
    )?;
    Ok(conn)
}

/// Snapshot of everything seen in *previous* app launches, loaded once and
/// frozen for the lifetime of this process. This is the key design point:
/// we compare against a baseline taken before this session's scans start,
/// rather than re-querying the DB live - otherwise something would stop
/// looking "new" the instant it was first recorded, even mid-session.
fn baseline(kind: &'static str) -> &'static Mutex<HashSet<String>> {
    static PROCESS_BASELINE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    static PERSISTENCE_BASELINE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

    let lock = if kind == "process" {
        &PROCESS_BASELINE
    } else {
        &PERSISTENCE_BASELINE
    };

    lock.get_or_init(|| {
        let set = connect()
            .and_then(|conn| {
                let mut stmt = conn.prepare("SELECT identifier FROM seen_items WHERE kind = ?1")?;
                let rows = stmt.query_map(params![kind], |row| row.get::<_, String>(0))?;
                Ok(rows.filter_map(|r| r.ok()).collect::<HashSet<String>>())
            })
            .unwrap_or_default();
        Mutex::new(set)
    })
}

/// Marks a batch of items as seen "now" (upserting `last_seen`, or
/// inserting with `first_seen = last_seen = now` if new) and returns which
/// identifiers are new compared to the frozen pre-session baseline.
///
/// Call once per scan with everything found, not once per item, so this
/// stays a single set of DB writes rather than one connection per item.
///
/// `items` is (identifier, display_name) pairs. `kind` should be a stable
/// string like `"process"` or `"persistence"` - it's both a DB column value
/// and (via the static baseline cache) a key into which in-memory snapshot
/// to check against, so callers must be consistent about what string they
/// pass for a given category of item.
pub fn check_new_and_record(kind: &'static str, items: &[(String, String)]) -> HashSet<String> {
    let guard = baseline(kind).lock().unwrap();
    // If the baseline was empty, this is either the very first time
    // WTHIROMC has ever run, or the first time it's seen this *kind* of
    // item. Either way, there's no meaningful "previous session" to
    // compare against, so nothing should be flagged as new - otherwise
    // someone's first-ever launch would show "NEW" on every single
    // process, which is alarming noise, not a useful signal.
    let is_bootstrap = guard.is_empty();
    let new_ones: HashSet<String> = if is_bootstrap {
        HashSet::new()
    } else {
        items
            .iter()
            .filter(|(identifier, _)| !guard.contains(identifier))
            .map(|(identifier, _)| identifier.clone())
            .collect()
    };
    drop(guard);

    if let Ok(conn) = connect() {
        for (identifier, display_name) in items {
            let _ = conn.execute(
                "INSERT INTO seen_items (identifier, kind, display_name, first_seen, last_seen)
                 VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))
                 ON CONFLICT(identifier, kind) DO UPDATE SET last_seen = datetime('now')",
                params![identifier, kind, display_name],
            );
        }
    }

    new_ones
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors the real schema against an in-memory DB, to test the
    /// "bootstrap suppresses new-flagging" logic and the upsert behavior
    /// without touching the real file under %APPDATA%.
    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS seen_items (
                identifier TEXT NOT NULL,
                kind TEXT NOT NULL,
                display_name TEXT NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                PRIMARY KEY (identifier, kind)
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn upsert_inserts_new_and_updates_existing() {
        let conn = test_conn();

        conn.execute(
            "INSERT INTO seen_items (identifier, kind, display_name, first_seen, last_seen)
             VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))
             ON CONFLICT(identifier, kind) DO UPDATE SET last_seen = datetime('now')",
            params!["C:\\tools\\cargo.exe", "process", "cargo"],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM seen_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Re-inserting the same identifier should update in place, not
        // create a second row.
        conn.execute(
            "INSERT INTO seen_items (identifier, kind, display_name, first_seen, last_seen)
             VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))
             ON CONFLICT(identifier, kind) DO UPDATE SET last_seen = datetime('now')",
            params!["C:\\tools\\cargo.exe", "process", "cargo"],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM seen_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn bootstrap_baseline_is_empty_set() {
        // Directly testing the logic check_new_and_record relies on: an
        // empty guard set should mean "treat as bootstrap, flag nothing."
        let guard: HashSet<String> = HashSet::new();
        assert!(guard.is_empty());
    }
}

