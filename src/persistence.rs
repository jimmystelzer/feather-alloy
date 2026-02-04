use std::fs;
use std::path::PathBuf;
use crate::profile::{WebProfile, AppSettings};

/// Retorna o caminho do arquivo de configuração de perfis
pub fn get_profiles_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("feather-alloy");
    
    // Criar diretório se não existir
    fs::create_dir_all(&config_dir)?;
    
    Ok(config_dir.join("profiles.json"))
}

/// Salva a lista de perfis em arquivo JSON
pub fn save_profiles(profiles: &[WebProfile]) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = get_profiles_file_path()?;
    let json = serde_json::to_string_pretty(profiles)?;
    
    fs::write(&file_path, json)?;
    println!("[Persistence] Profiles saved to: {:?}", file_path);
    
    Ok(())
}

/// Carrega a lista de perfis do arquivo JSON
pub fn load_profiles() -> Result<Vec<WebProfile>, Box<dyn std::error::Error>> {
    let file_path = get_profiles_file_path()?;
    
    // Se o arquivo não existir, retornar lista vazia
    if !file_path.exists() {
        println!("[Persistence] No profiles file found, starting with empty list");
        return Ok(Vec::new());
    }
    
    let json = fs::read_to_string(&file_path)?;
    
    // Tentar fazer parse do JSON
    match serde_json::from_str::<Vec<WebProfile>>(&json) {
        Ok(profiles) => {
            println!("[Persistence] Loaded {} profiles from: {:?}", 
                     profiles.len(), file_path);
            Ok(profiles)
        }
        Err(e) => {
            eprintln!("[Persistence] Failed to parse profiles file: {}", e);
            eprintln!("[Persistence] Starting with empty list");
            Ok(Vec::new())
        }
    }
}

/// Retorna o caminho do arquivo de configurações
pub fn get_settings_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("feather-alloy");
    
    // Criar diretório se não existir
    fs::create_dir_all(&config_dir)?;
    
    Ok(config_dir.join("settings.json"))
}

/// Salva as configurações em arquivo JSON
pub fn save_settings(settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = get_settings_file_path()?;
    let json = serde_json::to_string_pretty(settings)?;
    
    fs::write(&file_path, json)?;
    println!("[Persistence] Settings saved to: {:?}", file_path);
    
    Ok(())
}

/// Carrega as configurações do arquivo JSON
pub fn load_settings() -> Result<AppSettings, Box<dyn std::error::Error>> {
    let file_path = get_settings_file_path()?;
    
    if !file_path.exists() {
        println!("[Persistence] No settings file found, using defaults");
        return Ok(AppSettings::default());
    }
    
    let json = fs::read_to_string(&file_path)?;
    
    match serde_json::from_str::<AppSettings>(&json) {
        Ok(settings) => {
            println!("[Persistence] Loaded settings from: {:?}", file_path);
            Ok(settings)
        }
        Err(e) => {
            eprintln!("[Persistence] Failed to parse settings file: {}, using defaults", e);
            Ok(AppSettings::default())
        }
    }
}

/// Deleta os dados de um perfil do disco
pub fn delete_profile_data(uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profile_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("feather-alloy")
        .join("profiles")
        .join(uuid);
    
    if profile_dir.exists() {
        fs::remove_dir_all(&profile_dir)?;
        println!("[Persistence] Deleted profile data: {:?}", profile_dir);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_profiles() {
        let test_profile = WebProfile::new(
            "Test Service".to_string(),
            "https://example.com".to_string(),
            None,
            None,
        );
        
        let profiles = vec![test_profile];
        
        // Salvar
        save_profiles(&profiles).expect("Failed to save profiles");
        
        // Carregar
        let loaded = load_profiles().expect("Failed to load profiles");
        
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Test Service");
        assert_eq!(loaded[0].url, "https://example.com");
    }
}
