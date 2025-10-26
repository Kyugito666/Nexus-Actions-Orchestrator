// src/nexus/config.rs - Nexus node configuration

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusConfig {
    pub node_ids: Vec<String>,
    pub wallets: Vec<String>,
}

impl NexusConfig {
    pub fn load_from_files(nodes_file: &Path, wallets_file: &Path) -> Result<Self> {
        let nodes_content = fs::read_to_string(nodes_file)
            .context("Failed to read nodes.txt")?;
        
        let node_ids: Vec<String> = nodes_content
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let wallets_content = fs::read_to_string(wallets_file)
            .context("Failed to read wallets.txt")?;
        
        let wallets: Vec<String> = wallets_content
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if node_ids.is_empty() {
            bail!("No node IDs found in nodes.txt");
        }
        
        if wallets.is_empty() {
            bail!("No wallets found in wallets.txt");
        }
        
        if node_ids.len() != wallets.len() {
            bail!(
                "Node IDs and wallets count mismatch: {} nodes vs {} wallets",
                node_ids.len(),
                wallets.len()
            );
        }
        
        info!("Loaded {} node configurations", node_ids.len());
        
        Ok(Self { node_ids, wallets })
    }
    
    pub fn validate(&self) -> Result<()> {
        for (i, node_id) in self.node_ids.iter().enumerate() {
            if node_id.is_empty() {
                bail!("Node ID at index {} is empty", i);
            }
        }
        
        for (i, wallet) in self.wallets.iter().enumerate() {
            if !wallet.starts_with("0x") {
                bail!("Wallet at index {} does not start with 0x: {}", i, wallet);
            }
            
            if wallet.len() != 42 {
                warn!("Wallet at index {} has unusual length: {}", i, wallet);
            }
        }
        
        info!("All {} nodes validated successfully", self.node_ids.len());
        Ok(())
    }
    
    pub fn generate_matrix_json(&self, max_parallel: usize) -> Result<String> {
        let mut matrix_items = Vec::new();
        
        for (i, (node_id, wallet)) in self.node_ids.iter().zip(self.wallets.iter()).enumerate() {
            matrix_items.push(serde_json::json!({
                "index": i + 1,
                "node_id": node_id,
                "wallet": wallet
            }));
        }
        
        let matrix = serde_json::json!({
            "include": matrix_items
        });
        
        Ok(serde_json::to_string(&matrix)?)
    }
    
    pub fn split_for_github_free_tier(&self) -> Vec<NexusConfig> {
        const MAX_PER_BATCH: usize = 20; // GitHub free tier limit
        
        let mut batches = Vec::new();
        
        for chunk in self.node_ids.chunks(MAX_PER_BATCH) {
            let chunk_size = chunk.len();
            let wallets_chunk = &self.wallets[batches.len() * MAX_PER_BATCH..][..chunk_size];
            
            batches.push(NexusConfig {
                node_ids: chunk.to_vec(),
                wallets: wallets_chunk.to_vec(),
            });
        }
        
        batches
    }
    
    pub fn total_nodes(&self) -> usize {
        self.node_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_nexus_config_load() {
        let mut nodes_file = NamedTempFile::new().unwrap();
        let mut wallets_file = NamedTempFile::new().unwrap();
        
        writeln!(nodes_file, "node1").unwrap();
        writeln!(nodes_file, "node2").unwrap();
        
        writeln!(wallets_file, "0x8254a986319461bf29ae35940a96786e507ad9ac").unwrap();
        writeln!(wallets_file, "0x8254a986319461bf29ae35940a96786e507ad9ac").unwrap();
        
        let config = NexusConfig::load_from_files(nodes_file.path(), wallets_file.path()).unwrap();
        
        assert_eq!(config.node_ids.len(), 2);
        assert_eq!(config.wallets.len(), 2);
    }
    
    #[test]
    fn test_split_batches() {
        let mut node_ids = Vec::new();
        let mut wallets = Vec::new();
        
        for i in 0..45 {
            node_ids.push(format!("node{}", i));
            wallets.push(format!("0x{:040}", i));
        }
        
        let config = NexusConfig { node_ids, wallets };
        let batches = config.split_for_github_free_tier();
        
        assert_eq!(batches.len(), 3); // 20 + 20 + 5
        assert_eq!(batches[0].total_nodes(), 20);
        assert_eq!(batches[1].total_nodes(), 20);
        assert_eq!(batches[2].total_nodes(), 5);
    }
}
