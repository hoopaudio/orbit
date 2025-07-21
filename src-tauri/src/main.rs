// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[cfg(target_os = "macos")]
use tauri::{Runtime, WebviewWindow};

#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, ManagerExt, WebviewWindowExt, CollectionBehavior, StyleMask,
};

mod window;

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(OrbitPanel {
        config: {
            canBecomeKeyWindow: true,
            canBecomeMainWindow: false,
            becomesKeyOnlyIfNeeded: true,
            isFloatingPanel: true
        }
    })

    panel_event!(OrbitPanelEventHandler {})
}

#[tauri::command]
fn process_query(query: &str) -> String {
    if query.to_lowercase().contains("yeet") {
        return "Hello! I'm Orbit, your AI assistant. How can I help you today?".to_string();
    }
    "try typing 'yeet'".to_string()
}

#[tauri::command]
fn create_settings_window() -> Result<(), String> {
    println!("Creating settings window");
    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn show_panel(handle: tauri::AppHandle) {
    if let Ok(panel) = handle.get_webview_panel("main") {
        position_panel_smart(&handle);
        panel.show_and_make_key();
    }
}

#[cfg(target_os = "macos")]
#[tauri::command] 
fn hide_panel(handle: tauri::AppHandle) {
    // Save position before hiding
    remember_panel_position(handle.clone());
    if let Ok(panel) = handle.get_webview_panel("main") {
        panel.hide();
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn rescue_panel(handle: tauri::AppHandle) {
    println!("🆘 Rescuing lost panel!");
    
    // First, clear the bad stored position
    let last_pos = LAST_PANEL_POSITION.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(None)));
    *last_pos.lock().unwrap() = None;
    println!("🔄 Cleared stored position data");
    
    if let Some(window) = handle.get_webview_window("main") {
        // Reset to sensible default position
        let safe_x = 300.0;
        let safe_y = 200.0;
        let _ = window.set_position(tauri::PhysicalPosition::new(safe_x as i32, safe_y as i32));
        let _ = window.show();
        let _ = window.set_focus();
        
        // Store this safe position
        *last_pos.lock().unwrap() = Some((safe_x, safe_y));
        println!("📍 Reset panel to safe position ({:.0}, {:.0})", safe_x, safe_y);
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn reset_panel_position(handle: tauri::AppHandle) {
    println!("🔄 Resetting panel position storage");
    let last_pos = LAST_PANEL_POSITION.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(None)));
    *last_pos.lock().unwrap() = None;
    
    // Position at safe default
    if let Some(window) = handle.get_webview_window("main") {
        let _ = window.set_position(tauri::PhysicalPosition::new(300, 200));
        println!("📍 Panel reset to default position (300, 200)");
    }
}

#[cfg(target_os = "macos")]
// Shared storage for panel position - accessible by both functions
static LAST_PANEL_POSITION: std::sync::OnceLock<std::sync::Arc<std::sync::Mutex<Option<(f64, f64)>>>> = std::sync::OnceLock::new();

#[cfg(target_os = "macos")]
#[tauri::command]
fn position_panel_smart(handle: &tauri::AppHandle) {
    use tauri::PhysicalPosition;
    
    let last_pos = LAST_PANEL_POSITION.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(None)));
    
    if let Some(window) = handle.get_webview_window("main") {
        // Check if we have a remembered position
        let stored_position = last_pos.lock().unwrap().clone();
        
        if let Some((x, y)) = stored_position {
            // Use the stored position (physical coordinates)
            let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
            println!("🎯 Panel restored to remembered position ({:.0}, {:.0})", x, y);
        } else {
            // First time: use a sensible default position
            let default_x = 300.0;
            let default_y = 200.0;
            
            let _ = window.set_position(PhysicalPosition::new(default_x as i32, default_y as i32));
            
            // Store this as the initial position
            *last_pos.lock().unwrap() = Some((default_x, default_y));
            println!("🎯 Panel positioned at initial location ({:.0}, {:.0})", default_x, default_y);
        }
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn remember_panel_position(handle: tauri::AppHandle) {
    let last_pos = LAST_PANEL_POSITION.get_or_init(|| std::sync::Arc::new(std::sync::Mutex::new(None)));
    
    if let Some(window) = handle.get_webview_window("main") {
        if let Ok(position) = window.outer_position() {
            let x = position.x as f64;
            let y = position.y as f64;
            
            // Bounds check to prevent runaway coordinates
            if x >= 0.0 && x <= 10000.0 && y >= 0.0 && y <= 10000.0 {
                // Get the current stored position to prevent unnecessary updates
                let current_stored = last_pos.lock().unwrap().clone();
                
                // Only update if position actually changed significantly (> 5 pixels)
                let should_update = match current_stored {
                    Some((stored_x, stored_y)) => {
                        (x - stored_x).abs() > 5.0 || (y - stored_y).abs() > 5.0
                    }
                    None => true
                };
                
                if should_update {
                    *last_pos.lock().unwrap() = Some((x, y));
                    println!("📍 Remembered panel position ({:.0}, {:.0})", x, y);
                }
            } else {
                println!("❌ Invalid position detected ({:.0}, {:.0}) - ignoring", x, y);
            }
        }
    }
}

// ✅ Overlayed-inspired clean panel trait
#[cfg(target_os = "macos")]
pub trait WebviewWindowExtMacos {
    fn remove_shadow(&self);
    fn set_float_panel(&self, level: i32);
}

#[cfg(target_os = "macos")]
impl<R: Runtime> WebviewWindowExtMacos for WebviewWindow<R> {
    fn remove_shadow(&self) {
        // Shadow is already removed in setup with window.set_shadow(false)
        // This method is here for API compatibility with Overlayed's pattern
    }

    fn set_float_panel(&self, level: i32) {
        if let Ok(panel) = self.to_panel::<OrbitPanel>() {
            panel.set_level(level as i64);
            
            // Use tauri-nspanel's StyleMask instead of raw cocoa
            panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

            // Use tauri-nspanel's CollectionBehavior
            panel.set_collection_behavior(
                CollectionBehavior::new()
                    .full_screen_auxiliary()
                    .can_join_all_spaces()
                    .into(),
            );
            
            println!("🎯 Orbit panel configured with level {}", level);
        } else {
            println!("❌ Failed to convert window to panel");
        }
    }
}

// ✅ Professional window levels like Overlayed
#[cfg(target_os = "macos")]
const ORBIT_NORMAL_LEVEL: i32 = 8;

fn main() {
    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .setup(|app| {
            // ✅ Phase 1: Basic window setup (Overlayed pattern - no panic risk)
            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_decorations(false);
            let _ = window.set_shadow(false);
            
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                apply_orbit_panel_setup(&window);
                
                // Set up position tracking for memory (only on focus loss to avoid move loops)
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::Focused(false) => {
                            // Save position when losing focus (about to hide)
                            remember_panel_position(app_handle.clone());
                        }
                        _ => {}
                    }
                });
            }

            // ✅ Global shortcut setup
            setup_global_shortcuts(app)?;
            
            println!("🎯 Orbit is running! Press CommandOrControl+2 to toggle.");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            process_query,
            create_settings_window,
            #[cfg(target_os = "macos")]
            show_panel,
            #[cfg(target_os = "macos")]
            hide_panel,
            #[cfg(target_os = "macos")]
            rescue_panel,
            #[cfg(target_os = "macos")]
            remember_panel_position,
            #[cfg(target_os = "macos")]
            reset_panel_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ✅ Phase 2: Panel conversion (safe, after window exists - Overlayed pattern)
