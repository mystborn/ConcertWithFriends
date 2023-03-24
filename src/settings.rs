use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u16,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TicketMaster {
    pub token: String,
    pub secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub log: Log,
    pub ticketmaster: TicketMaster,
    pub autoreload_templates: bool,
    pub env: ENV,
}

const CONFIG_FILE_PATH: &str = "./config/Default.toml";
const CONFIG_FILE_PATH_PREFIX: &str = "./config/";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("SERVER_ENV").unwrap_or_else(|_| "Development".into());
        Config::builder()
            .set_default("env", env.clone())?
            .set_default("autoreload_templates", true)?
            .add_source(File::with_name(CONFIG_FILE_PATH))
            .add_source(File::with_name(&format!(
                "{}{}",
                CONFIG_FILE_PATH_PREFIX, env
            )))
            .build()?
            .try_deserialize()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum ENV {
    Development,
    Testing,
    Production,
}

impl fmt::Display for ENV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ENV::Development => write!(f, "Development"),
            ENV::Testing => write!(f, "Testing"),
            ENV::Production => write!(f, "Production"),
        }
    }
}

impl From<&str> for ENV {
    fn from(env: &str) -> Self {
        match env.to_lowercase().as_str() {
            "dev" | "development" => ENV::Development,
            "prod" | "production" => ENV::Production,
            _ => ENV::Testing,
        }
    }
}
