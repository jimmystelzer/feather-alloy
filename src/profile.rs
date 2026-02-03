use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::WebviewWindow;

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
pub struct AppState {
    pub profiles: std::sync::Mutex<Vec<WebProfile>>,
    pub webviews: std::sync::Mutex<HashMap<String, WebviewWindow>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            profiles: std::sync::Mutex::new(Vec::new()),
            webviews: std::sync::Mutex::new(HashMap::new()),
        }
    }
}
