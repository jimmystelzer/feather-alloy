use std::sync::Mutex;
use tao::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
};
use wry::{Rect, WebView, WebViewBuilder};

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
    _window: Window,
    toolbar_webview: WebView,
    content_webview: WebView,
    state: AppState,
}

impl WindowManager {
    pub fn new(
        event_loop: &EventLoop<AppEvent>,
        state: AppState,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Criar janela principal
        let window = WindowBuilder::new()
            .with_title("Feather Alloy")
            .with_inner_size(LogicalSize::new(1200.0, 800.0))
            .with_min_inner_size(LogicalSize::new(800.0, 600.0))
            .build(event_loop)?;

        let window_size = window.inner_size();
        
        // Criar proxy para enviar eventos customizados
        let proxy = event_loop.create_proxy();
        
        // Criar Toolbar Webview com IPC handler
        let toolbar_webview = Self::create_toolbar_webview(
            &window,
            window_size,
            state.clone(),
            proxy.clone(),
        )?;
        
        // Criar Content Webview com IPC handler
        let content_webview = Self::create_content_webview(
            &window,
            window_size,
            state.clone(),
            proxy,
        )?;

        Ok(Self {
            _window: window,
            toolbar_webview,
            content_webview,
            state,
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
        
        // Script de inicialização para garantir que window.ipc esteja disponível
        let init_script = r#"
            console.log('[Toolbar] Initialization script running');
            console.log('[Toolbar] window.ipc available:', typeof window.ipc !== 'undefined');
            
            // Adicionar listener para debug
            window.addEventListener('error', function(e) {
                console.error('[Toolbar] Error:', e.message, e.filename, e.lineno);
            });
            
            // Log quando o DOM estiver pronto
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', function() {
                    console.log('[Toolbar] DOM loaded, ipc available:', typeof window.ipc !== 'undefined');
                });
            } else {
                console.log('[Toolbar] DOM already loaded, ipc available:', typeof window.ipc !== 'undefined');
            }
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
                            // Enviar lista de perfis de volta
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

    fn create_content_webview(
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

        // Script de inicialização para content webview
        let init_script = r#"
            console.log('[Content] Initialization script running');
            console.log('[Content] window.ipc available:', typeof window.ipc !== 'undefined');
            
            window.addEventListener('error', function(e) {
                console.error('[Content] Error:', e.message, e.filename, e.lineno);
            });
        "#;

        let webview = WebViewBuilder::new()
            .with_bounds(content_bounds)
            .with_initialization_script(init_script)
            .with_html(welcome_html)
            .with_ipc_handler(move |request: http::Request<String>| {
                let body = request.body();
                
                if let Ok(message) = IpcMessage::from_json(body) {
                    println!("[Content IPC] Received: {:?}", message);
                    
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
                                println!("[Content] Response: {:?}", response);
                            }
                        }
                    }
                }
            })
            .build_as_child(window)?;

        Ok(webview)
    }

    pub fn show_add_profile_form(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let add_profile_html = include_str!("../ui/content/add-profile.html");
        self.content_webview.load_html(add_profile_html)?;
        println!("[WindowManager] Showing add profile form");
        Ok(())
    }

    pub fn show_welcome(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let welcome_html = include_str!("../ui/content/index.html");
        self.content_webview.load_html(welcome_html)?;
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
            self.content_webview.load_url(&url)?;
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
        
        // Atualizar toolbar
        self.update_toolbar_profiles()?;
        
        // Voltar para tela de boas-vindas
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
        let _ = self.content_webview.set_bounds(content_bounds);
    }

    pub fn run(mut self, event_loop: EventLoop<AppEvent>) -> ! {
        // Carregar perfis iniciais na toolbar
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
