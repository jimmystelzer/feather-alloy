use tauri::Manager;

mod server;

// Comandos Tauri serão adicionados nas próximas fases

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
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
