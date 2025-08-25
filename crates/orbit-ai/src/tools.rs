use crate::screenshot::ScreenshotService;
use async_trait::async_trait;
use langchain_rust::tools::Tool;
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;
use tauri::AppHandle;

pub struct ScreenshotTool<R: tauri::Runtime> {
    app_handle: Arc<AppHandle<R>>,
}

impl<R: tauri::Runtime> ScreenshotTool<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self {
            app_handle: Arc::new(app_handle),
        }
    }
}

#[async_trait]
impl<R: tauri::Runtime> Tool for ScreenshotTool<R> {
    fn name(&self) -> String {
        "take_screenshot".to_string()
    }

    fn description(&self) -> String {
        "Take a screenshot of the user's current screen and analyze its content. Use this tool when the user asks about what's on their screen, needs help with visible content, mentions visual elements, or asks about current applications. Returns information about what's visible on screen.".to_string()
    }

    async fn run(&self, _input: Value) -> Result<String, Box<dyn Error>> {
        let analysis = ScreenshotService::capture_with_ocr(&self.app_handle)
            .await
            .map_err(|e| format!("Failed to capture screenshot: {}", e))?;

        let extracted_text = analysis.ocr_result.extracted_text.trim();
        let has_meaningful_text =
            !extracted_text.is_empty() && !extracted_text.contains("Screenshot captured at");

        let result = if has_meaningful_text {
            format!(
                "I captured a screenshot of your screen ({}x{} pixels) and found the following text content:\n\n{}",
                analysis.screenshot_info.width,
                analysis.screenshot_info.height,
                extracted_text
            )
        } else {
            format!(
                "I captured a screenshot of your screen ({}x{} pixels). The image shows your current display but I couldn't extract readable text from it. This might be because the screen contains mostly graphics, images, or non-text content.",
                analysis.screenshot_info.width,
                analysis.screenshot_info.height
            )
        };

        Ok(result)
    }
}
