use std::path::PathBuf;
use std::collections::HashMap;
use tao::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
};
use wry::{Rect, WebView, WebViewBuilder, WebContext};
use tray_icon::TrayIconBuilder;

use crate::ipc::{IpcHandler, IpcMessage};
use crate::profile::AppState;

const TOOLBAR_WIDTH: f64 = 70.0;

// Eventos customizados para o event loop
#[derive(Debug, Clone)]
pub enum AppEvent {
    ShowAddProfileForm,
    CancelAddProfile,
    AddProfile {
        name: String,
        url: String,
        icon_path: Option<String>,
        user_agent: Option<String>,
    },
    UpdateProfile {
        uuid: String,
        name: String,
        url: String,
        icon_path: Option<String>,
        user_agent: Option<String>,
    },
    ShowProfile {
        uuid: String,
    },
    ShowProfileContextMenu {         uuid: String,         x: f64,         y: f64,     },
    ReloadProfile {
        uuid: String,
    },
    UpdateProfileIcon {
        uuid: String,
    },
    ShowEditProfile {
        uuid: String,
    },
    RemoveProfile {
        uuid: String,
    },
    UpdateToolbar,
    ShowWelcome,
    ShowSettings,
    UpdateSettings {
        minimize_on_open: bool,
        minimize_on_close: bool,
        hide_on_close: bool,
        enable_tray: bool,
    },
    ToggleWindow,
    Quit,
}

pub struct WindowManager {
    window: Window,
    toolbar_webview: WebView,
    welcome_webview: WebView,
    // WebViews por perfil (UUID -> WebView)
    profile_webviews: HashMap<String, WebView>,
    state: AppState,
    current_profile_uuid: Option<String>,
    proxy: EventLoopProxy<AppEvent>,
    // WebContexts por perfil
    web_contexts: HashMap<String, WebContext>,
    tray: Option<tray_icon::TrayIcon>,
    context_menu: Option<crate::context_menu::ProfileContextMenu>,
    context_menu_target_uuid: Option<String>,
}

impl WindowManager {
    pub fn new(
        event_loop: &EventLoop<AppEvent>,
        state: AppState,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Carregar perfis e configurações salvos
        let saved_profiles = crate::persistence::load_profiles()?;
        let saved_settings = crate::persistence::load_settings()?;
        
        {
            let mut data = state.lock().unwrap();
            data.profiles = saved_profiles;
            data.settings = saved_settings;
            println!("[WindowManager] Loaded {} profiles and settings from disk", data.profiles.len());
        }
        
        let icon = Self::load_icon().ok();
        
        let window = WindowBuilder::new()
            .with_title("Feather Alloy")
            .with_window_icon(icon)
            .with_inner_size(LogicalSize::new(1200.0, 800.0))
            .with_min_inner_size(LogicalSize::new(800.0, 600.0))
            .build(event_loop)?;

        let window_size = window.inner_size();
        let proxy = event_loop.create_proxy();
        
        let toolbar_webview = Self::create_toolbar_webview(
            &window,
            window_size,
            state.clone(),
            proxy.clone(),
        )?;
        
        let welcome_webview = Self::create_welcome_webview(
            &window,
            window_size,
            state.clone(),
            proxy.clone(),
        )?;

        let mut manager = Self {
            window,
            toolbar_webview,
            welcome_webview,
            profile_webviews: HashMap::new(),
            state,
            current_profile_uuid: None,
            proxy: proxy.clone(),
            web_contexts: HashMap::new(),
            tray: None,
            context_menu: crate::context_menu::ProfileContextMenu::new().ok(),
            context_menu_target_uuid: None,
        };

        if manager.state.lock().unwrap().settings.enable_tray {
            match Self::setup_tray(proxy.clone()) {
                Ok(tray) => manager.tray = Some(tray),
                Err(e) => eprintln!("[WindowManager] Failed to setup tray: {}", e),
            }
        }

        // Minimizar janela ao abrir se configurado
        if manager.state.lock().unwrap().settings.minimize_on_open {
            println!("[WindowManager] minimize_on_open is enabled, minimizing window");
            manager.window.set_minimized(true);
        }

        Ok(manager)
    }

