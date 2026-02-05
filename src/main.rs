// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Permitir Wayland no Linux, mas manter correções de renderização
    #[cfg(target_os = "linux")]
    {
        // Corrigir erro "Failed to create GBM buffer" (tela preta/flickering) em algumas GPUs/Drivers
        // Isso ainda é relevante para o WebKitGTK em muitos sistemas
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    feather_alloy_lib::run()
}
