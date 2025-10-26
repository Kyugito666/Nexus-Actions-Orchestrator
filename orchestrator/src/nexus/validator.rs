// src/nexus/validator.rs - Validate Nexus credentials

use anyhow::{Result, bail};
use log::{info, warn};

pub struct NexusValidator;

impl NexusValidator {
    pub fn validate_node_id(node_id: &str) -> Result<()> {
        if node_id.is_empty() {
            bail!("Node ID cannot be empty");
        }
        
        if node_id.len() < 5 {
            bail!("Node ID too short: {}", node_id);
        }
        
        Ok(())
    }
    
    pub fn validate_wallet(wallet: &str) -> Result<()> {
        if !wallet.starts_with("0x") {
            bail!("Wallet must start with 0x: {}", wallet);
        }
        
        if wallet.len() != 42 {
            bail!("Invalid wallet length (expected 42 chars): {}", wallet);
        }
        
        // Check hex characters
        let hex_part = &wallet[2..];
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("Wallet contains invalid hex characters: {}", wallet);
        }
        
        Ok(())
    }
    
    pub fn validate_all(node_ids: &[String], wallets: &[String]) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        
        for (i, node_id) in node_ids.iter().enumerate() {
            if let Err(e) = Self::validate_node_id(node_id) {
                errors.push(format!("Node {} (index {}): {}", node_id, i, e));
            }
        }
        
        for (i, wallet) in wallets.iter().enumerate() {
            if let Err(e) = Self::validate_wallet(wallet) {
                errors.push(format!("Wallet {} (index {}): {}", wallet, i, e));
            }
        }
        
        if !errors.is_empty() {
            for err in &errors {
                warn!("{}", err);
            }
            return Err(anyhow::anyhow!("Validation failed with {} errors", errors.len()));
        }
        
        info!("All {} nodes and wallets validated successfully", node_ids.len());
        Ok(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_wallet() {
        let wallet = "0x1234567890123456789012345678901234567890";
        assert!(NexusValidator::validate_wallet(wallet).is_ok());
    }
    
    #[test]
    fn test_invalid_wallet_no_prefix() {
        let wallet = "1234567890123456789012345678901234567890";
        assert!(NexusValidator::validate_wallet(wallet).is_err());
    }
    
    #[test]
    fn test_invalid_wallet_length() {
        let wallet = "0x123456";
        assert!(NexusValidator::validate_wallet(wallet).is_err());
    }
    
    #[test]
    fn test_valid_node_id() {
        assert!(NexusValidator::validate_node_id("node_abc123").is_ok());
    }
    
    #[test]
    fn test_invalid_node_id_empty() {
        assert!(NexusValidator::validate_node_id("").is_err());
    }
}
