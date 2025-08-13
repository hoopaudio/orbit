use anyhow::Result;
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
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
    pub async fn capture(width: u32, height: u32) -> Result<ScreenshotInfo> {
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

        let temp_dir = std::env::temp_dir();
        let filename = format!("orbit_screenshot_{}x{}_{}.png", width, height, timestamp);
        let file_path = temp_dir.join(&filename);

        resized.save_with_format(&file_path, ImageFormat::Png)?;

        Ok(ScreenshotInfo {
            file_path: file_path.to_string_lossy().to_string(),
            width: resized.width(),
            height: resized.height(),
            timestamp,
        })
    }

    pub async fn capture_hd() -> Result<ScreenshotInfo> {
        Self::capture(1280, 720).await
    }


    pub async fn capture_with_ocr() -> Result<ScreenshotAnalysis> {
        let screenshot = Self::capture_hd().await?;
        
        let fallback_message = format!(
            "Screenshot captured at {} with dimensions {}x{} pixels",
            screenshot.timestamp, screenshot.width, screenshot.height
        );
        
        let ocr_result = OcrService::extract_text_with_fallback(&screenshot.file_path, &fallback_message).await;

        Ok(ScreenshotAnalysis {
            screenshot_info: screenshot,
            ocr_result,
        })
    }
}

#[tauri::command]
pub async fn capture_screenshot() -> Result<ScreenshotInfo, String> {
    ScreenshotService::capture_hd()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn capture_screenshot_with_ocr() -> Result<ScreenshotAnalysis, String> {
    ScreenshotService::capture_with_ocr()
        .await
        .map_err(|e| e.to_string())
}
