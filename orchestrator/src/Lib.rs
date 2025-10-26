// src/lib.rs - Library interface for testing
pub mod core;
pub mod github;
pub mod nexus;
pub mod monitor;
pub mod orchestration;
pub mod utils;
pub mod ui;

pub use core::{AccountManager, BillingMonitor, ProxyManager, StateManager};
pub use github::{GitHubClient, ForkManager, SecretsManager, WorkflowController};
pub use nexus::{NexusConfig, NexusValidator};
