#![cfg(test)]

use crate::{Cog, Intent, IntentKind, Urgency};

#[test]
fn ignite_seeds_prime_and_shell() {
    let mut cog = Cog::new();
    let report = cog.ignite();
    assert!(cog.ignited);
    assert!(!report.stages.is_empty());
    assert_eq!(cog.vessels.len(), 1);
    assert_eq!(cog.vessels[0].name, "prime");
    assert!(cog.agents.iter().any(|a| a.name == "shell"));
    assert!(cog.fabric.nodes.len() >= 2);
}

#[test]
fn spawn_and_query_roundtrip() {
    let mut cog = Cog::new();
    cog.ignite();
    let intent = Intent::new(
        "test",
        IntentKind::SpawnAgent {
            name: "builder".into(),
            purpose: "compile".into(),
        },
        Urgency::Normal,
    );
    let result = cog.submit(intent);
    assert!(result.ok);

    let weave = Intent::new(
        "test",
        IntentKind::WeaveNode {
            title: "note".into(),
            node_kind: "doc".into(),
            body: "hello fabric".into(),
        },
        Urgency::Normal,
    );
    assert!(cog.submit(weave).ok);

    let q = Intent::new(
        "test",
        IntentKind::QueryFabric {
            query: "fabric".into(),
        },
        Urgency::Normal,
    );
    let qr = cog.submit(q);
    assert!(qr.ok);
    let hits = qr.data.get("hits").and_then(|h| h.as_array()).unwrap();
    assert!(!hits.is_empty());
}

#[test]
fn parse_spawn_line() {
    let cog = Cog::new();
    let intent = cog
        .parse_line_resolved("dev", "spawn builder :: compile workspace")
        .unwrap();
    match intent.kind {
        IntentKind::SpawnAgent { name, purpose } => {
            assert_eq!(name, "builder");
            assert_eq!(purpose, "compile workspace");
        }
        _ => panic!("expected spawn"),
    }
}

#[test]
fn sovereign_revision_requires_approve_before_apply() {
    let mut cog = Cog::new();
    cog.ignite();
    let propose = cog
        .parse_line_resolved("dev", "revise propose harden :: safer defaults")
        .unwrap();
    assert!(cog.submit(propose).ok);

    let apply_early = cog
        .parse_line_resolved("dev", "revise apply harden")
        .unwrap();
    assert!(!cog.submit(apply_early).ok);

    let approve = cog
        .parse_line_resolved("dev", "revise approve harden")
        .unwrap();
    assert!(cog.submit(approve).ok);
    let apply = cog
        .parse_line_resolved("dev", "revise apply harden")
        .unwrap();
    assert!(cog.submit(apply).ok);
}

#[test]
fn sticky_pref_blocks_silent_mutation() {
    let mut cog = Cog::new();
    cog.ignite();
    let silent = cog
        .parse_line_resolved("dev", "pref set telemetry on")
        .unwrap();
    let result = cog.submit(silent);
    assert!(!result.ok);
    assert!(result.message.contains("sticky"));
}

#[test]
fn explain_is_glass_box() {
    let mut cog = Cog::new();
    cog.ignite();
    let intent = cog.parse_line_resolved("dev", "explain system").unwrap();
    let result = cog.submit(intent);
    assert!(result.ok);
    assert!(result.data.get("why_not_reboot").is_some());
    assert!(result.data.get("doctrine").is_some());
}

#[test]
fn silicon_declare_bind_accel_nvidia_and_amd() {
    let mut cog = Cog::new();
    cog.ignite();

    let d1 = cog
        .parse_line_resolved(
            "dev",
            "silicon declare nvidia gpu rtx0 24576 128 :: NVIDIA GeForce RTX 4090",
        )
        .unwrap();
    assert!(cog.submit(d1).ok);

    let d2 = cog
        .parse_line_resolved(
            "dev",
            "silicon declare amd cpu zen0 65536 32 :: AMD Ryzen 9 7950X",
        )
        .unwrap();
    assert!(cog.submit(d2).ok);

    let d3 = cog
        .parse_line_resolved(
            "dev",
            "silicon declare amd gpu rx0 16384 96 :: AMD Radeon RX 7900 XTX",
        )
        .unwrap();
    assert!(cog.submit(d3).ok);

    assert!(cog.lattice.nvidia_gpus().count() >= 1);
    assert!(cog.lattice.amd_cpus().count() >= 1);
    assert!(cog.lattice.amd_gpus().count() >= 1);

    let spawn = cog
        .parse_line_resolved("dev", "spawn trainer :: train model on CUDA")
        .unwrap();
    assert!(cog.submit(spawn).ok);

    let bind = cog
        .parse_line_resolved("dev", "silicon bind trainer rtx0")
        .unwrap();
    assert!(cog.submit(bind).ok);

    let accel = cog
        .parse_line_resolved("dev", "silicon accel trainer matmul batch-1024")
        .unwrap();
    let ar = cog.submit(accel);
    assert!(ar.ok);
    assert!(ar.message.contains("accel"));
}
