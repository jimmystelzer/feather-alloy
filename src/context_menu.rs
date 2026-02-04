use muda::{Menu, MenuItem, MenuEvent, PredefinedMenuItem, ContextMenu};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileMenuAction {
    Reload,
    UpdateIcon,
    Edit,
    Remove,
}

pub struct ProfileContextMenu {
    menu: Menu,
    reload_item: MenuItem,
    update_icon_item: MenuItem,
    edit_item: MenuItem,
    remove_item: MenuItem,
}

impl ProfileContextMenu {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();
        
        let reload_item = MenuItem::new("🔄 Atualizar conteúdo", true, None);
        let update_icon_item = MenuItem::new("🎨 Atualizar ícone", true, None);
        let edit_item = MenuItem::new("✏️ Editar perfil", true, None);
        let remove_item = MenuItem::new("🗑️ Remover perfil", true, None);
        
        menu.append(&reload_item)?;
        menu.append(&update_icon_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&edit_item)?;
        menu.append(&remove_item)?;
        
        Ok(Self {
            menu,
            reload_item,
            update_icon_item,
            edit_item,
            remove_item,
        })
    }
    
    pub fn show_at(&self, window: &tao::window::Window, x: f64, y: f64) -> Result<(), Box<dyn std::error::Error>> {
        let position = muda::dpi::Position::Logical((x, y).into());
        
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            self.menu.show_context_menu_for_hwnd(window.hwnd() as _, Some(position));
        }
        
        #[cfg(target_os = "linux")]
        {
            use tao::platform::unix::WindowExtUnix;
            use gtk::prelude::*; // Necessário para traits de conversão
            
            let gtk_window = window.gtk_window();
            let window_ref: &gtk::Window = gtk_window.as_ref();
            self.menu.show_context_menu_for_gtk_window(window_ref, Some(position));
        }
        
        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::WindowExtMacOS;
            self.menu.show_context_menu_for_nsview(window.ns_view() as _, Some(position));
        }
        
        Ok(())
    }
    
    /// Verifica qual item foi clicado e retorna a ação correspondente
    pub fn get_action(&self, event: &MenuEvent) -> Option<ProfileMenuAction> {
        if event.id == self.reload_item.id() {
            Some(ProfileMenuAction::Reload)
        } else if event.id == self.update_icon_item.id() {
            Some(ProfileMenuAction::UpdateIcon)
        } else if event.id == self.edit_item.id() {
            Some(ProfileMenuAction::Edit)
        } else if event.id == self.remove_item.id() {
            Some(ProfileMenuAction::Remove)
        } else {
            None
        }
    }
}
