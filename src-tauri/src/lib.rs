//! WTHIROMC — What The Hell Is Running On My Computer.
//!
//! A three-layer pipeline, kept deliberately decoupled:
//!
//! 1. `collector` — gathers raw, uninterpreted facts about the system
//!    (running processes, startup/persistence entries, digital signatures).
//! 2. `rules` — pure functions that turn facts into a weighted risk score.
//!    No knowledge of collection or presentation.
//! 3. `explain` — turns a risk score into plain-English sentences a
//!    non-expert can read. No knowledge of how the score was computed.
//!
//! `types` defines the data that flows between them; `commands` exposes
//! the whole pipeline to the frontend over Tauri IPC.
//!
//! See `docs/plan.md` for the full design rationale and milestone plan.

pub mod collector;
pub mod commands;
pub mod explain;
pub mod rules;
pub mod types;
