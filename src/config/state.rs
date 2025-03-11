use crate::config::settings::Settings;
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub client: Client,
}
