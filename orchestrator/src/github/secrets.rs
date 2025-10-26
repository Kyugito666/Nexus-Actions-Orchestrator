// src/github/secrets.rs - Secrets management (ported from Datagram)

use anyhow::{Result, Context, bail};
use log::{info, debug, warn};
use std::thread;
use std::time::Duration;
use crate::github::api::GitHubClient;
use crate::utils::crypto::encrypt_for_github;

pub struct SecretsManager {
    client: GitHubClient,
}

impl SecretsManager {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }
    
    fn get_repo_public_key(&self, repo: &str) -> Result<(String, String)> {
        debug!("Getting public key for {}", repo);
        
        let response = self.client.api_call(
            &format!("repos/{}/actions/secrets/public-key", repo),
            "GET"
        )?;
        
        let json: serde_json::Value = serde_json::from_str(&response)
            .context("Failed to parse public key response")?;
        
        let key = json["key"]
            .as_str()
            .context("Public key not found")?
            .to_string();
        
        let key_id = json["key_id"]
            .as_str()
            .context("Key ID not found")?
            .to_string();
        
        Ok((key, key_id))
    }
    
    pub fn set_secret(&self, repo: &str, secret_name: &str, secret_value: &str) -> Result<()> {
        info!("Setting secret {} in {}", secret_name, repo);
        
        // Get public key
        let (public_key, key_id) = self.get_repo_public_key(repo)?;
        
        // Encrypt value
        let encrypted_value = encrypt_for_github(&public_key, secret_value)
            .context("Failed to encrypt secret value")?;
        
        // Prepare payload
        let payload = serde_json::json!({
            "encrypted_value": encrypted_value,
            "key_id": key_id
        });
        
        // Set secret
        self.client.api_call_with_data(
            &format!("repos/{}/actions/secrets/{}", repo, secret_name),
            "PUT",
            &payload.to_string()
        )?;
        
        info!("Secret {} set successfully", secret_name);
        
        // Verify secret was set
        thread::sleep(Duration::from_secs(2));
        
        match self.client.api_call(
            &format!("repos/{}/actions/secrets/{}", repo, secret_name),
            "GET"
        ) {
            Ok(_) => {
                info!("Secret {} verified", secret_name);
                Ok(())
            }
            Err(e) => {
                warn!("Secret verification failed: {}", e);
                Ok(()) // Don't fail if verification fails
            }
        }
    }
    
    pub fn set_nexus_secrets(
        &self,
        repo: &str,
        node_ids: &[String],
        wallets: &[String],
    ) -> Result<()> {
        if node_ids.len() != wallets.len() {
            bail!("Node IDs and wallets count mismatch: {} vs {}", node_ids.len(), wallets.len());
        }
        
        info!("Setting Nexus secrets for {} nodes", node_ids.len());
        
        // Create newline-separated strings
        let node_ids_str = node_ids.join("\n");
        let wallets_str = wallets.join("\n");
        
        // Set NEXUS_NODE_IDS
        self.set_secret(repo, "NEXUS_NODE_IDS", &node_ids_str)
            .context("Failed to set NEXUS_NODE_IDS")?;
        
        thread::sleep(Duration::from_secs(1));
        
        // Set NEXUS_WALLETS
        self.set_secret(repo, "NEXUS_WALLETS", &wallets_str)
            .context("Failed to set NEXUS_WALLETS")?;
        
        info!("All Nexus secrets set successfully");
        Ok(())
    }
    
    pub fn delete_secret(&self, repo: &str, secret_name: &str) -> Result<()> {
        debug!("Deleting secret {} from {}", secret_name, repo);
        
        self.client.api_call(
            &format!("repos/{}/actions/secrets/{}", repo, secret_name),
            "DELETE"
        )?;
        
        info!("Secret {} deleted", secret_name);
        Ok(())
    }
    
    pub fn list_secrets(&self, repo: &str) -> Result<Vec<String>> {
        debug!("Listing secrets for {}", repo);
        
        let response = self.client.api_call(
            &format!("repos/{}/actions/secrets", repo),
            "GET"
        )?;
        
        let json: serde_json::Value = serde_json::from_str(&response)
            .context("Failed to parse secrets list")?;
        
        let mut secret_names = Vec::new();
        
        if let Some(secrets) = json["secrets"].as_array() {
            for secret in secrets {
                if let Some(name) = secret["name"].as_str() {
                    secret_names.push(name.to_string());
                }
            }
        }
        
        Ok(secret_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_newline_join() {
        let nodes = vec!["node1".to_string(), "node2".to_string()];
        let joined = nodes.join("\n");
        assert_eq!(joined, "node1\nnode2");
    }
}
