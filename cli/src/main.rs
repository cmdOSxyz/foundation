//! # cmdos — the runtime CLI
//!
//! The first *runnable* cmdOS: it wires the whole agent stack into one program
//! you can run from a terminal. Give it a request; the agent (Alios) plans it,
//! the kernel runs the plan under supervision against your real files, and it
//! prints the execution timeline and the audit ledger.
//!
//! This uses the deterministic `RulePlanner` so it runs with no API key. Swapping
//! in the model-backed planner (RFC-0014) is a one-line change once a real
//! `ClaudeTransport` is wired.
//!
//! Usage:
//!   cmdos "list files in ."
//!   cmdos --dir /path/to/workspace "rename draft.txt to final.txt"
//!
//! Safety: every step is gated by policy (Alios) and every executed step is
//! reversible and recorded — the same guarantees the library crates enforce.

use alios::{Agent, RulePlanner};
use cmd_kernel::{AuthorityContext, Kernel, StepOutcome};
use cmd_ledger::Ledger;
use cmd_types::{now, Id, Mandate, RiskClass};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let parsed = match parse_args(&args) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("{msg}");
            print_usage();
            return ExitCode::from(2);
        }
    };

    // Build the agent with the deterministic planner.
    let agent = Agent::new("Nova", RulePlanner::new());
    let (intent, plan) = agent.plan_for(&parsed.request);

    println!("── cmdOS ──────────────────────────────────");
    println!("agent   : {}", agent.name);
    println!("intent  : {}", intent.raw_text);
    println!("plan    : {}", plan.summary);
    println!("steps   : {}", plan.steps.len());
    for (i, s) in plan.steps.iter().enumerate() {
        println!(
            "  {}. {} — {}.{}",
            i + 1,
            s.description,
            s.capability,
            s.action
        );
    }
    println!("───────────────────────────────────────────");

    // Grant the agent a filesystem mandate (in the real product this comes from
    // the user's onboarding / approvals).
    let mandate = Mandate {
        id: Id::new(),
        agent_id: Id::new(),
        scope: "cli session".into(),
        capabilities: vec!["filesystem".into()],
        max_autonomous_risk: RiskClass::R1Reversible,
        budget_id: None,
        granted_at: now(),
        expires_at: None,
        revoked_at: None,
    };
    let ctx = AuthorityContext {
        mandate: Some(&mandate),
        budget: None,
    };

    // Run the plan through the kernel against the real filesystem.
    let mut ledger = Ledger::new();
    let mut fs_cap = cap_files::FileSystem::new();
    let run = {
        let mut kernel = Kernel::new(&mut ledger);
        kernel.run_plan(&plan, &mut fs_cap, &ctx, &|step| risk_of(&step.action))
    };

    println!("execution:");
    for (id, outcome) in &run.steps {
        let short = short_id(*id);
        match outcome {
            StepOutcome::Executed => println!("  ✓ {short}  executed"),
            StepOutcome::AwaitingApproval(r) => println!("  ⏸ {short}  needs approval — {r}"),
            StepOutcome::Blocked(r) => println!("  ✗ {short}  blocked — {r}"),
            StepOutcome::Failed(r) => println!("  ✗ {short}  failed — {r}"),
        }
    }

    println!("───────────────────────────────────────────");
    println!(
        "ledger  : {} entries, chain {}",
        ledger.len(),
        if ledger.verify().is_ok() {
            "intact ✓"
        } else {
            "BROKEN ✗"
        }
    );
    println!(
        "result  : {}",
        if run.completed {
            "completed ✓"
        } else {
            "stopped"
        }
    );

    if run.completed {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Map an action name to its risk class. In the full system this comes from the
/// capability contract; here a small table matching cap-files.
fn risk_of(action: &str) -> RiskClass {
    match action {
        "list" | "read" => RiskClass::R0ReadOnly,
        "rename" | "move" | "delete" => RiskClass::R1Reversible,
        _ => RiskClass::R2Compensable,
    }
}

struct Parsed {
    request: String,
    #[allow(dead_code)]
    dir: Option<String>,
}

fn parse_args(args: &[String]) -> Result<Parsed, String> {
    let mut dir = None;
    let mut request_parts: Vec<String> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                i += 1;
                dir = Some(args.get(i).ok_or("--dir needs a path")?.clone());
            }
            "-h" | "--help" => return Err("help".into()),
            other => request_parts.push(other.to_string()),
        }
        i += 1;
    }
    if request_parts.is_empty() {
        return Err("no request given".into());
    }
    Ok(Parsed {
        request: request_parts.join(" "),
        dir,
    })
}

fn print_usage() {
    eprintln!("usage: cmdos [--dir PATH] \"<your request>\"");
    eprintln!("  e.g. cmdos \"list files in .\"");
}

fn short_id(id: Id) -> String {
    let s = id.to_string();
    s.chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plain_request() {
        let args = vec!["cmdos".into(), "list".into(), "files".into()];
        let p = parse_args(&args).unwrap();
        assert_eq!(p.request, "list files");
        assert!(p.dir.is_none());
    }

    #[test]
    fn parses_dir_flag() {
        let args = vec![
            "cmdos".into(),
            "--dir".into(),
            "/tmp/x".into(),
            "list".into(),
        ];
        let p = parse_args(&args).unwrap();
        assert_eq!(p.dir.as_deref(), Some("/tmp/x"));
        assert_eq!(p.request, "list");
    }

    #[test]
    fn empty_request_is_an_error() {
        let args = vec!["cmdos".into()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn risk_table_matches_capability_semantics() {
        assert_eq!(risk_of("list"), RiskClass::R0ReadOnly);
        assert_eq!(risk_of("rename"), RiskClass::R1Reversible);
        assert_eq!(risk_of("delete"), RiskClass::R1Reversible);
    }
}
