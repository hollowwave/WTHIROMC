use serde::Serialize;
use std::collections::HashMap;

/// Where a startup/persistence entry was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PersistenceType {
    RegistryRun,
    StartupFolder,
    ScheduledTask,
}

/// Raw facts about one thing set to run automatically (on login, on
/// schedule, etc.) — the persistence-scanner equivalent of ProcessFacts.
#[derive(Debug, Clone, Serialize)]
pub struct PersistenceFacts {
    pub name: String,
    /// The full command/path as found in the registry, startup folder, or task.
    pub command: String,
    pub source: PersistenceType,
    pub publisher: Option<String>,
    pub is_signed: bool,
    /// Why the file is unsigned, if it is - see `SignatureInfo::detail`.
    pub signature_detail: Option<String>,
    pub file_age_days: Option<i64>,
}

/// UI-facing shape for a persistence entry, mirroring ExplainedProcess.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainedPersistence {
    pub name: String,
    pub command: String,
    pub source: PersistenceType,
    pub publisher: Option<String>,
    pub risk_level: RiskLevel,
    pub score: i32,
    pub summary: String,
    pub explanations: Vec<String>,
    /// True if the user explicitly marked this safe via the allowlist -
    /// distinct from something that's naturally Green, so the UI can offer
    /// "remove from safe list" only where it actually applies.
    pub user_marked_safe: bool,
    /// True if this wasn't seen in any previous app launch - see the
    /// `history` module.
    pub is_new: bool,
}
/// "runs from a folder legitimate software rarely lives in".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RunLocation {
    System32,
    ProgramFiles,
    Temp,
    Downloads,
    AppData,
    Other,
}

/// Raw, uninterpreted facts about a single running process.
/// The Collector produces these. Nothing in this struct is a judgment call.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessFacts {
    pub pid: u32,
    pub name: String,
    pub exe_path: String,
    pub publisher: Option<String>,
    pub is_signed: bool,
    /// Why the file is unsigned, if it is - see `SignatureInfo::detail`.
    pub signature_detail: Option<String>,
    /// Age of the executable file in days, if we could determine it.
    pub file_age_days: Option<i64>,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub has_network_activity: bool,
    pub is_autostart: bool,
    pub run_location: RunLocation,
}

/// A single rule's verdict on a process. `context` holds values to be
/// substituted into that rule's explanation template (e.g. "days_old" -> "2").
#[derive(Debug, Clone)]
pub struct RuleHit {
    pub rule_id: &'static str,
    pub weight: i32,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum RiskLevel {
    Green,
    Yellow,
    Orange,
    Red,
    Black,
}

impl RiskLevel {
    pub fn from_score(score: i32) -> Self {
        match score {
            s if s < 10 => RiskLevel::Green,
            s if s < 30 => RiskLevel::Yellow,
            s if s < 50 => RiskLevel::Orange,
            s if s < 75 => RiskLevel::Red,
            _ => RiskLevel::Black,
        }
    }
}

/// Output of running every rule against one process's facts.
#[derive(Debug, Clone)]
pub struct RiskResult {
    pub score: i32,
    pub level: RiskLevel,
    pub hits: Vec<RuleHit>,
}

/// The final, UI-facing shape: facts + risk + human-readable explanation.
/// This is what gets sent to the frontend over Tauri IPC.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainedProcess {
    pub pid: u32,
    pub name: String,
    pub exe_path: String,
    pub publisher: Option<String>,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub risk_level: RiskLevel,
    pub score: i32,
    pub summary: String,
    pub explanations: Vec<String>,
    pub user_marked_safe: bool,
    /// True if this wasn't seen in any previous app launch - see the
    /// `history` module. Independent of the rule engine's score; a
    /// perfectly Green process can still be new.
    pub is_new: bool,
}

