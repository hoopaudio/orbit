use tauri::{Emitter, Manager, Runtime, WebviewWindow};
use tauri_nspanel::{
    cocoa::{
        appkit::{NSMainMenuWindowLevel, NSView, NSWindowCollectionBehavior},
        base::{id, YES},
        foundation::{NSPoint, NSRect, NSSize},
    },
    objc::{msg_send, sel, sel_impl},
    panel_delegate, Panel, WebviewWindowExt as PanelWebviewWindowExt,
};
use thiserror::Error;

type TauriError = tauri::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("Unable to convert window to panel")]
    Panel,
    #[error("Monitor not found")]
    MonitorNotFound,
    #[error("Invalid position: {0}")]
    InvalidPosition(String),
}

pub trait WebviewWindowExt {
    fn to_orbit_panel(&self) -> tauri::Result<Panel>;
    fn position(&self, position: &str) -> tauri::Result<()>;
}

impl<R: Runtime> WebviewWindowExt for WebviewWindow<R> {
    fn to_orbit_panel(&self) -> tauri::Result<Panel> {
        // Convert window to panel
        let panel = self
            .to_panel()
            .map_err(|_| TauriError::Anyhow(Error::Panel.into()))?;

        // Set panel level
        panel.set_level(NSMainMenuWindowLevel + 1);

        // Allows the panel to display on the same space as the full screen window
        panel.set_collection_behaviour(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
        );

        #[allow(non_upper_case_globals)]
        const NSWindowStyleMaskNonActivatingPanel: i32 = 1 << 7;

        // Ensures the panel cannot activate the App
        panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel);

        let panel_delegate = panel_delegate!(SpotlightPanelDelegate {
            window_did_resign_key,
            window_did_become_key
        });

        let app_handle = self.app_handle().clone();

        let label = self.label().to_string();

        panel_delegate.set_listener(Box::new(move |delegate_name: String| {
            match delegate_name.as_str() {
                "window_did_become_key" => {
                    let _ = app_handle.emit(format!("{}_panel_did_become_key", label).as_str(), ());
                }
                "window_did_resign_key" => {
                    let _ = app_handle.emit(format!("{}_panel_did_resign_key", label).as_str(), ());
                }
                _ => (),
            }
        }));

        panel.set_delegate(panel_delegate);

        Ok(panel)
    }

    fn position(&self, position: &str) -> tauri::Result<()> {
        let window_handle: id = self.ns_window()? as _;

        // Get the NSScreen for the current window
        let screen: id = unsafe { msg_send![window_handle, screen] };
        if screen.is_null() {
            return Err(TauriError::Anyhow(Error::MonitorNotFound.into()));
        }

        // Get the visible frame of the screen (excludes menu bar and dock)
        let visible_frame: NSRect = unsafe { msg_send![screen, visibleFrame] };
        let window_frame: NSRect = unsafe { window_handle.frame() };

        let new_size = NSSize {
            width: window_frame.size.width,
            height: window_frame.size.height,
        };

        let origin = match position {
            "top_right" => NSPoint {
                x: visible_frame.origin.x + visible_frame.size.width - new_size.width,
                y: visible_frame.origin.y + visible_frame.size.height - new_size.height,
            },
            "center" => NSPoint {
                x: visible_frame.origin.x + (visible_frame.size.width / 2.0)
                    - (new_size.width / 2.0),
                y: visible_frame.origin.y + (visible_frame.size.height / 2.0)
                    - (new_size.height / 2.0),
            },
            _ => {
                return Err(TauriError::Anyhow(
                    Error::InvalidPosition(position.to_string()).into(),
                ))
            }
        };

        let rect = NSRect {
            origin,
            size: new_size,
        };

        let _: () = unsafe { msg_send![window_handle, setFrame: rect display: YES] };

        Ok(())
    }
}
