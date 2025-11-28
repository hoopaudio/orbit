use crate::{consts::*, OrbitState, Pinned, TrayMenu};
use orbit_ai::OrbitAgent;
use orbit_core::{AbletonCommand, AbletonControlResponse, OrbitMessage};
use std::{
    ops::Deref,
    sync::{atomic::AtomicBool, Mutex},
};
use tauri::{image::Image, menu::Menu, AppHandle, Emitter, Manager, State, WebviewWindow, Wry};
use tauri_nspanel::ManagerExt;

#[tauri::command]
pub fn show(app_handle: AppHandle) {
    let window = app_handle.get_webview_window(ORBIT_LABEL).unwrap();
    let panel = app_handle.get_webview_panel(ORBIT_LABEL).unwrap();

    panel.show();
    window.set_focus().unwrap();

    // Focus the input field after showing the window
    let _ = window.eval("setTimeout(() => { const input = document.querySelector('textarea'); if (input) input.focus(); }, 50);");
}

#[tauri::command]
pub fn hide(app_handle: AppHandle) {
    let panel = app_handle.get_webview_panel(ORBIT_LABEL).unwrap();

    if panel.is_visible() {
        panel.order_out(None);
    }
}

#[tauri::command]
pub fn open_settings(window: WebviewWindow, update: bool) {
    let app = window.app_handle();
    let settings_windows = app.get_webview_window(SETTINGS_WINDOW_NAME);
    if let Some(settings_windows) = settings_windows {
        settings_windows.show();
        settings_windows.set_focus();
        if update {
            // emit to the settings window to show update
            settings_windows
                .emit_to(SETTINGS_WINDOW_NAME, SHOW_UPDATE_MODAL, ())
                .unwrap();
        }
    }
}

#[tauri::command]
pub fn close_settings(window: WebviewWindow) {
    let app = window.app_handle();
    let settings_windows = app.get_webview_window(SETTINGS_WINDOW_NAME);
    if let Some(settings_windows) = settings_windows {
        settings_windows.hide();
    }
}

#[tauri::command]
pub fn get_pin(storage: State<Pinned>) -> bool {
    storage.0.load(std::sync::atomic::Ordering::Relaxed)
}

#[tauri::command]
pub fn open_devtools(window: WebviewWindow) {
    window.open_devtools();
}

#[tauri::command]
pub fn toggle_pin(window: WebviewWindow, pin: State<Pinned>, menu: State<TrayMenu>) {
    let app = window.app_handle();
    let value = !get_pin(app.state::<Pinned>());

    _set_pin(value, &window, pin, menu);
}

#[tauri::command]
pub fn set_pin(window: WebviewWindow, pin: State<Pinned>, menu: State<TrayMenu>, value: bool) {
    _set_pin(value, &window, pin, menu);
}

impl Deref for Pinned {
    type Target = AtomicBool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for TrayMenu {
    type Target = Mutex<Menu<Wry>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn _set_pin(value: bool, window: &WebviewWindow, pinned: State<Pinned>, menu: State<TrayMenu>) {
    // @d0nutptr cooked here
    pinned.0.store(value, std::sync::atomic::Ordering::Relaxed);

    // let the client know
    window.emit(TRAY_TOGGLE_PIN, value).unwrap();

    // invert the label for the tray
    if let Some(toggle_pin_menu_item) = menu.0.lock().ok().and_then(|m| m.get(TRAY_TOGGLE_PIN)) {
        let enable_or_disable = if value { "Unpin" } else { "Pin" };
        toggle_pin_menu_item
            .as_menuitem_unchecked()
            .set_text(enable_or_disable);
    }

    #[cfg(target_os = "macos")]
    window.with_webview(move |webview| unsafe {
        #[cfg(target_os = "macos")]
        use cocoa::appkit::NSWindow;
        let id = webview.ns_window() as cocoa::base::id;

        #[cfg(target_arch = "aarch64")]
        id.setIgnoresMouseEvents_(value);

        // convert bool into number
        #[cfg(target_arch = "x86_64")]
        {
            let value = if value { 1 } else { 0 };
            id.setHasShadow_(value);
        }
    });

    window.set_ignore_cursor_events(value);

    // update the tray icon
    update_tray_icon(window.app_handle(), value);
}

pub fn update_tray_icon(app: &AppHandle, pinned: bool) {
    let icon_bytes = if pinned {
        include_bytes!("../icons/tray-icon.png").as_slice()
    } else {
        include_bytes!("../icons/tray-icon.png").as_slice()
    };

    if let Some(tray) = app.tray_by_id(ORBIT) {
        if let Ok(icon) = Image::from_bytes(icon_bytes) {
            tray.set_icon(Some(icon));
        }
    }
}

#[tauri::command]
pub fn resize_window(window: WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    use tauri::LogicalSize;

    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn resize_and_reposition_for_standard_mode(
    window: WebviewWindow,
    width: f64,
    height: f64,
) -> Result<(), String> {
    use tauri::LogicalSize;

    // First resize the window
    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;

    // Then reposition it to top_right with some padding from the top
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSPoint, NSRect, NSSize};
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let ns_window = window.ns_window().map_err(|e| e.to_string())? as id;

            // Get the screen the window is on
            let screen: id = msg_send![ns_window, screen];
            if screen == nil {
                return Err("No screen found".to_string());
            }

            // Get the visible frame (area not occupied by dock/menu bar)
            let visible_frame: NSRect = msg_send![screen, visibleFrame];

            // Position at top-right, exact same calculation as window.position("top_right")
            let new_frame = NSRect {
                origin: NSPoint {
                    x: visible_frame.origin.x + visible_frame.size.width - width,
                    y: visible_frame.origin.y + visible_frame.size.height - height,
                },
                size: NSSize { width, height },
            };

            // Set the frame without animation
            let _: () = msg_send![ns_window, setFrame:new_frame display:true animate:false];
        }
    }

