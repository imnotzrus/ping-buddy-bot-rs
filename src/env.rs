/// Environment configuration management
use std::env;

use url::Url;

use crate::error::{BotError, Result};

/// Application environment configuration
#[derive(Debug, Clone)]
pub struct Env {
    /// Inbound webhook URL
    pub inbound: Url,
    /// Outbound webhook URL
    pub outbound: Url,
    /// Telegram bot token
    #[allow(dead_code)]
    pub bot_token: String,
}

impl Env {
    /// Initialize and validate environment variables
    pub fn init() -> Result<Self> {
        // Load and validate bot token
        let bot_token = env::var("TELOXIDE_TOKEN").map_err(|_| {
            BotError::Config("TELOXIDE_TOKEN environment variable is required".to_string())
        })?;

        if bot_token.is_empty() {
            return Err(BotError::Config(
                "TELOXIDE_TOKEN cannot be empty".to_string(),
            ));
        }

        // Load and validate inbound URL
        let inbound_str = env::var("INBOUND").map_err(|_| {
            BotError::Config("INBOUND environment variable is required".to_string())
        })?;

        let inbound = Url::parse(&inbound_str).map_err(|e| {
            BotError::Config(format!("Invalid INBOUND URL: {}", e))
        })?;

        // Load and validate outbound URL
        let outbound_str = env::var("OUTBOUND").map_err(|_| {
            BotError::Config("OUTBOUND environment variable is required".to_string())
        })?;

        let outbound = Url::parse(&outbound_str).map_err(|e| {
            BotError::Config(format!("Invalid OUTBOUND URL: {}", e))
        })?;

        // Validate URL schemes
        if !matches!(inbound.scheme(), "http" | "https") {
            return Err(BotError::Config(
                "INBOUND URL must use http or https scheme".to_string(),
            ));
        }

        if !matches!(outbound.scheme(), "http" | "https") {
            return Err(BotError::Config(
                "OUTBOUND URL must use http or https scheme".to_string(),
            ));
        }

        log::info!("Environment configuration loaded successfully");
        log::debug!("Inbound URL: {}", inbound);
        log::debug!("Outbound URL: {}", outbound);

        Ok(Self {
            inbound,
            outbound,
            bot_token,
        })
    }
}
