// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Window;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn process_query(query: &str) -> String {
    // This is where you'd integrate with your AI service
    // For now, we'll just echo back with some processing
    if query.to_lowercase().contains("hello") {
        "Hello! I'm Orbit, your AI assistant. How can I help you today?".to_string()
    } else if query.to_lowercase().contains("weather") {
        "I'd love to help with the weather! I'm still learning that capability.".to_string()
    } else if query.to_lowercase().contains("time") {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("It's been {} seconds since epoch. I'm working on a better time display!", now)
    } else {
        format!("I heard you say: '{}'. I'm still learning how to respond to that!", query)
    }
}

#[tauri::command]
fn toggle_window_visibility(window: Window) {
    if window.is_visible().unwrap_or(false) {
        window.hide().unwrap();
    } else {
        window.show().unwrap();
        window.set_focus().unwrap();
    }
}

#[tauri::command]
fn hide_window(window: Window) {
    window.hide().unwrap();
}

#[tauri::command]
fn show_window(window: Window) {
    window.show().unwrap();
    window.set_focus().unwrap();
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Set up global hotkey (Cmd+Space on Mac, Ctrl+Space on others)
            #[cfg(target_os = "macos")]
            let hotkey = "cmd+space";
            #[cfg(not(target_os = "macos"))]
            let hotkey = "ctrl+space";
            
            println!("Orbit is running! Press {} to toggle.", hotkey);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            process_query,
            toggle_window_visibility,
            hide_window,
            show_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
