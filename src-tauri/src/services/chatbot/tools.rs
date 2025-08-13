use async_trait::async_trait;
use langchain_rust::tools::Tool;
use serde_json::Value;
use std::error::Error;
use crate::services::screenshot::ScreenshotService;

pub struct ScreenshotTool;

impl ScreenshotTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ScreenshotTool {
    fn name(&self) -> String {
        "take_screenshot".to_string()
    }

    fn description(&self) -> String {
        "Take a screenshot of the user's current screen and analyze its content. Use this tool when the user asks about what's on their screen, needs help with visible content, mentions visual elements, or asks about current applications. Returns information about what's visible on screen.".to_string()
    }

    async fn run(&self, _input: Value) -> Result<String, Box<dyn Error>> {
        match ScreenshotService::capture_with_ocr().await {
            Ok(analysis) => {
                let result = if analysis.ocr_result.extracted_text.trim().is_empty() || analysis.ocr_result.extracted_text.contains("Screenshot captured at") {
                    format!(
                        "I captured a screenshot of your screen ({}x{} pixels). The image shows your current display but I couldn't extract readable text from it. This might be because the screen contains mostly graphics, images, or non-text content.",
                        analysis.screenshot_info.width,
                        analysis.screenshot_info.height
                    )
                } else {
                    format!(
                        "I captured a screenshot of your screen ({}x{} pixels) and found the following text content:\n\n{}",
                        analysis.screenshot_info.width,
                        analysis.screenshot_info.height,
                        analysis.ocr_result.extracted_text
                    )
                };
                Ok(result)
            }
            Err(e) => Err(format!("Failed to capture screenshot: {}", e).into())
        }
    }
}