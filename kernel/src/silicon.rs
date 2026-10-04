//! Silicon Lattice — AIOS hardware substrate for NVIDIA GPUs and AMD chips.
//!
//! Not a Unix device tree. Dies and lanes are cognition-facing compute units.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use uuid::Uuid;

/// Silicon vendor identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Vendor {
    Nvidia,
    Amd,
    /// Present for host visibility only; AIOS compute focus is NVIDIA + AMD.
    Other,
}

impl Vendor {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nvidia => "nvidia",
            Self::Amd => "amd",
            Self::Other => "other",
        }
    }
}

/// Kind of die in the lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DieKind {
    Gpu,
    Cpu,
}

/// A compute pathway on a die.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lane {
    pub name: String,
    pub kind: String,
    pub tflops_est: f32,
    pub available: bool,
}

/// One silicon unit (GPU or CPU package).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Die {
    pub id: Uuid,
    pub name: String,
    pub vendor: Vendor,
    pub kind: DieKind,
    pub model: String,
    pub memory_mb: u64,
    pub compute_units: u32,
    pub lanes: Vec<Lane>,
    pub driver_hint: String,
    pub source: String,
    pub bound_agents: Vec<String>,
    pub discovered_at: DateTime<Utc>,
}

impl Die {
    pub fn facet_prefix(&self) -> String {
        match (self.vendor, self.kind) {
            (Vendor::Nvidia, DieKind::Gpu) => "silicon.nvidia".into(),
            (Vendor::Amd, DieKind::Gpu) => "silicon.amd.rocm".into(),
            (Vendor::Amd, DieKind::Cpu) => "silicon.amd.zen".into(),
            _ => "silicon.other".into(),
        }
    }

    pub fn primary_facets(&self) -> Vec<String> {
        let mut out = vec![format!("silicon.bind")];
        match (self.vendor, self.kind) {
            (Vendor::Nvidia, DieKind::Gpu) => {
                out.push("silicon.nvidia.cuda".into());
                if self.lanes.iter().any(|l| l.kind == "tensor") {
                    out.push("silicon.nvidia.tensor".into());
                }
            }
            (Vendor::Amd, DieKind::Gpu) => {
                out.push("silicon.amd.rocm".into());
            }
            (Vendor::Amd, DieKind::Cpu) => {
                out.push("silicon.amd.zen".into());
            }
            _ => {}
        }
        out
    }
}

/// A scheduled compute burst on a bound lane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccelJob {
    pub id: Uuid,
    pub agent: String,
    pub die_id: Uuid,
    pub lane: String,
    pub work: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// The Silicon Lattice — AIOS hardware view.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lattice {
    pub dies: Vec<Die>,
    pub jobs: Vec<AccelJob>,
    pub last_probe_at: Option<DateTime<Utc>>,
    pub probe_notes: Vec<String>,
}

