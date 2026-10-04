export type AgentState =
  | "seed"
  | "awake"
  | "attend"
  | "act"
  | "rest"
  | "dissolve";

export type RevisionState =
  | "proposed"
  | "approved"
  | "applied"
  | "rolledback"
  | "rejected";

export interface Agent {
  id: string;
  name: string;
  purpose: string;
  state: AgentState;
  vessel: string;
  facets: string[];
  attention: number;
  createdAt: string;
}

export interface Vessel {
  id: string;
  name: string;
  policy: string;
  sealed: boolean;
  maxAgents: number;
}

export interface FabricNode {
  id: string;
  title: string;
  kind: string;
  body: string;
  createdAt: string;
}

export interface Relation {
  from: string;
  to: string;
  label: string;
}

export interface IntentResult {
  ok: boolean;
  message: string;
  data?: unknown;
}

export interface BootStage {
  name: string;
  detail: string;
}

export interface PulseEvent {
  agent: string;
  note: string;
  attention: number;
  at: string;
}

export interface Revision {
  id: string;
  title: string;
  summary: string;
  state: RevisionState;
  checkpoint: string;
}

export interface StickyPref {
  key: string;
  value: string;
  sticky: boolean;
}

export interface ContinuitySnap {
  name: string;
  agentNames: string[];
  prefs: StickyPref[];
}

export interface Lane {
  name: string;
  kind: string;
  tflopsEst: number;
  available: boolean;
}

export interface Die {
  id: string;
  name: string;
  vendor: "nvidia" | "amd" | "other";
  kind: "gpu" | "cpu";
  model: string;
  memoryMb: number;
  computeUnits: number;
  lanes: Lane[];
  boundAgents: string[];
  source: string;
}

export interface AccelJob {
  id: string;
  agent: string;
  dieId: string;
  lane: string;
  work: string;
  status: string;
}

export interface CogState {
  version: string;
  lineage: string;
  doctrine: string;
  ignited: boolean;
  bootStages: BootStage[];
  vessels: Vessel[];
  agents: Agent[];
  nodes: FabricNode[];
  relations: Relation[];
  pulseTicks: number;
  pulseHistory: PulseEvent[];
  revisions: Revision[];
  focusShield: boolean;
  blockedClaims: number;
  prefs: StickyPref[];
  continuity: ContinuitySnap[];
  dies: Die[];
  accelJobs: AccelJob[];
  probeNotes: string[];
  log: { input: string; result: IntentResult; at: string }[];
  lastExplain?: string;
}
