use crate::profile::{AppState, WebProfile};
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

/// Adiciona um novo perfil
#[tauri::command]
pub async fn add_profile(
    state: State<'_, AppState>,
    name: String,
    url: String,
    icon_path: Option<String>,
    user_agent: Option<String>,
) -> Result<WebProfile, String> {
    let profile = WebProfile::new(name, url, icon_path, user_agent);
    
    let mut profiles = state.profiles.lock().unwrap();
    profiles.push(profile.clone());
    
    Ok(profile)
}

/// Lista todos os perfis cadastrados
#[tauri::command]
pub async fn get_profiles(state: State<'_, AppState>) -> Result<Vec<WebProfile>, String> {
    let profiles = state.profiles.lock().unwrap();
    Ok(profiles.clone())
}

/// Remove um perfil pelo UUID
#[tauri::command]
pub async fn remove_profile(state: State<'_, AppState>, uuid: String) -> Result<(), String> {
    let mut profiles = state.profiles.lock().unwrap();
    profiles.retain(|p| p.uuid != uuid);
    
    // Remove também a webview associada se existir
    let mut webviews = state.webviews.lock().unwrap();
    if let Some(webview) = webviews.remove(&uuid) {
        let _ = webview.close();
    }
    
    Ok(())
}

/// Cria e exibe uma webview para o perfil especificado
#[tauri::command]
pub async fn show_webview(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    uuid: String,
) -> Result<(), String> {
    // Busca o perfil
    let profiles = state.profiles.lock().unwrap();
    let profile = profiles
        .iter()
        .find(|p| p.uuid == uuid)
        .ok_or("Perfil não encontrado")?
        .clone();
    drop(profiles);
    
    // Verifica se a webview já existe
    let webviews = state.webviews.lock().unwrap();
    if let Some(existing_webview) = webviews.get(&uuid) {
        // Se já existe, apenas foca/mostra
        existing_webview.show().map_err(|e| e.to_string())?;
        existing_webview.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    drop(webviews);
    
    // Define o diretório de dados isolado para este perfil
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let profile_data_dir = app_data_dir.join("profiles").join(&uuid);
    
    // Cria o diretório se não existir
    std::fs::create_dir_all(&profile_data_dir).map_err(|e| e.to_string())?;
    
    // Cria uma nova webview window com contexto isolado
    let label = format!("webview_{}", uuid);
    let mut builder = WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::External(profile.url.parse().map_err(|e: url::ParseError| e.to_string())?)
    )
    .title(&profile.name)
    .inner_size(1200.0, 800.0)
    .data_directory(profile_data_dir);
    
    // Define user-agent customizado se especificado
    if let Some(ua) = &profile.user_agent {
        builder = builder.user_agent(ua);
    }
    
    let webview = builder.build().map_err(|e| e.to_string())?;
    
    // Armazena a referência da webview
    let mut webviews = state.webviews.lock().unwrap();
    webviews.insert(uuid.clone(), webview);
    
    Ok(())
}

/// Oculta uma webview específica
#[tauri::command]
pub async fn hide_webview(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<(), String> {
    let webviews = state.webviews.lock().unwrap();
    
    if let Some(webview) = webviews.get(&uuid) {
        webview.hide().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// Fecha uma webview específica
#[tauri::command]
pub async fn close_webview(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<(), String> {
    let mut webviews = state.webviews.lock().unwrap();
    
    if let Some(webview) = webviews.remove(&uuid) {
        webview.close().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
