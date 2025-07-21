use tauri::Window;

#[tauri::command]
pub fn toggle_window_visibility(window: Window) {
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
    #[cfg(target_os = "macos")]
    {
        let _ = window.set_always_on_top(true);
    }
}

#[tauri::command]
fn resize_to_content(window: Window, content_height: u32) -> Result<(), String> {
    use tauri::LogicalSize;
    let new_height = (content_height + 40).clamp(100, 600);
    window
        .set_size(LogicalSize::new(500, new_height))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_window_badge(text: Option<String>) -> Result<(), String> {
    println!("Setting badge to: {:?}", text);
    Ok(())
}
