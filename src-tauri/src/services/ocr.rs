use anyhow::Result;
use serde::{Deserialize, Serialize};
use tesseract::Tesseract;

#[derive(Serialize, Deserialize, Debug)]
pub struct OcrResult {
    pub extracted_text: String,
    pub confidence: i32,
}

pub struct OcrService;

impl OcrService {
    pub async fn extract_text(image_path: &str) -> Result<OcrResult> {
        let mut tesseract = Tesseract::new(None, Some("eng"))?
            .set_variable("tessedit_pageseg_mode", "3")? // Fully automatic page segmentation
            .set_variable("tessedit_ocr_engine_mode", "3")? // Default engine (more stable)
            .set_image(image_path)?;

        let text = tesseract.get_text()?;
        let confidence = tesseract.mean_text_conf();

        Ok(OcrResult {
            extracted_text: text.trim().to_string(),
            confidence,
        })
    }

    pub async fn extract_text_with_fallback(image_path: &str, fallback_message: &str) -> OcrResult {
        match Self::extract_text(image_path).await {
            Ok(result) => {
                if result.extracted_text.is_empty() {
                    OcrResult {
                        extracted_text: "No text detected in image".to_string(),
                        confidence: 0,
                    }
                } else {
                    result
                }
            }
            Err(e) => {
                log::warn!("OCR failed: {}. Using fallback message.", e);
                OcrResult {
                    extracted_text: format!("{}. OCR failed: {}", fallback_message, e),
                    confidence: 0,
                }
            }
        }
    }
}