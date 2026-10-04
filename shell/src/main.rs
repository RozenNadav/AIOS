use std::io::{self, Write};
use std::path::PathBuf;

use aios_kernel::{Cog, IntentKind, AIOS_LINEAGE, AIOS_VERSION};
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "aios",
    about = "AIOS — AI-native operating system for developers",
    version = AIOS_VERSION,
    long_about = "Intent shell for the AIOS Cognition Core.\nNot Unix. Not Windows. Purpose-scheduled."
)]
struct Cli {
    /// Path to Cog state file
    #[arg(long, global = true)]
    state: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Ignite Cognition Core (boot)
    Boot,
    /// Run one intent and exit
    Intent {
        /// Intent line, e.g. "spawn builder :: compile workspace"
        text: Vec<String>,
    },
    /// Dump a JSON snapshot of Cog
    Snapshot,
    /// Export snapshot for the developer console
    Export {
        #[arg(short, long, default_value = "console/public/cog-snapshot.json")]
        out: PathBuf,
    },
    /// Interactive intent shell (default)
    Shell,
}

fn state_path(cli: &Cli) -> PathBuf {
    cli.state
        .clone()
        .unwrap_or_else(Cog::default_state_path)
}

fn load_or_new(path: &PathBuf) -> Cog {
    if path.exists() {
        match Cog::load(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "{} could not load state ({}): starting fresh",
                    "warn".yellow().bold(),
                    e
                );
                Cog::new()
            }
        }
    } else {
        Cog::new()
    }
}

fn print_banner() {
    println!("{}", "╔══════════════════════════════════════════╗".cyan());
    println!("{}", "║     AIOS  ·  Cognition Core  ·  Dev      ║".cyan().bold());
    println!("{}", "╚══════════════════════════════════════════╝".cyan());
    println!("{} {}", "lineage".dimmed(), AIOS_LINEAGE.white());
    println!("{} v{}", "version".dimmed(), AIOS_VERSION);
    println!();
}

fn print_help() {
    println!("{}", "Intent verbs".bold());
    println!("  attend [topic]                 Ask Cog to attend");
    println!("  explain [topic]                Glass-box diagnostics (no reboot)");
    println!("  spawn <name> :: <purpose>      Birth an agent");
    println!("  rest <agent>                   Quiet an agent");
    println!("  dissolve <agent>               End an agent");
    println!("  weave <title>|<kind>|<body>    Write Knowledge Fabric");
    println!("  query <text>                  Search fabric");
    println!("  vessel open <name> [policy]    Open vessel (developer|sandbox|private)");
    println!("  vessel seal <name>             Seal a vessel");
    println!("  clarity <agent> <facet>        Grant capability (consent)");
    println!("  revise propose t :: summary    Sovereign revision (anti forced-update)");
    println!("  revise approve|apply|rollback  Approve / apply / undo revision");
    println!("  revise list                    List revisions");
    println!("  focus [on|off]                 Focus shield (anti-interruption)");
    println!("  pref set|get …                 Sticky preferences");
    println!("  continuity save|restore <name> Surface continuity");
    println!("  silicon probe|list             NVIDIA / AMD Silicon Lattice");
    println!("  silicon declare nvidia gpu …   Declare target GPU (lab hosts)");
    println!("  silicon declare amd cpu|gpu …  Declare AMD Zen / ROCm die");
    println!("  silicon bind <agent> <die>     Attach compute facets");
    println!("  silicon accel <agent> <work>   Run compute burst on bound die");
    println!("  note <text>                    Weave a developer note");
    println!("  tick                           Pulse scheduler step");
    println!("  status                         Compact status");
    println!("  help                           This list");
    println!("  exit / quit                    Leave shell");
    println!();
}