#[cfg(target_os = "macos")]
fn apply_orbit_panel_setup(window: &WebviewWindow) {
    window.remove_shadow();
    window.set_float_panel(ORBIT_NORMAL_LEVEL);
}

// ✅ Clean global shortcut setup
fn setup_global_shortcuts(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
    
    let hotkey = "CommandOrControl+2";
    let rescue_hotkey = "CommandOrControl+Shift+2"; // Rescue shortcut
    
    let shortcut: Shortcut = hotkey.parse()?;
    let rescue_shortcut: Shortcut = rescue_hotkey.parse()?;
    
    #[cfg(target_os = "macos")]
    let app_handle = app.handle().clone();
    #[cfg(target_os = "macos")]
    let rescue_app_handle = app.handle().clone();

    app.handle()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, scut, event| {
                    if let ShortcutState::Pressed = event.state() {
                        if scut == &shortcut {
                            #[cfg(target_os = "macos")]
                            {
                                match app_handle.get_webview_panel("main") {
                                    Ok(panel) => {
                                        if panel.is_visible() {
                                            panel.hide();
                                        } else {
                                            position_panel_smart(&app_handle);
                                            panel.show_and_make_key();
                                        }
                                    }
                                    Err(_) => {
                                        // Fallback to regular window
                                        if let Some(window) = app_handle.get_webview_window("main") {
                                            if window.is_visible().unwrap_or(false) {
                                                window.hide().unwrap();
                                            } else {
                                                window.show().unwrap();
                                                window.set_focus().unwrap();
                                            }
                                        }
                                    }
                                }
                            }
                            #[cfg(not(target_os = "macos"))]
                            {
                                println!("🎯 Toggle shortcut pressed!");
                            }
                        } else if scut == &rescue_shortcut {
                            #[cfg(target_os = "macos")]
                            {
                                println!("🆘 Rescue shortcut pressed!");
                                if let Some(window) = rescue_app_handle.get_webview_window("main") {
                                    let _ = window.set_position(tauri::PhysicalPosition::new(100, 100));
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(),
        )?;

    app.global_shortcut().register(shortcut)?;
    app.global_shortcut().register(rescue_shortcut)?;
    
    println!("🎯 Hotkeys registered: {} (toggle), {} (rescue)", hotkey, rescue_hotkey);
    println!("📍 Panel will remember its position between toggles!");
    
    Ok(())
}
