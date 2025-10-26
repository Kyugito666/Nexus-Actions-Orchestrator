// src/core/proxy.rs - Proxy management (1 PAT = 1 Proxy)

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub url: String,           // http://user:pass@ip:port
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
}

impl ProxyConfig {
    pub fn from_url(url: &str) -> Result<Self> {
        // Parse: http://user:pass@ip:port
        let url = url.trim();
        
        if !url.starts_with("http://") && !url.starts_with("https://") {
            bail!("Proxy URL must start with http:// or https://");
        }
        
        let without_scheme = url.split("://").nth(1)
            .context("Invalid proxy URL format")?;
        
        let parts: Vec<&str> = without_scheme.split('@').collect();
        if parts.len() != 2 {
            bail!("Proxy URL must contain credentials: http://user:pass@host:port");
        }
        
        let credentials = parts[0];
        let host_port = parts[1];
        
        let cred_parts: Vec<&str> = credentials.split(':').collect();
        if cred_parts.len() != 2 {
            bail!("Invalid credentials format in proxy URL");
        }
        
        let username = cred_parts[0].to_string();
        let password = cred_parts[1].to_string();
        
        let host_parts: Vec<&str> = host_port.split(':').collect();
        if host_parts.len() != 2 {
            bail!("Invalid host:port format in proxy URL");
        }
        
        let host = host_parts[0].to_string();
        let port = host_parts[1].parse::<u16>()
            .context("Invalid port number")?;
        
        Ok(Self {
            url: url.to_string(),
            username,
            password,
            host,
            port,
        })
    }
    
    pub fn to_curl_format(&self) -> String {
        format!("http://{}:{}@{}:{}", self.username, self.password, self.host, self.port)
    }
}

pub struct ProxyManager {
    mappings: HashMap<String, ProxyConfig>,  // PAT token -> Proxy
    cache_file: std::path::PathBuf,
}

impl ProxyManager {
    pub fn new(cache_dir: &Path) -> Self {
        let cache_file = cache_dir.join("proxymap.json");
        
        Self {
            mappings: HashMap::new(),
            cache_file,
        }
    }
    
    pub fn load_from_file(&mut self, proxies_file: &Path, tokens: &[String]) -> Result<()> {
        let content = fs::read_to_string(proxies_file)
            .context("Failed to read proxies.txt")?;
        
        let proxy_lines: Vec<String> = content
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if proxy_lines.len() < tokens.len() {
            bail!(
                "Not enough proxies! Need {} proxies for {} tokens",
                tokens.len(),
                tokens.len()
            );
        }
        
        info!("Mapping {} tokens to {} proxies (1:1)", tokens.len(), proxy_lines.len());
        
        for (i, token) in tokens.iter().enumerate() {
            let proxy_url = &proxy_lines[i];
            let proxy_config = ProxyConfig::from_url(proxy_url)
                .with_context(|| format!("Invalid proxy URL at line {}: {}", i + 1, proxy_url))?;
            
            self.mappings.insert(token.clone(), proxy_config);
        }
        
        info!("Successfully mapped {} PAT-Proxy pairs", self.mappings.len());
        self.save_cache()?;
        
        Ok(())
    }
    
    pub fn load_cache(&mut self) -> Result<()> {
        if !self.cache_file.exists() {
            info!("Proxy cache not found, skipping load");
            return Ok(());
        }
        
        let content = fs::read_to_string(&self.cache_file)
            .context("Failed to read proxy cache")?;
        
        self.mappings = serde_json::from_str(&content)
            .context("Failed to parse proxy cache")?;
        
        info!("Loaded {} proxy mappings from cache", self.mappings.len());
        Ok(())
    }
    
    pub fn save_cache(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.mappings)
            .context("Failed to serialize proxy mappings")?;
        
        fs::write(&self.cache_file, json)
            .context("Failed to write proxy cache")?;
        
        Ok(())
    }
    
    pub fn get_proxy(&self, token: &str) -> Option<&ProxyConfig> {
        self.mappings.get(token)
    }
    
    pub fn test_proxy(&self, proxy: &ProxyConfig) -> Result<bool> {
        use std::process::Command;
        use std::time::Duration;
        
        info!("Testing proxy: {}:{}", proxy.host, proxy.port);
        
        // Test with curl to github.com
        let output = Command::new("curl")
            .args(&[
                "--proxy", &proxy.to_curl_format(),
                "--connect-timeout", "10",
                "--max-time", "15",
                "-s",
                "-o", "/dev/null",
                "-w", "%{http_code}",
                "https://api.github.com/",
            ])
            .output()
            .context("Failed to execute curl for proxy test")?;
        
        let status_code = String::from_utf8_lossy(&output.stdout);
        let is_ok = status_code.trim() == "200";
        
        if is_ok {
            info!("Proxy test OK: {}:{}", proxy.host, proxy.port);
        } else {
            warn!(
                "Proxy test failed: {}:{} (HTTP {})",
                proxy.host, proxy.port, status_code
            );
        }
        
        Ok(is_ok)
    }
    
    pub fn validate_all(&self) -> Result<Vec<String>> {
        let mut failed_tokens = Vec::new();
        
        for (token, proxy) in &self.mappings {
            match self.test_proxy(proxy) {
                Ok(true) => {}
                Ok(false) => {
                    warn!("Proxy validation failed for token: {}...", &token[..12]);
                    failed_tokens.push(token.clone());
                }
                Err(e) => {
                    warn!("Proxy test error for token {}...: {}", &token[..12], e);
                    failed_tokens.push(token.clone());
                }
            }
        }
        
        if failed_tokens.is_empty() {
            info!("All {} proxies validated successfully", self.mappings.len());
        } else {
            warn!("{} proxies failed validation", failed_tokens.len());
        }
        
        Ok(failed_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proxy_parsing() {
        let url = "http://user123:pass456@1.2.3.4:8080";
        let config = ProxyConfig::from_url(url).unwrap();
        
        assert_eq!(config.username, "user123");
        assert_eq!(config.password, "pass456");
        assert_eq!(config.host, "1.2.3.4");
        assert_eq!(config.port, 8080);
    }
    
    #[test]
    fn test_invalid_proxy() {
        let url = "invalid_url";
        assert!(ProxyConfig::from_url(url).is_err());
    }
}
