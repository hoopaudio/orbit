use anyhow::Result;
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tauri::{AppHandle, Manager};
use xcap::Monitor;

use super::ocr::{OcrResult, OcrService};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScreenshotInfo {
    pub file_path: String,
    pub width: u32,
    pub height: u32,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScreenshotAnalysis {
    pub screenshot_info: ScreenshotInfo,
    pub ocr_result: OcrResult,
}

pub struct ScreenshotService;

impl ScreenshotService {
    pub async fn capture<R: tauri::Runtime>(
        app_handle: &AppHandle<R>,
        width: u32,
        height: u32,
    ) -> Result<ScreenshotInfo> {
        let monitors = Monitor::all()?;
        let primary = monitors
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("No primary monitor found"))?;

        let captured_image = primary.capture_image()?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();

        // Convert to DynamicImage for resizing to specified resolution
        let dynamic_image = DynamicImage::ImageRgba8(captured_image);
        let resized =
            dynamic_image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);

        // Use Tauri's cache directory for screenshots
        let cache_dir = app_handle
            .path()
            .cache_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get cache directory: {}", e))?;
        let screenshot_dir = cache_dir.join("orbit_screenshots");

        // Ensure the screenshot directory exists
        std::fs::create_dir_all(&screenshot_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create screenshot directory: {}", e))?;

        let filename = format!("orbit_screenshot_{}x{}_{}.png", width, height, timestamp);
        let file_path = screenshot_dir.join(&filename);

        resized.save_with_format(&file_path, ImageFormat::Png)?;

        Ok(ScreenshotInfo {
            file_path: file_path.to_string_lossy().to_string(),
            width: resized.width(),
            height: resized.height(),
            timestamp,
        })
    }

    pub async fn capture_hd<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> Result<ScreenshotInfo> {
        Self::capture(app_handle, 1280, 720).await
    }

    pub async fn capture_with_ocr<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> Result<ScreenshotAnalysis> {
        let screenshot = Self::capture_hd(app_handle).await?;

        let fallback_message = format!(
            "Screenshot captured at {} with dimensions {}x{} pixels",
            screenshot.timestamp, screenshot.width, screenshot.height
        );

        let ocr_result =
            OcrService::extract_text_with_fallback(&screenshot.file_path, &fallback_message).await;

        Ok(ScreenshotAnalysis {
            screenshot_info: screenshot,
            ocr_result,
        })
    }

    /// Clean up all screenshot files (called on app exit)
    pub fn cleanup_all_screenshots<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> Result<()> {
        let cache_dir = app_handle
            .path()
            .cache_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get cache directory: {}", e))?;
        let screenshot_dir = cache_dir.join("orbit_screenshots");

        if !screenshot_dir.exists() {
            return Ok(());
        }

        if let Ok(read_dir) = std::fs::read_dir(&screenshot_dir) {
            for entry_result in read_dir {
                if let Ok(entry) = entry_result {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "png") {
                        if let Err(e) = std::fs::remove_file(&path) {
                            log::warn!("Failed to remove screenshot {:?}: {}", path, e);
                        } else {
                            log::debug!("Cleaned up screenshot: {:?}", path);
                        }
                    }
                }
            }
        }

        // Remove the directory if it's empty
        if let Err(e) = std::fs::remove_dir(&screenshot_dir) {
            log::debug!("Screenshot directory not empty or failed to remove: {}", e);
        }

        Ok(())
    }
}