    Ok(())
}

#[tauri::command]
pub fn resize_window_for_producer_mode(window: WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::NSRect;
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let ns_window = window.ns_window().map_err(|e| e.to_string())? as id;

            // Get the screen the window is on
            let screen: id = msg_send![ns_window, screen];
            if screen == nil {
                return Err("No screen found".to_string());
            }

            // Get the visible frame (area not occupied by dock/menu bar)
            let visible_frame: NSRect = msg_send![screen, visibleFrame];

            // Set the window frame to match the visible area
            // We need to position it at the top of the visible area
            let new_frame = NSRect {
                origin: cocoa::foundation::NSPoint {
                    x: visible_frame.origin.x + visible_frame.size.width - 450.0, // Align flush to right edge
                    y: visible_frame.origin.y, // Bottom of visible area
                },
                size: cocoa::foundation::NSSize {
                    width: 450.0,
                    height: visible_frame.size.height,
                },
            };

            // Set the frame without animation for instant resize
            let _: () = msg_send![ns_window, setFrame:new_frame display:true animate:false];

            println!(
                "Set producer mode window to height: {}",
                visible_frame.size.height
            );
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Fallback for other platforms
        window
            .set_size(tauri::LogicalSize::new(550.0, 800.0))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_available_screen_dimensions(window: WebviewWindow) -> Result<(f64, f64), String> {
    // Get the monitor that the window is currently on
    let current_monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;

    // Get the monitor's size (this is the full screen size)
    let _monitor_size = current_monitor.size();

    // On macOS, we need to account for the menu bar and dock
    // The available work area is the screen minus system UI elements
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::{NSArray, NSPoint, NSRect};
        use objc::{class, msg_send, sel, sel_impl};

        unsafe {
            // Get the window's current position to find which screen it's on
            let ns_window = window.ns_window().map_err(|e| e.to_string())? as id;
            let window_frame: NSRect = msg_send![ns_window, frame];

            // Get all screens and find the one containing our window
            let screens: id = msg_send![class!(NSScreen), screens];
            let screen_count: usize = msg_send![screens, count];

            let mut target_screen: id = nil;
            for i in 0..screen_count {
                let screen: id = msg_send![screens, objectAtIndex:i];
                let screen_frame: NSRect = msg_send![screen, frame];

                // Check if window center is within this screen
                let window_center_x = window_frame.origin.x + window_frame.size.width / 2.0;
                let window_center_y = window_frame.origin.y + window_frame.size.height / 2.0;

                if window_center_x >= screen_frame.origin.x
                    && window_center_x <= screen_frame.origin.x + screen_frame.size.width
                    && window_center_y >= screen_frame.origin.y
                    && window_center_y <= screen_frame.origin.y + screen_frame.size.height
                {
                    target_screen = screen;
                    break;
                }
            }

            // If we didn't find the screen, use the main screen
            if target_screen == nil {
                target_screen = msg_send![screens, objectAtIndex:0];
            }

            // Get the visible frame (excludes menu bar and dock) and full frame
            let visible_frame: NSRect = msg_send![target_screen, visibleFrame];
            let full_frame: NSRect = msg_send![target_screen, frame];

            // Log for debugging
            println!("Full frame height: {}", full_frame.size.height);
            println!("Visible frame height: {}", visible_frame.size.height);
            println!(
                "Difference (menu + dock): {}",
                full_frame.size.height - visible_frame.size.height
            );

            // Return the visible frame dimensions
            Ok((visible_frame.size.width, visible_frame.size.height))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // For other platforms, return the full monitor size
        // You may need to adjust this based on platform-specific requirements
        Ok((monitor_size.width as f64, monitor_size.height as f64))
    }
}

#[tauri::command]
pub fn get_visible_frame(window: WebviewWindow) -> Result<(f64, f64, f64, f64), String> {
    #[cfg(target_os = "macos")]
    {
        use cocoa::base::{id, nil};
        use cocoa::foundation::NSRect;
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let ns_window = window.ns_window().map_err(|e| e.to_string())? as id;
            let screen: id = msg_send![ns_window, screen];
            if screen == nil {
                return Err("No screen found".to_string());
            }
            let visible_frame: NSRect = msg_send![screen, visibleFrame];
            Ok((
                visible_frame.origin.x,
                visible_frame.origin.y,
                visible_frame.size.width,
                visible_frame.size.height,
            ))
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok((0.0, 0.0, 0.0, 0.0))
    }
}

/// Connect to Ableton Live
#[tauri::command]
pub async fn connect_ableton(orbit_state: State<'_, OrbitState>) -> Result<String, String> {
    let mut connector = orbit_state.0.lock().await;

    match connector.connect_ableton().await {
        Ok(_) => Ok("Connected to Ableton Live".to_string()),
        Err(e) => Err(format!("Failed to connect to Ableton Live: {}", e)),
    }
}

/// Disconnect from Ableton Live
#[tauri::command]
pub async fn disconnect_ableton(orbit_state: State<'_, OrbitState>) -> Result<String, String> {
    let mut connector = orbit_state.0.lock().await;
    connector.disconnect_ableton().await;
    Ok("Disconnected from Ableton Live".to_string())
}

/// Send a command to Ableton Live
#[tauri::command]
pub async fn send_ableton_command(
    orbit_state: State<'_, OrbitState>,
    command: AbletonCommand,
) -> Result<AbletonControlResponse, String> {
    let connector = orbit_state.0.lock().await;

    let message = OrbitMessage::ableton_control(command);

    match connector.handle_message(message).await {
        Ok(Some(response)) => {
            // Parse the response
            let response_data: AbletonControlResponse = serde_json::from_value(response.payload)
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(response_data)
        }
        Ok(None) => Err("No response from Ableton".to_string()),
        Err(e) => Err(format!("Command failed: {}", e)),
    }
}

/// Test OSC connection to Ableton Live
#[tauri::command]
pub async fn test_ableton_connection() -> Result<String, String> {
    println!("test_ableton_connection called");

    let result = tokio::task::spawn_blocking(move || {
        orbit_ai::pyo3::Python::with_gil(|py| {
            // Import the ableton_client module
            let sys = py
                .import("sys")
                .map_err(|e| format!("Failed to import sys: {}", e))?;
            let path = sys
                .getattr("path")
                .map_err(|e| format!("Failed to get sys.path: {}", e))?;

            // Add the orbit-connector path
            #[cfg(debug_assertions)]
            let module_path = {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let project_root = if cwd.ends_with("crates/orbit-app") {
                    cwd.parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.to_path_buf())
                        .unwrap_or(cwd.clone())
                } else {
                    cwd.clone()
                };
                project_root
                    .join("crates/orbit-connector/src/python")
                    .to_string_lossy()
                    .to_string()
            };

            #[cfg(not(debug_assertions))]
            let module_path = {
                let exe_path =
                    std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
                exe_dir.join("python").to_string_lossy().to_string()
            };

            let path_list: Vec<String> = path
                .extract()
                .map_err(|e| format!("Failed to extract path: {}", e))?;
            if !path_list.iter().any(|p| p.contains(&module_path)) {
                path.call_method1("append", (&module_path,))
                    .map_err(|e| format!("Failed to append path: {}", e))?;
            }

            // Import ableton_client
            let ableton_client = py
                .import("ableton_client")
                .map_err(|e| format!("Failed to import ableton_client: {}", e))?;
            let get_ableton_client = ableton_client
                .getattr("get_ableton_client")
                .map_err(|e| format!("Failed to get get_ableton_client: {}", e))?;
            let client = get_ableton_client
                .call0()
                .map_err(|e| format!("Failed to create client: {}", e))?;

            // Test basic connection by sending a simple OSC message
            let send_message = client
                .getattr("send_message")
                .map_err(|e| format!("Failed to get send_message method: {}", e))?;
            let result = send_message
                .call1(("/live/test",))
                .map_err(|e| format!("Failed to send test message: {}", e))?;
            let success: bool = result.extract().unwrap_or(false);

            if success {
                Ok("OSC message sent successfully - connection working".to_string())
            } else {
                Err("Failed to send OSC message - connection may be down".to_string())
            }
        })
    })
    .await;

    match result {
        Ok(Ok(msg)) => {
            println!("Connection test successful: {}", msg);
            Ok(msg)
        }
        Ok(Err(e)) => {
            let error_msg = format!("Connection test failed: {}", e);
            println!("{}", error_msg);
            Err(error_msg)
        }
        Err(e) => {
            let error_msg = format!("Task execution failed: {}", e);
            println!("{}", error_msg);
            Err(error_msg)
        }
    }
}

