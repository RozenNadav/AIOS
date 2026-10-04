use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::AgentId;
use crate::fabric::NodeId;
use crate::vessel::VesselId;

/// How urgently Cog should attend to an intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Urgency {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            "critical" | "crit" => Some(Self::Critical),
            _ => None,
        }
    }
}

/// Kind of purpose expressed to Cog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "verb")]
pub enum IntentKind {
    /// Ignite / query Cog itself.
    Attend { topic: String },
    /// Birth a new agent with a purpose.
    SpawnAgent { name: String, purpose: String },
    /// Quiet an agent (Rest state).
    RestAgent { agent: AgentId },
    /// Dissolve an agent entirely.
    DissolveAgent { agent: AgentId },
    /// Weave a semantic node into the fabric.
    WeaveNode {
        title: String,
        node_kind: String,
        body: String,
    },
    /// Relate two fabric nodes.
    Relate {
        from: NodeId,
        to: NodeId,
        label: String,
    },
    /// Query fabric by semantic text.
    QueryFabric { query: String },
    /// Open a vessel with a policy label.
    OpenVessel { name: String, policy: String },
    /// Seal a vessel.
    SealVessel { vessel: VesselId },
    /// Grant temporary clarity (capability elevation).
    ClarityGrant { agent: AgentId, facet: String },
    /// Emit a free-form developer note into fabric.
    Note { text: String },
    /// Pulse: force one scheduler tick.
    Tick,
    /// Snapshot kernel state.
    Snapshot,
    /// Glass-box diagnostics — explain what Cog is doing (anti-"just reboot").
    Explain { topic: String },
    /// Propose a sovereign revision (anti-forced-update).
    ProposeRevision { title: String, summary: String },
    /// Approve a proposed revision by id prefix or title.
    ApproveRevision { rev: String },
    /// Reject a proposed revision.
    RejectRevision { rev: String },
    /// Apply an approved revision.
    ApplyRevision { rev: String },
    /// Roll back an applied revision.
    RollbackRevision { rev: String },
    /// List revisions.
    ListRevisions,
    /// Enable / disable Focus Shield (anti-interruption).
    Focus { enabled: bool },
    /// Set a sticky preference (immune to silent mutation).
    PrefSet { key: String, value: String, sticky: bool },
    /// Read preferences.
    PrefGet { key: String },
    /// Capture continuity snapshot of live agents + prefs.
    ContinuitySave { name: String },
    /// Restore a continuity snapshot by name.
    ContinuityRestore { name: String },
    /// Probe host for NVIDIA GPUs and AMD CPU/GPU silicon.
    SiliconProbe,
    /// List silicon lattice dies and lanes.
    SiliconList,
    /// Bind an agent to a die (grants vendor compute facets).
    SiliconBind { agent: AgentId, die: String },
    /// Unbind an agent from all dies.
    SiliconUnbind { agent: AgentId },
    /// Schedule a compute burst on the agent's bound die.
    SiliconAccel { agent: AgentId, work: String },
    /// Declare a lab/target die when hardware is not locally present.
    SiliconDeclare {
        vendor: String,
        kind: String,
        name: String,
        model: String,
        memory_mb: u64,
        units: u32,
    },
}

/// A typed purpose message on the Intent Bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub id: Uuid,
    pub actor: String,
    pub kind: IntentKind,
    pub urgency: Urgency,
    pub created_at: DateTime<Utc>,
}

impl Intent {
    pub fn new(actor: impl Into<String>, kind: IntentKind, urgency: Urgency) -> Self {
        Self {
            id: Uuid::new_v4(),
            actor: actor.into(),
            kind,
            urgency,
            created_at: Utc::now(),
        }
    }
}

/// Outcome of Cog resolving an intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub intent_id: Uuid,
    pub ok: bool,
    pub message: String,
    pub data: serde_json::Value,
}

impl IntentResult {
    pub fn ok(intent_id: Uuid, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            intent_id,
            ok: true,
            message: message.into(),
            data,
        }
    }

    pub fn err(intent_id: Uuid, message: impl Into<String>) -> Self {
        Self {
            intent_id,
            ok: false,
            message: message.into(),
            data: serde_json::Value::Null,
        }
    }
}
