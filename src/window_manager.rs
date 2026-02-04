use std::path::PathBuf;
use std::collections::HashMap;
use tao::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
};
use wry::{Rect, WebView, WebViewBuilder, WebContext};

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
    ShowProfile {
        uuid: String,
    },
    UpdateToolbar,
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
}

impl WindowManager {
    pub fn new(
        event_loop: &EventLoop<AppEvent>,
        state: AppState,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let window = WindowBuilder::new()
            .with_title("Feather Alloy")
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

        Ok(Self {
            window,
            toolbar_webview,
            welcome_webview,
            profile_webviews: HashMap::new(),
            state,
            current_profile_uuid: None,
            proxy,
            web_contexts: HashMap::new(),
        })
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
        
        let init_script = r#"
            console.log('[Toolbar] Initialization script running');
            console.log('[Toolbar] window.ipc available:', typeof window.ipc !== 'undefined');
        "#;
        
        let webview = WebViewBuilder::new()
            .with_bounds(toolbar_bounds)
            .with_initialization_script(init_script)
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
                        IpcMessage::GetProfiles => {
                            let _ = proxy.send_event(AppEvent::UpdateToolbar);
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

        let init_script = r#"
            console.log('[Welcome] Initialization script running');
        "#;

        let webview = WebViewBuilder::new()
            .with_bounds(content_bounds)
            .with_initialization_script(init_script)
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
                        IpcMessage::CancelAddProfile => {
                            let _ = proxy.send_event(AppEvent::CancelAddProfile);
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

        let init_script = r#"
            console.log('[Profile] Initialization script running');
        "#;

        let webview = WebViewBuilder::new_with_web_context(web_context)
            .with_bounds(content_bounds)
            .with_initialization_script(init_script)
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
        let profiles = self.state.lock().unwrap();
        
        if let Some(profile) = profiles.iter().find(|p| p.uuid == uuid) {
            let url = profile.url.clone();
            let name = profile.name.clone();
            drop(profiles);
            
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
        
        let mut profiles = self.state.lock().unwrap();
        profiles.push(profile.clone());
        drop(profiles);
        
        println!("[WindowManager] Profile added: {} ({})", name, url);
        
        self.update_toolbar_profiles()?;
        self.show_welcome()?;
        
        Ok(())
    }

    pub fn update_toolbar_profiles(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let profiles = self.state.lock().unwrap();
        let profiles_json = serde_json::to_string(&*profiles)?;
        drop(profiles);
        
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

            match event {
                Event::UserEvent(app_event) => {
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
                        AppEvent::ShowProfile { uuid } => {
                            let _ = self.navigate_to_profile(&uuid);
                        }
                        AppEvent::UpdateToolbar => {
                            let _ = self.update_toolbar_profiles();
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
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