impl Lattice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn nvidia_gpus(&self) -> impl Iterator<Item = &Die> {
        self.dies
            .iter()
            .filter(|d| d.vendor == Vendor::Nvidia && d.kind == DieKind::Gpu)
    }

    pub fn amd_cpus(&self) -> impl Iterator<Item = &Die> {
        self.dies
            .iter()
            .filter(|d| d.vendor == Vendor::Amd && d.kind == DieKind::Cpu)
    }

    pub fn amd_gpus(&self) -> impl Iterator<Item = &Die> {
        self.dies
            .iter()
            .filter(|d| d.vendor == Vendor::Amd && d.kind == DieKind::Gpu)
    }

    pub fn find_die(&self, ref_str: &str) -> Option<&Die> {
        let n = ref_str.to_ascii_lowercase();
        self.dies.iter().find(|d| {
            d.id.to_string().starts_with(ref_str)
                || d.name.to_ascii_lowercase() == n
                || d.model.to_ascii_lowercase().contains(&n)
                || d.name.to_ascii_lowercase().contains(&n)
        })
    }

    pub fn find_die_mut(&mut self, ref_str: &str) -> Option<&mut Die> {
        let n = ref_str.to_ascii_lowercase();
        self.dies.iter_mut().find(|d| {
            d.id.to_string().starts_with(ref_str)
                || d.name.to_ascii_lowercase() == n
                || d.model.to_ascii_lowercase().contains(&n)
                || d.name.to_ascii_lowercase().contains(&n)
        })
    }

    /// Probe the host for NVIDIA GPUs and AMD CPU/GPU silicon.
    pub fn probe(&mut self) -> Vec<String> {
        let mut notes = Vec::new();
        let mut found: Vec<Die> = Vec::new();

        match probe_nvidia() {
            Ok(dies) => {
                notes.push(format!("nvidia: discovered {} GPU die(s)", dies.len()));
                found.extend(dies);
            }
            Err(e) => notes.push(format!("nvidia: {e}")),
        }

        match probe_amd_cpu() {
            Ok(Some(die)) => {
                notes.push(format!("amd-cpu: discovered {}", die.model));
                found.push(die);
            }
            Ok(None) => notes.push("amd-cpu: no AuthenticAMD package on this host".into()),
            Err(e) => notes.push(format!("amd-cpu: {e}")),
        }

        match probe_amd_gpu() {
            Ok(dies) => {
                notes.push(format!("amd-gpu: discovered {} GPU die(s)", dies.len()));
                found.extend(dies);
            }
            Err(e) => notes.push(format!("amd-gpu: {e}")),
        }

        // Preserve declared dies (source == "declare") across probes.
        let declared: Vec<Die> = self
            .dies
            .iter()
            .filter(|d| d.source == "declare")
            .cloned()
            .collect();

        // Merge: probed replaces previous probed; declared kept unless same name.
        let mut merged = found;
        for d in declared {
            if !merged.iter().any(|m| m.name == d.name) {
                merged.push(d);
            }
        }

        self.dies = merged;
        self.last_probe_at = Some(Utc::now());
        self.probe_notes = notes.clone();
        notes
    }

    pub fn declare_nvidia_gpu(
        &mut self,
        name: &str,
        model: &str,
        memory_mb: u64,
        sm_count: u32,
    ) -> Die {
        let die = Die {
            id: Uuid::new_v4(),
            name: name.into(),
            vendor: Vendor::Nvidia,
            kind: DieKind::Gpu,
            model: model.into(),
            memory_mb,
            compute_units: sm_count,
            lanes: nvidia_lanes(sm_count, memory_mb),
            driver_hint: "NVIDIA CUDA / AIOS silicon.nvidia.* facets".into(),
            source: "declare".into(),
            bound_agents: Vec::new(),
            discovered_at: Utc::now(),
        };
        self.dies.retain(|d| d.name != die.name);
        self.dies.push(die.clone());
        die
    }

    pub fn declare_amd_cpu(&mut self, name: &str, model: &str, cores: u32, memory_mb: u64) -> Die {
        let die = Die {
            id: Uuid::new_v4(),
            name: name.into(),
            vendor: Vendor::Amd,
            kind: DieKind::Cpu,
            model: model.into(),
            memory_mb,
            compute_units: cores,
            lanes: amd_zen_lanes(cores),
            driver_hint: "AMD Zen / AIOS silicon.amd.zen facet".into(),
            source: "declare".into(),
            bound_agents: Vec::new(),
            discovered_at: Utc::now(),
        };
        self.dies.retain(|d| d.name != die.name);
        self.dies.push(die.clone());
        die
    }

    pub fn declare_amd_gpu(
        &mut self,
        name: &str,
        model: &str,
        memory_mb: u64,
        cu_count: u32,
    ) -> Die {
        let die = Die {
            id: Uuid::new_v4(),
            name: name.into(),
            vendor: Vendor::Amd,
            kind: DieKind::Gpu,
            model: model.into(),
            memory_mb,
            compute_units: cu_count,
            lanes: amd_rocm_lanes(cu_count, memory_mb),
            driver_hint: "AMD ROCm / AIOS silicon.amd.rocm facet".into(),
            source: "declare".into(),
            bound_agents: Vec::new(),
            discovered_at: Utc::now(),
        };
        self.dies.retain(|d| d.name != die.name);
        self.dies.push(die.clone());
        die
    }

    pub fn bind_agent(&mut self, agent: &str, die_ref: &str) -> Result<&Die, String> {
        // Two-step to avoid borrow issues.
        let die_id = self
            .find_die(die_ref)
            .map(|d| d.id)
            .ok_or_else(|| format!("unknown die '{die_ref}'"))?;
        let die = self
            .dies
            .iter_mut()
            .find(|d| d.id == die_id)
            .ok_or_else(|| format!("unknown die '{die_ref}'"))?;
        if !die.bound_agents.iter().any(|a| a == agent) {
            die.bound_agents.push(agent.into());
        }
        Ok(self.dies.iter().find(|d| d.id == die_id).unwrap())
    }

    pub fn unbind_agent(&mut self, agent: &str) -> usize {
        let mut n = 0;
        for die in &mut self.dies {
            let before = die.bound_agents.len();
            die.bound_agents.retain(|a| a != agent);
            n += before - die.bound_agents.len();
        }
        n
    }

    pub fn accel(&mut self, agent: &str, work: &str) -> Result<AccelJob, String> {
        let die = self
            .dies
            .iter()
            .find(|d| d.bound_agents.iter().any(|a| a == agent))
            .ok_or_else(|| {
                format!("agent '{agent}' is not bound to any die — use: silicon bind {agent} <die>")
            })?;
        let lane = die
            .lanes
            .iter()
            .find(|l| l.available)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| "default".into());
        let job = AccelJob {
            id: Uuid::new_v4(),
            agent: agent.into(),
            die_id: die.id,
            lane: lane.clone(),
            work: work.into(),
            status: "completed".into(),
            created_at: Utc::now(),
        };
        self.jobs.push(job.clone());
        if self.jobs.len() > 100 {
            let drain = self.jobs.len() - 100;
            self.jobs.drain(0..drain);
        }
        Ok(job)
    }

    pub fn summary_json(&self) -> serde_json::Value {
        serde_json::json!({
            "dies": self.dies,
            "nvidia_gpus": self.nvidia_gpus().count(),
            "amd_cpus": self.amd_cpus().count(),
            "amd_gpus": self.amd_gpus().count(),
            "jobs": self.jobs.len(),
            "last_probe_at": self.last_probe_at,
            "probe_notes": self.probe_notes,
        })
    }
}

