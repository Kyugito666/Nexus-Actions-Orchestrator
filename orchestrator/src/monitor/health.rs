// src/monitor/health.rs - Health monitoring for workflows

use anyhow::Result;
use std::path::PathBuf;
use log::info;
use crate::core::{
    state::StateManager,
    account::AccountManager,
    billing::BillingMonitor,
    proxy::ProxyManager,
};

pub struct HealthMonitor {
    state_manager: StateManager,
    billing_monitor: BillingMonitor,
}

impl HealthMonitor {
    pub fn new(config_dir: &PathBuf) -> Result<Self> {
        let state_manager = StateManager::new(config_dir)?;
        let billing_monitor = BillingMonitor::default();
        
        Ok(Self {
            state_manager,
            billing_monitor,
        })
    }
    
    pub fn check_all_accounts(
        &self,
        accounts: &[crate::core::account::AccountInfo],
        proxy_manager: &ProxyManager,
    ) -> Result<Vec<crate::core::billing::BillingInfo>> {
        let mut billing_infos = Vec::new();
        
        for account in accounts {
            let proxy = proxy_manager.get_proxy(&account.token)
                .map(|p| p.to_curl_format());
            
            match self.billing_monitor.check_billing(
                &account.username,
                &account.token,
                proxy.as_deref(),
            ) {
                Ok(info) => {
                    self.billing_monitor.display_billing(&info);
                    billing_infos.push(info);
                }
                Err(e) => {
                    eprintln!("Failed to check billing for {}: {}", account.username, e);
                }
            }
            
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        
        Ok(billing_infos)
    }
}

pub fn show_billing_all() -> Result<()> {
    let config_dir = PathBuf::from("config");
    
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║          BILLING STATUS - ALL ACCOUNTS               ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");
    
    let tokens_file = config_dir.join("tokens.txt");
    let proxies_file = config_dir.join("proxies.txt");
    let cache_dir = config_dir.join("cache");
    
    let mut account_mgr = AccountManager::new(&cache_dir);
    account_mgr.load_tokens(&tokens_file)?;
    
    let mut proxy_mgr = ProxyManager::new(&cache_dir);
    
    if proxies_file.exists() {
        let tokens: Vec<String> = std::fs::read_to_string(&tokens_file)?
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        proxy_mgr.load_from_file(&proxies_file, &tokens)?;
    }
    
    let health_monitor = HealthMonitor::new(&config_dir)?;
    
    let billing_infos = health_monitor.check_all_accounts(
        account_mgr.get_all_accounts(),
        &proxy_mgr,
    )?;
    
    println!("\n─────────────────────────────────────────────────────────");
    println!("Summary:");
    
    let total = billing_infos.len();
    let exhausted = billing_infos.iter().filter(|b| b.is_exhausted).count();
    let warning = billing_infos.iter().filter(|b| b.is_warning && !b.is_exhausted).count();
    let ok = total - exhausted - warning;
    
    println!("  Total Accounts: {}", total);
    println!("  🟢 OK: {}", ok);
    println!("  🟡 Warning: {}", warning);
    println!("  🔴 Exhausted: {}", exhausted);
    println!("─────────────────────────────────────────────────────────\n");
    
    Ok(())
}