fn print_result(cog: &Cog, result: &aios_kernel::IntentResult) {
    if result.ok {
        println!("{} {}", "ok".green().bold(), result.message);
    } else {
        println!("{} {}", "err".red().bold(), result.message);
    }
    if let Some(hits) = result.data.get("hits").and_then(|h| h.as_array()) {
        for h in hits {
            let title = h.get("title").and_then(|v| v.as_str()).unwrap_or("?");
            let kind = h.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
            let id = h.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            println!(
                "  {} {} {}",
                kind.magenta(),
                title.white().bold(),
                id.dimmed()
            );
        }
    }
    if let Some(agents) = result.data.get("agents").and_then(|a| a.as_array()) {
        if result.message.starts_with("explain:") {
            for a in agents {
                let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let state = a.get("state").and_then(|v| v.as_str()).unwrap_or("?");
                let purpose = a.get("purpose").and_then(|v| v.as_str()).unwrap_or("");
                println!(
                    "  {} {}  {}",
                    name.green(),
                    state.yellow(),
                    purpose.dimmed()
                );
            }
            if let Some(d) = result.data.get("doctrine").and_then(|v| v.as_str()) {
                println!("  {}", d.cyan());
            }
        }
    }
    if let Some(revs) = result.data.get("revisions").and_then(|r| r.as_array()) {
        for r in revs {
            let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("?");
            let state = r.get("state").and_then(|v| v.as_str()).unwrap_or("?");
            let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            println!(
                "  {} {} {}",
                state.yellow(),
                title.white().bold(),
                &id[..8.min(id.len())].dimmed()
            );
        }
    }
    let dies = result
        .data
        .get("dies")
        .or_else(|| result.data.get("lattice").and_then(|l| l.get("dies")))
        .and_then(|d| d.as_array());
    if let Some(dies) = dies {
        for d in dies {
            let name = d.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let vendor = d.get("vendor").and_then(|v| v.as_str()).unwrap_or("?");
            let kind = d.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
            let model = d.get("model").and_then(|v| v.as_str()).unwrap_or("?");
            let mem = d.get("memory_mb").and_then(|v| v.as_u64()).unwrap_or(0);
            println!(
                "  {} {} {}  {}  {}MB",
                vendor.magenta(),
                kind.yellow(),
                name.green().bold(),
                model.white(),
                mem
            );
        }
    }
    if result
        .data
        .get("topic")
        .and_then(|t| t.as_str())
        .map(|t| t == "help" || t == "status")
        .unwrap_or(false)
    {
        if result.data.get("topic").and_then(|t| t.as_str()) == Some("help") {
            print_help();
        } else {
            print_status(cog);
        }
    }
}

fn print_status(cog: &Cog) {
    println!(
        "{} ignited={} vessels={} agents={} nodes={} pulses={}",
        "cog".cyan().bold(),
        cog.ignited,
        cog.vessels.len(),
        cog.agents
            .iter()
            .filter(|a| a.state != aios_kernel::AgentState::Dissolve)
            .count(),
        cog.fabric.nodes.len(),
        cog.pulse.ticks
    );
    for a in cog
        .agents
        .iter()
        .filter(|a| a.state != aios_kernel::AgentState::Dissolve)
    {
        println!(
            "  agent {}  {}  attn={:.2}  {}",
            a.name.green(),
            format!("{:?}", a.state).yellow(),
            a.attention,
            a.purpose.dimmed()
        );
    }
    for v in &cog.vessels {
        let seal = if v.sealed { "sealed" } else { "open" };
        println!(
            "  vessel {}  {}  policy={}",
            v.name.blue(),
            seal,
            v.policy.label
        );
    }
    println!(
        "{} nvidia={} amd-cpu={} amd-gpu={} dies={}",
        "silicon".cyan().bold(),
        cog.lattice.nvidia_gpus().count(),
        cog.lattice.amd_cpus().count(),
        cog.lattice.amd_gpus().count(),
        cog.lattice.dies.len()
    );
    for d in &cog.lattice.dies {
        println!(
            "  die {}  {:?} {:?}  {}  lanes={}",
            d.name.green(),
            d.vendor,
            d.kind,
            d.model.dimmed(),
            d.lanes.len()
        );
    }
}

