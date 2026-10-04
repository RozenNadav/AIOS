import type {
  AccelJob,
  Agent,
  BootStage,
  CogState,
  ContinuitySnap,
  Die,
  FabricNode,
  IntentResult,
  Lane,
  PulseEvent,
  Revision,
  StickyPref,
  Vessel,
} from "./types";

const uid = () =>
  crypto.randomUUID?.() ??
  `id-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

const now = () => new Date().toISOString();

const DOCTRINE =
  "Consent-first · Sovereign revisions · Attention guard · Glass-box explain · Silicon Lattice";

function vesselPolicy(label: string): Pick<Vessel, "policy" | "maxAgents"> {
  if (label === "sandbox") return { policy: "sandbox", maxAgents: 8 };
  if (label === "private") return { policy: "private", maxAgents: 32 };
  return { policy: "developer", maxAgents: 64 };
}

function nvidiaLanes(sm: number, mem: number): Lane[] {
  const scale = Math.max(0.25, sm / 64);
  return [
    { name: "cuda", kind: "cuda", tflopsEst: 20 * scale, available: true },
    { name: "tensor", kind: "tensor", tflopsEst: 80 * scale, available: mem >= 8192 },
    { name: "rt", kind: "rt", tflopsEst: 10 * scale, available: true },
  ];
}

function amdZenLanes(cores: number): Lane[] {
  return [{ name: "zen", kind: "cpu", tflopsEst: cores * 0.05, available: true }];
}

function amdRocmLanes(cu: number, mem: number): Lane[] {
  const scale = Math.max(0.25, cu / 60);
  return [
    { name: "rocm", kind: "rocm", tflopsEst: 18 * scale, available: true },
    { name: "rdna", kind: "rdna", tflopsEst: 22 * scale, available: mem >= 4096 },
  ];
}

function labDies(): Die[] {
  // Browser Surface cannot call nvidia-smi; seed a realistic NVIDIA + AMD lattice for DevCore.
  return [
    {
      id: uid(),
      name: "rtx0",
      vendor: "nvidia",
      kind: "gpu",
      model: "NVIDIA GeForce RTX 4090",
      memoryMb: 24576,
      computeUnits: 128,
      lanes: nvidiaLanes(128, 24576),
      boundAgents: [],
      source: "lab",
    },
    {
      id: uid(),
      name: "zen0",
      vendor: "amd",
      kind: "cpu",
      model: "AMD Ryzen 9 7950X",
      memoryMb: 65536,
      computeUnits: 32,
      lanes: amdZenLanes(32),
      boundAgents: [],
      source: "lab",
    },
    {
      id: uid(),
      name: "rx0",
      vendor: "amd",
      kind: "gpu",
      model: "AMD Radeon RX 7900 XTX",
      memoryMb: 16384,
      computeUnits: 96,
      lanes: amdRocmLanes(96, 16384),
      boundAgents: [],
      source: "lab",
    },
  ];
}

function findDie(state: CogState, ref: string) {
  const n = ref.toLowerCase();
  return state.dies.find(
    (d) =>
      d.id.startsWith(ref) ||
      d.name.toLowerCase() === n ||
      d.model.toLowerCase().includes(n),
  );
}

export function createCog(): CogState {
  return {
    version: "0.2.1",
    lineage: "AIOS Cognition Core — independent AI-native lineage",
    doctrine: DOCTRINE,
    ignited: false,
    bootStages: [],
    vessels: [],
    agents: [],
    nodes: [],
    relations: [],
    pulseTicks: 0,
    pulseHistory: [],
    revisions: [],
    focusShield: false,
    blockedClaims: 0,
    prefs: [],
    continuity: [],
    dies: [],
    accelJobs: [],
    probeNotes: [],
    log: [],
  };
}

export function ignite(state: CogState): CogState {
  if (state.ignited) return state;

  const stages: BootStage[] = [
    { name: "spark.presence", detail: "Establish host presence for Cognition Core" },
    { name: "spark.intent_bus", detail: "Open Intent Bus channels" },
    { name: "spark.fabric", detail: "Attach Knowledge Fabric substrate" },
    { name: "spark.attention_guard", detail: "Arm Attention Guard — no unsolicited nags" },
    { name: "spark.sovereign", detail: "Enable Sovereign Revisions — no forced updates" },
    { name: "spark.silicon", detail: "Probe Silicon Lattice — NVIDIA CUDA · AMD Zen/ROCm" },
    { name: "spark.pulse", detail: "Arm Pulse attention scheduler" },
    { name: "spark.vessel.prime", detail: "Open prime developer vessel (private-ready)" },
    { name: "spark.cog.awaken", detail: "Awaken Cognition Core" },
  ];

  const dies = labDies();

  const primeId = uid();
  const shell: Agent = {
    id: uid(),
    name: "shell",
    purpose: "Interpret developer intents",
    state: "awake",
    vessel: primeId,
    facets: ["intent.emit", "fabric.read", "fabric.weave", "agent.spawn"],
    attention: 1.2,
    createdAt: now(),
  };

  const prefs: StickyPref[] = [
    { key: "telemetry", value: "off", sticky: true },
    { key: "assistant.unsolicited", value: "off", sticky: true },
    { key: "updates.auto_apply", value: "off", sticky: true },
    { key: "ads", value: "off", sticky: true },
    { key: "silicon.prefer", value: "nvidia,amd", sticky: true },
  ];

  const nodes: FabricNode[] = [
    {
      id: uid(),
      title: "AIOS",
      kind: "system",
      body: "Cognition-first OS. Not Unix. Not Windows. Purpose-scheduled.",
      createdAt: now(),
    },
    {
      id: uid(),
      title: "Doctrine",
      kind: "doctrine",
      body: DOCTRINE,
      createdAt: now(),
    },
    {
      id: uid(),
      title: "Silicon Lattice",
      kind: "silicon",
      body: `NVIDIA GPUs + AMD Zen/ROCm as dies/lanes — not /dev nodes. Lab dies: ${dies
        .map((d) => d.name)
        .join(", ")}.`,
      createdAt: now(),
    },
    {
      id: uid(),
      title: "Pain → Future",
      kind: "design",
      body: "Forced updates→Sovereign Revisions; nags→Attention Guard; reboot mystery→Explain; resets→Sticky Prefs; tool sprawl→Intent Bus; focus loss→Focus Shield.",
      createdAt: now(),
    },
  ];

  return {
    ...state,
    ignited: true,
    bootStages: stages,
    vessels: [{ id: primeId, name: "prime", sealed: false, ...vesselPolicy("developer") }],
    agents: [shell],
    nodes,
    prefs,
    dies,
    probeNotes: [
      "console: seeded lab lattice (RTX 4090 + Ryzen 7950X + RX 7900 XTX)",
      "host probe available via Rust shell: silicon probe",
    ],
  };
}

function aliveAgents(state: CogState) {
  return state.agents.filter((a) => a.state !== "dissolve");
}

function findAgent(state: CogState, ref: string) {
  return aliveAgents(state).find(
    (a) => a.id === ref || a.name.toLowerCase() === ref.toLowerCase(),
  );
}

function findVessel(state: CogState, ref: string) {
  return state.vessels.find(
    (v) => v.id === ref || v.name.toLowerCase() === ref.toLowerCase(),
  );
}

function findRevision(state: CogState, rev: string) {
  const n = rev.toLowerCase();
  return state.revisions.find(
    (r) =>
      r.id.startsWith(rev) ||
      r.title.toLowerCase() === n ||
      r.title.toLowerCase().includes(n),
  );
}

function checkpoint(state: CogState) {
  return `nodes=${state.nodes.length} agents=${aliveAgents(state)
    .map((a) => a.name)
    .join(",")}`;
}

function tickPulse(state: CogState): { state: CogState; events: PulseEvent[] } {
  const events: PulseEvent[] = [];
  let agents = [...state.agents];

  if (state.focusShield) {
    agents = agents.map((a) =>
      a.name !== "shell" && a.state !== "dissolve"
        ? { ...a, state: "rest" as const, attention: Math.max(0.05, a.attention * 0.5) }
        : a,
    );
  }

  agents = [...agents].sort((a, b) => b.attention - a.attention).map((agent) => {
    if (agent.state === "dissolve" || agent.state === "rest") return agent;
    let updated = { ...agent };
    if (updated.state === "seed") {
      updated.state = "awake";
      events.push({
        agent: updated.name,
        note: `pulse awoke '${updated.name}'`,
        attention: updated.attention,
        at: now(),
      });
    } else if (updated.state === "awake" || updated.state === "attend") {
      updated.state = "act";
      updated.attention = Math.max(0.1, updated.attention - 0.15);
      events.push({
        agent: updated.name,
        note: `pulse engaged '${updated.name}'`,
        attention: updated.attention,
        at: now(),
      });
    } else if (updated.state === "act") {
      if (updated.attention < 0.4) {
        updated.state = "rest";
        updated.attention = Math.max(0.05, updated.attention * 0.5);
        events.push({
          agent: updated.name,
          note: `pulse rested '${updated.name}'`,
          attention: updated.attention,
          at: now(),
        });
      } else {
        updated.state = "attend";
        updated.attention = Math.min(10, updated.attention + 0.05);
        events.push({
          agent: updated.name,
          note: `pulse re-attended '${updated.name}'`,
          attention: updated.attention,
          at: now(),
        });
      }
    }
    return updated;
  });

  // Keep original order ids
  const byId = new Map(agents.map((a) => [a.id, a]));
  const ordered = state.agents.map((a) => byId.get(a.id) ?? a);

  return {
    state: {
      ...state,
      agents: ordered,
      pulseTicks: state.pulseTicks + 1,
      pulseHistory: [...state.pulseHistory, ...events].slice(-40),
    },
    events,
  };
}

function explain(state: CogState, topic: string): IntentResult {
  const lines = [
    `topic: ${topic}`,
    DOCTRINE,
    `focus_shield=${state.focusShield} blocked_claims=${state.blockedClaims}`,
    `agents=${aliveAgents(state).length} nodes=${state.nodes.length} pulses=${state.pulseTicks}`,
    ...aliveAgents(state).map(
      (a) => `  · ${a.name} [${a.state}] attn=${a.attention.toFixed(2)} — ${a.purpose}`,
    ),
    `prefs: ${state.prefs.map((p) => `${p.key}=${p.value}${p.sticky ? "*" : ""}`).join(", ")}`,
    `revisions open: ${state.revisions.filter((r) => r.state === "proposed" || r.state === "approved").length}`,
    "why_not_reboot: AIOS explains live state instead of asking you to reboot.",
  ];
  return {
    ok: true,
    message: `explain: ${topic} — glass box open`,
    data: { text: lines.join("\n") },
  };
}

export function submitIntent(
  state: CogState,
  line: string,
): { state: CogState; result: IntentResult } {
  let s = state.ignited ? { ...state } : ignite(state);
  const input = line.trim();
  if (!input) {
    return { state: s, result: { ok: false, message: "empty intent" } };
  }

  const [verbRaw, ...restParts] = input.split(/\s+/);
  const verb = verbRaw.toLowerCase();
  const rest = restParts.join(" ");

  let result: IntentResult;
  let lastExplain = s.lastExplain;

  switch (verb) {
    case "help":
      result = {
        ok: true,
        message:
          "verbs: explain, focus, revise, pref, continuity, silicon, spawn, weave, query, tick, status",
      };
      break;
    case "status":
    case "attend": {
      result = {
        ok: true,
        message: `Cog attending: ${rest || "status"}`,
        data: {
          agents: aliveAgents(s).length,
          nodes: s.nodes.length,
          ticks: s.pulseTicks,
          focus: s.focusShield,
        },
      };
      break;
    }
    case "explain": {
      result = explain(s, rest || "system");
      lastExplain = String((result.data as { text?: string })?.text ?? result.message);
      break;
    }
    case "spawn": {
      if (s.focusShield) {
        s = { ...s, blockedClaims: s.blockedClaims + 1 };
        result = {
          ok: false,
          message: "focus shield blocked spawn — emit `focus off` first (attention guard)",
        };
        break;
      }
      let name = "agent";
      let purpose = "unspecified purpose";
      if (rest.includes("::")) {
        const [n, p] = rest.split("::");
        name = n.trim() || name;
        purpose = p.trim() || purpose;
      } else {
        const [n, ...p] = restParts;
        name = n || name;
        if (p.length) purpose = p.join(" ");
      }
      const prime = s.vessels[0];
      if (!prime || prime.sealed) {
        result = { ok: false, message: "no open prime vessel" };
        break;
      }
      const agent: Agent = {
        id: uid(),
        name,
        purpose,
        state: "awake",
        vessel: prime.id,
        facets: ["intent.emit", "fabric.read"],
        attention: 1,
        createdAt: now(),
      };
      s = { ...s, agents: [...s.agents, agent] };
      result = { ok: true, message: `agent '${name}' seeded and awake` };
      break;
    }
    case "rest": {
      const agent = findAgent(s, restParts[0] || "");
      if (!agent) {
        result = { ok: false, message: "unknown agent" };
        break;
      }
      s = {
        ...s,
        agents: s.agents.map((a) =>
          a.id === agent.id
            ? { ...a, state: "rest", attention: Math.max(0.05, a.attention * 0.5) }
            : a,
        ),
      };
      result = { ok: true, message: `agent '${agent.name}' resting` };
      break;
    }
    case "dissolve": {
      const agent = findAgent(s, restParts[0] || "");
      if (!agent) {
        result = { ok: false, message: "unknown agent" };
        break;
      }
      s = {
        ...s,
        agents: s.agents.map((a) =>
          a.id === agent.id ? { ...a, state: "dissolve", attention: 0 } : a,
        ),
      };
      result = { ok: true, message: `agent '${agent.name}' dissolved` };
      break;
    }
    case "weave": {
      const chunks = input
        .slice(verb.length)
        .split("|")
        .map((c) => c.trim())
        .filter(Boolean);
      if (chunks.length < 3) {
        result = { ok: false, message: "weave syntax: weave <title> | <kind> | <body>" };
        break;
      }
      const node: FabricNode = {
        id: uid(),
        title: chunks[0],
        kind: chunks[1],
        body: chunks.slice(2).join(" | "),
        createdAt: now(),
      };
      s = { ...s, nodes: [node, ...s.nodes] };
      result = { ok: true, message: `wove node '${node.title}'` };
      break;
    }
    case "query": {
      const q = rest.toLowerCase();
      const hits = s.nodes.filter(
        (n) =>
          n.title.toLowerCase().includes(q) ||
          n.kind.toLowerCase().includes(q) ||
          n.body.toLowerCase().includes(q),
      );
      result = { ok: true, message: `${hits.length} fabric hit(s)`, data: { hits } };
      break;
    }
    case "vessel": {
      const sub = (restParts[0] || "").toLowerCase();
      if (sub === "open") {
        const name = restParts[1] || "vessel";
        const policy = restParts[2] || "developer";
        const v: Vessel = { id: uid(), name, sealed: false, ...vesselPolicy(policy) };
        s = { ...s, vessels: [...s.vessels, v] };
        result = { ok: true, message: `vessel '${name}' opened (policy ${v.policy})` };
      } else if (sub === "seal") {
        const v = findVessel(s, restParts[1] || "");
        if (!v) {
          result = { ok: false, message: "unknown vessel" };
          break;
        }
        s = {
          ...s,
          vessels: s.vessels.map((x) => (x.id === v.id ? { ...x, sealed: true } : x)),
        };
        result = { ok: true, message: `vessel '${v.name}' sealed` };
      } else {
        result = { ok: false, message: "vessel subcommands: open, seal" };
      }
      break;
    }
    case "clarity": {
      const agent = findAgent(s, restParts[0] || "");
      const facet = restParts[1];
      if (!agent || !facet) {
        result = { ok: false, message: "clarity <agent> <facet>" };
        break;
      }
      if (facet.includes("telemetry")) {
        s = { ...s, blockedClaims: s.blockedClaims + 1 };
        result = {
          ok: false,
          message: "telemetry facet denied — private-by-default (consent-first)",
        };
        break;
      }
      s = {
        ...s,
        agents: s.agents.map((a) =>
          a.id === agent.id
            ? {
                ...a,
                facets: a.facets.includes(facet) ? a.facets : [...a.facets, facet],
                attention: Math.min(10, a.attention + 1.5),
                state: "attend",
              }
            : a,
        ),
      };
      result = { ok: true, message: `clarity '${facet}' granted to '${agent.name}'` };
      break;
    }
    case "note": {
      if (!rest) {
        result = { ok: false, message: "note requires text" };
        break;
      }
      const node: FabricNode = {
        id: uid(),
        title: "developer-note",
        kind: "note",
        body: rest,
        createdAt: now(),
      };
      s = { ...s, nodes: [node, ...s.nodes] };
      result = { ok: true, message: "note woven into fabric" };
      break;
    }
    case "tick":
    case "pulse": {
      const pulsed = tickPulse(s);
      s = pulsed.state;
      result = {
        ok: true,
        message: `pulse tick #${s.pulseTicks} — ${pulsed.events.length} event(s)`,
      };
      break;
    }
    case "focus": {
      const enabled = !(rest.toLowerCase() === "off" || rest === "0" || rest === "false");
      s = {
        ...s,
        focusShield: enabled,
        agents: enabled
          ? s.agents.map((a) =>
              a.name !== "shell" && a.state !== "dissolve"
                ? { ...a, state: "rest" as const }
                : a,
            )
          : s.agents,
      };
      result = {
        ok: true,
        message: enabled
          ? "focus shield ON — interruptions blocked"
          : "focus shield OFF",
      };
      break;
    }
    case "revise":
    case "revision": {
      const sub = (restParts[0] || "list").toLowerCase();
      if (sub === "propose") {
        const body = restParts.slice(1).join(" ");
        const [titleRaw, summaryRaw] = body.includes("::")
          ? body.split("::")
          : [body, body];
        const title = titleRaw.trim() || "untitled";
        const summary = (summaryRaw || title).trim();
        const rev: Revision = {
          id: uid(),
          title,
          summary,
          state: "proposed",
          checkpoint: checkpoint(s),
        };
        s = { ...s, revisions: [...s.revisions, rev] };
        result = {
          ok: true,
          message: `revision proposed '${title}' — approve required before apply`,
        };
      } else if (sub === "list") {
        result = {
          ok: true,
          message: `${s.revisions.length} revision(s)`,
          data: { revisions: s.revisions },
        };
      } else if (["approve", "reject", "apply", "rollback"].includes(sub)) {
        const key = restParts[1] || "";
        const target = findRevision(s, key);
        if (!target) {
          result = { ok: false, message: "unknown revision" };
          break;
        }
        let next: Revision = { ...target };
        let ok = true;
        let msg = "";
        if (sub === "approve") {
          if (next.state !== "proposed") {
            ok = false;
            msg = "expected proposed";
          } else {
            next.state = "approved";
            msg = `revision '${next.title}' approved`;
          }
        } else if (sub === "reject") {
          next.state = "rejected";
          msg = `revision '${next.title}' rejected`;
        } else if (sub === "apply") {
          if (next.state !== "approved") {
            ok = false;
            msg = "revision must be approved before apply (sovereign rule)";
          } else {
            next.state = "applied";
            msg = `revision '${next.title}' applied (rollback available)`;
            s = {
              ...s,
              nodes: [
                {
                  id: uid(),
                  title: `revision:${next.title}`,
                  kind: "revision",
                  body: "Sovereign revision applied with developer consent.",
                  createdAt: now(),
                },
                ...s.nodes,
              ],
            };
          }
        } else {
          if (next.state !== "applied") {
            ok = false;
            msg = "only applied revisions can roll back";
          } else {
            next.state = "rolledback";
            msg = `revision '${next.title}' rolled back → ${next.checkpoint}`;
          }
        }
        if (ok) {
          s = {
            ...s,
            revisions: s.revisions.map((r) => (r.id === next.id ? next : r)),
          };
          result = { ok: true, message: msg };
        } else {
          result = { ok: false, message: msg };
        }
      } else {
        result = {
          ok: false,
          message: "revise: propose|approve|reject|apply|rollback|list",
        };
      }
      break;
    }
    case "pref": {
      const sub = (restParts[0] || "get").toLowerCase();
      if (sub === "set") {
        let vals = restParts.slice(1);
        const key = vals.shift() || "";
        let sticky = false;
        if (vals[vals.length - 1] === "sticky" || vals[vals.length - 1] === "true") {
          sticky = true;
          vals = vals.slice(0, -1);
        }
        const value = vals.join(" ");
        if (!key || !value) {
          result = { ok: false, message: "pref set <key> <value> [sticky]" };
          break;
        }
        const existing = s.prefs.find((p) => p.key === key);
        if (existing?.sticky && !sticky) {
          s = { ...s, blockedClaims: s.blockedClaims + 1 };
          result = {
            ok: false,
            message: `sticky pref '${key}' refuses silent mutation — add 'sticky' to change deliberately`,
          };
          break;
        }
        const pref: StickyPref = { key, value, sticky: sticky || !!existing?.sticky };
        s = {
          ...s,
          prefs: existing
            ? s.prefs.map((p) => (p.key === key ? pref : p))
            : [...s.prefs, pref],
        };
        result = { ok: true, message: `pref '${key}' = '${value}' (sticky=${pref.sticky})` };
      } else {
        const key = sub === "get" || sub === "list" ? restParts[1] || "*" : sub;
        if (!key || key === "*") {
          result = {
            ok: true,
            message: `${s.prefs.length} pref(s)`,
            data: { prefs: s.prefs },
          };
        } else {
          const p = s.prefs.find((x) => x.key === key);
          result = p
            ? { ok: true, message: `${p.key}=${p.value}`, data: p }
            : { ok: false, message: `unknown pref '${key}'` };
        }
      }
      break;
    }
    case "continuity": {
      const sub = (restParts[0] || "").toLowerCase();
      const name = restParts[1] || "default";
      if (sub === "save") {
        const snap: ContinuitySnap = {
          name,
          agentNames: aliveAgents(s).map((a) => a.name),
          prefs: s.prefs,
        };
        s = {
          ...s,
          continuity: [...s.continuity.filter((c) => c.name !== name), snap],
        };
        result = { ok: true, message: `continuity '${name}' saved` };
      } else if (sub === "restore") {
        const snap = s.continuity.find((c) => c.name === name);
        if (!snap) {
          result = { ok: false, message: `unknown continuity '${name}'` };
          break;
        }
        s = {
          ...s,
          prefs: snap.prefs,
          agents: s.agents.map((a) =>
            snap.agentNames.includes(a.name) && a.state === "rest"
              ? { ...a, state: "awake" as const }
              : a,
          ),
        };
        result = {
          ok: true,
          message: `continuity '${name}' restored — ${snap.agentNames.length} agents`,
        };
      } else {
        result = { ok: false, message: "continuity save|restore <name>" };
      }
      break;
    }
    case "silicon": {
      const sub = (restParts[0] || "list").toLowerCase();
      if (sub === "probe" || sub === "list") {
        if (sub === "probe" && s.dies.length === 0) {
          s = { ...s, dies: labDies(), probeNotes: ["re-probed lab lattice"] };
        }
        result = {
          ok: true,
          message: `${s.dies.length} silicon die(s) (nvidia=${s.dies.filter((d) => d.vendor === "nvidia").length} amd=${s.dies.filter((d) => d.vendor === "amd").length})`,
          data: { dies: s.dies },
        };
      } else if (sub === "declare") {
        const vendor = (restParts[1] || "").toLowerCase();
        const kind = (restParts[2] || "").toLowerCase() as "gpu" | "cpu";
        const name = restParts[3] || "die";
        const memoryMb = Number(restParts[4] || 8192);
        const units = Number(restParts[5] || 64);
        const modelParts = restParts.slice(6);
        const model = modelParts.join(" ").replace(/^::\s*/, "") || name;
        let die: Die;
        if (vendor === "nvidia" && kind === "gpu") {
          die = {
            id: uid(),
            name,
            vendor: "nvidia",
            kind: "gpu",
            model,
            memoryMb,
            computeUnits: units,
            lanes: nvidiaLanes(units, memoryMb),
            boundAgents: [],
            source: "declare",
          };
        } else if (vendor === "amd" && kind === "cpu") {
          die = {
            id: uid(),
            name,
            vendor: "amd",
            kind: "cpu",
            model,
            memoryMb,
            computeUnits: units,
            lanes: amdZenLanes(units),
            boundAgents: [],
            source: "declare",
          };
        } else if (vendor === "amd" && kind === "gpu") {
          die = {
            id: uid(),
            name,
            vendor: "amd",
            kind: "gpu",
            model,
            memoryMb,
            computeUnits: units,
            lanes: amdRocmLanes(units, memoryMb),
            boundAgents: [],
            source: "declare",
          };
        } else {
          result = {
            ok: false,
            message: "silicon declare <nvidia|amd> <gpu|cpu> <name> <mem> <units> :: <model>",
          };
          break;
        }
        s = { ...s, dies: [...s.dies.filter((d) => d.name !== name), die] };
        result = { ok: true, message: `declared ${vendor} ${kind} '${name}' (${model})` };
      } else if (sub === "bind") {
        const aname = restParts[1] || "";
        const dref = restParts[2] || "";
        const agent = findAgent(s, aname);
        const die = findDie(s, dref);
        if (!agent || !die) {
          result = { ok: false, message: "silicon bind <agent> <die>" };
          break;
        }
        const facets =
          die.vendor === "nvidia"
            ? ["silicon.bind", "silicon.nvidia.cuda", "silicon.nvidia.tensor"]
            : die.kind === "cpu"
              ? ["silicon.bind", "silicon.amd.zen"]
              : ["silicon.bind", "silicon.amd.rocm"];
        s = {
          ...s,
          dies: s.dies.map((d) =>
            d.id === die.id
              ? {
                  ...d,
                  boundAgents: d.boundAgents.includes(agent.name)
                    ? d.boundAgents
                    : [...d.boundAgents, agent.name],
                }
              : d,
          ),
          agents: s.agents.map((a) =>
            a.id === agent.id
              ? {
                  ...a,
                  facets: [...new Set([...a.facets, ...facets])],
                  state: "attend",
                  attention: Math.min(10, a.attention + 0.8),
                }
              : a,
          ),
        };
        result = { ok: true, message: `bound agent '${agent.name}' → die '${die.name}'` };
      } else if (sub === "unbind") {
        const aname = restParts[1] || "";
        const agent = findAgent(s, aname);
        if (!agent) {
          result = { ok: false, message: "unknown agent" };
          break;
        }
        s = {
          ...s,
          dies: s.dies.map((d) => ({
            ...d,
            boundAgents: d.boundAgents.filter((x) => x !== agent.name),
          })),
          agents: s.agents.map((a) =>
            a.id === agent.id
              ? {
                  ...a,
                  facets: a.facets.filter(
                    (f) => !f.startsWith("silicon.nvidia") && !f.startsWith("silicon.amd"),
                  ),
                }
              : a,
          ),
        };
        result = { ok: true, message: `unbound '${agent.name}'` };
      } else if (sub === "accel") {
        const aname = restParts[1] || "";
        const work = restParts.slice(2).join(" ");
        const agent = findAgent(s, aname);
        if (!agent || !work) {
          result = { ok: false, message: "silicon accel <agent> <work...>" };
          break;
        }
        const die = s.dies.find((d) => d.boundAgents.includes(agent.name));
        if (!die) {
          result = {
            ok: false,
            message: `agent '${agent.name}' is not bound — use silicon bind`,
          };
          break;
        }
        const lane = die.lanes.find((l) => l.available)?.name || "default";
        const job: AccelJob = {
          id: uid(),
          agent: agent.name,
          dieId: die.id,
          lane,
          work,
          status: "completed",
        };
        s = {
          ...s,
          accelJobs: [job, ...s.accelJobs].slice(0, 40),
          agents: s.agents.map((a) =>
            a.id === agent.id ? { ...a, state: "act" as const } : a,
          ),
        };
        result = {
          ok: true,
          message: `accel '${agent.name}' on lane '${lane}' — ${work}`,
        };
      } else {
        result = {
          ok: false,
          message: "silicon: probe|list|declare|bind|unbind|accel",
        };
      }
      break;
    }
    default:
      result = { ok: true, message: `Cog attending: ${input}`, data: { natural: true } };
  }

  s = {
    ...s,
    lastExplain,
    log: [{ input, result, at: now() }, ...s.log].slice(0, 50),
  };

  return { state: s, result };
}
