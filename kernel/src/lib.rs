//! AIOS Cognition Core
//!
//! This crate is the kernel of AIOS. It deliberately does **not** expose
//! POSIX, Unix, or Windows abstractions. Primaries are intents, agents,
//! facets, vessels, and the knowledge fabric.

pub mod agent;
pub mod boot;
pub mod cog;
pub mod fabric;
pub mod intent;
pub mod pulse;
pub mod silicon;
pub mod sovereign;
pub mod vessel;

#[cfg(test)]
mod tests;

pub use agent::{Agent, AgentId, AgentState};
pub use boot::{BootReport, Spark};
pub use cog::{Cog, CogSnapshot};
pub use fabric::{Fabric, Node, NodeId, Relation};
pub use intent::{Intent, IntentKind, IntentResult, Urgency};
pub use pulse::{PulseEvent, PulseScheduler};
pub use silicon::{AccelJob, Die, DieKind, Lane, Lattice, Vendor};
pub use sovereign::{AttentionGuard, Revision, RevisionState, StickyPref};
pub use vessel::{Vessel, VesselId, VesselPolicy};

/// AIOS kernel version string.
pub const AIOS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Human-readable identity of this OS lineage.
pub const AIOS_LINEAGE: &str = "AIOS Cognition Core — independent AI-native lineage";

/// Doctrine tagline for Surfaces.
pub const AIOS_DOCTRINE: &str =
    "Consent-first · Sovereign revisions · Attention guard · Glass-box explain";
