// src/main.rs - Complete entry point
mod core;
mod github;
mod nexus;
mod monitor;
mod orchestration;
mod utils;
mod ui;

use anyhow::Result;
use log::{info, error};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Initialize logger
    let log_dir = PathBuf::from("logs");
    std::fs::create_dir_all(&log_dir).ok();
    
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
    
    // Initialize libsodium
    if unsafe { utils::crypto::init_crypto().is_err() } {
        error!("Failed to initialize libsodium");
        return Err(anyhow::anyhow!("Crypto initialization failed"));
    }
    
    info!("Nexus GitHub Orchestrator v2.0 starting...");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "status" => {
                return core::state::show_status();
            }
            "billing" => {
                return monitor::health::show_billing_all();
            }
            "cleanup" => {
                return github::fork::cleanup_exhausted_forks();
            }
            "rotate" => {
                let rotator = orchestration::Rotator::new(PathBuf::from("config"));
                let rotated = rotator.check_and_rotate()?;
                if rotated {
                    println!("✅ Account rotated successfully");
                } else {
                    println!("ℹ️  No rotation needed");
                }
                return Ok(());
            }
            "version" | "-v" | "--version" => {
                println!("Nexus GitHub Orchestrator v2.0.0");
                return Ok(());
            }
            "help" | "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                print_help();
                return Ok(());
            }
        }
    }
    
    // Start interactive UI
    ui::menu::run_menu()
}

fn print_help() {
    println!("Nexus GitHub Orchestrator v2.0.0");
    println!();
    println!("USAGE:");
    println!("    nexus-orchestrator [COMMAND]");
    println!();
    println!("COMMANDS:");
    println!("    (none)      Start interactive menu");
    println!("    status      Show orchestrator status");
    println!("    billing     Show billing for all accounts");
    println!("    cleanup     Clean up exhausted forks");
    println!("    rotate      Force account rotation");
    println!("    version     Show version");
    println!("    help        Show this help");
}
