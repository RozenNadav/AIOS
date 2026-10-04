# AIOS Architecture

AIOS (AI Operating System) is a **cognition-first** operating system. It is **not** derived from Windows, Unix, Linux, POSIX, or Mach. Primaries are intents, agents, and semantic memory — not processes, files, or syscalls.

## Design thesis

Traditional OSes schedule *code*. AIOS schedules *purpose*.

The Cognition Core (Cog) is the kernel. It consumes developer intents, routes capability through Facets, wakes Agents, and persists meaning in the Knowledge Fabric.

## Layers

```
┌─────────────────────────────────────────────────────────┐
│  SURFACE          Developer Console / future consumer UI │
├─────────────────────────────────────────────────────────┤
│  VESSEL           Capability envelopes (isolation)       │
├─────────────────────────────────────────────────────────┤
│  PULSE            Attention / urgency scheduler          │
├─────────────────────────────────────────────────────────┤
│  FABRIC           Knowledge Fabric (semantic memory)     │
├─────────────────────────────────────────────────────────┤
│  AGENT RUNTIME    Living computational entities          │
├─────────────────────────────────────────────────────────┤
│  INTENT BUS       Typed purpose messages                 │
├─────────────────────────────────────────────────────────┤
│  COGNITION CORE   AI kernel (Cog)                        │
├─────────────────────────────────────────────────────────┤
│  SPARK            Bootstrap / host ignition              │
└─────────────────────────────────────────────────────────┘
```

## Primitive glossary (non-Unix)

| AIOS term | Role | Not the same as |
|-----------|------|-----------------|
| **Spark** | Boot ignition sequence | BIOS/UEFI bootloader |
| **Cog** | Cognition Core — the kernel | Linux/Windows kernel |
| **Intent** | Declared purpose message | syscall / IPC call |
| **Agent** | Living unit of work + identity | process / thread |
| **Facet** | Typed capability port | file descriptor / socket |
| **Node** | Semantic memory object | file / inode |
| **Pulse** | Attention scheduling tick | timer interrupt / CFS |
| **Vessel** | Isolation + capability envelope | container / VM / cgroup |
| **Surface** | Human–machine presentation | desktop / shell UI |
| **Die / Lane** | Silicon unit + compute pathway (NVIDIA/AMD) | `/dev/nvidia*`, WDDM device |
| **Lattice** | Discovered NVIDIA GPUs + AMD CPU/GPU set | device manager |

## Intent model

An Intent is `{ actor, verb, target, payload, urgency }`.

Examples:

- `spawn agent build-runner with purpose "compile workspace"`
- `weave node "crate:aios-kernel" under project:aios`
- `attend agent:shell urgency:high`

Cog resolves intents through policy + routing — there is no POSIX syscall table.

## Agent lifecycle

`Seed → Awake → Attend → Act → Rest → Dissolve`

Agents hold a purpose string, facet set, vessel id, and fabric slice. They do not inherit Unix PIDs, uid/gid, or fork/exec semantics.

## Knowledge Fabric

A graph of Nodes with Relations. Paths are semantic queries, not directory trees. Persistence is JSON-backed in v0.1 for developers; later stages swap in a native fabric store.

## Security model

Capability-based via Facets and Vessels. An agent may only emit intents its vessel permits. There is no global root user in the Unix sense — elevation is a temporary **Clarity Grant** issued by Cog.

## Roadmap stages

1. **DevCore (now)** — kernel crate, intent shell, developer console
2. **Tooling** — package weave, debugger, agent marketplace
3. **Hosted** — multi-vessel networking, remote Cog
4. **Consumer** — friendlier Surface, assistants, app model
