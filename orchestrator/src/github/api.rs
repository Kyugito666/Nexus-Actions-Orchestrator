// src/github/api.rs - GitHub API wrapper with proxy support

use anyhow::{Result, Context, bail};
use std::process::{Command, Output};
use std::time::Duration;
use std::thread;
use log::{debug, warn};
use crate::utils::retry::{retry_with_backoff, RetryConfig};

pub struct GitHubClient {
    token: String,
    proxy: Option<String>,
    retry_config: RetryConfig,
}

impl GitHubClient {
    pub fn new(token: String, proxy: Option<String>) -> Self {
        Self {
            token,
            proxy,
            retry_config: RetryConfig::default(),
        }
    }
    
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }
    
    fn execute_gh(&self, args: &[&str]) -> Result<Output> {
        let mut cmd = Command::new("gh");
        cmd.args(args);
        cmd.env("GH_TOKEN", &self.token);
        
        if let Some(proxy_url) = &self.proxy {
            cmd.env("https_proxy", proxy_url);
            cmd.env("http_proxy", proxy_url);
        }
        
        debug!("Executing: gh {}", args.join(" "));
        
        let output = cmd.output()
            .context("Failed to execute gh command")?;
        
        Ok(output)
    }
    
    pub fn api_call(&self, endpoint: &str, method: &str) -> Result<String> {
        let args = if method == "GET" {
            vec!["api", endpoint]
        } else {
            vec!["api", "-X", method, endpoint]
        };
        
        let operation = || {
            let output = self.execute_gh(&args)?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // Check for rate limit
                if stderr.contains("rate limit") || stderr.contains("403") {
                    warn!("Rate limit hit, waiting 60s...");
                    thread::sleep(Duration::from_secs(60));
                    bail!("Rate limit exceeded (retry)");
                }
                
                // Check for temporary errors
                if stderr.contains("timeout") || stderr.contains("connection") {
                    bail!("Network error: {}", stderr);
                }
                
                bail!("API call failed: {}", stderr);
            }
            
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        };
        
        retry_with_backoff(&self.retry_config, "GitHub API call", operation)
    }
    
    pub fn api_call_with_data(&self, endpoint: &str, method: &str, json_data: &str) -> Result<String> {
        let mut args = vec!["api", "-X", method, endpoint, "--input", "-"];
        
        let mut cmd = Command::new("gh");
        cmd.args(&args);
        cmd.env("GH_TOKEN", &self.token);
        
        if let Some(proxy_url) = &self.proxy {
            cmd.env("https_proxy", proxy_url);
            cmd.env("http_proxy", proxy_url);
        }
        
        use std::io::Write;
        use std::process::Stdio;
        
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let mut child = cmd.spawn()
            .context("Failed to spawn gh command")?;
        
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(json_data.as_bytes())
                .context("Failed to write to stdin")?;
        }
        
        let output = child.wait_with_output()
            .context("Failed to wait for gh command")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("API call failed: {}", stderr);
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    pub fn get_username(&self) -> Result<String> {
        let response = self.api_call("user", "GET")?;
        let json: serde_json::Value = serde_json::from_str(&response)
            .context("Failed to parse user response")?;
        
        json["login"]
            .as_str()
            .map(|s| s.to_string())
            .context("Username not found in response")
    }
    
    pub fn check_repo_exists(&self, repo: &str) -> Result<bool> {
        match self.api_call(&format!("repos/{}", repo), "GET") {
            Ok(_) => Ok(true),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("404") || error_str.contains("Not Found") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
    
    pub fn create_fork(&self, source_repo: &str) -> Result<String> {
        debug!("Creating fork of {}", source_repo);
        
        let response = self.api_call(
            &format!("repos/{}/forks", source_repo),
            "POST"
        )?;
        
        let json: serde_json::Value = serde_json::from_str(&response)
            .context("Failed to parse fork response")?;
        
        json["full_name"]
            .as_str()
            .map(|s| s.to_string())
            .context("Fork name not found in response")
    }
    
    pub fn delete_repo(&self, repo: &str) -> Result<()> {
        debug!("Deleting repository {}", repo);
        
        self.api_call(&format!("repos/{}", repo), "DELETE")?;
        
        // Wait to ensure deletion is processed
        thread::sleep(Duration::from_secs(5));
        
        Ok(())
    }
    
    pub fn get_workflow_id(&self, repo: &str, workflow_file: &str) -> Result<Option<u64>> {
        let response = self.api_call(&format!("repos/{}/actions/workflows", repo), "GET")?;
        
        let json: serde_json::Value = serde_json::from_str(&response)
            .context("Failed to parse workflows response")?;
        
        if let Some(workflows) = json["workflows"].as_array() {
            for workflow in workflows {
                if let Some(path) = workflow["path"].as_str() {
                    if path.contains(workflow_file) {
                        return Ok(workflow["id"].as_u64());
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    pub fn enable_workflow(&self, repo: &str, workflow_id: u64) -> Result<()> {
        debug!("Enabling workflow {} in {}", workflow_id, repo);
        
        match self.api_call(
            &format!("repos/{}/actions/workflows/{}/enable", repo, workflow_id),
            "PUT"
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("already enabled") {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }
    
    pub fn disable_workflow(&self, repo: &str, workflow_id: u64) -> Result<()> {
        debug!("Disabling workflow {} in {}", workflow_id, repo);
        
        match self.api_call(
            &format!("repos/{}/actions/workflows/{}/disable", repo, workflow_id),
            "PUT"
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("already disabled") || error_str.contains("not enabled") {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }
    
    pub fn trigger_workflow(&self, repo: &str, workflow_file: &str, ref_name: &str) -> Result<()> {
        debug!("Triggering workflow {} in {} on ref {}", workflow_file, repo, ref_name);
        
        let data = serde_json::json!({
            "ref": ref_name
        });
        
        self.api_call_with_data(
            &format!("repos/{}/actions/workflows/{}/dispatches", repo, workflow_file),
            "POST",
            &data.to_string()
        )?;
        
        Ok(())
    }
    
    pub fn get_latest_workflow_run(&self, repo: &str) -> Result<Option<u64>> {
        let response = self.api_call(
            &format!("repos/{}/actions/runs?per_page=1", repo),
            "GET"
        )?;
        
        let json: serde_json::Value = serde_json::from_str(&response)
            .context("Failed to parse workflow runs response")?;
        
        if let Some(runs) = json["workflow_runs"].as_array() {
            if let Some(first_run) = runs.first() {
                return Ok(first_run["id"].as_u64());
            }
        }
        
        Ok(None)
    }
    
    pub fn get_workflow_status(&self, repo: &str, run_id: u64) -> Result<(String, Option<String>)> {
        let response = self.api_call(
            &format!("repos/{}/actions/runs/{}", repo, run_id),
            "GET"
        )?;
        
        let json: serde_json::Value = serde_json::from_str(&response)
            .context("Failed to parse workflow run response")?;
        
        let status = json["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        
        let conclusion = json["conclusion"]
            .as_str()
            .map(|s| s.to_string());
        
        Ok((status, conclusion))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new("test_token".to_string(), None);
        assert_eq!(client.token, "test_token");
        assert!(client.proxy.is_none());
    }
}
