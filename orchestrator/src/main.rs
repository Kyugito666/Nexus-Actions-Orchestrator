// src/main.rs - Entry point

mod core;
mod github;
mod nexus;
mod monitor;
mod utils;
mod ui;
mod orchestration;

use orchestration::{Deployer, Rotator};
use anyhow::Result;
use log::{info, error};
use std::env;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
    
    // Initialize libsodium
    unsafe {
        let init_result = utils::crypto::crypto_init();
        if init_result < 0 {
            error!("Failed to initialize libsodium");
            return Err(anyhow::anyhow!("Crypto initialization failed"));
        }
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
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Available: status, billing, cleanup");
                return Ok(());
            }
        }
    }
    
    // Start interactive UI
    ui::menu::run_menu()
}
