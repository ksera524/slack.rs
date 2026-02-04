use std::env;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Settings {
    pub slack_bot_token: String,
    pub slack_api_base_url: String,
}
#[derive(Debug, Error)]
pub enum SettingError {
    #[error("Missing environment variable {0}")]
    MissingEnvVar(String),
}

impl Settings {
    pub fn new() -> Result<Self, SettingError> {
        let slack_bot_token = env::var("SLACK_BOT_TOKEN")
            .map_err(|_| SettingError::MissingEnvVar("SLACK_BOT_TOKEN".into()))?;
        let slack_api_base_url = env::var("SLACK_API_BASE_URL")
            .unwrap_or_else(|_| "https://slack.com/api".to_string())
            .trim_end_matches('/')
            .to_string();
        Ok(Self {
            slack_bot_token,
            slack_api_base_url,
        })
    }
}
