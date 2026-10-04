use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::vessel::VesselId;

/// Stable identity of an agent (not a PID).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lifecycle of an AIOS agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentState {
    Seed,
    Awake,
    Attend,
    Act,
    Rest,
    Dissolve,
}

/// Living computational entity — AIOS substitute for "process".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub purpose: String,
    pub state: AgentState,
    pub vessel: VesselId,
    /// Capability ports this agent exposes or may use.
    pub facets: Vec<String>,
    pub attention: f32,
    pub created_at: DateTime<Utc>,
    pub last_pulse: DateTime<Utc>,
}

impl Agent {
    pub fn seed(name: impl Into<String>, purpose: impl Into<String>, vessel: VesselId) -> Self {
        let now = Utc::now();
        Self {
            id: AgentId::new(),
            name: name.into(),
            purpose: purpose.into(),
            state: AgentState::Seed,
            vessel,
            facets: vec!["intent.emit".into(), "fabric.read".into()],
            attention: 1.0,
            created_at: now,
            last_pulse: now,
        }
    }

    pub fn awaken(&mut self) {
        if self.state == AgentState::Seed || self.state == AgentState::Rest {
            self.state = AgentState::Awake;
            self.last_pulse = Utc::now();
        }
    }

    pub fn rest(&mut self) {
        if self.state != AgentState::Dissolve {
            self.state = AgentState::Rest;
            self.attention = (self.attention * 0.5).max(0.05);
            self.last_pulse = Utc::now();
        }
    }

    pub fn dissolve(&mut self) {
        self.state = AgentState::Dissolve;
        self.attention = 0.0;
        self.last_pulse = Utc::now();
    }

    pub fn attend(&mut self, boost: f32) {
        if self.state == AgentState::Dissolve {
            return;
        }
        self.state = AgentState::Attend;
        self.attention = (self.attention + boost).clamp(0.0, 10.0);
        self.last_pulse = Utc::now();
    }

    pub fn act(&mut self) {
        if matches!(
            self.state,
            AgentState::Awake | AgentState::Attend | AgentState::Act
        ) {
            self.state = AgentState::Act;
            self.attention = (self.attention - 0.15).max(0.1);
            self.last_pulse = Utc::now();
        }
    }
}
