//! Gathers raw, uninterpreted facts about the system: running processes
//! (`processes`), autostart entries (`persistence`), and digital signature
//! verification (`signature`, used by both of the above).
//!
//! Nothing in this module makes a judgment call about whether something is
//! safe or dangerous — that's the `rules` module's job. Keeping that
//! separation means every rule can be unit-tested against fabricated facts
//! without touching a real system.

pub mod persistence;
pub mod processes;
pub mod signature;
