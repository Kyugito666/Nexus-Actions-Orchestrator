// src/core/state.rs - State management and persistence

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkChainNode {
    pub pat_index: usize,
    pub username: String,
    pub repo: String,
    pub parent: Option<String>,
    pub billing_used: f32,
    pub status: ForkStatus,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForkStatus {
    Active,
    Exhausted,
    Disabled,
    Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorState {
    pub fork_chain: Vec<ForkChainNode>,
    pub current_active_index: usize,
    pub total_accounts: usize,
    pub last_rotation: Option<DateTime<Utc>>,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            fork_chain: Vec::new(),
            current_active_index: 0,
            total_accounts: 0,
            last_rotation: None,
        }
    }
}

pub struct StateManager {
    cache_dir: PathBuf,
    state_file: PathBuf,
}

impl StateManager {
    pub fn new(config_dir: &Path) -> Result<Self> {
        let cache_dir = config_dir.join("cache");
        fs::create_dir_all(&cache_dir)?;
        
        let state_file = cache_dir.join("active.json");
        
        Ok(Self {
            cache_dir,
            state_file,
        })
    }
    
    pub fn load_state(&self) -> Result<OrchestratorState> {
        if !self.state_file.exists() {
            info!("State file not found, using default state");
            return Ok(OrchestratorState::default());
        }
        
        let content = fs::read_to_string(&self.state_file)
            .context("Failed to read state file")?;
        
        let state: OrchestratorState = serde_json::from_str(&content)
            .context("Failed to parse state JSON")?;
        
        info!("Loaded state: {} accounts in chain", state.fork_chain.len());
        Ok(state)
    }
    
    pub fn save_state(&self, state: &OrchestratorState) -> Result<()> {
        let json = serde_json::to_string_pretty(state)
            .context("Failed to serialize state")?;
        
        // Write to temp file first
        let temp_file = self.state_file.with_extension("tmp");
        fs::write(&temp_file, json)
            .context("Failed to write temp state file")?;
        
        // Atomic rename
        fs::rename(&temp_file, &self.state_file)
            .context("Failed to rename state file")?;
        
        info!("State saved successfully");
        Ok(())
    }
    
    pub fn add_fork_node(&self, mut state: OrchestratorState, node: ForkChainNode) -> Result<OrchestratorState> {
        state.fork_chain.push(node);
        state.last_rotation = Some(Utc::now());
        self.save_state(&state)?;
        Ok(state)
    }
    
    pub fn update_fork_status(
        &self,
        mut state: OrchestratorState,
        index: usize,
        status: ForkStatus,
    ) -> Result<OrchestratorState> {
        if let Some(node) = state.fork_chain.get_mut(index) {
            node.status = status;
            node.last_updated = Utc::now();
            self.save_state(&state)?;
        }
        Ok(state)
    }
    
    pub fn get_active_fork(&self, state: &OrchestratorState) -> Option<&ForkChainNode> {
        state.fork_chain.iter().find(|n| n.status == ForkStatus::Active)
    }
    
    pub fn get_cache_file(&self, filename: &str) -> PathBuf {
        self.cache_dir.join(filename)
    }
}

pub fn show_status() -> Result<()> {
    let config_dir = PathBuf::from("config");
    let state_mgr = StateManager::new(&config_dir)?;
    let state = state_mgr.load_state()?;
    
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║          ORCHESTRATOR STATUS                          ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");
    
    println!("Total Accounts: {}", state.total_accounts);
    println!("Fork Chain Length: {}", state.fork_chain.len());
    println!("Current Active Index: {}", state.current_active_index);
    
    if let Some(last_rotation) = state.last_rotation {
        println!("Last Rotation: {}", last_rotation.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    println!("\nFork Chain:");
    println!("─────────────────────────────────────────────────────────");
    
    for (i, node) in state.fork_chain.iter().enumerate() {
        let status_icon = match node.status {
            ForkStatus::Active => "🟢",
            ForkStatus::Exhausted => "🔴",
            ForkStatus::Disabled => "⚪",
            ForkStatus::Source => "🔵",
        };
        
        println!(
            "{} [{:2}] @{:<20} | {} | Billing: {:.1}/120.0",
            status_icon,
            i,
            node.username,
            node.repo,
            node.billing_used
        );
        
        if let Some(parent) = &node.parent {
            println!("       └─ Forked from: {}", parent);
        }
    }
    
    println!("─────────────────────────────────────────────────────────\n");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_state_persistence() {
        let temp_dir = tempdir().unwrap();
        let state_mgr = StateManager::new(temp_dir.path()).unwrap();
        
        let mut state = OrchestratorState::default();
        state.total_accounts = 5;
        
        state_mgr.save_state(&state).unwrap();
        
        let loaded = state_mgr.load_state().unwrap();
        assert_eq!(loaded.total_accounts, 5);
    }
}
