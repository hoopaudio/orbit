use crate::{consts::*, Pinned, TrayMenu};
use orbit_ai::{LangChainChatBot, PythonBotWrapper};
use std::{
    ops::Deref,
    sync::{atomic::AtomicBool, Mutex},
};
use tauri::{image::Image, menu::Menu, AppHandle, Emitter, Manager, State, WebviewWindow, Wry};
use tauri_nspanel::ManagerExt;

/// Initialize the Python bot at startup to avoid cold start delays
pub fn initialize_python_bot() {
    println!("Initializing Python bot at startup...");

    // Spawn a background task to initialize the bot
    std::thread::spawn(|| {
        match PythonBotWrapper::initialize() {
            Ok(_) => {
                println!("Python bot successfully warmed up at startup");
            }
            Err(e) => {
                eprintln!("Failed to initialize Python bot at startup: {}", e);
                // Non-fatal - will retry on first query
            }
        }
    });
}

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
pub async fn process_query(query: String, app_handle: AppHandle) -> Result<String, String> {
    let orbit_agent = LangChainChatBot::new(app_handle).map_err(|e| e.to_string())?;

    orbit_agent
        .ask_orbit(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn process_query_stream(query: String, app_handle: AppHandle) -> Result<(), String> {
    let chatbot = LangChainChatBot::new(app_handle.clone()).map_err(|e| e.to_string())?;

    chatbot
        .ask_orbit_stream(&query, app_handle)
        .await
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn process_query_python(query: String) -> Result<String, String> {
    println!("process_query_python called with query: {}", query);

    // Create a lightweight wrapper - the Python singleton handles the actual instance
    let bot = PythonBotWrapper::new()
        .map_err(|e| format!("Failed to create bot wrapper: {}", e))?;

    match bot.ask_orbit(&query).await {
        Ok(response) => {
            println!("Got response from Python bot: {}", response);
            Ok(response)
        }
        Err(e) => {
            let error_msg = format!("Failed to get response from Python bot: {}", e);
            println!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn process_query_python_stream(query: String, app_handle: AppHandle) -> Result<(), String> {
    println!("process_query_python_stream called with query: {}", query);

    // Create a lightweight wrapper - reuses the Python singleton
    let bot = PythonBotWrapper::new()
        .map_err(|e| format!("Failed to create bot wrapper: {}", e))?;

    // Stream the response
    match bot.ask_orbit_stream(&query, app_handle).await {
        Ok(_) => {
            println!("Streaming completed successfully");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to stream response: {}", e);
            println!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn clear_python_memory() -> Result<String, String> {
    println!("Clearing Python bot memory");

    let bot = PythonBotWrapper::new()
        .map_err(|e| format!("Failed to create bot wrapper: {}", e))?;

    match bot.clear_memory().await {
        Ok(response) => {
            println!("Memory cleared: {}", response);
            Ok(response)
        }
        Err(e) => {
            let error_msg = format!("Failed to clear memory: {}", e);
            println!("{}", error_msg);
            Err(error_msg)
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