    fn setup_tray(proxy: EventLoopProxy<AppEvent>) -> Result<tray_icon::TrayIcon, Box<dyn std::error::Error>> {
        use tray_icon::menu::{Menu, MenuItem};
        
        println!("[WindowManager] Loading tray icon (32x32.png)...");
        let icon_bytes = include_bytes!("../icons/32x32.png");
        let image = image::load_from_memory(icon_bytes)?.to_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let tray_icon = tray_icon::Icon::from_rgba(rgba, width, height)?;

        // Criar menu para o tray (necessário no Linux onde eventos de clique não funcionam)
        let menu = Menu::new();
        let toggle_item = MenuItem::new("Show/Hide", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        
        menu.append(&toggle_item)?;
        menu.append(&quit_item)?;

        // Armazenar os IDs antes de criar a closure
        let toggle_id = toggle_item.id().clone();
        let quit_id = quit_item.id().clone();

        // Configurar handler para eventos de menu
        let proxy_clone = proxy.clone();
        tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
            println!("[WindowManager] Menu event: {:?}", event);
            if event.id == toggle_id {
                let _ = proxy_clone.send_event(AppEvent::ToggleWindow);
            } else if event.id == quit_id {
                let _ = proxy_clone.send_event(AppEvent::Quit);
            }
        }));

        println!("[WindowManager] Building tray icon with menu");
        let tray = TrayIconBuilder::new()
            .with_icon(tray_icon)
            .with_tooltip("Feather Alloy")
            .with_title("Feather Alloy")
            .with_id("feather-alloy-tray")
            .with_menu(Box::new(menu))
            .build()?;
            
        println!("[WindowManager] Tray icon built successfully");
        Ok(tray)
    }

    fn load_icon() -> Result<tao::window::Icon, Box<dyn std::error::Error>> {
        let icon_bytes = include_bytes!("../icons/128x128.png");
        let image = image::load_from_memory(icon_bytes)?.to_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        tao::window::Icon::from_rgba(rgba, width, height).map_err(|e| e.into())
    }

    fn asset_protocol_handler(_id: wry::WebViewId, request: http::Request<Vec<u8>>) -> http::Response<std::borrow::Cow<'static, [u8]>> {
        let uri = request.uri();
        
        // Em WRY, o esquema asset://caminho/para/arquivo pode vir com o host sendo a primeira parte do caminho
        let mut path_str = if let Some(host) = uri.host() {
            format!("{}{}", host, uri.path())
        } else {
            uri.path().to_string()
        };
        
        // Remover barra inicial se existir para facilitar o join com o path relativo
        if path_str.starts_with('/') {
            path_str.remove(0);
        }

        let path = std::path::PathBuf::from(&path_str);
        // println!("[AssetProtocol] Requesting: {:?} (from URI: {})", path, uri);
        
        match std::fs::read(&path) {
            Ok(content) => {
                let mime_type = if path.extension().map(|e| e == "png").unwrap_or(false) {
                    "image/png"
                } else if path.extension().map(|e| e == "svg").unwrap_or(false) {
                    "image/svg+xml"
                } else if path.extension().map(|e| e == "ico").unwrap_or(false) {
                    "image/x-icon"
                } else if path.extension().map(|e| e == "html").unwrap_or(false) {
                    "text/html"
                } else if path.extension().map(|e| e == "css").unwrap_or(false) {
                    "text/css"
                } else if path.extension().map(|e| e == "js").unwrap_or(false) {
                    "text/javascript"
                } else {
                    "application/octet-stream"
                };
                
                http::Response::builder()
                    .header("Content-Type", mime_type)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(std::borrow::Cow::from(content))
                    .unwrap()
            }
            Err(e) => {
                eprintln!("[AssetProtocol] Failed to read asset {:?}: {}", path, e);
                http::Response::builder()
                    .status(404)
                    .body(std::borrow::Cow::from(Vec::new()))
                    .unwrap()
            }
        }
    }

    fn create_toolbar_webview(
        window: &Window,
        window_size: PhysicalSize<u32>,
        state: AppState,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Result<WebView, Box<dyn std::error::Error>> {
        let toolbar_bounds = Rect {
            position: PhysicalPosition::new(0, 0).into(),
            size: PhysicalSize::new(TOOLBAR_WIDTH as u32, window_size.height).into(),
        };

        let toolbar_html = include_str!("../ui/toolbar/index.html");
        
        let disable_context_menu = r#"
            document.addEventListener('contextmenu', function(e) {
                e.preventDefault();
                return false;
            }, false);
        "#;
        
        let init_script = r#"
            console.log('[Toolbar] Initialization script running');
            console.log('[Toolbar] window.ipc available:', typeof window.ipc !== 'undefined');
        "#;
        
        let webview = WebViewBuilder::new()
            .with_bounds(toolbar_bounds)
            .with_custom_protocol("asset".into(), Self::asset_protocol_handler)
            // Scripts de inicialização executados em ordem
            .with_initialization_script(disable_context_menu)
            .with_initialization_script(init_script)
            .with_devtools(false) // Desabilitar DevTools
            .with_html(toolbar_html)
            .with_ipc_handler(move |request: http::Request<String>| {
                let body = request.body();
                
                if let Ok(message) = IpcMessage::from_json(body) {
                    println!("[Toolbar IPC] Received: {:?}", message);
                    
                    match message {
                        IpcMessage::ShowAddProfileForm => {
                            let _ = proxy.send_event(AppEvent::ShowAddProfileForm);
                        }
                        IpcMessage::ShowProfile { uuid } => {
                            let _ = proxy.send_event(AppEvent::ShowProfile { uuid });
                        }
                        IpcMessage::ShowWelcome => {
                            let _ = proxy.send_event(AppEvent::ShowWelcome);
                        }
                        IpcMessage::GetProfiles => {
                            let _ = proxy.send_event(AppEvent::UpdateToolbar);
                        }
                        IpcMessage::ShowSettings => {
                            let _ = proxy.send_event(AppEvent::ShowSettings);
                        }
                        IpcMessage::ShowProfileContextMenu { uuid, x, y } => {
                            let _ = proxy.send_event(AppEvent::ShowProfileContextMenu { uuid, x, y });
                        }
                        IpcMessage::ReloadProfile { uuid } => {
                            let _ = proxy.send_event(AppEvent::ReloadProfile { uuid });
                        }
                        IpcMessage::UpdateProfileIcon { uuid } => {
                            let _ = proxy.send_event(AppEvent::UpdateProfileIcon { uuid });
                        }
                        IpcMessage::EditProfile { uuid } => {
                            let _ = proxy.send_event(AppEvent::ShowEditProfile { uuid });
                        }
                        IpcMessage::RemoveProfile { uuid } => {
                            let _ = proxy.send_event(AppEvent::RemoveProfile { uuid });
                        }

                        _ => {
                            let handler = IpcHandler::new(state.clone());
                            if let Some(response) = handler.handle_message(message) {
                                println!("[Toolbar] Response: {:?}", response);
                            }
                        }
                    }
                }
            })
            .build_as_child(window)?;

        Ok(webview)
    }

    fn create_welcome_webview(
        window: &Window,
        window_size: PhysicalSize<u32>,
        state: AppState,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Result<WebView, Box<dyn std::error::Error>> {
        let content_bounds = Rect {
            position: PhysicalPosition::new(TOOLBAR_WIDTH as i32, 0).into(),
            size: PhysicalSize::new(
                window_size.width - TOOLBAR_WIDTH as u32,
                window_size.height,
            ).into(),
        };

        let welcome_html = include_str!("../ui/content/index.html");

        let disable_context_menu = r#"
            document.addEventListener('contextmenu', function(e) {
                e.preventDefault();
                return false;
            }, false);
        "#;
        
        let init_script = r#"
            console.log('[Welcome] Initialization script running');
        "#;

        let webview = WebViewBuilder::new()
            .with_bounds(content_bounds)
            .with_custom_protocol("asset".into(), Self::asset_protocol_handler)
            // Scripts executados em ordem
            .with_initialization_script(disable_context_menu)
            .with_initialization_script(init_script)
            .with_devtools(false) // Desabilitar DevTools
            .with_html(welcome_html)
            .with_ipc_handler(move |request: http::Request<String>| {
                let body = request.body();
                
                if let Ok(message) = IpcMessage::from_json(body) {
                    println!("[Welcome IPC] Received: {:?}", message);
                    
                    match message {
                        IpcMessage::AddProfile { name, url, icon_path, user_agent } => {
                            let _ = proxy.send_event(AppEvent::AddProfile {
                                name,
                                url,
                                icon_path,
                                user_agent,
                            });
                        }
                        IpcMessage::UpdateProfile { uuid, name, url, icon_path, user_agent } => {
                            let _ = proxy.send_event(AppEvent::UpdateProfile {
                                uuid,
                                name,
                                url,
                                icon_path,
                                user_agent,
                            });
                        }
                        IpcMessage::UpdateSettings { minimize_on_open, minimize_on_close, hide_on_close, enable_tray } => {
                            let _ = proxy.send_event(AppEvent::UpdateSettings {
                                minimize_on_open,
                                minimize_on_close,
                                hide_on_close,
                                enable_tray,
                            });
                        }
                        IpcMessage::CancelAddProfile => {
                            let _ = proxy.send_event(AppEvent::CancelAddProfile);
                        }
                        IpcMessage::ShowWelcome => {
                            println!("[Welcome IPC] ShowWelcome received, sending ShowWelcome event");
                            let _ = proxy.send_event(AppEvent::ShowWelcome);
                        }
                        IpcMessage::QuitApp => {
                            println!("[Welcome IPC] QuitApp received, sending Quit event");
                            let _ = proxy.send_event(AppEvent::Quit);
                        }
                        _ => {
                            let handler = IpcHandler::new(state.clone());
                            if let Some(response) = handler.handle_message(message) {
                                println!("[Welcome] Response: {:?}", response);
                            }
                        }
                    }
                }
            })
            .build_as_child(window)?;

        Ok(webview)
    }

    fn create_profile_webview(
        window: &Window,
        window_size: PhysicalSize<u32>,
        web_context: &mut WebContext,
        url: &str,
    ) -> Result<WebView, Box<dyn std::error::Error>> {
        let content_bounds = Rect {
            position: PhysicalPosition::new(TOOLBAR_WIDTH as i32, 0).into(),
            size: PhysicalSize::new(
                window_size.width - TOOLBAR_WIDTH as u32,
                window_size.height,
            ).into(),
        };

        let disable_context_menu = r#"
            document.addEventListener('contextmenu', function(e) {
                e.preventDefault();
                return false;
            }, false);
        "#;
        
        let init_script = r#"
            console.log('[Profile] Initialization script running');
        "#;

        let webview = WebViewBuilder::new_with_web_context(web_context)
            .with_bounds(content_bounds)
            // Scripts executados em ordem
            .with_initialization_script(disable_context_menu)
            .with_initialization_script(init_script)
            .with_devtools(false) // Desabilitar DevTools
            .with_url(url)
            .with_visible(false) // Iniciar oculto
            .build_as_child(window)?;

        Ok(webview)
    }

    fn get_or_create_web_context(&mut self, uuid: &str) -> Result<&mut WebContext, Box<dyn std::error::Error>> {
        if !self.web_contexts.contains_key(uuid) {
            let data_dir = Self::get_profile_data_directory(uuid)?;
            println!("[WindowManager] Creating WebContext for profile {} with data directory: {:?}", uuid, data_dir);
            
            let web_context = WebContext::new(Some(data_dir));
            self.web_contexts.insert(uuid.to_string(), web_context);
        }
        
        Ok(self.web_contexts.get_mut(uuid).unwrap())
    }

    fn get_profile_data_directory(uuid: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let app_data_dir = dirs::data_dir()
            .ok_or("Failed to get data directory")?
            .join("feather-alloy")
            .join("profiles")
            .join(uuid);

        std::fs::create_dir_all(&app_data_dir)?;
        Ok(app_data_dir)
    }

    pub fn show_add_profile_form(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ocultar todas as webviews de perfis
        for webview in self.profile_webviews.values() {
            webview.set_visible(false)?;
        }
        
        let add_profile_html = include_str!("../ui/content/add-profile.html");
        self.welcome_webview.load_html(add_profile_html)?;
        self.welcome_webview.set_visible(true)?;
        
        self.current_profile_uuid = None;
        println!("[WindowManager] Showing add profile form");
        Ok(())
    }

    pub fn show_welcome(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ocultar todas as webviews de perfis
        for webview in self.profile_webviews.values() {
            webview.set_visible(false)?;
        }
        
        let welcome_html = include_str!("../ui/content/index.html");
        self.welcome_webview.load_html(welcome_html)?;
        self.welcome_webview.set_visible(true)?;
        
        self.current_profile_uuid = None;
        println!("[WindowManager] Showing welcome screen");
        Ok(())
    }

    pub fn navigate_to_profile(&mut self, uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.state.lock().unwrap();
        
        if let Some(profile) = data.profiles.iter().find(|p| p.uuid == uuid) {
            let url = profile.url.clone();
            let name = profile.name.clone();
            drop(data);
            
            println!("[WindowManager] Navigating to profile: {} ({})", name, url);
            
            // Ocultar welcome webview
            self.welcome_webview.set_visible(false)?;
            
            // Ocultar todas as outras webviews de perfis
            for (other_uuid, webview) in &self.profile_webviews {
                if other_uuid != uuid {
                    webview.set_visible(false)?;
                }
            }
            
            // Se a webview do perfil já existe, apenas mostrar
            if self.profile_webviews.contains_key(uuid) {
                println!("[WindowManager] Showing existing webview for profile {}", uuid);
                self.profile_webviews.get(uuid).unwrap().set_visible(true)?;
            } else {
                // Criar nova webview para este perfil
                println!("[WindowManager] Creating new webview for profile {}", uuid);
                
                let window_size = self.window.inner_size();
                let window_ptr = &self.window as *const Window;
                
                let web_context = self.get_or_create_web_context(uuid)?;
                let window_ref = unsafe { &*window_ptr };
                
                let webview = Self::create_profile_webview(
                    window_ref,
                    window_size,
                    web_context,
                    &url,
                )?;
                
                // Mostrar a nova webview
                webview.set_visible(true)?;
                
                self.profile_webviews.insert(uuid.to_string(), webview);
            }
            
            self.current_profile_uuid = Some(uuid.to_string());
            Ok(())
        } else {
            Err("Perfil não encontrado".into())
        }
    }

    pub fn add_profile(
        &mut self,
        name: String,
        url: String,
        icon_path: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let profile = crate::profile::WebProfile::new(name.clone(), url.clone(), icon_path, user_agent);
        
        let mut data = self.state.lock().unwrap();
        data.profiles.push(profile.clone());
        
        // Salvar perfis em disco
        if let Err(e) = crate::persistence::save_profiles(&data.profiles) {
            eprintln!("[WindowManager] Failed to save profiles: {}", e);
        }
        
        drop(data);
        
        println!("[WindowManager] Profile added: {} ({})", name, url);
        
        self.update_toolbar_profiles()?;
        self.show_welcome()?;
        
        Ok(())
    }

    pub fn update_profile(
        &mut self,
        uuid: String,
        name: String,
        url: String,
        icon_path: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.state.lock().unwrap();
        
        if let Some(profile) = data.profiles.iter_mut().find(|p| p.uuid == uuid) {
            profile.name = name.clone();
            profile.url = url.clone();
            profile.icon_path = icon_path;
            profile.user_agent = user_agent;
            
            // Salvar perfis em disco
            if let Err(e) = crate::persistence::save_profiles(&data.profiles) {
                eprintln!("[WindowManager] Failed to save profiles: {}", e);
            }
            
            drop(data);
            
            println!("[WindowManager] Profile updated: {} ({})", name, url);
            
            self.update_toolbar_profiles()?;
            self.show_welcome()?;
            
            Ok(())
        } else {
            drop(data);
            Err("Perfil não encontrado".into())
        }
    }

    pub fn reload_profile(&mut self, uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(webview) = self.profile_webviews.get(uuid) {
            // Recarregar a webview
            let data = self.state.lock().unwrap();
            if let Some(profile) = data.profiles.iter().find(|p| p.uuid == uuid) {
                let url = profile.url.clone();
                drop(data);
                
                webview.load_url(&url)?;
                println!("[WindowManager] Profile {} reloaded", uuid);
                Ok(())
            } else {
                drop(data);
                Err("Perfil não encontrado".into())
            }
        } else {
            Err("WebView do perfil não encontrada".into())
        }
    }

    pub fn update_profile_icon(&mut self, uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implementar busca de favicon
        println!("[WindowManager] Update icon for profile {}", uuid);
        Ok(())
    }

    pub fn show_edit_profile(&mut self, uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.state.lock().unwrap();
        
        if let Some(profile) = data.profiles.iter().find(|p| p.uuid == uuid) {
            let profile_json = serde_json::to_string(profile)?;
            drop(data);
            
            // Ocultar todas as webviews de perfis
            for webview in self.profile_webviews.values() {
                webview.set_visible(false)?;
            }
            
            let edit_profile_html = include_str!("../ui/content/edit-profile.html");
            
            // Injetar dados diretamente no HTML antes de carregar
            // Isso garante que os dados estejam disponíveis imediatamente
            let html_with_data = edit_profile_html.replace(
                "</body>",
                &format!(
                    r#"<script>
                    console.log('[EditProfile] Injected data script running');
                    window.__PROFILE_DATA__ = {};
                    if (window.loadProfileData) {{
                        console.log('[EditProfile] Calling loadProfileData immediately');
                        window.loadProfileData(window.__PROFILE_DATA__);
                    }} else {{
                        document.addEventListener('DOMContentLoaded', function() {{
                            console.log('[EditProfile] DOMContentLoaded, calling loadProfileData');
                            if (window.loadProfileData) {{
                                window.loadProfileData(window.__PROFILE_DATA__);
                            }}
                        }});
                    }}
                    </script></body>"#,
                    profile_json
                )
            );
            
            self.welcome_webview.load_html(&html_with_data)?;
            self.welcome_webview.set_visible(true)?;
            
            self.current_profile_uuid = None;
            println!("[WindowManager] Showing edit profile form for {}", uuid);
            Ok(())
        } else {
            drop(data);
            Err("Perfil não encontrado".into())
        }
    }

    pub fn remove_profile(&mut self, uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Remover webview se existir
        if let Some(webview) = self.profile_webviews.remove(uuid) {
            drop(webview);
        }
        
        // Remover web context
        self.web_contexts.remove(uuid);
        
        // Remover do estado
        let mut data = self.state.lock().unwrap();
        data.profiles.retain(|p| p.uuid != uuid);
        
        // Salvar perfis em disco
        if let Err(e) = crate::persistence::save_profiles(&data.profiles) {
            eprintln!("[WindowManager] Failed to save profiles: {}", e);
        }
        
        drop(data);
        
        // Limpar dados do perfil do disco
        if let Err(e) = crate::persistence::delete_profile_data(uuid) {
            eprintln!("[WindowManager] Failed to delete profile data: {}", e);
        }
        
        println!("[WindowManager] Profile {} removed", uuid);
        
        self.update_toolbar_profiles()?;
        self.show_welcome()?;
        
        Ok(())
    }

    pub fn show_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ocultar todas as webviews de perfis
        for webview in self.profile_webviews.values() {
            webview.set_visible(false)?;
        }
        
        // Obter dados das configurações
        let data = self.state.lock().unwrap();
        let settings_json = serde_json::to_string(&data.settings)?;
        drop(data);
        
        println!("[WindowManager] Loading settings with data: {}", settings_json);
        
        // Carregar HTML e injetar dados diretamente
        let settings_html = include_str!("../ui/content/settings.html");
        
        // Adicionar script de inicialização com os dados no HTML
        let html_with_data = settings_html.replace(
            "</body>",
            &format!(
                r#"<script>
                console.log('[Settings] Injected data script running');
                window.__SETTINGS_DATA__ = {};
                if (window.loadSettings) {{
                    console.log('[Settings] Calling loadSettings immediately');
                    window.loadSettings(window.__SETTINGS_DATA__);
                }} else {{
                    console.log('[Settings] loadSettings not ready, will retry');
                    document.addEventListener('DOMContentLoaded', function() {{
                        console.log('[Settings] DOMContentLoaded, calling loadSettings');
                        if (window.loadSettings) {{
                            window.loadSettings(window.__SETTINGS_DATA__);
                        }}
                    }});
                }}
                </script></body>"#,
                settings_json
            )
        );
        
        self.welcome_webview.load_html(&html_with_data)?;
        self.welcome_webview.set_visible(true)?;
        
        self.current_profile_uuid = None;
        println!("[WindowManager] Settings screen loaded");
        Ok(())
    }

    pub fn update_settings(
        &mut self,
        minimize_on_open: bool,
        minimize_on_close: bool,
        hide_on_close: bool,
        enable_tray: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.state.lock().unwrap();
        data.settings.minimize_on_open = minimize_on_open;
        data.settings.minimize_on_close = minimize_on_close;
        data.settings.hide_on_close = hide_on_close;
        data.settings.enable_tray = enable_tray;
        
        // Salvar configurações em disco
        if let Err(e) = crate::persistence::save_settings(&data.settings) {
            eprintln!("[WindowManager] Failed to save settings: {}", e);
        }
        
        drop(data);
        
        println!("[WindowManager] Settings updated");
        
        self.show_welcome()?;
        
        Ok(())
    }

    pub fn update_toolbar_profiles(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.state.lock().unwrap();
        let profiles_json = serde_json::to_string(&data.profiles)?;
        drop(data);
        
        let script = format!(
            "if (window.handleProfilesUpdate) {{ window.handleProfilesUpdate({}); }}",
            profiles_json
        );
        
        self.toolbar_webview.evaluate_script(&script)?;
        println!("[WindowManager] Toolbar profiles updated");
        Ok(())
    }

    pub fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        let toolbar_bounds = Rect {
            position: PhysicalPosition::new(0, 0).into(),
            size: PhysicalSize::new(TOOLBAR_WIDTH as u32, new_size.height).into(),
        };
        let _ = self.toolbar_webview.set_bounds(toolbar_bounds);

        let content_bounds = Rect {
            position: PhysicalPosition::new(TOOLBAR_WIDTH as i32, 0).into(),
            size: PhysicalSize::new(
                new_size.width - TOOLBAR_WIDTH as u32,
                new_size.height,
            ).into(),
        };
        
        let _ = self.welcome_webview.set_bounds(content_bounds);
        
        for webview in self.profile_webviews.values() {
            let _ = webview.set_bounds(content_bounds);
        }
    }

    pub fn run(mut self, event_loop: EventLoop<AppEvent>) -> ! {
        let _ = self.update_toolbar_profiles();

        event_loop.run(move |event, _elwt, control_flow| {
            *control_flow = ControlFlow::Wait;

            #[cfg(target_os = "linux")]
            {
                while gtk::events_pending() {
                    gtk::main_iteration_do(false);
                }
            }

            // Processar eventos do menu nativo
            if let Ok(event) = muda::MenuEvent::receiver().try_recv() {
                if let Some(context_menu) = &self.context_menu {
                    if let Some(action) = context_menu.get_action(&event) {
                        // Precisamos saber qual UUID foi clicado.
                        // Como o menu é modal (ou quase), podemos guardar o UUID temporariamente no WindowManager
                        // quando abrimos o menu.
                        // Mas 'get_action' só retorna a ação.
                        // Vamos precisar de um campo 'context_menu_target_uuid' no WindowManager.
                        // Ou simplificar: assumir que a ação se aplica ao perfil que abriu o menu.
                        // Mas espera, o menu é assíncrono?
                        // O receiver recebe o evento quando clicado.
                        // Se guardarmos o UUID alvo quando abrimos o menu, podemos usar aqui.
                        
                        if let Some(uuid) = &self.context_menu_target_uuid {
                             match action {
                                crate::context_menu::ProfileMenuAction::Reload => {
                                    let _ = self.proxy.send_event(AppEvent::ReloadProfile { uuid: uuid.clone() });
                                }
                                crate::context_menu::ProfileMenuAction::UpdateIcon => {
                                    let _ = self.proxy.send_event(AppEvent::UpdateProfileIcon { uuid: uuid.clone() });
                                }
                                crate::context_menu::ProfileMenuAction::Edit => {
                                    let _ = self.proxy.send_event(AppEvent::ShowEditProfile { uuid: uuid.clone() });
                                }
                                crate::context_menu::ProfileMenuAction::Remove => {
                                    let _ = self.proxy.send_event(AppEvent::RemoveProfile { uuid: uuid.clone() });
                                }
                            }
                        }
                    }
                }
            }

            match event {
                Event::UserEvent(app_event) => {
                    println!("[WindowManager] >>> RECEIVED USER EVENT: {:?}", app_event);
                    match app_event {
                        AppEvent::ShowAddProfileForm => {
                            let _ = self.show_add_profile_form();
                        }
                        AppEvent::CancelAddProfile => {
                            let _ = self.show_welcome();
                        }
                        AppEvent::AddProfile { name, url, icon_path, user_agent } => {
                            let _ = self.add_profile(name, url, icon_path, user_agent);
                        }
                        AppEvent::UpdateProfile { uuid, name, url, icon_path, user_agent } => {
                            let _ = self.update_profile(uuid, name, url, icon_path, user_agent);
                        }
                        AppEvent::ShowProfile { uuid } => {
                            let _ = self.navigate_to_profile(&uuid);
                        }
                        AppEvent::ReloadProfile { uuid } => {
                            let _ = self.reload_profile(&uuid);
                        }
                        AppEvent::UpdateProfileIcon { uuid } => {
                            let _ = self.update_profile_icon(&uuid);
                        }
                        AppEvent::ShowEditProfile { uuid } => {
                            let _ = self.show_edit_profile(&uuid);
                        }
                        AppEvent::RemoveProfile { uuid } => {
                            let _ = self.remove_profile(&uuid);
                        }
                        AppEvent::UpdateToolbar => {
                            let _ = self.update_toolbar_profiles();
                        }
                        AppEvent::ShowWelcome => {
                            let _ = self.show_welcome();
                        }
                        AppEvent::ShowSettings => {
                            let _ = self.show_settings();
                        }
                        AppEvent::UpdateSettings { minimize_on_open, minimize_on_close, hide_on_close, enable_tray } => {
                            let _ = self.update_settings(minimize_on_open, minimize_on_close, hide_on_close, enable_tray);
                        }
                        AppEvent::ToggleWindow => {
                            println!("[WindowManager] >>> TOGGLE WINDOW EVENT");
                            let is_visible = self.window.is_visible();
                            self.window.set_visible(!is_visible);
                            if !is_visible { 
                                self.window.set_focus(); 
                            }
                        }
                        AppEvent::ShowProfileContextMenu { uuid, x, y } => {
                            self.context_menu_target_uuid = Some(uuid);
                            if let Some(context_menu) = &self.context_menu {
                                let _ = context_menu.show_at(&self.window, x, y);
                            }
                        }
                        AppEvent::Quit => {
                            println!("[WindowManager] >>> QUIT EVENT");
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    let settings = self.state.lock().unwrap().settings.clone();
                    
                    if settings.hide_on_close {
                        self.window.set_visible(false);
                    } else if settings.minimize_on_close {
                        self.window.set_minimized(true);
                    } else {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    self.handle_resize(new_size);
                }
                _ => {}
            }
        })
    }
}
