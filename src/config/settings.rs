use std::env;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Settings {
    pub slack_bot_token: String,
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
        Ok(Self { slack_bot_token })
    }
}
