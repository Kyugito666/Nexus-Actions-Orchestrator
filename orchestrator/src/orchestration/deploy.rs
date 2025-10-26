// Update imports at top of src/orchestration/deploy.rs
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use log::info;
use crate::core::{account, state, StateManager};
use crate::github::{GitHubClient, SecretsManager, WorkflowController};
use crate::nexus::NexusConfig;

pub struct Deployer {
    config_dir: PathBuf,
}

impl Deployer {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }
    
    pub fn deploy_main_workflow(&self) -> Result<()> {
        info!("Deploying main workflow");
        let workflow_path = Path::new(".github/workflows/nexus.yml");
        let setup = self.load_setup()?;
        let main_repo = format!("{}/{}", setup.owner, setup.repo_name);
        
        let tokens = self.load_tokens()?;
        let main_token = tokens.first().unwrap();
        
        let client = GitHubClient::new(main_token.clone(), None);
        let controller = WorkflowController::new(workflow_path)?;
        
        controller.deploy_to_repo(&main_repo, &client)?;
        info!("Main workflow deployed to {}", main_repo);
        Ok(())
    }
    
    pub fn set_all_secrets(&self) -> Result<()> {
        info!("Setting secrets for all repos");
        let nexus_config = NexusConfig::load_from_files(
            &self.config_dir.join("nodes.txt"),
            &self.config_dir.join("wallets.txt")
        )?;
        
        let state_mgr = StateManager::new(&self.config_dir)?;
        let state = state_mgr.load_state()?;
        
        for node in &state.fork_chain {
            if node.status != state::ForkStatus::Active {
                continue;
            }
            
            let account = self.get_account_by_index(node.pat_index)?;
            let client = GitHubClient::new(account.token.clone(), None);
            let secrets_mgr = SecretsManager::new(client);
            
            secrets_mgr.set_nexus_secrets(
                &node.repo,
                &nexus_config.node_ids,
                &nexus_config.wallets
            )?;
            
            info!("Secrets set for {}", node.repo);
        }
        
        Ok(())
    }
    
    fn load_setup(&self) -> Result<SetupConfig> {
        let content = std::fs::read_to_string(self.config_dir.join("setup.json"))?;
        Ok(serde_json::from_str(&content)?)
    }
    
    fn load_tokens(&self) -> Result<Vec<String>> {
        let content = std::fs::read_to_string(self.config_dir.join("tokens.txt"))?;
        Ok(content.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
    }
    
    fn get_account_by_index(&self, index: usize) -> Result<account::AccountInfo> {
        let mut mgr = account::AccountManager::new(&self.config_dir.join("cache"));
        mgr.load_tokens(&self.config_dir.join("tokens.txt"))?;
        mgr.get_account(index).cloned().ok_or_else(|| anyhow::anyhow!("Account not found"))
    }
}

#[derive(serde::Deserialize)]
struct SetupConfig {
    main_repo_owner: String,
    main_repo_name: String,
}

impl SetupConfig {
    fn owner(&self) -> &str { &self.main_repo_owner }
    fn repo_name(&self) -> &str { &self.main_repo_name }
}
