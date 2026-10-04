import { FormEvent, useEffect, useRef, useState } from "react";
import { createCog, ignite, submitIntent } from "./cog/engine";
import type { CogState } from "./cog/types";

const PAINS = [
  { pain: "Forced updates", fix: "Sovereign Revisions" },
  { pain: "Ads & nags", fix: "Attention Guard" },
  { pain: "“Just reboot”", fix: "Explain Mode" },
  { pain: "Settings reset", fix: "Sticky Prefs" },
  { pain: "Lost focus", fix: "Focus Shield" },
  { pain: "Tool sprawl", fix: "One Intent Bus" },
];

const SUGGESTIONS = [
  "silicon list",
  "spawn trainer :: train on CUDA tensor cores",
  "silicon bind trainer rtx0",
  "silicon accel trainer matmul batch-1024",
  "silicon bind trainer zen0",
  "silicon bind trainer rx0",
  "explain system",
  "tick",
];

export default function App() {
  const [phase, setPhase] = useState<"boot" | "work">("boot");
  const [booting, setBooting] = useState(false);
  const [visibleStages, setVisibleStages] = useState(0);
  const [cog, setCog] = useState<CogState>(() => createCog());
  const [line, setLine] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!booting || !cog.ignited) return;
    if (visibleStages >= cog.bootStages.length) {
      const t = window.setTimeout(() => setPhase("work"), 550);
      return () => window.clearTimeout(t);
    }
    const t = window.setTimeout(() => setVisibleStages((n) => n + 1), 280);
    return () => window.clearTimeout(t);
  }, [booting, cog.bootStages.length, cog.ignited, visibleStages]);

  useEffect(() => {
    if (phase === "work") inputRef.current?.focus();
  }, [phase]);

  function startIgnition() {
    const next = ignite(createCog());
    setCog(next);
    setVisibleStages(0);
    setBooting(true);
  }

  function enterWorkspace() {
    setCog((c) => (c.ignited ? c : ignite(c)));
    setPhase("work");
  }

  function run(raw: string) {
    const text = raw.trim();
    if (!text) return;
    setCog((prev) => submitIntent(prev, text).state);
    setLine("");
  }

  function onSubmit(e: FormEvent) {
    e.preventDefault();
    run(line);
  }

  if (phase === "boot") {
    return (
      <section className="boot" aria-label="AIOS ignition">
        <div className="boot-field" />
        <div className="boot-orb" aria-hidden />
        <div className="boot-content">
          <h1 className="boot-brand">AIOS</h1>
          <p className="boot-tag">Future-driven · Developer Edition · v0.2</p>
          <p className="boot-lead">
            Built against what current OSes get wrong: forced updates, attention
            theft, opaque failures, and AI that nags. Consent-first cognition.
          </p>
          <div className="boot-actions">
            <button className="btn btn-primary" onClick={startIgnition} disabled={booting}>
              {booting ? "Igniting…" : "Ignite Cog"}
            </button>
            <button className="btn btn-ghost" onClick={enterWorkspace}>
              Skip to Surface
            </button>
          </div>
          {!booting && (
            <ul className="pain-grid" aria-label="Pain to future mapping">
              {PAINS.map((p) => (
                <li key={p.pain}>
                  <span className="pain">{p.pain}</span>
                  <span className="arrow">→</span>
                  <span className="fix">{p.fix}</span>
                </li>
              ))}
            </ul>
          )}
          {booting && (
            <div className="boot-stages" aria-live="polite">
              {cog.bootStages.map((stage, i) => (
                <div
                  key={stage.name}
                  className={`boot-stage ${i < visibleStages ? "visible" : ""}`}
                >
                  <span className="mark">✓</span>
                  <span className="name">{stage.name}</span>
                  <span className="detail">— {stage.detail}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </section>
    );
  }

  const agents = cog.agents.filter((a) => a.state !== "dissolve");

  return (
    <div className="workspace">
      <header className="topbar">
        <div className="topbar-brand">
          AIOS <span>Dev</span>
        </div>
        <div className="topbar-meta">
          <span>v{cog.version}</span>
          <span className={cog.focusShield ? "hot" : ""}>
            focus {cog.focusShield ? "ON" : "off"}
          </span>
          <span>
            NVIDIA {cog.dies.filter((d) => d.vendor === "nvidia").length}
          </span>
          <span>
            AMD {cog.dies.filter((d) => d.vendor === "amd").length}
          </span>
          <span>{agents.length} agents</span>
          <span>pulse {cog.pulseTicks}</span>
        </div>
      </header>

      <p className="doctrine-strip">{cog.doctrine}</p>

      <main className="layout">
        <section>
          <div className="panel">
            <h2>Intent Bus</h2>
            <p className="sub">One bus instead of fourteen tools. Speak purpose.</p>
            <form className="intent-box" onSubmit={onSubmit}>
              <input
                ref={inputRef}
                value={line}
                onChange={(e) => setLine(e.target.value)}
                placeholder="explain system"
                aria-label="Intent line"
              />
              <button type="submit">Emit</button>
            </form>
            <div className="chips">
              {SUGGESTIONS.map((s) => (
                <button key={s} type="button" className="chip" onClick={() => run(s)}>
                  {s}
                </button>
              ))}
            </div>
            <div className="log">
              {cog.log.map((entry, i) => (
                <div
                  key={`${entry.at}-${i}`}
                  className={`log-item ${entry.result.ok ? "" : "err"}`}
                >
                  <div className="in">› {entry.input}</div>
                  <div className="out">
                    {entry.result.ok ? "ok" : "err"} — {entry.result.message}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {cog.lastExplain && (
            <div className="panel explain-panel">
              <h2>Explain</h2>
              <p className="sub">Glass box — not “just reboot”.</p>
              <pre className="explain-body">{cog.lastExplain}</pre>
            </div>
          )}

          <div className="panel" style={{ marginTop: "1.75rem" }}>
            <h2>Agents</h2>
            <p className="sub">Living units of work — not processes.</p>
            <div className="agent-list">
              {agents.map((a) => (
                <div key={a.id} className="agent-row">
                  <div>
                    <span className={`state-pill ${a.state}`}>{a.state}</span>
                    <strong>{a.name}</strong>
                  </div>
                  <div className="meta">{a.purpose}</div>
                  <div className="attn" title={`attention ${a.attention.toFixed(2)}`}>
                    <i style={{ width: `${Math.min(100, a.attention * 20)}%` }} />
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>

        <aside className="side-stack">
          <div className="panel">
            <h2>Silicon Lattice</h2>
            <p className="sub">NVIDIA CUDA · AMD Zen/ROCm — dies & lanes, not /dev nodes.</p>
            <div className="vessel-list">
              {cog.dies.length === 0 && (
                <div className="meta">No dies — emit `silicon probe` or declare.</div>
              )}
              {cog.dies.map((d) => (
                <div key={d.id} className="vessel-row">
                  <strong>
                    <span className={`state-pill ${d.vendor}`}>{d.vendor}</span>
                    {d.name}{" "}
                    <span className="meta">{d.kind}</span>
                  </strong>
                  <div className="meta">
                    {d.model} · {d.memoryMb}MB · {d.computeUnits}{" "}
                    {d.kind === "cpu" ? "cores" : "CU/SM"} · lanes{" "}
                    {d.lanes.map((l) => l.name).join(", ")}
                  </div>
                  {d.boundAgents.length > 0 && (
                    <div className="meta">bound: {d.boundAgents.join(", ")}</div>
                  )}
                </div>
              ))}
            </div>
            {cog.accelJobs[0] && (
              <div className="meta" style={{ marginTop: "0.75rem" }}>
                last accel: {cog.accelJobs[0].agent} / {cog.accelJobs[0].lane} —{" "}
                {cog.accelJobs[0].work}
              </div>
            )}
          </div>

          <div className="panel">
            <h2>Sovereign Revisions</h2>
            <p className="sub">Propose → approve → apply → rollback. Never forced.</p>
            <div className="vessel-list">
              {cog.revisions.length === 0 && (
                <div className="meta">No revisions yet — try `revise propose …`</div>
              )}
              {cog.revisions.map((r) => (
                <div key={r.id} className="vessel-row">
                  <strong>{r.title}</strong>
                  <div className="meta">
                    <span className={`state-pill ${r.state}`}>{r.state}</span>
                    {r.summary}
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="panel">
            <h2>Sticky Prefs</h2>
            <p className="sub">Silent mutation refused. Telemetry off by default.</p>
            <div className="vessel-list">
              {cog.prefs.map((p) => (
                <div key={p.key} className="vessel-row">
                  <strong>
                    {p.key}={p.value}
                  </strong>
                  <div className="meta">{p.sticky ? "sticky · immune to silent reset" : "mutable"}</div>
                </div>
              ))}
            </div>
          </div>

          <div className="panel">
            <h2>Knowledge Fabric</h2>
            <p className="sub">Semantic memory — kills knowledge silos.</p>
            <div className="node-list">
              {cog.nodes.slice(0, 8).map((n) => (
                <div key={n.id} className="node-row">
                  <strong>{n.title}</strong>
                  <div className="meta">
                    {n.kind} · {n.body}
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="panel">
            <h2>Vessels</h2>
            <p className="sub">Capability envelopes — try policy `private`.</p>
            <div className="vessel-list">
              {cog.vessels.map((v) => (
                <div key={v.id} className="vessel-row">
                  <strong>{v.name}</strong>
                  <div className="meta">
                    {v.sealed ? "sealed" : "open"} · {v.policy}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </aside>
      </main>
    </div>
  );
}
