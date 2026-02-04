use std::sync::{Arc, Mutex};
use tao::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use wry::{Rect, WebView, WebViewBuilder};

use crate::ipc::{IpcHandler, IpcMessage};
use crate::profile::AppState;

const TOOLBAR_WIDTH: f64 = 70.0;

pub struct WindowManager {
    _window: Window,
    toolbar_webview: WebView,
    content_webview: WebView,
    ipc_handler: Arc<Mutex<IpcHandler>>,
    current_profile_uuid: Arc<Mutex<Option<String>>>,
}

impl WindowManager {
    pub fn new(event_loop: &EventLoop<()>, state: AppState) -> Result<Self, Box<dyn std::error::Error>> {
        // Criar janela principal
        let window = WindowBuilder::new()
            .with_title("Feather Alloy")
            .with_inner_size(LogicalSize::new(1200.0, 800.0))
            .with_min_inner_size(LogicalSize::new(800.0, 600.0))
            .build(event_loop)?;

        let window_size = window.inner_size();
        
        // Criar IPC handler
        let ipc_handler = Arc::new(Mutex::new(IpcHandler::new(state.clone())));
        let current_profile_uuid = Arc::new(Mutex::new(None));

        // Criar Toolbar Webview (esquerda)
        let toolbar_webview = Self::create_toolbar_webview(&window, window_size, ipc_handler.clone(), current_profile_uuid.clone())?;
        
        // Criar Content Webview (direita)
        let content_webview = Self::create_content_webview(&window, window_size, ipc_handler.clone())?;

        Ok(Self {
            _window: window,
            toolbar_webview,
            content_webview,
            ipc_handler,
            current_profile_uuid,
        })
    }

    fn create_toolbar_webview(
        window: &Window,
        window_size: PhysicalSize<u32>,
        ipc_handler: Arc<Mutex<IpcHandler>>,
        current_profile_uuid: Arc<Mutex<Option<String>>>,
    ) -> Result<WebView, Box<dyn std::error::Error>> {
        let toolbar_bounds = Rect {
            position: PhysicalPosition::new(0, 0).into(),
            size: PhysicalSize::new(TOOLBAR_WIDTH as u32, window_size.height).into(),
        };

        // Preparar HTML da toolbar
        let toolbar_html = include_str!("../ui/toolbar/index.html");
        
        let ipc_handler_clone = ipc_handler.clone();
        let current_profile_clone = current_profile_uuid.clone();

        let webview = WebViewBuilder::new()
            .with_bounds(toolbar_bounds)
            .with_html(toolbar_html)
            .with_ipc_handler(move |request: http::Request<String>| {
                let body = request.body();
                
                if let Ok(message) = IpcMessage::from_json(body) {
                    let handler = ipc_handler_clone.lock().unwrap();
                    
                    // Processar mensagem ShowProfile para atualizar estado
                    if let IpcMessage::ShowProfile { ref uuid } = message {
                        let mut current = current_profile_clone.lock().unwrap();
                        *current = Some(uuid.clone());
                    }
                    
                    if let Some(response) = handler.handle_message(message) {
                        // TODO: Enviar resposta de volta para a webview
                        // Isso requer acesso à webview, que será implementado posteriormente
                        println!("Response: {:?}", response);
                    }
                }
            })
            .build_as_child(window)?;

        Ok(webview)
    }

    fn create_content_webview(
        window: &Window,
        window_size: PhysicalSize<u32>,
        _ipc_handler: Arc<Mutex<IpcHandler>>,
    ) -> Result<WebView, Box<dyn std::error::Error>> {
        let content_bounds = Rect {
            position: PhysicalPosition::new(TOOLBAR_WIDTH as i32, 0).into(),
            size: PhysicalSize::new(
                window_size.width - TOOLBAR_WIDTH as u32,
                window_size.height,
            ).into(),
        };

        // Preparar HTML de boas-vindas
        let welcome_html = include_str!("../ui/content/index.html");

        let webview = WebViewBuilder::new()
            .with_bounds(content_bounds)
            .with_html(welcome_html)
            .with_ipc_handler(move |request: http::Request<String>| {
                // Content webview pode enviar mensagens se necessário
                println!("Content webview message: {}", request.body());
            })
            .build_as_child(window)?;

        Ok(webview)
    }

    pub fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        // Atualizar bounds da toolbar webview
        let toolbar_bounds = Rect {
            position: PhysicalPosition::new(0, 0).into(),
            size: PhysicalSize::new(TOOLBAR_WIDTH as u32, new_size.height).into(),
        };
        let _ = self.toolbar_webview.set_bounds(toolbar_bounds);

        // Atualizar bounds da content webview
        let content_bounds = Rect {
            position: PhysicalPosition::new(TOOLBAR_WIDTH as i32, 0).into(),
            size: PhysicalSize::new(
                new_size.width - TOOLBAR_WIDTH as u32,
                new_size.height,
            ).into(),
        };
        let _ = self.content_webview.set_bounds(content_bounds);
    }

    pub fn navigate_to_profile(&mut self, uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let handler = self.ipc_handler.lock().unwrap();
        let message = IpcMessage::ShowProfile { uuid: uuid.to_string() };
        
        if let Some(IpcMessage::NavigateToUrl { url, user_agent: _ }) = handler.handle_message(message) {
            drop(handler);
            
            // Navegar a content webview para a URL do perfil
            self.content_webview.load_url(&url)?;
            
            // TODO: Aplicar user_agent se fornecido
            // Isso pode requerer recriar a webview com o user_agent configurado
            
            Ok(())
        } else {
            Err("Perfil não encontrado".into())
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        event_loop.run(move |event, _elwt, control_flow| {
            *control_flow = ControlFlow::Wait;

            #[cfg(target_os = "linux")]
            {
                while gtk::events_pending() {
                    gtk::main_iteration_do(false);
                }
            }

            match event {
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
