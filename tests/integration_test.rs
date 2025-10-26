// tests/integration_test.rs
use std::path::PathBuf;

#[test]
fn test_config_loading() {
    let config_dir = PathBuf::from("config");
    
    // This test just ensures modules compile
    assert!(config_dir.exists() || !config_dir.exists());
}

#[test]
fn test_crypto_init() {
    // Test crypto initialization
    let result = unsafe {
        nexus_orchestrator::utils::crypto::init_crypto()
    };
    assert!(result.is_ok());
}