fn nvidia_lanes(sm_count: u32, memory_mb: u64) -> Vec<Lane> {
    let scale = (sm_count as f32 / 64.0).max(0.25);
    vec![
        Lane {
            name: "cuda".into(),
            kind: "cuda".into(),
            tflops_est: 20.0 * scale,
            available: true,
        },
        Lane {
            name: "tensor".into(),
            kind: "tensor".into(),
            tflops_est: 80.0 * scale,
            available: memory_mb >= 8192,
        },
        Lane {
            name: "rt".into(),
            kind: "rt".into(),
            tflops_est: 10.0 * scale,
            available: true,
        },
    ]
}

fn amd_zen_lanes(cores: u32) -> Vec<Lane> {
    vec![Lane {
        name: "zen".into(),
        kind: "cpu".into(),
        tflops_est: cores as f32 * 0.05,
        available: true,
    }]
}

fn amd_rocm_lanes(cu_count: u32, memory_mb: u64) -> Vec<Lane> {
    let scale = (cu_count as f32 / 60.0).max(0.25);
    vec![
        Lane {
            name: "rocm".into(),
            kind: "rocm".into(),
            tflops_est: 18.0 * scale,
            available: true,
        },
        Lane {
            name: "rdna".into(),
            kind: "rdna".into(),
            tflops_est: 22.0 * scale,
            available: memory_mb >= 4096,
        },
    ]
}

fn probe_nvidia() -> Result<Vec<Die>, String> {
    // Prefer nvidia-smi query when present.
    if let Ok(output) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,compute_cap",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut dies = Vec::new();
            for (idx, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
                let parts: Vec<_> = line.split(',').map(|s| s.trim()).collect();
                let model = parts.first().copied().unwrap_or("NVIDIA GPU").to_string();
                let memory_mb: u64 = parts
                    .get(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(8192);
                let sm_count = 64; // smi doesn't give SM count directly; estimate later
                dies.push(Die {
                    id: Uuid::new_v4(),
                    name: format!("nvidia-{idx}"),
                    vendor: Vendor::Nvidia,
                    kind: DieKind::Gpu,
                    model,
                    memory_mb,
                    compute_units: sm_count,
                    lanes: nvidia_lanes(sm_count, memory_mb),
                    driver_hint: "nvidia-smi / CUDA".into(),
                    source: "probe".into(),
                    bound_agents: Vec::new(),
                    discovered_at: Utc::now(),
                });
            }
            if !dies.is_empty() {
                return Ok(dies);
            }
        }
    }

    // Sysfs PCI vendor 10de (NVIDIA)
    let pci = read_pci_vendor_devices("10de");
    if !pci.is_empty() {
        let mut dies = Vec::new();
        for (idx, label) in pci.into_iter().enumerate() {
            dies.push(Die {
                id: Uuid::new_v4(),
                name: format!("nvidia-{idx}"),
                vendor: Vendor::Nvidia,
                kind: DieKind::Gpu,
                model: label,
                memory_mb: 0,
                compute_units: 0,
                lanes: nvidia_lanes(48, 8192),
                driver_hint: "PCI 10de — install NVIDIA driver for full lanes".into(),
                source: "probe-pci".into(),
                bound_agents: Vec::new(),
                discovered_at: Utc::now(),
            });
        }
        return Ok(dies);
    }

    if std::path::Path::new("/dev/nvidia0").exists() {
        return Ok(vec![Die {
            id: Uuid::new_v4(),
            name: "nvidia-0".into(),
            vendor: Vendor::Nvidia,
            kind: DieKind::Gpu,
            model: "NVIDIA GPU (/dev/nvidia0 present)".into(),
            memory_mb: 0,
            compute_units: 64,
            lanes: nvidia_lanes(64, 8192),
            driver_hint: "NVIDIA device node present".into(),
            source: "probe-dev".into(),
            bound_agents: Vec::new(),
            discovered_at: Utc::now(),
        }]);
    }

    Err("no NVIDIA GPU detected (nvidia-smi / PCI 10de / /dev/nvidia*)".into())
}

