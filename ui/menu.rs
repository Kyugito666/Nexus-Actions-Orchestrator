// src/ui/menu.rs - Main menu system (mirip Nexus lama)

use anyhow::Result;
use crate::ui::{display, input};
use std::path::PathBuf;

pub fn run_menu() -> Result<()> {
    loop {
        display::clear_screen();
        display::print_banner();
        
        display::print_menu_item(1, "🔧 Setup & Configuration");
        display::print_menu_item(2, "🚀 Deployment");
        display::print_menu_item(3, "📊 Monitoring");
        display::print_menu_item(4, "🔄 Operations");
        display::print_menu_item(5, "⚙️  Advanced");
        println!();
        display::print_menu_item(0, "🚪 Exit");
        
        display::print_separator();
        
        let choice = input::read_number("\nSelect option: ", 0, 5)?;
        
        let result = match choice {
            0 => {
                display::print_info("Goodbye!");
                return Ok(());
            }
            1 => menu_setup(),
            2 => menu_deployment(),
            3 => menu_monitoring(),
            4 => menu_operations(),
            5 => menu_advanced(),
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            display::print_error(&format!("Error: {}", e));
            display::pause();
        }
    }
}

fn menu_setup() -> Result<()> {
    loop {
        display::clear_screen();
        display::print_section("SETUP & CONFIGURATION");
        
        display::print_submenu_item(1, "Initialize Configuration");
        display::print_submenu_item(2, "Import Tokens");
        display::print_submenu_item(3, "Import Proxies");
        display::print_submenu_item(4, "Import Nodes & Wallets");
        display::print_submenu_item(5, "Validate All");
        println!();
        display::print_submenu_item(0, "← Back");
        
        display::print_separator();
        
        let choice = input::read_number("\nSelect: ", 0, 5)?;
        
        match choice {
            0 => return Ok(()),
            1 => {
                // Initialize config - implemented in next batch
                display::print_info("Feature: Initialize Configuration");
                display::pause();
            }
            2 => {
                // Import tokens
                display::print_info("Feature: Import Tokens");
                display::pause();
            }
            3 => {
                // Import proxies
                display::print_info("Feature: Import Proxies");
                display::pause();
            }
            4 => {
                // Import nodes
                display::print_info("Feature: Import Nodes & Wallets");
                display::pause();
            }
            5 => {
                // Validate all
                validate_all_command()?;
            }
            _ => {}
        }
    }
}

fn menu_deployment() -> Result<()> {
    loop {
        display::clear_screen();
        display::print_section("DEPLOYMENT");
        
        display::print_submenu_item(1, "Deploy Main Workflow");
        display::print_submenu_item(2, "Create Fork Chain");
        display::print_submenu_item(3, "Set Secrets");
        display::print_submenu_item(4, "Trigger Workflow");
        println!();
        display::print_submenu_item(0, "← Back");
        
        display::print_separator();
        
        let choice = input::read_number("\nSelect: ", 0, 4)?;
        
        match choice {
            0 => return Ok(()),
            _ => {
                display::print_info("Feature under development");
                display::pause();
            }
        }
    }
}

fn menu_monitoring() -> Result<()> {
    loop {
        display::clear_screen();
        display::print_section("MONITORING");
        
        display::print_submenu_item(1, "Show Billing Status");
        display::print_submenu_item(2, "Show Workflow Status");
        display::print_submenu_item(3, "Show Fork Chain");
        display::print_submenu_item(4, "View Logs");
        println!();
        display::print_submenu_item(0, "← Back");
        
        display::print_separator();
        
        let choice = input::read_number("\nSelect: ", 0, 4)?;
        
        match choice {
            0 => return Ok(()),
            1 => {
                crate::monitor::health::show_billing_all()?;
                display::pause();
            }
            3 => {
                crate::core::state::show_status()?;
                display::pause();
            }
            _ => {
                display::print_info("Feature under development");
                display::pause();
            }
        }
    }
}

fn menu_operations() -> Result<()> {
    loop {
        display::clear_screen();
        display::print_section("OPERATIONS");
        
        display::print_submenu_item(1, "Force Account Switch");
        display::print_submenu_item(2, "Manual Fork Creation");
        display::print_submenu_item(3, "Disable All Workflows");
        display::print_submenu_item(4, "Cleanup Exhausted Forks");
        println!();
        display::print_submenu_item(0, "← Back");
        
        display::print_separator();
        
        let choice = input::read_number("\nSelect: ", 0, 4)?;
        
        match choice {
            0 => return Ok(()),
            4 => {
                if input::read_yes_no("Delete all exhausted forks?") {
                    crate::github::fork::cleanup_exhausted_forks()?;
                    display::print_success("Cleanup complete");
                }
                display::pause();
            }
            _ => {
                display::print_info("Feature under development");
                display::pause();
            }
        }
    }
}

fn menu_advanced() -> Result<()> {
    loop {
        display::clear_screen();
        display::print_section("ADVANCED");
        
        display::print_submenu_item(1, "Edit Thresholds");
        display::print_submenu_item(2, "Test Proxy Connections");
        display::print_submenu_item(3, "Export Reports");
        display::print_submenu_item(4, "Reset All Cache");
        println!();
        display::print_submenu_item(0, "← Back");
        
        display::print_separator();
        
        let choice = input::read_number("\nSelect: ", 0, 4)?;
        
        match choice {
            0 => return Ok(()),
            2 => {
                test_proxies_command()?;
            }
            _ => {
                display::print_info("Feature under development");
                display::pause();
            }
        }
    }
}

fn validate_all_command() -> Result<()> {
    use crate::core::{account::AccountManager, proxy::ProxyManager};
    use crate::nexus::config::NexusConfig;
    
    let config_dir = PathBuf::from("config");
    let cache_dir = config_dir.join("cache");
    
    display::print_info("Validating tokens...");
    
    let mut account_mgr = AccountManager::new(&cache_dir);
    account_mgr.load_tokens(&config_dir.join("tokens.txt"))?;
    
    let mut proxy_mgr = ProxyManager::new(&cache_dir);
    
    let tokens: Vec<String> = std::fs::read_to_string(config_dir.join("tokens.txt"))?
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    proxy_mgr.load_from_file(&config_dir.join("proxies.txt"), &tokens)?;
    account_mgr.validate_all(&proxy_mgr)?;
    
    display::print_success("Tokens validated");
    
    display::print_info("Validating nodes and wallets...");
    
    let nexus_config = NexusConfig::load_from_files(
        &config_dir.join("nodes.txt"),
        &config_dir.join("wallets.txt")
    )?;
    
    nexus_config.validate()?;
    
    display::print_success("Nodes and wallets validated");
    display::print_success(&format!("Total: {} nodes", nexus_config.total_nodes()));
    
    display::pause();
    Ok(())
}

fn test_proxies_command() -> Result<()> {
    use crate::core::proxy::ProxyManager;
    
    let config_dir = PathBuf::from("config");
    let cache_dir = config_dir.join("cache");
    
    let mut proxy_mgr = ProxyManager::new(&cache_dir);
    proxy_mgr.load_cache(
