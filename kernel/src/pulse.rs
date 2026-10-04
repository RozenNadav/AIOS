use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::agent::{AgentId, AgentState};

/// One observation emitted by the Pulse scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseEvent {
    pub agent: AgentId,
    pub from_state: AgentState,
    pub to_state: AgentState,
    pub attention: f32,
    pub at: DateTime<Utc>,
    pub note: String,
}

/// Attention-based scheduler — not a Unix time-slice CFS clone.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PulseScheduler {
    pub ticks: u64,
    pub history: Vec<PulseEvent>,
}

impl PulseScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance cognition: prefer high-attention awake/attend agents.
    pub fn tick(
        &mut self,
        agents: &mut [crate::agent::Agent],
    ) -> Vec<PulseEvent> {
        self.ticks += 1;
        let mut events = Vec::new();

        let mut order: Vec<usize> = (0..agents.len()).collect();
        order.sort_by(|a, b| {
            agents[*b]
                .attention
                .partial_cmp(&agents[*a].attention)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for idx in order {
            let agent = &mut agents[idx];
            if agent.state == AgentState::Dissolve || agent.state == AgentState::Rest {
                continue;
            }

            let from = agent.state;
            match agent.state {
                AgentState::Seed => {
                    agent.awaken();
                    events.push(PulseEvent {
                        agent: agent.id,
                        from_state: from,
                        to_state: agent.state,
                        attention: agent.attention,
                        at: Utc::now(),
                        note: format!("pulse awoke '{}'", agent.name),
                    });
                }
                AgentState::Awake | AgentState::Attend => {
                    agent.act();
                    events.push(PulseEvent {
                        agent: agent.id,
                        from_state: from,
                        to_state: agent.state,
                        attention: agent.attention,
                        at: Utc::now(),
                        note: format!("pulse engaged '{}'", agent.name),
                    });
                }
                AgentState::Act => {
                    // Cool down after acting so others can receive attention.
                    if agent.attention < 0.4 {
                        agent.rest();
                        events.push(PulseEvent {
                            agent: agent.id,
                            from_state: from,
                            to_state: agent.state,
                            attention: agent.attention,
                            at: Utc::now(),
                            note: format!("pulse rested '{}'", agent.name),
                        });
                    } else {
                        agent.attend(0.05);
                        events.push(PulseEvent {
                            agent: agent.id,
                            from_state: from,
                            to_state: agent.state,
                            attention: agent.attention,
                            at: Utc::now(),
                            note: format!("pulse re-attended '{}'", agent.name),
                        });
                    }
                }
                AgentState::Rest | AgentState::Dissolve => {}
            }
        }

        self.history.extend(events.clone());
        if self.history.len() > 200 {
            let drain = self.history.len() - 200;
            self.history.drain(0..drain);
        }
        events
    }
}
