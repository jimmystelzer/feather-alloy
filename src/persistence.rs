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


/// Salva o ícone do perfil na pasta de dados do perfil
pub fn save_profile_icon(uuid: &str, source_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let source = PathBuf::from(source_path);
    if !source.exists() {
        return Err(format!("Source icon not found: {}", source_path).into());
    }

    let extension = source.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    
    let profile_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("feather-alloy")
        .join("profiles")
        .join(uuid);

    // Ensure dir exists
    fs::create_dir_all(&profile_dir)?;

    let dest_filename = format!("icon.{}", extension);
    let dest_path = profile_dir.join(&dest_filename);

    fs::copy(&source, &dest_path)?;
    println!("[Persistence] Icon copied to: {:?}", dest_path);

    // Return relative path: profiles/{uuid}/{filename}
    let relative_path = format!("profiles/{}/{}", uuid, dest_filename);
    Ok(relative_path)
}

/// Remove o ícone do perfil se existir
pub fn delete_profile_icon(uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
    let profile_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("feather-alloy")
        .join("profiles")
        .join(uuid);

    if !profile_dir.exists() {
        return Ok(());
    }

    // List files and delete any that look like icons
    for entry in fs::read_dir(&profile_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(stem) = path.file_stem() {
            if stem == "icon" {
                fs::remove_file(&path)?;
                println!("[Persistence] Deleted icon: {:?}", path);
            }
        }
    }
    
    // Also delete favicon.ico if it exists
    let favicon = profile_dir.join("favicon.ico");
    if favicon.exists() {
        fs::remove_file(favicon)?;
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