fn probe_amd_cpu() -> Result<Option<Die>, String> {
    let vendor = read_cpu_vendor()?;
    if vendor != "AuthenticAMD" {
        return Ok(None);
    }
    let model = read_cpu_model().unwrap_or_else(|| "AMD CPU".into());
    let cores = read_cpu_cores().unwrap_or(1);
    let mem = read_mem_mb().unwrap_or(0);
    Ok(Some(Die {
        id: Uuid::new_v4(),
        name: "amd-cpu-0".into(),
        vendor: Vendor::Amd,
        kind: DieKind::Cpu,
        model,
        memory_mb: mem,
        compute_units: cores,
        lanes: amd_zen_lanes(cores),
        driver_hint: "AMD CPUID AuthenticAMD".into(),
        source: "probe".into(),
        bound_agents: Vec::new(),
        discovered_at: Utc::now(),
    }))
}

fn probe_amd_gpu() -> Result<Vec<Die>, String> {
    if let Ok(output) = Command::new("rocminfo").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut dies = Vec::new();
            for (idx, line) in text.lines().enumerate() {
                if line.contains("Marketing Name") || line.contains("Card Name") {
                    let model = line
                        .split(':')
                        .nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|| "AMD GPU".into());
                    dies.push(Die {
                        id: Uuid::new_v4(),
                        name: format!("amd-gpu-{idx}"),
                        vendor: Vendor::Amd,
                        kind: DieKind::Gpu,
                        model,
                        memory_mb: 8192,
                        compute_units: 60,
                        lanes: amd_rocm_lanes(60, 8192),
                        driver_hint: "ROCm rocminfo".into(),
                        source: "probe".into(),
                        bound_agents: Vec::new(),
                        discovered_at: Utc::now(),
                    });
                }
            }
            if !dies.is_empty() {
                return Ok(dies);
            }
        }
    }

    let pci = read_pci_vendor_devices("1002");
    if !pci.is_empty() {
        let mut dies = Vec::new();
        for (idx, label) in pci.into_iter().enumerate() {
            dies.push(Die {
                id: Uuid::new_v4(),
                name: format!("amd-gpu-{idx}"),
                vendor: Vendor::Amd,
                kind: DieKind::Gpu,
                model: label,
                memory_mb: 0,
                compute_units: 0,
                lanes: amd_rocm_lanes(40, 8192),
                driver_hint: "PCI 1002 — install ROCm for full lanes".into(),
                source: "probe-pci".into(),
                bound_agents: Vec::new(),
                discovered_at: Utc::now(),
            });
        }
        return Ok(dies);
    }

    Err("no AMD GPU detected (rocminfo / PCI 1002)".into())
}

fn read_cpu_vendor() -> Result<String, String> {
    let data = std::fs::read_to_string("/proc/cpuinfo")
        .map_err(|e| format!("cannot read cpuinfo: {e}"))?;
    for line in data.lines() {
        if let Some(v) = line.strip_prefix("vendor_id") {
            let v = v.trim().trim_start_matches(':').trim();
            return Ok(v.to_string());
        }
    }
    Err("vendor_id missing".into())
}

fn read_cpu_model() -> Option<String> {
    let data = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in data.lines() {
        if let Some(v) = line.strip_prefix("model name") {
            return Some(v.trim().trim_start_matches(':').trim().to_string());
        }
    }
    None
}

fn read_cpu_cores() -> Option<u32> {
    let data = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let n = data.lines().filter(|l| l.starts_with("processor")).count();
    Some(n.max(1) as u32)
}

fn read_mem_mb() -> Option<u64> {
    let data = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in data.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            let kb: u64 = v.split_whitespace().next()?.parse().ok()?;
            return Some(kb / 1024);
        }
    }
    None
}

/// Best-effort PCI vendor scan via sysfs (works without lspci).
fn read_pci_vendor_devices(vendor_hex: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices") else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let vendor = std::fs::read_to_string(path.join("vendor"))
            .unwrap_or_default()
            .trim()
            .trim_start_matches("0x")
            .to_ascii_lowercase();
        if vendor != vendor_hex {
            continue;
        }
        let device = std::fs::read_to_string(path.join("device"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let class = std::fs::read_to_string(path.join("class"))
            .unwrap_or_default()
            .trim()
            .to_string();
        // Display / 3D controllers roughly 0x03xxxx
        if class.starts_with("0x03") || class.starts_with("0x3") {
            out.push(format!(
                "PCI {}:{} class {}",
                path.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                device,
                class
            ));
        }
    }
    out
}
