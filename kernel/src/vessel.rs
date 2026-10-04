use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Isolation envelope identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VesselId(pub Uuid);

impl VesselId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for VesselId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VesselId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Capability policy attached to a vessel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VesselPolicy {
    pub label: String,
    pub allowed_facets: Vec<String>,
    pub max_agents: usize,
    pub clarity_allowed: bool,
}

impl VesselPolicy {
    pub fn developer() -> Self {
        Self {
            label: "developer".into(),
            allowed_facets: vec![
                "intent.emit".into(),
                "fabric.read".into(),
                "fabric.weave".into(),
                "agent.spawn".into(),
                "vessel.open".into(),
                "clarity.grant".into(),
                "silicon.bind".into(),
                "silicon.nvidia.cuda".into(),
                "silicon.nvidia.tensor".into(),
                "silicon.amd.zen".into(),
                "silicon.amd.rocm".into(),
            ],
            max_agents: 64,
            clarity_allowed: true,
        }
    }

    pub fn sandbox() -> Self {
        Self {
            label: "sandbox".into(),
            allowed_facets: vec!["intent.emit".into(), "fabric.read".into()],
            max_agents: 8,
            clarity_allowed: false,
        }
    }

    /// Private vessel — no telemetry facet can ever be granted.
    pub fn private() -> Self {
        Self {
            label: "private".into(),
            allowed_facets: vec![
                "intent.emit".into(),
                "fabric.read".into(),
                "fabric.weave".into(),
                "agent.spawn".into(),
                "silicon.bind".into(),
                "silicon.nvidia.cuda".into(),
                "silicon.amd.zen".into(),
                "silicon.amd.rocm".into(),
            ],
            max_agents: 32,
            clarity_allowed: false,
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label.to_ascii_lowercase().as_str() {
            "sandbox" => Self::sandbox(),
            "private" => Self::private(),
            _ => Self::developer(),
        }
    }

    pub fn allows(&self, facet: &str) -> bool {
        self.allowed_facets.iter().any(|f| f == facet)
    }
}

/// Vessel — AIOS isolation + capability boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vessel {
    pub id: VesselId,
    pub name: String,
    pub policy: VesselPolicy,
    pub sealed: bool,
    pub created_at: DateTime<Utc>,
}

impl Vessel {
    pub fn open(name: impl Into<String>, policy: VesselPolicy) -> Self {
        Self {
            id: VesselId::new(),
            name: name.into(),
            policy,
            sealed: false,
            created_at: Utc::now(),
        }
    }

    pub fn seal(&mut self) {
        self.sealed = true;
    }
}
