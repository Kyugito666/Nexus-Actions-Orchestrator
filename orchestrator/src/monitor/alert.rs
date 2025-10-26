// src/monitor/alert.rs - Alert system (Telegram/Discord)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub discord_webhook: Option<String>,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            telegram_bot_token: None,
            telegram_chat_id: None,
            discord_webhook: None,
        }
    }
}

pub struct AlertManager {
    config: AlertConfig,
}

impl AlertManager {
    pub fn new(config_file: &Path) -> Result<Self> {
        let config = if config_file.exists() {
            let content = fs::read_to_string(config_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AlertConfig::default()
        };
        
        Ok(Self { config })
    }
    
    pub fn send_alert(&self, message: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        if let (Some(token), Some(chat_id)) = (&self.config.telegram_bot_token, &self.config.telegram_chat_id) {
            self.send_telegram(token, chat_id, message)?;
        }
        
        if let Some(webhook) = &self.config.discord_webhook {
            self.send_discord(webhook, message)?;
        }
        
        Ok(())
    }
    
    fn send_telegram(&self, bot_token: &str, chat_id: &str, message: &str) -> Result<()> {
        use std::process::Command;
        
        let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
        
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": message,
            "parse_mode": "Markdown"
        });
        
        let output = Command::new("curl")
            .args(&[
                "-X", "POST",
                &url,
                "-H", "Content-Type: application/json",
                "-d", &payload.to_string(),
                "-s"
            ])
            .output()?;
        
        if output.status.success() {
            info!("Telegram alert sent");
        } else {
            warn!("Failed to send Telegram alert: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    fn send_discord(&self, webhook: &str, message: &str) -> Result<()> {
        use std::process::Command;
        
        let payload = serde_json::json!({
            "content": message
        });
        
        let output = Command::new("curl")
            .args(&[
                "-X", "POST",
                webhook,
                "-H", "Content-Type: application/json",
                "-d", &payload.to_string(),
                "-s"
            ])
            .output()?;
        
        if output.status.success() {
            info!("Discord alert sent");
        } else {
            warn!("Failed to send Discord alert: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
}
