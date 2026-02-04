use serde::{Deserialize, Serialize};

/// Mensagens IPC entre as webviews e o backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcMessage {
    // Mensagens da Toolbar para o Backend
    AddProfile {
        name: String,
        url: String,
        icon_path: Option<String>,
        user_agent: Option<String>,
    },
    ShowProfile {
        uuid: String,
    },
    RemoveProfile {
        uuid: String,
    },
    GetProfiles,
    ShowAddProfileForm,
    CancelAddProfile,
    
    // Mensagens do Backend para a Toolbar
    ProfileAdded {
        profile: crate::profile::WebProfile,
    },
    ProfileRemoved {
        uuid: String,
    },
    ProfilesList {
        profiles: Vec<crate::profile::WebProfile>,
    },
    
    // Mensagens do Backend para a Content Webview
    NavigateToUrl {
        url: String,
        user_agent: Option<String>,
    },
    ShowWelcome,
    
    // Respostas genéricas
    Success {
        message: String,
    },
    Error {
        message: String,
    },
}

impl IpcMessage {
    /// Parse uma mensagem IPC de uma string JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    /// Converte a mensagem para JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Handler de IPC que processa mensagens
pub struct IpcHandler {
    state: crate::profile::AppState,
}

impl IpcHandler {
    pub fn new(state: crate::profile::AppState) -> Self {
        Self { state }
    }
    
    /// Processa uma mensagem IPC e retorna a resposta
    pub fn handle_message(&self, message: IpcMessage) -> Option<IpcMessage> {
        match message {
            IpcMessage::AddProfile { name, url, icon_path, user_agent } => {
                let profile = crate::profile::WebProfile::new(name, url, icon_path, user_agent);
                
                let mut data = self.state.lock().unwrap();
                data.profiles.push(profile.clone());
                drop(data);
                
                Some(IpcMessage::ProfileAdded { profile })
            }
            
            IpcMessage::RemoveProfile { uuid } => {
                let mut data = self.state.lock().unwrap();
                data.profiles.retain(|p| p.uuid != uuid);
                drop(data);
                
                Some(IpcMessage::ProfileRemoved { uuid })
            }
            
            IpcMessage::GetProfiles => {
                let data = self.state.lock().unwrap();
                let profiles_list = data.profiles.clone();
                drop(data);
                
                Some(IpcMessage::ProfilesList { profiles: profiles_list })
            }
            
            IpcMessage::ShowProfile { uuid } => {
                let data = self.state.lock().unwrap();
                if let Some(profile) = data.profiles.iter().find(|p| p.uuid == uuid) {
                    let url = profile.url.clone();
                    let user_agent = profile.user_agent.clone();
                    drop(data);
                    
                    Some(IpcMessage::NavigateToUrl { url, user_agent })
                } else {
                    drop(data);
                    Some(IpcMessage::Error {
                        message: "Perfil não encontrado".to_string(),
                    })
                }
            }
            
            // Outras mensagens não precisam de resposta ou são apenas para notificação
            _ => None,
        }
    }
}
