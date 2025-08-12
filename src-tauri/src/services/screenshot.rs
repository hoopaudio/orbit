use xcap::Monitor;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use image::{DynamicImage, ImageFormat};

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
    pub extracted_text: String,
    pub confidence: i32,
}

pub struct ScreenshotService;

impl ScreenshotService {
    pub async fn capture_720p() -> Result<ScreenshotInfo> {
        let monitors = Monitor::all()?;
        let primary = monitors
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("No primary monitor found"))?;

        let captured_image = primary.capture_image()?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        
        // Convert to DynamicImage for resizing to 720p
        let dynamic_image = DynamicImage::ImageRgba8(captured_image);
        let resized = dynamic_image.resize_exact(1280, 720, image::imageops::FilterType::Lanczos3);
        
        let temp_dir = std::env::temp_dir();
        let filename = format!("orbit_screenshot_720p_{}.png", timestamp);
        let file_path = temp_dir.join(&filename);
        
        resized.save_with_format(&file_path, ImageFormat::Png)?;

        Ok(ScreenshotInfo {
            file_path: file_path.to_string_lossy().to_string(),
            width: resized.width(),
            height: resized.height(),
            timestamp,
        })
    }

    pub async fn capture_and_analyze() -> Result<ScreenshotAnalysis> {
        let screenshot = Self::capture_720p().await?;
        
        // For now, return basic screenshot info without OCR
        // OCR can be added later when tesseract system dependencies are available
        let analysis_text = format!(
            "Screenshot captured at {} with dimensions {}x{} pixels. The image has been saved to: {}",
            screenshot.timestamp, screenshot.width, screenshot.height, screenshot.file_path
        );
        
        Ok(ScreenshotAnalysis {
            screenshot_info: screenshot,
            extracted_text: analysis_text,
            confidence: 100, // High confidence for basic image capture info
        })
    }
}

#[tauri::command]
pub async fn take_screenshot() -> Result<ScreenshotInfo, String> {
    ScreenshotService::capture_720p()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn screenshot_and_analyze() -> Result<ScreenshotAnalysis, String> {
    ScreenshotService::capture_and_analyze()
        .await
        .map_err(|e| e.to_string())
}