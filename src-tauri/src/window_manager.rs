use std::collections::HashMap;
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::EventLoopWindowTarget,
    platform::macos::{WindowBuilderExtMacOS, WindowExtMacOS},
    window::{Window, WindowBuilder, WindowId},
};

pub struct OrbitWindowManager {
    windows: HashMap<WindowId, OrbitWindow>,
    current_monitor: u32,
}

pub struct OrbitWindow {
    window: Window,
    window_type: WindowType,
    is_visible: bool,
}

#[derive(Debug, Clone)]
pub enum WindowType {
    Main,      // The main overlay
    Settings,  // Settings panel
    History,   // Conversation history
    Assistant, // Floating assistant
}

impl OrbitWindowManager {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            current_monitor: 0,
        }
    }

    pub fn create_main_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
    ) -> Result<WindowId, Box<dyn std::error::Error>> {
        let window = WindowBuilder::new()
            .with_title("Orbit")
            .with_inner_size(LogicalSize::new(500, 100))
            .with_position(LogicalPosition::new(1060, 30))
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(true)
            .with_visible(false)
            // macOS-specific enhancements
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true)
            .with_movable_by_window_background(true)
            .with_has_shadow(true)
            .build(event_loop)?;

        let window_id = window.id();
        let orbit_window = OrbitWindow {
            window,
            window_type: WindowType::Main,
            is_visible: false,
        };

        self.windows.insert(window_id, orbit_window);
        Ok(window_id)
    }

    pub fn create_settings_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
    ) -> Result<WindowId, Box<dyn std::error::Error>> {
        let window = WindowBuilder::new()
            .with_title("Orbit Settings")
            .with_inner_size(LogicalSize::new(400, 600))
            .with_decorations(true)
            .with_transparent(false)
            .with_visible(false)
            // macOS-specific styling
            .with_titlebar_transparent(false)
            .with_has_shadow(true)
            .build(event_loop)?;

        let window_id = window.id();
        let orbit_window = OrbitWindow {
            window,
            window_type: WindowType::Settings,
            is_visible: false,
        };

        self.windows.insert(window_id, orbit_window);
        Ok(window_id)
    }

    pub fn toggle_window(&mut self, window_id: WindowId) {
        let should_position_near_cursor =
            if let Some(orbit_window) = self.windows.get_mut(&window_id) {
                if orbit_window.is_visible {
                    orbit_window.window.set_visible(false);
                    orbit_window.is_visible = false;
                    false
                } else {
                    orbit_window.window.set_visible(true);
                    orbit_window.window.set_focus();
                    orbit_window.is_visible = true;

                    // Check if we need to position near cursor
                    matches!(orbit_window.window_type, WindowType::Main)
                }
            } else {
                false
            };

        // Position near cursor after releasing the mutable borrow
        if should_position_near_cursor {
            if let Some(orbit_window) = self.windows.get(&window_id) {
                self.position_near_cursor(&orbit_window.window);
            }
        }
    }

    pub fn resize_main_window(&mut self, window_id: WindowId, content_height: u32) {
        if let Some(orbit_window) = self.windows.get(&window_id) {
            if matches!(orbit_window.window_type, WindowType::Main) {
                let new_height = (content_height + 40).clamp(100, 600); // Add padding, clamp to limits
                orbit_window
                    .window
                    .set_inner_size(LogicalSize::new(500, new_height));
            }
        }
    }

    pub fn position_near_cursor(&self, window: &Window) {
        // Get cursor position and position window nearby
        // This would require additional platform-specific cursor tracking
        let position = LogicalPosition::new(1060, 30); // Fallback position
        window.set_outer_position(position);
    }

    pub fn handle_window_event(&mut self, window_id: WindowId, event: &WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(orbit_window) = self.windows.get_mut(&window_id) {
                    orbit_window.window.set_visible(false);
                    orbit_window.is_visible = false;
                }
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    // Auto-hide main window when it loses focus
                    if let Some(orbit_window) = self.windows.get_mut(&window_id) {
                        if matches!(orbit_window.window_type, WindowType::Main) {
                            orbit_window.window.set_visible(false);
                            orbit_window.is_visible = false;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn set_badge(&self, text: Option<String>) {
        // Set badge on dock icon
        for orbit_window in self.windows.values() {
            orbit_window.window.set_badge_label(text.clone());
            break; // Only need to set it once
        }
    }

    pub fn get_main_window(&self) -> Option<&Window> {
        for orbit_window in self.windows.values() {
            if matches!(orbit_window.window_type, WindowType::Main) {
                return Some(&orbit_window.window);
            }
        }
        None
    }
}
