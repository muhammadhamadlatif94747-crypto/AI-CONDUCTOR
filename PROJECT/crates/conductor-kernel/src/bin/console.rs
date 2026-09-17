//! Runnable entry point for the Developer Reliability Console
//! (P1-W11-T01). Thin CLI wrapper over `conductor_kernel::console` — all
//! logic lives in the library module so it's independently testable; this
//! file only parses argv and prints.
//!
//! Usage:
//!   conductor-console <store-dir> snapshot [--chain-id ID] [--tail N]
//!   conductor-console <store-dir> cancel --mission M [--step S] [--attempt A] --from STATUS
//!
//! `<store-dir>` must contain (or will create, for cancel) events.jsonl,
//! anchors.jsonl, outbox.json, sequence.json — the same real stores every
//! other part of the kernel reads and writes.

use conductor_kernel::console::{render, request_attempt_cancellation, snapshot};
use conductor_kernel::correlation::CorrelationContext;
use conductor_kernel::state::AttemptStatus;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(store_dir) = args.first() else {
        eprintln!("usage: conductor-console <store-dir> <snapshot|cancel> [options]");
        return ExitCode::FAILURE;
    };
    let store_dir = PathBuf::from(store_dir);
    let Some(command) = args.get(1) else {
        eprintln!("usage: conductor-console <store-dir> <snapshot|cancel> [options]");
        return ExitCode::FAILURE;
    };

    // Discovered during takeover audit: this module's own doc comment
    // above promises the store directory "will create" on cancel, but
    // EventLog::open only creates the FILE, not missing parent
    // directories -- so a genuinely fresh store-dir failed with ENOENT
    // on both snapshot and cancel. Every existing unit test pre-created
    // its tempdir, so this gap wasn't exercised until an independent
    // hands-on run against a real, never-before-seen path. Fixed at the
    // narrowest correct point: the CLI process boundary, not the
    // library (snapshot() is read-only by contract and must not gain a
    // side effect of creating a directory just by being queried; a
    // fresh, empty snapshot for a genuinely empty store is already
    // correctly handled once the directory exists).
    if let Err(e) = std::fs::create_dir_all(&store_dir) {
        eprintln!("could not create store directory {}: {e}", store_dir.display());
        return ExitCode::FAILURE;
    }

    match command.as_str() {
        "snapshot" => {
            let chain_id = flag_value(&args, "--chain-id").unwrap_or_else(|| "default".into());
            let tail_n: usize = flag_value(&args, "--tail")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            match snapshot(&store_dir, &chain_id, tail_n) {
                Ok(snap) => {
                    print!("{}", render(&snap));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("snapshot failed: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "cancel" => {
            let Some(mission) = flag_value(&args, "--mission") else {
                eprintln!("cancel requires --mission M");
                return ExitCode::FAILURE;
            };
            let Some(from_str) = flag_value(&args, "--from") else {
                eprintln!("cancel requires --from <Pending|Running|Unknown|...>");
                return ExitCode::FAILURE;
            };
            let Some(from) = parse_attempt_status(&from_str) else {
                eprintln!("unrecognized --from status: {from_str}");
                return ExitCode::FAILURE;
            };

            let mut ctx = CorrelationContext::for_mission(mission);
            if let Some(step) = flag_value(&args, "--step") {
                ctx = ctx.with_step(step);
            }
            if let Some(attempt) = flag_value(&args, "--attempt") {
                match ctx.with_attempt(attempt) {
                    Ok(c) => ctx = c,
                    Err(e) => {
                        eprintln!("invalid --attempt: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }

            // AC-12: this binary is the one legitimate place a real
            // wall-clock read belongs -- it's the process boundary, not the
            // deterministic kernel core. The library itself never reads
            // the clock directly (see lib.rs doc comment).
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            match request_attempt_cancellation(&store_dir, ctx, from, now_ms) {
                Ok(Ok(())) => {
                    println!("cancellation request AUTHORIZED and recorded.");
                    ExitCode::SUCCESS
                }
                Ok(Err(reason)) => {
                    println!("cancellation request DENIED: {reason}");
                    println!("(denial recorded as a real event -- not silently dropped)");
                    ExitCode::FAILURE
                }
                Err(e) => {
                    eprintln!("cancel failed: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        other => {
            eprintln!("unknown command '{other}'; expected 'snapshot' or 'cancel'");
            ExitCode::FAILURE
        }
    }
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn parse_attempt_status(s: &str) -> Option<AttemptStatus> {
    match s {
        "Pending" => Some(AttemptStatus::Pending),
        "Running" => Some(AttemptStatus::Running),
        "Succeeded" => Some(AttemptStatus::Succeeded),
        "Failed" => Some(AttemptStatus::Failed),
        "Cancelled" => Some(AttemptStatus::Cancelled),
        "Unknown" => Some(AttemptStatus::Unknown),
        _ => None,
    }
}
