use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::agent::{Agent, AgentId, AgentState};
use crate::boot::{BootReport, Spark};
use crate::fabric::Fabric;
use crate::intent::{Intent, IntentKind, IntentResult, Urgency};
use crate::pulse::PulseScheduler;
use crate::silicon::Lattice;
use crate::sovereign::{AttentionGuard, Revision, RevisionState, StickyPref};
use crate::vessel::{Vessel, VesselId, VesselPolicy};
use crate::{AIOS_DOCTRINE, AIOS_LINEAGE, AIOS_VERSION};

/// Serializable kernel view for console / shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CogSnapshot {
    pub version: String,
    pub lineage: String,
    pub doctrine: String,
    pub ignited: bool,
    pub boot: Option<BootReport>,
    pub vessels: Vec<Vessel>,
    pub agents: Vec<Agent>,
    pub fabric_nodes: usize,
    pub fabric_relations: usize,
    pub pulse_ticks: u64,
    pub clarity_grants: Vec<ClarityGrant>,
    pub intent_log: Vec<IntentLogEntry>,
    pub revisions: Vec<Revision>,
    pub attention_guard: AttentionGuard,
    pub prefs: Vec<StickyPref>,
    pub continuity: Vec<ContinuitySnap>,
    pub lattice: Lattice,
    pub saved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuitySnap {
    pub name: String,
    pub agent_names: Vec<String>,
    pub prefs: Vec<StickyPref>,
    pub saved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarityGrant {
    pub agent: AgentId,
    pub facet: String,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentLogEntry {
    pub intent: Intent,
    pub result: IntentResult,
}

/// Cognition Core — the AIOS kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cog {
    pub version: String,
    pub lineage: String,
    pub ignited: bool,
    pub boot: Option<BootReport>,
    pub vessels: Vec<Vessel>,
    pub agents: Vec<Agent>,
    pub fabric: Fabric,
    pub pulse: PulseScheduler,
    pub clarity_grants: Vec<ClarityGrant>,
    pub intent_log: Vec<IntentLogEntry>,
    pub prime_vessel: Option<VesselId>,
    #[serde(default)]
    pub revisions: Vec<Revision>,
    #[serde(default)]
    pub attention_guard: AttentionGuard,
    #[serde(default)]
    pub prefs: Vec<StickyPref>,
    #[serde(default)]
    pub continuity: Vec<ContinuitySnap>,
    #[serde(default)]
    pub lattice: Lattice,
}

impl Default for Cog {
    fn default() -> Self {
        Self::new()
    }
}

impl Cog {
    pub fn new() -> Self {
        Self {
            version: AIOS_VERSION.into(),
            lineage: AIOS_LINEAGE.into(),
            ignited: false,
            boot: None,
            vessels: Vec::new(),
            agents: Vec::new(),
            fabric: Fabric::new(),
            pulse: PulseScheduler::new(),
            clarity_grants: Vec::new(),
            intent_log: Vec::new(),
            prime_vessel: None,
            revisions: Vec::new(),
            attention_guard: AttentionGuard::default(),
            prefs: Vec::new(),
            continuity: Vec::new(),
            lattice: Lattice::new(),
        }
    }

    /// Default fabric path under the developer's AIOS home.
    pub fn default_state_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".aios").join("cog-state.json")
    }

    pub fn ignite(&mut self) -> BootReport {
        let report = Spark::ignite();
        let prime = Vessel::open("prime", VesselPolicy::developer());
        self.prime_vessel = Some(prime.id);
        self.vessels.push(prime);

        // Seed fabric with lineage markers + doctrine.
        self.fabric.weave(
            "AIOS",
            "system",
            "Cognition-first operating system. Independent lineage — not Unix, not Windows.",
        );
        self.fabric.weave(
            "Developer Surface",
            "surface",
            "Primary interface for AIOS DevCore: intent shell + developer console.",
        );
        self.fabric.weave(
            "Doctrine",
            "doctrine",
            "Consent-first AI. Sovereign revisions (no forced updates). Attention guard (no nags). Glass-box explain (no 'just reboot'). Sticky prefs. Focus shield.",
        );

        // Sticky defaults — immune to silent mutation.
        self.prefs.push(StickyPref::set("telemetry", "off", true));
        self.prefs.push(StickyPref::set("assistant.unsolicited", "off", true));
        self.prefs.push(StickyPref::set("updates.auto_apply", "off", true));
        self.prefs.push(StickyPref::set("ads", "off", true));
        self.prefs.push(StickyPref::set("silicon.prefer", "nvidia,amd", true));

        // Probe Silicon Lattice for NVIDIA GPUs and AMD chips.
        let notes = self.lattice.probe();
        let n_gpu = self.lattice.nvidia_gpus().count();
        let a_cpu = self.lattice.amd_cpus().count();
        let a_gpu = self.lattice.amd_gpus().count();
        self.fabric.weave(
            "Silicon Lattice",
            "silicon",
            &format!(
                "NVIDIA GPUs={n_gpu} · AMD CPUs={a_cpu} · AMD GPUs={a_gpu}. Notes: {}",
                notes.join("; ")
            ),
        );

        // Shell agent lives in the prime vessel.
        if let Some(vid) = self.prime_vessel {
            let mut shell = Agent::seed("shell", "Interpret developer intents", vid);
            shell.facets.push("fabric.weave".into());
            shell.facets.push("agent.spawn".into());
            shell.facets.push("silicon.bind".into());
            shell.awaken();
            self.agents.push(shell);
        }

        self.boot = Some(report.clone());
        self.ignited = true;
        report
    }

    fn checkpoint_marker(&self) -> String {
        let agents: Vec<_> = self
            .agents
            .iter()
            .filter(|a| a.state != AgentState::Dissolve)
            .map(|a| a.name.clone())
            .collect();
        format!("nodes={} agents={}", self.fabric.nodes.len(), agents.join(","))
    }

    fn find_revision_mut(&mut self, rev: &str) -> Option<&mut Revision> {
        let needle = rev.to_ascii_lowercase();
        self.revisions.iter_mut().find(|r| {
            r.id.to_string().starts_with(rev)
                || r.title.to_ascii_lowercase() == needle
                || r.title.to_ascii_lowercase().contains(&needle)
        })
    }

    fn find_revision(&self, rev: &str) -> Option<&Revision> {
        let needle = rev.to_ascii_lowercase();
        self.revisions.iter().find(|r| {
            r.id.to_string().starts_with(rev)
                || r.title.to_ascii_lowercase() == needle
                || r.title.to_ascii_lowercase().contains(&needle)
        })
    }

    fn explain_report(&self, topic: &str) -> serde_json::Value {
        let alive: Vec<_> = self
            .agents
            .iter()
            .filter(|a| a.state != AgentState::Dissolve)
            .map(|a| {
                serde_json::json!({
                    "name": a.name,
                    "state": format!("{:?}", a.state).to_ascii_lowercase(),
                    "purpose": a.purpose,
                    "attention": a.attention,
                })
            })
            .collect();
        let recent: Vec<_> = self
            .intent_log
            .iter()
            .rev()
            .take(5)
            .map(|e| {
                serde_json::json!({
                    "ok": e.result.ok,
                    "message": e.result.message,
                    "actor": e.intent.actor,
                })
            })
            .collect();
        serde_json::json!({
            "topic": topic,
            "doctrine": AIOS_DOCTRINE,
            "why_not_reboot": "AIOS explains live state instead of asking you to reboot.",
            "attention_guard": self.attention_guard,
            "focus_shield": self.attention_guard.focus_shield,
            "blocked_claims": self.attention_guard.blocked_claims,
            "agents": alive,
            "vessels": self.vessels.iter().map(|v| serde_json::json!({
                "name": v.name,
                "policy": v.policy.label,
                "sealed": v.sealed,
            })).collect::<Vec<_>>(),
            "prefs": self.prefs,
            "revisions_open": self.revisions.iter().filter(|r| matches!(r.state, RevisionState::Proposed | RevisionState::Approved)).count(),
            "fabric_nodes": self.fabric.nodes.len(),
            "pulse_ticks": self.pulse.ticks,
            "silicon": {
                "nvidia_gpus": self.lattice.nvidia_gpus().count(),
                "amd_cpus": self.lattice.amd_cpus().count(),
                "amd_gpus": self.lattice.amd_gpus().count(),
                "dies": self.lattice.dies.len(),
                "probe_notes": self.lattice.probe_notes,
            },
            "recent_intents": recent,
        })
    }

    pub fn ensure_ignited(&mut self) {
        if !self.ignited {
            self.ignite();
        }
    }

    fn vessel_mut(&mut self, id: VesselId) -> Option<&mut Vessel> {
        self.vessels.iter_mut().find(|v| v.id == id)
    }

    fn agent_mut(&mut self, id: AgentId) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id == id)
    }

    fn find_agent_by_name(&self, name: &str) -> Option<&Agent> {
        self.agents
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(name) && a.state != AgentState::Dissolve)
    }

    fn resolve_agent_ref(&self, raw: &str) -> Option<AgentId> {
        if let Ok(u) = uuid::Uuid::parse_str(raw) {
            let id = AgentId(u);
            if self.agents.iter().any(|a| a.id == id) {
                return Some(id);
            }
        }
        self.find_agent_by_name(raw).map(|a| a.id)
    }

    fn resolve_vessel_ref(&self, raw: &str) -> Option<VesselId> {
        if let Ok(u) = uuid::Uuid::parse_str(raw) {
            let id = VesselId(u);
            if self.vessels.iter().any(|v| v.id == id) {
                return Some(id);
            }
        }
        self.vessels
            .iter()
            .find(|v| v.name.eq_ignore_ascii_case(raw))
            .map(|v| v.id)
    }

    /// Submit an intent on the Intent Bus.
    pub fn submit(&mut self, intent: Intent) -> IntentResult {
        self.ensure_ignited();
        let result = self.dispatch(&intent);
        self.intent_log.push(IntentLogEntry {
            intent,
            result: result.clone(),
        });
        if self.intent_log.len() > 100 {
            let drain = self.intent_log.len() - 100;
            self.intent_log.drain(0..drain);
        }
        result
    }

    fn dispatch(&mut self, intent: &Intent) -> IntentResult {
        let id = intent.id;
        match &intent.kind {
            IntentKind::Attend { topic } => IntentResult::ok(
                id,
                format!("Cog attending: {topic}"),
                serde_json::json!({
                    "topic": topic,
                    "agents": self.agents.len(),
                    "nodes": self.fabric.nodes.len(),
                    "ticks": self.pulse.ticks,
                }),
            ),
            IntentKind::SpawnAgent { name, purpose } => {
                let vessel = match self.prime_vessel {
                    Some(v) => v,
                    None => return IntentResult::err(id, "no prime vessel"),
                };
                if let Some(v) = self.vessels.iter().find(|x| x.id == vessel) {
                    if v.sealed {
                        return IntentResult::err(id, "prime vessel is sealed");
                    }
                    let alive = self
                        .agents
                        .iter()
                        .filter(|a| a.vessel == vessel && a.state != AgentState::Dissolve)
                        .count();
                    if alive >= v.policy.max_agents {
                        return IntentResult::err(id, "vessel agent limit reached");
                    }
                    if !v.policy.allows("agent.spawn") {
                        return IntentResult::err(id, "vessel policy denies agent.spawn");
                    }
                }
                let mut agent = Agent::seed(name, purpose, vessel);
                agent.awaken();
                let aid = agent.id;
                self.agents.push(agent);
                IntentResult::ok(
                    id,
                    format!("agent '{name}' seeded and awake"),
                    serde_json::json!({ "agent_id": aid }),
                )
            }
            IntentKind::RestAgent { agent } => {
                match self.agent_mut(*agent) {
                    Some(a) => {
                        a.rest();
                        IntentResult::ok(id, format!("agent '{}' resting", a.name), serde_json::json!({}))
                    }
                    None => IntentResult::err(id, "unknown agent"),
                }
            }
            IntentKind::DissolveAgent { agent } => {
                match self.agent_mut(*agent) {
                    Some(a) => {
                        let name = a.name.clone();
                        a.dissolve();
                        IntentResult::ok(id, format!("agent '{name}' dissolved"), serde_json::json!({}))
                    }
                    None => IntentResult::err(id, "unknown agent"),
                }
            }
            IntentKind::WeaveNode {
                title,
                node_kind,
                body,
            } => {
                let node = self.fabric.weave(title, node_kind, body);
                IntentResult::ok(
                    id,
                    format!("wove node '{}'", node.title),
                    serde_json::json!({ "node_id": node.id, "kind": node.kind }),
                )
            }
            IntentKind::Relate { from, to, label } => match self.fabric.relate(*from, *to, label) {
                Ok(rel) => IntentResult::ok(
                    id,
                    format!("related {} -[{}]-> {}", rel.from, rel.label, rel.to),
                    serde_json::json!({ "from": rel.from, "to": rel.to, "label": rel.label }),
                ),
                Err(e) => IntentResult::err(id, e),
            },
            IntentKind::QueryFabric { query } => {
                let hits: Vec<_> = self
                    .fabric
                    .query(query)
                    .into_iter()
                    .map(|n| {
                        serde_json::json!({
                            "id": n.id,
                            "title": n.title,
                            "kind": n.kind,
                            "body": n.body,
                        })
                    })
                    .collect();
                IntentResult::ok(
                    id,
                    format!("{} fabric hit(s)", hits.len()),
                    serde_json::json!({ "hits": hits }),
                )
            }
            IntentKind::OpenVessel { name, policy } => {
                let v = Vessel::open(name, VesselPolicy::from_label(policy));
                let vid = v.id;
                self.vessels.push(v);
                IntentResult::ok(
                    id,
                    format!("vessel '{name}' opened"),
                    serde_json::json!({ "vessel_id": vid, "policy": policy }),
                )
            }
            IntentKind::SealVessel { vessel } => match self.vessel_mut(*vessel) {
                Some(v) => {
                    v.seal();
                    IntentResult::ok(id, format!("vessel '{}' sealed", v.name), serde_json::json!({}))
                }
                None => IntentResult::err(id, "unknown vessel"),
            },
            IntentKind::ClarityGrant { agent, facet } => {
                let vessel_ok = self
                    .agents
                    .iter()
                    .find(|a| a.id == *agent)
                    .and_then(|a| self.vessels.iter().find(|v| v.id == a.vessel))
                    .map(|v| v.policy.clarity_allowed && v.policy.allows("clarity.grant"))
                    .unwrap_or(false);
                if !vessel_ok {
                    return IntentResult::err(id, "clarity grant denied by vessel policy");
                }
                if let Some(a) = self.agent_mut(*agent) {
                    if !a.facets.iter().any(|f| f == facet) {
                        a.facets.push(facet.clone());
                    }
                    a.attend(1.5);
                    let name = a.name.clone();
                    self.clarity_grants.push(ClarityGrant {
                        agent: *agent,
                        facet: facet.clone(),
                        granted_at: Utc::now(),
                    });
                    IntentResult::ok(
                        id,
                        format!("clarity '{facet}' granted to '{name}'"),
                        serde_json::json!({ "agent": agent, "facet": facet }),
                    )
                } else {
                    IntentResult::err(id, "unknown agent")
                }
            }
            IntentKind::Note { text } => {
                let node = self.fabric.weave("developer-note", "note", text);
                IntentResult::ok(
                    id,
                    "note woven into fabric",
                    serde_json::json!({ "node_id": node.id }),
                )
            }
            IntentKind::Tick => {
                // Focus shield parks non-shell agents into Rest before ticking.
                if self.attention_guard.focus_shield {
                    for a in self.agents.iter_mut() {
                        if a.name != "shell" && a.state != AgentState::Dissolve {
                            a.rest();
                        }
                    }
                }
                let events = self.pulse.tick(&mut self.agents);
                IntentResult::ok(
                    id,
                    format!("pulse tick #{} — {} event(s)", self.pulse.ticks, events.len()),
                    serde_json::json!({
                        "events": events,
                        "focus_shield": self.attention_guard.focus_shield,
                    }),
                )
            }
            IntentKind::Snapshot => {
                let snap = self.snapshot();
                IntentResult::ok(
                    id,
                    "cog snapshot",
                    serde_json::to_value(snap).unwrap_or(serde_json::Value::Null),
                )
            }
            IntentKind::Explain { topic } => {
                let report = self.explain_report(topic);
                IntentResult::ok(
                    id,
                    format!("explain: {topic} — glass box open"),
                    report,
                )
            }
            IntentKind::ProposeRevision { title, summary } => {
                let rev = Revision::propose(title, summary, self.checkpoint_marker());
                let rid = rev.id;
                self.revisions.push(rev);
                IntentResult::ok(
                    id,
                    format!("revision proposed '{title}' — approve required before apply"),
                    serde_json::json!({ "revision_id": rid, "state": "proposed" }),
                )
            }
            IntentKind::ApproveRevision { rev } => match self.find_revision_mut(rev) {
                Some(r) => match r.approve() {
                    Ok(()) => IntentResult::ok(
                        id,
                        format!("revision '{}' approved", r.title),
                        serde_json::json!({ "revision_id": r.id }),
                    ),
                    Err(e) => IntentResult::err(id, e),
                },
                None => IntentResult::err(id, "unknown revision"),
            },
