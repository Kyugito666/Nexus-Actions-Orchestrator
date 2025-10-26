// src/core/billing.rs - Billing monitor (ported from Nexus Rust billing.rs)

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::process::Command;
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingInfo {
    pub username: String,
    pub total_core_hours_used: f32,
    pub total_minutes_used: f32,
    pub included_minutes: f32,
    pub hours_remaining: f32,
    pub is_exhausted: bool,
    pub is_warning: bool,
}

#[derive(Deserialize, Debug)]
struct UsageItem {
    product: String,
    #[serde(rename = "unitType")]
    unit_type: String,
    quantity: f32,
}

#[derive(Deserialize, Debug)]
struct BillingResponse {
    #[serde(rename = "usageItems")]
    usage_items: Vec<UsageItem>,
}

pub struct BillingMonitor {
    warning_threshold: f32,    // 118.0 for free tier (120 total)
    critical_threshold: f32,   // 119.5 for free tier
}

impl Default for BillingMonitor {
    fn default() -> Self {
        Self {
            warning_threshold: 118.0,
            critical_threshold: 119.5,
        }
    }
}

impl BillingMonitor {
    pub fn new(warning_threshold: f32, critical_threshold: f32) -> Self {
        Self {
            warning_threshold,
            critical_threshold,
        }
    }
    
    pub fn check_billing(
        &self,
        username: &str,
        token: &str,
        proxy: Option<&str>,
    ) -> Result<BillingInfo> {
        let endpoint = format!("/users/{}/settings/billing/usage", username);
        
        let mut cmd = Command::new("gh");
        cmd.args(&[
            "api",
            &endpoint,
            "-H", "Accept: application/vnd.github+json",
        ]);
        
        if let Some(proxy_url) = proxy {
            cmd.env("https_proxy", proxy_url);
            cmd.env("http_proxy", proxy_url);
        }
        
        cmd.env("GH_TOKEN", token);
        
        let output = cmd.output()
            .context("Failed to execute gh command")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Billing API call failed for {}: {}", username, stderr);
            
            // Return safe default (assume exhausted)
            return Ok(BillingInfo {
                username: username.to_string(),
                total_core_hours_used: 999.0,
                total_minutes_used: 999.0,
                included_minutes: 2000.0,
                hours_remaining: 0.0,
                is_exhausted: true,
                is_warning: true,
            });
        }
        
        let response_text = String::from_utf8_lossy(&output.stdout);
        let response: BillingResponse = serde_json::from_str(&response_text)
            .context("Failed to parse billing response")?;
        
        let mut total_minutes = 0.0;
        
        for item in response.usage_items {
            if item.product == "actions" && item.unit_type == "Minutes" {
                total_minutes += item.quantity;
            }
        }
        
        // Free tier: 2000 minutes = ~33.3 hours at 1x multiplier
        // But Actions use multipliers: 2x for Linux = 120 "core-hours"
        let included_minutes = 2000.0;
        let total_core_hours = total_minutes * 2.0 / 60.0; // 2x multiplier for Linux
        let max_core_hours = 120.0;
        
        let hours_remaining = (max_core_hours - total_core_hours).max(0.0);
        
        let is_warning = total_core_hours >= self.warning_threshold;
        let is_exhausted = total_core_hours >= self.critical_threshold;
        
        Ok(BillingInfo {
            username: username.to_string(),
            total_core_hours_used: total_core_hours,
            total_minutes_used: total_minutes,
            included_minutes,
            hours_remaining,
            is_exhausted,
            is_warning,
        })
    }
    
    pub fn display_billing(&self, info: &BillingInfo) {
        let status_icon = if info.is_exhausted {
            "🔴"
        } else if info.is_warning {
            "🟡"
        } else {
            "🟢"
        };
        
        println!(
            "{} @{:<20} | {:.1}/120.0 core-hours | {:.1}h remaining",
            status_icon,
            info.username,
            info.total_core_hours_used,
            info.hours_remaining
        );
        
        if info.is_exhausted {
            println!("   ⚠️  CRITICAL: Quota exhausted!");
        } else if info.is_warning {
            println!("   ⚠️  WARNING: Quota low");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_billing_thresholds() {
        let monitor = BillingMonitor::default();
        assert_eq!(monitor.warning_threshold, 118.0);
        assert_eq!(monitor.critical_threshold, 119.5);
    }
}
