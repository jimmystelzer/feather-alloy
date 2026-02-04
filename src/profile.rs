use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Representa um perfil/serviço web configurado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebProfile {
    pub uuid: String,
    pub name: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    pub auto_hibernate: bool,
}

impl WebProfile {
    pub fn new(
        name: String,
        url: String,
        icon_path: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            uuid: uuid::Uuid::new_v4().to_string(),
            name,
            url,
            icon_path,
            user_agent,
            auto_hibernate: true,
        }
    }
}

/// Estado global da aplicação
pub type AppState = Arc<Mutex<Vec<WebProfile>>>;

pub fn create_app_state() -> AppState {
    Arc::new(Mutex::new(Vec::new()))
}
