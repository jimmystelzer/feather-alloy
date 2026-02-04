pub mod profile;
pub mod ipc;
pub mod window_manager;
pub mod persistence;
pub mod context_menu;
pub mod favicon;

use profile::create_app_state;
use window_manager::{WindowManager, AppEvent};

#[cfg(target_os = "linux")]
use gtk;

pub fn run() {
    // Inicializar GTK no Linux
    #[cfg(target_os = "linux")]
    {
        gtk::init().expect("Failed to initialize GTK");
    }

    // Criar estado da aplicação
    let state = create_app_state();

    // Criar event loop com eventos customizados
    let event_loop = tao::event_loop::EventLoopBuilder::<AppEvent>::with_user_event().build();

    // Criar window manager com as duas webviews
    let window_manager = WindowManager::new(&event_loop, state)
        .expect("Failed to create window manager");

    // Executar event loop
    window_manager.run(event_loop);
}
