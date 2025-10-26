// src/github/workflow.rs - Workflow deployment and control

use anyhow::{Result, Context};
use log::{info, debug, warn};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use crate::github::api::GitHubClient;

pub struct WorkflowController {
    workflow_content: String,
}

impl WorkflowController {
    pub fn new(workflow_file: &Path) -> Result<Self> {
        let content = fs::read_to_string(workflow_file)
            .context("Failed to read workflow file")?;
        
        Ok(Self {
            workflow_content: content,
        })
    }
    
    pub fn deploy_to_repo(&self, repo: &str, client: &GitHubClient) -> Result<()> {
        info!("Deploying workflow to {}", repo);
        
        use std::process::Command;
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();
        
        // Clone repo
        debug!("Cloning repository...");
        let clone_output = Command::new("git")
            .args(&["clone", "--depth", "1", &format!("https://github.com/{}", repo), "."])
            .current_dir(repo_path)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()?;
        
        if !clone_output.status.success() {
            anyhow::bail!("Git clone failed: {}", String::from_utf8_lossy(&clone_output.stderr));
        }
        
        // Create .github/workflows directory
        let workflows_dir = repo_path.join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir)?;
        
        // Write workflow file
        let workflow_path = workflows_dir.join("nexus.yml");
        fs::write(&workflow_path, &self.workflow_content)?;
        
        debug!("Workflow file written");
        
        // Configure git
        Command::new("git")
            .args(&["config", "user.name", "Nexus Bot"])
            .current_dir(repo_path)
            .output()?;
        
        Command::new("git")
            .args(&["config", "user.email", "bot@nexus.local"])
            .current_dir(repo_path)
            .output()?;
        
        // Add and commit
        Command::new("git")
            .args(&["add", ".github/workflows/nexus.yml"])
            .current_dir(repo_path)
            .output()?;
        
        let commit_output = Command::new("git")
            .args(&["commit", "-m", "Deploy Nexus workflow"])
            .current_dir(repo_path)
            .output()?;
        
        let commit_stdout = String::from_utf8_lossy(&commit_output.stdout);
        if commit_stdout.contains("nothing to commit") {
            info!("Workflow already up to date");
            return Ok(());
        }
        
        // Push
        debug!("Pushing changes...");
        let push_output = Command::new("git")
            .args(&["push"])
            .current_dir(repo_path)
            .output()?;
        
        if !push_output.status.success() {
            anyhow::bail!("Git push failed: {}", String::from_utf8_lossy(&push_output.stderr));
        }
        
        info!("Workflow deployed successfully");
        
        thread::sleep(Duration::from_secs(3));
        
        Ok(())
    }
    
    pub fn enable_workflow(&self, repo: &str, client: &GitHubClient) -> Result<()> {
        if let Some(workflow_id) = client.get_workflow_id(repo, "nexus.yml")? {
            client.enable_workflow(repo, workflow_id)?;
            info!("Workflow enabled in {}", repo);
        } else {
            warn!("Workflow not found in {}", repo);
        }
        
        Ok(())
    }
    
    pub fn trigger_workflow(&self, repo: &str, client: &GitHubClient) -> Result<()> {
        info!("Triggering workflow in {}", repo);
        
        client.trigger_workflow(repo, "nexus.yml", "main")?;
        
        info!("Workflow triggered successfully");
        
        Ok(())
    }
    
    pub fn wait_for_completion(
        &self,
        repo: &str,
        run_id: u64,
        client: &GitHubClient,
        timeout_minutes: u64,
    ) -> Result<String> {
        info!("Monitoring workflow run #{} in {}", run_id, repo);
        
        let timeout = Duration::from_secs(timeout_minutes * 60);
        let start = std::time::Instant::now();
        
        loop {
            if start.elapsed() > timeout {
                warn!("Workflow monitoring timeout after {} minutes", timeout_minutes);
                return Ok("timeout".to_string());
            }
            
            let (status, conclusion) = client.get_workflow_status(repo, run_id)?;
            
            debug!("Workflow status: {}, conclusion: {:?}", status, conclusion);
            
            if status == "completed" {
                let result = conclusion.unwrap_or_else(|| "unknown".to_string());
                info!("Workflow completed with result: {}", result);
                return Ok(result);
            }
            
            thread::sleep(Duration::from_secs(30));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    
    #[test]
    fn test_workflow_controller_creation() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        fs::write(path, "name: test\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest").unwrap();
        
        let controller = WorkflowController::new(path).unwrap();
        assert!(controller.workflow_content.contains("name: test"));
    }
}
