use tauri::Manager;

mod server;
mod profile;
mod commands;

use profile::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Iniciar servidor HTTP para arquivos estáticos
    #[cfg(debug_assertions)]
    server::serve_static_files();

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::add_profile,
            commands::get_profiles,
            commands::remove_profile,
            commands::show_webview,
            commands::hide_webview,
            commands::close_webview,
        ])
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
