use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(rename = "app")]
    pub app: AppSettings,
    #[serde(rename = "kafka")]
    pub kafka: KafkaSettings,
    #[serde(rename = "dubbo")] 
    pub dubbo: DubboSettings,
    #[serde(rename = "storage")]
    pub storage: StorageSettings,
    #[serde(rename = "markets")]
    pub markets: MarketsSettings,
    #[serde(rename = "ai_quant", default)]
    pub ai_quant: AIQuantSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub name: String,
    pub log_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KafkaSettings {
    pub brokers: String,
    pub topic: String,
    pub producer_timeout_ms: u64,
    pub producer_retry_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DubboSettings {
    pub port: u16,
    pub registry_address: String,
    pub group: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageSettings {
    pub wal_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketsSettings {
    pub markets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIQuantSettings {
    pub enabled: bool,
    pub provider_url: String,
    pub api_key: String,
    pub model_name: String,
}

impl Default for AIQuantSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider_url: "http://localhost:11434/v1/chat/completions".to_string(),
            api_key: "".to_string(),
            model_name: "meta-llama/Llama-2-7b-chat-hf".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let result = dotenvy::dotenv();
        match result {
            Ok(path) => println!("Loaded environment variables from .env file {:?}", path.to_str()),
            Err(e) => println!("Failed to load environment variables from .env file: {}", e),
        }
        
        // Always print environment variables for debugging
        println!("==== Environment Variables ====");
        for (key, value) in std::env::vars() {
            println!("{}={}", key, value);
        }

        // Use double underscore for nesting to support keys with underscores (e.g. registry__address)
        let prefixed_env = Environment::with_prefix("ORDER_MATCH")
            .prefix_separator("__")
            .separator("__")
            .try_parsing(true);

        let config = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(prefixed_env)
            .build()
            .map_err(|e| {
            println!("Failed to build config: {}", e);
                e
            })?;

        // Always print loaded config keys
        println!("==== Config Keys ====");
        if let Ok(settings) = config.clone().try_deserialize::<HashMap<String, String>>() {
            for (key, value) in settings {
                println!("{}={:?}", key, value);
            }
        }

        let config = config.try_deserialize::<Self>()?;
        
        // Validate required fields
        if config.app.name.is_empty() {
            return Err(ConfigError::Message("app.name is required".to_string()));
        }

        println!("Successfully loaded configuration: {:?}", config);
        Ok(config)
    }

    pub fn load_with_env_prefix(prefix: &str) -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let config = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(
                Environment::with_prefix(prefix)
                    .prefix_separator("_")
                    .separator("_")
                    .try_parsing(true)
            )
            .build()?;

        let config = config.try_deserialize::<Self>()?;
        
        // Validate required fields
        if config.app.name.is_empty() {
            return Err(ConfigError::Message("app.name is required".to_string()));
        }

        Ok(config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                name: "order-match".to_string(),
                log_level: "info".to_string(),
            },
            kafka: KafkaSettings {
                brokers: "localhost:9092".to_string(),
                topic: "dex-trades".to_string(),
                producer_timeout_ms: 5000,
                producer_retry_count: 3,
            },
            dubbo: DubboSettings {
                port: 20086,
                registry_address: "127.0.0.1:2181".to_string(),
                group: "/dev/metawebthree".to_string(),
                version: "".to_string(),
            },
            storage: StorageSettings {
                wal_dir: "/tmp".to_string(),
            },
            markets: MarketsSettings {
                markets: vec![
                    "BTC/USDT".to_string(),
                    "ETH/USDT".to_string(),
                    "DOGE/USDT".to_string(),
                ],
            },
            ai_quant: AIQuantSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.app.name, "order-match");
        assert_eq!(config.kafka.brokers, "localhost:9092");
        assert_eq!(config.dubbo.port, 20086);
    }

    #[test]
    fn test_config_load() {
        if std::path::Path::new("config.toml").exists() {
            let config = AppConfig::load();
            assert!(config.is_ok());
        }
    }
}
