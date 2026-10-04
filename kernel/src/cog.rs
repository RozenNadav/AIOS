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
            IntentKind::RejectRevision { rev } => match self.find_revision_mut(rev) {
                Some(r) => match r.reject() {
                    Ok(()) => IntentResult::ok(
                        id,
                        format!("revision '{}' rejected", r.title),
                        serde_json::json!({ "revision_id": r.id }),
                    ),
                    Err(e) => IntentResult::err(id, e),
                },
                None => IntentResult::err(id, "unknown revision"),
            },
            IntentKind::ApplyRevision { rev } => {
                let title = match self.find_revision(rev) {
                    Some(r) => r.title.clone(),
                    None => return IntentResult::err(id, "unknown revision"),
                };
                match self.find_revision_mut(rev) {
                    Some(r) => match r.apply() {
                        Ok(()) => {
                            // Record apply as fabric history — never silent.
                            self.fabric.weave(
                                &format!("revision:{title}"),
                                "revision",
                                "Sovereign revision applied with developer consent.",
                            );
                            IntentResult::ok(
                                id,
                                format!("revision '{title}' applied (rollback available)"),
                                serde_json::json!({ "title": title }),
                            )
                        }
                        Err(e) => IntentResult::err(id, e),
                    },
                    None => IntentResult::err(id, "unknown revision"),
                }
            },
            IntentKind::RollbackRevision { rev } => {
                let (title, checkpoint) = match self.find_revision(rev) {
                    Some(r) => (r.title.clone(), r.checkpoint.clone()),
                    None => return IntentResult::err(id, "unknown revision"),
                };
                match self.find_revision_mut(rev) {
                    Some(r) => match r.rollback() {
                        Ok(()) => {
                            self.fabric.weave(
                                &format!("rollback:{title}"),
                                "revision",
                                &format!("Rolled back to checkpoint {checkpoint}"),
                            );
                            IntentResult::ok(
                                id,
                                format!("revision '{title}' rolled back → {checkpoint}"),
                                serde_json::json!({ "checkpoint": checkpoint }),
                            )
                        }
                        Err(e) => IntentResult::err(id, e),
                    },
                    None => IntentResult::err(id, "unknown revision"),
                }
            },
            IntentKind::ListRevisions => IntentResult::ok(
                id,
                format!("{} revision(s)", self.revisions.len()),
                serde_json::json!({ "revisions": self.revisions }),
            ),
            IntentKind::Focus { enabled } => {
                self.attention_guard.focus_shield = *enabled;
                if *enabled {
                    for a in self.agents.iter_mut() {
                        if a.name != "shell" && a.state != AgentState::Dissolve {
                            a.rest();
                        }
                    }
                    IntentResult::ok(
                        id,
                        "focus shield ON — non-essential agents parked; unsolicited attention blocked",
                        serde_json::json!({ "focus_shield": true }),
                    )
                } else {
                    IntentResult::ok(
                        id,
                        "focus shield OFF — agents may receive pulse again",
                        serde_json::json!({ "focus_shield": false }),
                    )
                }
            }
            IntentKind::PrefSet { key, value, sticky } => {
                if let Some(p) = self.prefs.iter().find(|p| p.key == *key) {
                    if p.sticky && !*sticky {
                        self.attention_guard.blocked_claims += 1;
                        return IntentResult::err(
                            id,
                            format!(
                                "sticky pref '{key}' refuses silent mutation (was '{}') — use: pref set {key} {value} sticky",
                                p.value
                            ),
                        );
                    }
                }
                if let Some(p) = self.prefs.iter_mut().find(|p| p.key == *key) {
                    p.value = value.clone();
                    p.sticky = *sticky || p.sticky;
                    p.updated_at = Utc::now();
                } else {
                    self.prefs.push(StickyPref::set(key, value, *sticky));
                }
                IntentResult::ok(
                    id,
                    format!("pref '{key}' = '{value}' (sticky={sticky})"),
                    serde_json::json!({ "key": key, "value": value, "sticky": sticky }),
                )
            }
            IntentKind::PrefGet { key } => {
                if key.is_empty() || key == "*" {
                    IntentResult::ok(
                        id,
                        format!("{} pref(s)", self.prefs.len()),
                        serde_json::json!({ "prefs": self.prefs }),
                    )
                } else if let Some(p) = self.prefs.iter().find(|p| p.key == *key) {
                    IntentResult::ok(
                        id,
                        format!("{}={}", p.key, p.value),
                        serde_json::to_value(p).unwrap_or(serde_json::Value::Null),
                    )
                } else {
                    IntentResult::err(id, format!("unknown pref '{key}'"))
                }
            }
            IntentKind::ContinuitySave { name } => {
                let snap = ContinuitySnap {
                    name: name.clone(),
                    agent_names: self
                        .agents
                        .iter()
                        .filter(|a| a.state != AgentState::Dissolve)
                        .map(|a| a.name.clone())
                        .collect(),
                    prefs: self.prefs.clone(),
                    saved_at: Utc::now(),
                };
                self.continuity.retain(|c| c.name != *name);
                self.continuity.push(snap.clone());
                IntentResult::ok(
                    id,
                    format!("continuity '{}' saved", name),
                    serde_json::to_value(snap).unwrap_or(serde_json::Value::Null),
                )
            }
            IntentKind::ContinuityRestore { name } => {
                let snap = match self.continuity.iter().find(|c| c.name == *name).cloned() {
                    Some(s) => s,
                    None => return IntentResult::err(id, format!("unknown continuity '{name}'")),
                };
                // Restore sticky prefs from snapshot; awaken named agents if present.
                self.prefs = snap.prefs.clone();
                for a in self.agents.iter_mut() {
                    if snap.agent_names.iter().any(|n| n == &a.name) {
                        if a.state == AgentState::Rest {
                            a.awaken();
                        }
                    }
                }
                IntentResult::ok(
                    id,
                    format!(
                        "continuity '{}' restored — {} agents, {} prefs",
                        name,
                        snap.agent_names.len(),
                        snap.prefs.len()
                    ),
                    serde_json::json!({ "agents": snap.agent_names, "prefs": snap.prefs.len() }),
                )
            }
            IntentKind::SiliconProbe => {
                let notes = self.lattice.probe();
                IntentResult::ok(
                    id,
                    format!(
                        "silicon probe — {} die(s) (nvidia={} amd-cpu={} amd-gpu={})",
                        self.lattice.dies.len(),
                        self.lattice.nvidia_gpus().count(),
                        self.lattice.amd_cpus().count(),
                        self.lattice.amd_gpus().count()
                    ),
                    serde_json::json!({
                        "notes": notes,
                        "lattice": self.lattice.summary_json(),
                    }),
                )
            }
            IntentKind::SiliconList => IntentResult::ok(
                id,
                format!("{} silicon die(s)", self.lattice.dies.len()),
                self.lattice.summary_json(),
            ),
            IntentKind::SiliconBind { agent, die } => {
                let agent_name = match self.agents.iter().find(|a| a.id == *agent) {
                    Some(a) => a.name.clone(),
                    None => return IntentResult::err(id, "unknown agent"),
                };
                // Vessel must allow silicon.bind
                let vessel_ok = self
                    .agents
                    .iter()
                    .find(|a| a.id == *agent)
                    .and_then(|a| self.vessels.iter().find(|v| v.id == a.vessel))
                    .map(|v| v.policy.allows("silicon.bind"))
                    .unwrap_or(false);
                if !vessel_ok {
                    return IntentResult::err(id, "vessel policy denies silicon.bind");
                }
                let facets = match self.lattice.bind_agent(&agent_name, die) {
                    Ok(d) => d.primary_facets(),
                    Err(e) => return IntentResult::err(id, e),
                };
                if let Some(a) = self.agent_mut(*agent) {
                    for f in &facets {
                        if !a.facets.iter().any(|x| x == f) {
                            a.facets.push(f.clone());
                        }
                    }
                    a.attend(0.8);
                }
                IntentResult::ok(
                    id,
                    format!("bound agent '{agent_name}' → die '{die}'"),
                    serde_json::json!({ "agent": agent_name, "die": die, "facets": facets }),
                )
            }
            IntentKind::SiliconUnbind { agent } => {
                let agent_name = match self.agents.iter().find(|a| a.id == *agent) {
                    Some(a) => a.name.clone(),
                    None => return IntentResult::err(id, "unknown agent"),
                };
                let n = self.lattice.unbind_agent(&agent_name);
                if let Some(a) = self.agent_mut(*agent) {
                    a.facets
                        .retain(|f| !f.starts_with("silicon.nvidia") && !f.starts_with("silicon.amd"));
                }
                IntentResult::ok(
                    id,
                    format!("unbound '{agent_name}' from {n} die(s)"),
                    serde_json::json!({ "agent": agent_name, "removed": n }),
                )
            }
            IntentKind::SiliconAccel { agent, work } => {
                let agent_name = match self.agents.iter().find(|a| a.id == *agent) {
                    Some(a) => a.name.clone(),
                    None => return IntentResult::err(id, "unknown agent"),
                };
                // Require a silicon compute facet.
                let has_compute = self
                    .agents
                    .iter()
                    .find(|a| a.id == *agent)
                    .map(|a| {
                        a.facets.iter().any(|f| {
                            f.starts_with("silicon.nvidia") || f.starts_with("silicon.amd")
                        })
                    })
                    .unwrap_or(false);
                if !has_compute {
                    return IntentResult::err(
                        id,
                        format!("agent '{agent_name}' lacks silicon compute facets — bind first"),
                    );
                }
                match self.lattice.accel(&agent_name, work) {
                    Ok(job) => {
                        if let Some(a) = self.agent_mut(*agent) {
                            a.act();
                        }
                        IntentResult::ok(
                            id,
                            format!(
                                "accel '{}' on lane '{}' — {}",
                                agent_name, job.lane, work
                            ),
                            serde_json::to_value(job).unwrap_or(serde_json::Value::Null),
                        )
                    }
                    Err(e) => IntentResult::err(id, e),
                }
            }
            IntentKind::SiliconDeclare {
                vendor,
                kind,
                name,
                model,
                memory_mb,
                units,
            } => {
                let die = match (vendor.to_ascii_lowercase().as_str(), kind.to_ascii_lowercase().as_str())
                {
                    ("nvidia", "gpu") => self.lattice.declare_nvidia_gpu(name, model, *memory_mb, *units),
                    ("amd", "cpu") => self.lattice.declare_amd_cpu(name, model, *units, *memory_mb),
                    ("amd", "gpu") => self.lattice.declare_amd_gpu(name, model, *memory_mb, *units),
                    _ => {
                        return IntentResult::err(
                            id,
                            "silicon declare: vendor=nvidia|amd kind=gpu|cpu (amd cpu|gpu, nvidia gpu)",
                        );
                    }
                };
                self.fabric.weave(
                    &format!("die:{}", die.name),
                    "silicon",
                    &format!("{:?} {:?} {} — declared for lattice", die.vendor, die.kind, die.model),
                );
                IntentResult::ok(
                    id,
                    format!(
                        "declared {} {} '{}' ({})",
                        vendor, kind, name, model
                    ),
                    serde_json::to_value(die).unwrap_or(serde_json::Value::Null),
                )
            }
        }
    }

    pub fn snapshot(&self) -> CogSnapshot {
        CogSnapshot {
            version: self.version.clone(),
            lineage: self.lineage.clone(),
            doctrine: AIOS_DOCTRINE.into(),
            ignited: self.ignited,
            boot: self.boot.clone(),
            vessels: self.vessels.clone(),
            agents: self.agents.clone(),
            fabric_nodes: self.fabric.nodes.len(),
            fabric_relations: self.fabric.relations.len(),
            pulse_ticks: self.pulse.ticks,
            clarity_grants: self.clarity_grants.clone(),
            intent_log: self.intent_log.clone(),
            revisions: self.revisions.clone(),
            attention_guard: self.attention_guard.clone(),
            prefs: self.prefs.clone(),
            continuity: self.continuity.clone(),
            lattice: self.lattice.clone(),
            saved_at: Utc::now(),
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let data = fs::read_to_string(path)?;
        let cog: Cog = serde_json::from_str(&data)?;
        Ok(cog)
    }

    /// Parse a developer intent line into a structured Intent.
    ///
    /// Grammar (whitespace-flexible):
    ///   attend <topic...>
    ///   spawn <name> :: <purpose...>
    ///   rest <agent>
    ///   dissolve <agent>
    ///   weave <title> | <kind> | <body...>
    ///   query <text...>
    ///   vessel open <name> [policy]
    ///   vessel seal <vessel>
    ///   clarity <agent> <facet>
    ///   note <text...>
    ///   tick
    ///   snapshot
    ///   status
    pub fn parse_line(actor: &str, line: &str) -> Result<Intent, String> {
        let line = line.trim();
        if line.is_empty() {
            return Err("empty intent".into());
        }
        let mut parts = line.split_whitespace();
        let verb = parts.next().unwrap_or("").to_ascii_lowercase();
        let rest = line[verb.len()..].trim();

        let kind = match verb.as_str() {
            "attend" | "status" => {
                let topic = if rest.is_empty() {
                    "status".into()
                } else {
                    rest.to_string()
                };
                IntentKind::Attend { topic }
            }
            "spawn" => {
                let (name, purpose) = if let Some((n, p)) = rest.split_once("::") {
                    (n.trim().to_string(), p.trim().to_string())
                } else {
                    let mut it = rest.split_whitespace();
                    let n = it.next().unwrap_or("agent").to_string();
                    let p = it.collect::<Vec<_>>().join(" ");
                    (n, if p.is_empty() { "unspecified purpose".into() } else { p })
                };
                if name.is_empty() {
                    return Err("spawn requires a name".into());
                }
                IntentKind::SpawnAgent { name, purpose }
            }
            "rest" => {
                // Placeholder — resolved later with cog context via parse_and_submit
                return Err("__need_cog__:rest".into());
            }
            "dissolve" => return Err("__need_cog__:dissolve".into()),
            "weave" => {
                let chunks: Vec<&str> = rest.split('|').map(|s| s.trim()).collect();
                if chunks.len() < 3 {
                    return Err("weave syntax: weave <title> | <kind> | <body>".into());
                }
                IntentKind::WeaveNode {
                    title: chunks[0].into(),
                    node_kind: chunks[1].into(),
                    body: chunks[2..].join(" | "),
                }
            }
            "query" => {
                if rest.is_empty() {
                    return Err("query requires text".into());
                }
                IntentKind::QueryFabric {
                    query: rest.to_string(),
                }
            }
            "vessel" => {
                let mut it = rest.split_whitespace();
                let sub = it.next().unwrap_or("").to_ascii_lowercase();
                match sub.as_str() {
                    "open" => {
                        let name = it.next().unwrap_or("vessel").to_string();
                        let policy = it.next().unwrap_or("developer").to_string();
                        IntentKind::OpenVessel { name, policy }
                    }
                    "seal" => return Err("__need_cog__:vessel_seal".into()),
                    _ => return Err("vessel subcommands: open, seal".into()),
                }
            }
            "clarity" => return Err("__need_cog__:clarity".into()),
            "note" => {
                if rest.is_empty() {
                    return Err("note requires text".into());
                }
                IntentKind::Note {
                    text: rest.to_string(),
                }
            }
            "tick" | "pulse" => IntentKind::Tick,
            "snapshot" => IntentKind::Snapshot,
            "explain" => IntentKind::Explain {
                topic: if rest.is_empty() {
                    "system".into()
                } else {
                    rest.to_string()
                },
            },
            "revise" | "revision" => {
                let mut it = rest.split_whitespace();
                let sub = it.next().unwrap_or("").to_ascii_lowercase();
                match sub.as_str() {
                    "propose" => {
                        let body = it.collect::<Vec<_>>().join(" ");
                        let (title, summary) = if let Some((t, s)) = body.split_once("::") {
                            (t.trim().to_string(), s.trim().to_string())
                        } else if body.is_empty() {
                            return Err("revise propose <title> :: <summary>".into());
                        } else {
                            (body.clone(), body)
                        };
                        IntentKind::ProposeRevision { title, summary }
                    }
                    "approve" => {
                        let rev = it.next().ok_or("revise approve <id|title>")?.to_string();
                        IntentKind::ApproveRevision { rev }
                    }
                    "reject" => {
                        let rev = it.next().ok_or("revise reject <id|title>")?.to_string();
                        IntentKind::RejectRevision { rev }
                    }
                    "apply" => {
                        let rev = it.next().ok_or("revise apply <id|title>")?.to_string();
                        IntentKind::ApplyRevision { rev }
                    }
                    "rollback" => {
                        let rev = it.next().ok_or("revise rollback <id|title>")?.to_string();
                        IntentKind::RollbackRevision { rev }
                    }
                    "list" | "" => IntentKind::ListRevisions,
                    _ => return Err("revise: propose|approve|reject|apply|rollback|list".into()),
                }
            }
            "focus" => {
                let on = rest.to_ascii_lowercase();
                let enabled = !(on == "off" || on == "false" || on == "0");
                IntentKind::Focus { enabled }
            }
            "pref" => {
                let mut it = rest.split_whitespace();
                let sub = it.next().unwrap_or("get").to_ascii_lowercase();
                match sub.as_str() {
                    "set" => {
                        let key = it.next().ok_or("pref set <key> <value> [sticky]")?.to_string();
                        let mut vals: Vec<&str> = it.collect();
                        let sticky = vals
                            .last()
                            .map(|v| *v == "sticky" || *v == "true")
                            .unwrap_or(false);
                        if sticky {
                            vals.pop();
                        }
                        if vals.is_empty() {
                            return Err("pref set <key> <value> [sticky]".into());
                        }
                        IntentKind::PrefSet {
                            key,
                            value: vals.join(" "),
                            sticky,
                        }
                    }
                    "get" | "list" => IntentKind::PrefGet {
                        key: it.next().unwrap_or("*").to_string(),
                    },
                    _ => IntentKind::PrefGet {
                        key: sub.to_string(),
                    },
                }
            }
            "continuity" => {
                let mut it = rest.split_whitespace();
                let sub = it.next().unwrap_or("").to_ascii_lowercase();
                let name = it.next().unwrap_or("default").to_string();
                match sub.as_str() {
                    "save" => IntentKind::ContinuitySave { name },
                    "restore" => IntentKind::ContinuityRestore { name },
                    _ => return Err("continuity save|restore <name>".into()),
                }
            }
            "silicon" => {
                let mut it = rest.split_whitespace();
                let sub = it.next().unwrap_or("list").to_ascii_lowercase();
                match sub.as_str() {
                    "probe" => IntentKind::SiliconProbe,
                    "list" | "" => IntentKind::SiliconList,
                    "bind" | "unbind" | "accel" => {
                        return Err(format!("__need_cog__:silicon_{sub}"));
                    }
                    "declare" => {
                        // silicon declare <nvidia|amd> <gpu|cpu> <name> <memory_mb> <units> :: <model...>
                        let vendor = it.next().ok_or("silicon declare <vendor> <kind> <name> <mem_mb> <units> :: <model>")?.to_string();
                        let kind = it.next().ok_or("silicon declare …")?.to_string();
                        let name = it.next().ok_or("silicon declare …")?.to_string();
                        let memory_mb: u64 = it
                            .next()
                            .ok_or("silicon declare … needs memory_mb")?
                            .parse()
                            .map_err(|_| "memory_mb must be a number")?;
                        let units: u32 = it
                            .next()
                            .ok_or("silicon declare … needs units (SMs/CUs/cores)")?
                            .parse()
                            .map_err(|_| "units must be a number")?;
                        let rem = it.collect::<Vec<_>>().join(" ");
                        let model = rem
                            .strip_prefix("::")
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .or_else(|| {
                                let t = rem.trim().trim_start_matches("::").trim();
                                if t.is_empty() {
                                    None
                                } else {
                                    Some(t.to_string())
                                }
                            })
                            .unwrap_or_else(|| name.clone());
                        IntentKind::SiliconDeclare {
                            vendor,
                            kind,
                            name,
                            model,
                            memory_mb,
                            units,
                        }
                    }
                    _ => return Err("silicon: probe|list|bind|unbind|accel|declare".into()),
                }
            }
            "help" => IntentKind::Attend {
                topic: "help".into(),
            },
            _ => {
                // Natural-language fallback: treat whole line as an attend topic / note.
                IntentKind::Attend {
                    topic: line.to_string(),
                }
            }
        };

        Ok(Intent::new(actor, kind, Urgency::Normal))
    }

    /// Parse a line with access to live ids (agents/vessels by name).
    pub fn parse_line_resolved(&self, actor: &str, line: &str) -> Result<Intent, String> {
        let line = line.trim();
        let mut parts = line.split_whitespace();
        let verb = parts.next().unwrap_or("").to_ascii_lowercase();
        let _rest = line[verb.len()..].trim();

        match verb.as_str() {
            "rest" => {
                let name = parts.next().ok_or("rest requires agent name or id")?;
                let agent = self
                    .resolve_agent_ref(name)
                    .ok_or_else(|| format!("unknown agent '{name}'"))?;
                Ok(Intent::new(actor, IntentKind::RestAgent { agent }, Urgency::Normal))
            }
            "dissolve" => {
                let name = parts.next().ok_or("dissolve requires agent name or id")?;
                let agent = self
                    .resolve_agent_ref(name)
                    .ok_or_else(|| format!("unknown agent '{name}'"))?;
                Ok(Intent::new(
                    actor,
                    IntentKind::DissolveAgent { agent },
                    Urgency::High,
                ))
            }
            "clarity" => {
                let name = parts.next().ok_or("clarity <agent> <facet>")?;
                let facet = parts.next().ok_or("clarity <agent> <facet>")?;
                let agent = self
                    .resolve_agent_ref(name)
                    .ok_or_else(|| format!("unknown agent '{name}'"))?;
                Ok(Intent::new(
                    actor,
                    IntentKind::ClarityGrant {
                        agent,
                        facet: facet.to_string(),
                    },
                    Urgency::High,
                ))
            }
            "vessel" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                if sub == "seal" {
                    let name = parts.next().ok_or("vessel seal <name|id>")?;
                    let vessel = self
                        .resolve_vessel_ref(name)
                        .ok_or_else(|| format!("unknown vessel '{name}'"))?;
                    Ok(Intent::new(
                        actor,
                        IntentKind::SealVessel { vessel },
                        Urgency::High,
                    ))
                } else {
                    Self::parse_line(actor, line)
                }
            }
            "silicon" => {
                let sub = parts.next().unwrap_or("list").to_ascii_lowercase();
                match sub.as_str() {
                    "bind" => {
                        let aname = parts.next().ok_or("silicon bind <agent> <die>")?;
                        let die = parts.next().ok_or("silicon bind <agent> <die>")?.to_string();
                        let agent = self
                            .resolve_agent_ref(aname)
                            .ok_or_else(|| format!("unknown agent '{aname}'"))?;
                        Ok(Intent::new(
                            actor,
                            IntentKind::SiliconBind { agent, die },
                            Urgency::High,
                        ))
                    }
                    "unbind" => {
                        let aname = parts.next().ok_or("silicon unbind <agent>")?;
                        let agent = self
                            .resolve_agent_ref(aname)
                            .ok_or_else(|| format!("unknown agent '{aname}'"))?;
                        Ok(Intent::new(
                            actor,
                            IntentKind::SiliconUnbind { agent },
                            Urgency::Normal,
                        ))
                    }
                    "accel" => {
                        let aname = parts.next().ok_or("silicon accel <agent> <work...>")?;
                        let work = parts.collect::<Vec<_>>().join(" ");
                        if work.is_empty() {
                            return Err("silicon accel <agent> <work...>".into());
                        }
                        let agent = self
                            .resolve_agent_ref(aname)
                            .ok_or_else(|| format!("unknown agent '{aname}'"))?;
                        Ok(Intent::new(
                            actor,
                            IntentKind::SiliconAccel { agent, work },
                            Urgency::High,
                        ))
                    }
                    _ => Self::parse_line(actor, line),
                }
            }
            _ => Self::parse_line(actor, line),
        }
    }
}
