// src/github/fork.rs - Fork chain management

use anyhow::{Result, Context, bail};
use log::{info, warn, debug};
use std::thread;
use std::time::Duration;
use crate::core::state::{StateManager, ForkChainNode, ForkStatus, OrchestratorState};
use crate::core::account::AccountInfo;
use crate::github::api::GitHubClient;

pub struct ForkManager {
    state_manager: StateManager,
}

impl ForkManager {
    pub fn new(state_manager: StateManager) -> Self {
        Self { state_manager }
    }
    
    pub fn create_fork_chain(
        &self,
        state: OrchestratorState,
        account: &AccountInfo,
        parent_repo: &str,
        proxy: Option<String>,
    ) -> Result<(OrchestratorState, String)> {
        let client = GitHubClient::new(account.token.clone(), proxy);
        
        info!("Creating fork for @{} from {}", account.username, parent_repo);
        
        // Check if fork already exists
        let expected_fork = format!("{}/{}", account.username, parent_repo.split('/').nth(1).unwrap());
        
        if client.check_repo_exists(&expected_fork)? {
            info!("Fork already exists: {}", expected_fork);
            
            // Check if it's in our chain
            if state.fork_chain.iter().any(|n| n.repo == expected_fork) {
                return Ok((state, expected_fork));
            }
        } else {
            // Create new fork
            let fork_name = client.create_fork(parent_repo)?;
            info!("Fork created: {}", fork_name);
            
            // Wait for fork to be ready
            self.wait_for_fork_ready(&client, &fork_name)?;
        }
        
        // Add to chain
        let node = ForkChainNode {
            pat_index: account.index,
            username: account.username.clone(),
            repo: expected_fork.clone(),
            parent: Some(parent_repo.to_string()),
            billing_used: 0.0,
            status: ForkStatus::Active,
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        };
        
        let new_state = self.state_manager.add_fork_node(state, node)?;
        
        Ok((new_state, expected_fork))
    }
    
    fn wait_for_fork_ready(&self, client: &GitHubClient, fork_repo: &str) -> Result<()> {
        info!("Waiting for fork to be ready: {}", fork_repo);
        
        let max_attempts = 24; // 2 minutes total (24 * 5s)
        let mut attempts = 0;
        
        while attempts < max_attempts {
            thread::sleep(Duration::from_secs(5));
            
            match client.check_repo_exists(fork_repo) {
                Ok(true) => {
                    info!("Fork is ready: {}", fork_repo);
                    return Ok(());
                }
                Ok(false) => {
                    debug!("Fork not ready yet, attempt {}/{}", attempts + 1, max_attempts);
                }
                Err(e) => {
                    warn!("Error checking fork: {}", e);
                }
            }
            
            attempts += 1;
        }
        
        bail!("Timeout waiting for fork to be ready: {}", fork_repo)
    }
    
    pub fn disable_fork_workflow(
        &self,
        repo: &str,
        workflow_file: &str,
        client: &GitHubClient,
    ) -> Result<()> {
        info!("Disabling workflow in fork: {}", repo);
        
        match client.get_workflow_id(repo, workflow_file)? {
            Some(workflow_id) => {
                client.disable_workflow(repo, workflow_id)?;
                info!("Workflow disabled successfully");
                Ok(())
            }
            None => {
                warn!("Workflow not found in {}, skipping disable", repo);
                Ok(())
            }
        }
    }
    
    pub fn delete_fork(
        &self,
        mut state: OrchestratorState,
        fork_index: usize,
        client: &GitHubClient,
    ) -> Result<OrchestratorState> {
        if let Some(node) = state.fork_chain.get(fork_index) {
            let repo = &node.repo;
            
            info!("Deleting fork: {}", repo);
            
            // First disable workflow
            match client.get_workflow_id(repo, "nexus.yml") {
                Ok(Some(workflow_id)) => {
                    client.disable_workflow(repo, workflow_id).ok();
                }
                _ => {}
            }
            
            thread::sleep(Duration::from_secs(3));
            
            // Delete repository
            client.delete_repo(repo)?;
            
            info!("Fork deleted: {}", repo);
            
            // Update state
            state = self.state_manager.update_fork_status(state, fork_index, ForkStatus::Disabled)?;
        }
        
        Ok(state)
    }
    
    pub fn get_next_parent_repo(&self, state: &OrchestratorState) -> Option<String> {
        // Find the last active or exhausted fork to use as parent
        state.fork_chain
            .iter()
            .rev()
            .find(|n| n.status == ForkStatus::Active || n.status == ForkStatus::Exhausted)
            .map(|n| n.repo.clone())
    }
}

pub fn cleanup_exhausted_forks() -> Result<()> {
    info!("Starting cleanup of exhausted forks...");
    
    let config_dir = std::path::PathBuf::from("config");
    let state_mgr = StateManager::new(&config_dir)?;
    let mut state = state_mgr.load_state()?;
    
    let exhausted_forks: Vec<_> = state.fork_chain
        .iter()
        .enumerate()
        .filter(|(_, n)| n.status == ForkStatus::Exhausted)
        .map(|(i, n)| (i, n.clone()))
        .collect();
    
    if exhausted_forks.is_empty() {
        info!("No exhausted forks to clean up");
        return Ok(());
    }
    
    info!("Found {} exhausted forks to delete", exhausted_forks.len());
    
    // Load tokens to get credentials
    let tokens_file = config_dir.join("tokens.txt");
    let mut account_mgr = crate::core::account::AccountManager::new(&config_dir.join("cache"));
    account_mgr.load_tokens(&tokens_file)?;
    
    let fork_mgr = ForkManager::new(state_mgr);
    
    for (index, node) in exhausted_forks {
        if let Some(account) = account_mgr.get_account(node.pat_index) {
            info!("Deleting fork: {} (index {})", node.repo, index);
            
            let client = GitHubClient::new(account.token.clone(), None);
            
            match fork_mgr.delete_fork(state.clone(), index, &client) {
                Ok(new_state) => {
                    state = new_state;
                    info!("Successfully deleted: {}", node.repo);
                }
                Err(e) => {
                    warn!("Failed to delete {}: {}", node.repo, e);
                }
            }
            
            thread::sleep(Duration::from_secs(2));
        }
    }
    
    info!("Cleanup complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fork_manager_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state_mgr = StateManager::new(temp_dir.path()).unwrap();
        let fork_mgr = ForkManager::new(state_mgr);
        // Just test construction
    }
}