fn run_intent(cog: &mut Cog, actor: &str, line: &str) -> Result<()> {
    let intent = match cog.parse_line_resolved(actor, line) {
        Ok(i) => i,
        Err(e) => {
            println!("{} {}", "err".red().bold(), e);
            return Ok(());
        }
    };

    if matches!(&intent.kind, IntentKind::Attend { topic } if topic == "help") {
        print_help();
        return Ok(());
    }

    let result = cog.submit(intent);
    print_result(cog, &result);
    Ok(())
}

fn persist(cog: &Cog, path: &PathBuf) {
    if let Err(e) = cog.save(path) {
        eprintln!("{} save failed: {e}", "warn".yellow().bold());
    }
}

fn cmd_boot(path: &PathBuf) -> Result<()> {
    print_banner();
    let mut cog = load_or_new(path);
    let report = cog.ignite();
    println!("{} Cognition Core ignited", "spark".cyan().bold());
    for stage in &report.stages {
        let mark = if stage.ok {
            "✓".green()
        } else {
            "✗".red()
        };
        println!(
            "  {mark} {} — {}",
            stage.name.white(),
            stage.detail.dimmed()
        );
    }
    persist(&cog, path);
    println!();
    print_status(&cog);
    Ok(())
}

fn cmd_shell(path: &PathBuf) -> Result<()> {
    print_banner();
    let mut cog = load_or_new(path);
    if !cog.ignited {
        let _ = cog.ignite();
        println!("{} auto-ignited prime vessel", "spark".cyan().bold());
        persist(&cog, path);
    }
    print_status(&cog);
    println!(
        "\n{} type {} for verbs, {} to leave\n",
        "ready".green().bold(),
        "help".white().bold(),
        "exit".white().bold()
    );

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        write!(stdout, "{} ", "aios›".cyan().bold())?;
        stdout.flush()?;
        let mut line = String::new();
        let n = stdin.read_line(&mut line)?;
        if n == 0 {
            persist(&cog, path);
            println!();
            break;
        }
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if lower == "exit" || lower == "quit" {
            persist(&cog, path);
            println!("{}", "cog resting — state saved".dimmed());
            break;
        }
        run_intent(&mut cog, "developer", &line)?;
        persist(&cog, path);
    }
    Ok(())
}

fn cmd_export(path: &PathBuf, out: &PathBuf) -> Result<()> {
    let mut cog = load_or_new(path);
    cog.ensure_ignited();
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let snap = cog.snapshot();
    let mut value = serde_json::to_value(&snap)?;
    if let Some(obj) = value.as_object_mut() {
        let nodes: Vec<_> = cog.fabric.nodes.values().cloned().collect();
        obj.insert(
            "fabric".into(),
            serde_json::json!({
                "nodes": nodes,
                "relations": cog.fabric.relations,
            }),
        );
        obj.insert(
            "pulse_history".into(),
            serde_json::to_value(&cog.pulse.history)?,
        );
    }
    std::fs::write(out, serde_json::to_string_pretty(&value)?)?;
    persist(&cog, path);
    println!("{} {}", "exported".green().bold(), out.display());
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = state_path(&cli);

    match cli.command.unwrap_or(Commands::Shell) {
        Commands::Boot => cmd_boot(&path)?,
        Commands::Intent { text } => {
            let mut cog = load_or_new(&path);
            cog.ensure_ignited();
            let line = text.join(" ");
            run_intent(&mut cog, "developer", &line)?;
            persist(&cog, &path);
        }
        Commands::Snapshot => {
            let mut cog = load_or_new(&path);
            cog.ensure_ignited();
            println!("{}", serde_json::to_string_pretty(&cog.snapshot())?);
            persist(&cog, &path);
        }
        Commands::Export { out } => cmd_export(&path, &out)?,
        Commands::Shell => cmd_shell(&path)?,
    }
    Ok(())
}
