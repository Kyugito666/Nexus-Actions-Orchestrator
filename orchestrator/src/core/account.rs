// src/core/account.rs - Account management and rotation

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub username: String,
    pub token: String,
    pub index: usize,
}

pub struct AccountManager {
    accounts: Vec<AccountInfo>,
    cache_file: std::path::PathBuf,
}

impl AccountManager {
    pub fn new(cache_dir: &Path) -> Self {
        let cache_file = cache_dir.join("tokenmap.json");
        
        Self {
            accounts: Vec::new(),
            cache_file,
        }
    }
    
    pub fn load_tokens(&mut self, tokens_file: &Path) -> Result<()> {
        let content = fs::read_to_string(tokens_file)
            .context("Failed to read tokens.txt")?;
        
        let tokens: Vec<String> = content
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && (s.starts_with("ghp_") || s.starts_with("github_pat_")))
            .collect();
        
        if tokens.is_empty() {
            bail!("No valid tokens found in tokens.txt");
        }
        
        info!("Loaded {} tokens from file", tokens.len());
        
        // Try to load cached usernames first
        let cached_usernames = self.load_cache().unwrap_or_default();
        
        for (i, token) in tokens.iter().enumerate() {
            let username = cached_usernames.get(token)
                .cloned()
                .unwrap_or_else(|| format!("user_{}", i));
            
            self.accounts.push(AccountInfo {
                username,
                token: token.clone(),
                index: i,
            });
        }
        
        Ok(())
    }
    
    pub fn validate_all(&mut self, proxy_manager: &crate::core::proxy::ProxyManager) -> Result<()> {
        use std::process::Command;
        
        info!("Validating {} accounts...", self.accounts.len());
        
        let mut valid_accounts = Vec::new();
        let mut cache_map = HashMap::new();
        
        for (i, account) in self.accounts.iter().enumerate() {
            print!("  [{}/{}] Validating {}... ", i + 1, self.accounts.len(), account.username);
            
            let proxy = proxy_manager.get_proxy(&account.token);
            
            let mut cmd = Command::new("gh");
            cmd.args(&["api", "user", "--jq", ".login"]);
            cmd.env("GH_TOKEN", &account.token);
            
            if let Some(proxy_config) = proxy {
                let proxy_url = proxy_config.to_curl_format();
                cmd.env("https_proxy", &proxy_url);
                cmd.env("http_proxy", &proxy_url);
            }
            
            match cmd.output() {
                Ok(output) if output.status.success() => {
                    let username = String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .to_string();
                    
                    println!("✅ @{}", username);
                    
                    let mut validated_account = account.clone();
                    validated_account.username = username.clone();
                    
                    cache_map.insert(account.token.clone(), username);
                    valid_accounts.push(validated_account);
                }
                Ok(output) => {
                    let error = String::from_utf8_lossy(&output.stderr);
                    println!("❌ Invalid: {}", error.lines().next().unwrap_or("Unknown error"));
                }
                Err(e) => {
                    println!("❌ Error: {}", e);
                }
            }
            
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        
        if valid_accounts.is_empty() {
            bail!("No valid accounts found after validation!");
        }
        
        info!("Validation complete: {}/{} accounts valid", valid_accounts.len(), self.accounts.len());
        
        self.accounts = valid_accounts;
        self.save_cache(&cache_map)?;
        
        Ok(())
    }
    
    fn load_cache(&self) -> Result<HashMap<String, String>> {
        if !self.cache_file.exists() {
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(&self.cache_file)?;
        let cache: HashMap<String, String> = serde_json::from_str(&content)?;
        
        Ok(cache)
    }
    
    fn save_cache(&self, cache: &HashMap<String, String>) -> Result<()> {
        let json = serde_json::to_string_pretty(cache)?;
        fs::write(&self.cache_file, json)?;
        Ok(())
    }
    
    pub fn get_account(&self, index: usize) -> Option<&AccountInfo> {
        self.accounts.get(index)
    }
    
    pub fn get_all_accounts(&self) -> &[AccountInfo] {
        &self.accounts
    }
    
    pub fn total_accounts(&self) -> usize {
        self.accounts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    #[test]
    fn test_load_tokens() {
        let temp_dir = tempdir().unwrap();
        let tokens_file = temp_dir.path().join("tokens.txt");
        
        fs::write(&tokens_file, "ghp_test123\nghp_test456\n").unwrap();
        
        let cache_dir = temp_dir.path().join("cache");
        fs::create_dir(&cache_dir).unwrap();
        
        let mut manager = AccountManager::new(&cache_dir);
        manager.load_tokens(&tokens_file).unwrap();
        
        assert_eq!(manager.total_accounts(), 2);
    }
}
