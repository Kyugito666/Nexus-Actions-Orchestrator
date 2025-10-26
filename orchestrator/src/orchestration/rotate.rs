// Update imports at top of src/orchestration/rotate.rs
use anyhow::{Result, Context};
use std::path::PathBuf;
use log::{info, warn};
use crate::core::{account, billing, proxy, state, StateManager};
use crate::github::{fork, GitHubClient};

pub struct Rotator {
    config_dir: PathBuf,
}

impl Rotator {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }
    
    pub fn check_and_rotate(&self) -> Result<bool> {
        let state_mgr = StateManager::new(&self.config_dir)?;
        let mut state = state_mgr.load_state()?;
        
        let active_fork = match state_mgr.get_active_fork(&state) {
            Some(f) => f,
            None => return Ok(false),
        };
        
        let account = self.load_account(active_fork.pat_index)?;
        let proxy = self.load_proxy(&account.token)?;
        
        let billing_mon = billing::BillingMonitor::default();
        let billing = billing_mon.check_billing(&account.username, &account.token, proxy.as_deref())?;
        
        if billing.is_exhausted {
            info!("Account {} exhausted, rotating", account.username);
            
            let client = GitHubClient::new(account.token.clone(), proxy);
            let fork_mgr = fork::ForkManager::new(state_mgr.clone());
            
            fork_mgr.disable_fork_workflow(&active_fork.repo, "nexus.yml", &client)?;
            
            std::thread::sleep(std::time::Duration::from_secs(5));
            
            state = state_mgr.update_fork_status(state, active_fork.pat_index, state::ForkStatus::Exhausted)?;
            
            let next_index = (active_fork.pat_index + 1) % state.total_accounts;
            state.current_active_index = next_index;
            state_mgr.save_state(&state)?;
            
            info!("Rotated to account index {}", next_index);
            return Ok(true);
        }
        
        Ok(false)
    }
    
    fn load_account(&self, index: usize) -> Result<account::AccountInfo> {
        let mut mgr = account::AccountManager::new(&self.config_dir.join("cache"));
        mgr.load_tokens(&self.config_dir.join("tokens.txt"))?;
        mgr.get_account(index).cloned().ok_or_else(|| anyhow::anyhow!("Account not found"))
    }
    
    fn load_proxy(&self, token: &str) -> Result<Option<String>> {
        let mut proxy_mgr = proxy::ProxyManager::new(&self.config_dir.join("cache"));
        proxy_mgr.load_cache().ok();
        Ok(proxy_mgr.get_proxy(token).map(|p| p.to_curl_format()))
    }
}
