use log::trace;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub llm_api_key: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_enable_reasoning: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or(
            ".env".into()
        );
        if config_path.ends_with(".toml") {
            trace!("Loading configuration from file: {}", config_path);
            return Self::from_file(&config_path)
                .map_err(|_| std::env::VarError::NotPresent);
        } else if config_path.ends_with(".env") {
            trace!("Loading configuration from .env file: {}", config_path);
            dotenv::from_filename(&config_path).ok();
        }
        
        trace!("Loading configuration from environment variables");
        let server_host = std::env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".into());
        trace!("SERVER_HOST: {}", server_host);
        let server_port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .map_err(|_| std::env::VarError::NotPresent)?;
        trace!("SERVER_PORT: {}", server_port);
        let llm_api_key = std::env::var("LLM_API_KEY")?;
        trace!("LLM_API_KEY: {}", llm_api_key);
        let llm_base_url = std::env::var("LLM_BASE_URL")?;
        trace!("LLM_BASE_URL: {}", llm_base_url);
        let llm_model = std::env::var("LLM_MODEL")?;
        trace!("LLM_MODEL: {}", llm_model);
        let llm_enable_reasoning = std::env::var("LLM_ENABLE_REASONING")
            .unwrap_or_else(|_| "false".into())
            .parse()
            .unwrap_or(false);
        trace!("LLM_ENABLE_REASONING: {}", llm_enable_reasoning);

        Ok(Config {
            server_host,
            server_port,
            llm_api_key,
            llm_base_url,
            llm_model,
            llm_enable_reasoning,
        })
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config {{ server_host: {}, server_port: {}, llm_api_key: ****, llm_base_url: {}, llm_model: {} }}",
            self.server_host, self.server_port, self.llm_base_url, self.llm_model
        )
    }
}