/// Get track information from Ableton Live
/// refactor dog shit function
#[tauri::command]
pub async fn get_ableton_tracks() -> Result<String, String> {
    println!("get_ableton_tracks called");

    // Use a simpler approach that calls the OSC client directly through the existing Python integration
    let result = tokio::task::spawn_blocking(move || {
        orbit_ai::pyo3::Python::with_gil(|py| {
            // Import the ableton_client module
            let sys = py
                .import("sys")
                .map_err(|e| format!("Failed to import sys: {}", e))?;
            let path = sys
                .getattr("path")
                .map_err(|e| format!("Failed to get sys.path: {}", e))?;

            // Add the orbit-connector path
            #[cfg(debug_assertions)]
            let module_path = {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let project_root = if cwd.ends_with("crates/orbit-app") {
                    cwd.parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.to_path_buf())
                        .unwrap_or(cwd.clone())
                } else {
                    cwd.clone()
                };
                project_root
                    .join("crates/orbit-connector/src/python")
                    .to_string_lossy()
                    .to_string()
            };

            #[cfg(not(debug_assertions))]
            let module_path = {
                let exe_path =
                    std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
                exe_dir.join("python").to_string_lossy().to_string()
            };

            let path_list: Vec<String> = path
                .extract()
                .map_err(|e| format!("Failed to extract path: {}", e))?;
            if !path_list.iter().any(|p| p.contains(&module_path)) {
                path.call_method1("append", (&module_path,))
                    .map_err(|e| format!("Failed to append path: {}", e))?;
            }

            // Import ableton_client
            let ableton_client = py
                .import("ableton_client")
                .map_err(|e| format!("Failed to import ableton_client: {}", e))?;
            let get_ableton_client = ableton_client
                .getattr("get_ableton_client")
                .map_err(|e| format!("Failed to get get_ableton_client: {}", e))?;
            let client = get_ableton_client
                .call0()
                .map_err(|e| format!("Failed to create client: {}", e))?;

            // Call get_track_names
            let get_track_names = client
                .getattr("get_track_names")
                .map_err(|e| format!("Failed to get get_track_names method: {}", e))?;
            let tracks_result = get_track_names
                .call0()
                .map_err(|e| format!("Failed to call get_track_names: {}", e))?;

            // Extract the result as a Python list
            if let Ok(tracks) = tracks_result.downcast::<orbit_ai::pyo3::types::PyList>() {
                // Convert Python objects to JSON
                let mut track_list = Vec::new();
                for track_obj in tracks.iter() {
                    if let Ok(track_dict) = track_obj.downcast::<orbit_ai::pyo3::types::PyDict>() {
                        let mut track_json = serde_json::Map::new();
                        for (key, value) in track_dict.iter() {
                            let key_str: String =
                                key.extract().unwrap_or_else(|_| "unknown".to_string());

                            if let Ok(string_val) = value.extract::<String>() {
                                track_json.insert(key_str, serde_json::Value::String(string_val));
                            } else if let Ok(int_val) = value.extract::<i64>() {
                                track_json.insert(
                                    key_str,
                                    serde_json::Value::Number(serde_json::Number::from(int_val)),
                                );
                            } else if let Ok(bool_val) = value.extract::<bool>() {
                                track_json.insert(key_str, serde_json::Value::Bool(bool_val));
                            }
                        }
                        track_list.push(serde_json::Value::Object(track_json));
                    }
                }
                Ok(serde_json::to_string(&track_list).unwrap_or_else(|_| "[]".to_string()))
            } else {
                Err("No tracks found or connection failed".to_string())
            }
        })
    })
    .await;

    match result {
        Ok(Ok(tracks_json)) => {
            println!("Successfully got tracks: {}", tracks_json);
            Ok(tracks_json)
        }
        Ok(Err(e)) => {
            let error_msg = format!("Failed to get tracks from Ableton: {}", e);
            println!("{}", error_msg);
            Err(error_msg)
        }
        Err(e) => {
            let error_msg = format!("Task execution failed: {}", e);
            println!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn ask_orbit(message: String) -> Result<String, String> {
    // Run in spawn_blocking since Python GIL blocks the thread
    tokio::task::spawn_blocking(move || {
        let agent = OrbitAgent::new().map_err(|e| format!("Failed to create OrbitAgent: {}", e))?;
        agent
            .run(&message, Some("thread-1"))
            .map_err(|e| format!("Agent run failed: {}", e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
