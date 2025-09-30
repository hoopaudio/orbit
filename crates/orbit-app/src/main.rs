#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod consts;
mod tray;
mod window;

use consts::{MAIN_WINDOW_NAME, ORBIT_LABEL, SETTINGS_WINDOW_NAME};
use tray::Tray;
use commands::*;
use dotenv::dotenv;
use orbit_ai::ScreenshotService;
use std::{
    str::FromStr,
    sync::{atomic::AtomicBool, Mutex},
};
use tauri::{generate_handler, menu::Menu, Listener, Manager, Wry};
use tauri_nspanel::ManagerExt;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
use window::WebviewWindowExt;

pub struct Pinned(AtomicBool);
pub struct TrayMenu(Mutex<Menu<Wry>>);


fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    let flags = StateFlags::POSITION | StateFlags::SIZE;
    let window_state_plugin = tauri_plugin_window_state::Builder::default().with_state_flags(flags);
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let log_level_filter = log::LevelFilter::from_str(&log_level).unwrap_or(log::LevelFilter::Info);

    let log_plugin_builder = tauri_plugin_log::Builder::new()
        .targets([Target::new(TargetKind::LogDir { file_name: None })])
        .level_for("orbit", log_level_filter)
        .level_for("reqwest", log_level_filter)
        .level_for("tokio_tungstenite", log::LevelFilter::Off)
        .level_for("tungstenite", log::LevelFilter::Off)
        .level_for("tao", log::LevelFilter::Off);

    let mut app = tauri::Builder::default()
        .plugin(window_state_plugin.build())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_http::init())
        .plugin(log_plugin_builder.build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
        }))
        // Register a global shortcut (⌘+O) to toggle the visibility of the spotlight panel
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(Shortcut::new(Some(Modifiers::SUPER), Code::KeyO))
                .unwrap()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed
                        && shortcut.matches(Modifiers::SUPER, Code::KeyO)
                    {
                        let window = app.get_webview_window(ORBIT_LABEL).unwrap();

                        let panel = app.get_webview_panel(ORBIT_LABEL).unwrap();

                        if panel.is_visible() {
                            panel.order_out(None);
                        } else {
                            window.position("top_right").unwrap();

                            panel.show();
                            window.set_focus().unwrap();

                            // Focus the input field after showing the window
                            let _ = window.eval("setTimeout(() => { const input = document.querySelector('textarea'); if (input) input.focus(); }, 50);");
                        }
                    }
                })
                .build(),
        );

    #[cfg(target_os = "macos")]
    {
        app = app.plugin(tauri_nspanel::init());
    }

    app = app
        .manage(Pinned(AtomicBool::new(false)))
        .setup(move |app| {
            // Set activation poicy to Accessory to prevent the app icon from showing on the dock
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Initialize Python bot at startup to avoid cold start delays on first query
            commands::initialize_python_bot();

            let handle = app.app_handle();

            let window = handle.get_webview_window(ORBIT_LABEL).unwrap();

            #[cfg(not(target_os = "macos"))]
            window.set_always_on_top(true);

            window.set_decorations(false);
            window.set_shadow(false);

            #[cfg(target_os = "macos")]
            unsafe {
                use cocoa::appkit::NSWindow;
                use objc::{msg_send, sel, sel_impl};

                #[allow(non_upper_case_globals)]
                const NSFocusRingTypeNone: i32 = 1;

                let ns_window = window.ns_window().unwrap() as cocoa::base::id;
                let ns_view = ns_window.contentView();
                let _: () = msg_send![ns_view, setFocusRingType:NSFocusRingTypeNone];
            }

            // Convert the window to a spotlight panel
            let panel = window.to_orbit_panel()?;

            handle.listen(format!("{}_panel_did_resign_key", ORBIT_LABEL), move |_| {
                // Hide the panel when it's no longer the key window
                // This ensures the panel doesn't remain visible when it's not actively being used
                panel.order_out(None);
            });

            Tray::update_tray(app.app_handle());

            Ok(())
        })
        .invoke_handler(generate_handler![
            toggle_pin,
            get_pin,
            set_pin,
            open_devtools,
            close_settings,
            open_settings,
            process_query,
            process_query_stream,
            process_query_python,
            resize_window,
            resize_and_reposition_for_standard_mode,
            show,
            hide,
            get_visible_frame,
            resize_window_for_producer_mode,
            clear_python_memory
        ]);

    app.build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::WindowEvent {
                event: tauri::WindowEvent::CloseRequested { api, .. },
                label,
                ..
            } = event
            {
                if label == SETTINGS_WINDOW_NAME {
                    let win = app.get_webview_window(label.as_str()).unwrap();
                    win.hide().unwrap();
                }

                if label == MAIN_WINDOW_NAME {
                    app.save_window_state(StateFlags::POSITION | StateFlags::SIZE);

                    // Clean up all screenshots before exit
                    if let Err(e) = ScreenshotService::cleanup_all_screenshots(&app.app_handle()) {
                        log::warn!("Failed to clean up screenshots on exit: {}", e);
                    }

                    std::process::exit(0);
                } else {
                    api.prevent_close();
                }
            }
        });
}
