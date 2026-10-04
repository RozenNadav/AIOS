use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{AIOS_LINEAGE, AIOS_VERSION};

/// Stages of Spark ignition (AIOS bootstrap — not UEFI/GRUB).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootStage {
    pub name: String,
    pub detail: String,
    pub ok: bool,
    pub at: DateTime<Utc>,
}

/// Report produced when Spark finishes igniting Cog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootReport {
    pub version: String,
    pub lineage: String,
    pub stages: Vec<BootStage>,
    pub ignited_at: DateTime<Utc>,
}

/// Spark — bootstrap sequence that ignites the Cognition Core.
pub struct Spark;

impl Spark {
    pub fn ignite() -> BootReport {
        let mut stages = Vec::new();

        stages.push(BootStage {
            name: "spark.presence".into(),
            detail: "Establish host presence for Cognition Core".into(),
            ok: true,
            at: Utc::now(),
        });

        stages.push(BootStage {
            name: "spark.intent_bus".into(),
            detail: "Open Intent Bus channels".into(),
            ok: true,
            at: Utc::now(),
        });

        stages.push(BootStage {
            name: "spark.fabric".into(),
            detail: "Attach Knowledge Fabric substrate".into(),
            ok: true,
            at: Utc::now(),
        });

        stages.push(BootStage {
            name: "spark.pulse".into(),
            detail: "Arm Pulse attention scheduler".into(),
            ok: true,
            at: Utc::now(),
        });

        stages.push(BootStage {
            name: "spark.silicon".into(),
            detail: "Probe Silicon Lattice (NVIDIA GPUs · AMD Zen/ROCm)".into(),
            ok: true,
            at: Utc::now(),
        });

        stages.push(BootStage {
            name: "spark.vessel.prime".into(),
            detail: "Open prime developer vessel".into(),
            ok: true,
            at: Utc::now(),
        });

        stages.push(BootStage {
            name: "spark.cog.awaken".into(),
            detail: "Awaken Cognition Core".into(),
            ok: true,
            at: Utc::now(),
        });

        BootReport {
            version: AIOS_VERSION.into(),
            lineage: AIOS_LINEAGE.into(),
            stages,
            ignited_at: Utc::now(),
        }
    }
}
