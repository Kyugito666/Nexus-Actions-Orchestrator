// src/utils/retry.rs - Retry logic with exponential backoff

use std::thread;
use std::time::Duration;
use anyhow::Result;
use log::{warn, debug};

#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
        }
    }
}

pub fn retry_with_backoff<F, T>(
    config: &RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;
    
    loop {
        attempt += 1;
        
        debug!(
            "Attempting {} (attempt {}/{})",
            operation_name, attempt, config.max_attempts
        );
        
        match operation() {
            Ok(result) => {
                if attempt > 1 {
                    debug!("{} succeeded on attempt {}", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt >= config.max_attempts {
                    warn!(
                        "{} failed after {} attempts: {}",
                        operation_name, attempt, e
                    );
                    return Err(e);
                }
                
                warn!(
                    "{} failed (attempt {}): {}. Retrying in {}ms...",
                    operation_name, attempt, e, delay
                );
                
                thread::sleep(Duration::from_millis(delay));
                
                // Exponential backoff
                delay = ((delay as f64) * config.multiplier) as u64;
                if delay > config.max_delay_ms {
                    delay = config.max_delay_ms;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    
    #[test]
    fn test_retry_success() {
        let counter = AtomicU32::new(0);
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };
        
        let result = retry_with_backoff(&config, "test", || {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                anyhow::bail!("Simulated failure {}", count);
            }
            Ok(42)
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
    
    #[test]
    fn test_retry_failure() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };
        
        let result = retry_with_backoff(&config, "test", || {
            anyhow::bail!("Always fails");
        });
        
        assert!(result.is_err());
    }
}
