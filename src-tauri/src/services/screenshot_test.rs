use crate::services::ocr::OcrService;
use crate::services::screenshot::ScreenshotService;
use std::path::Path;
use tokio;

#[tokio::test]
async fn test_screenshot_to_testdata() -> Result<(), Box<dyn std::error::Error>> {
    let screenshot_info = ScreenshotService::capture_hd().await?;

    let testdata_dir = Path::new("../testdata");
    std::fs::create_dir_all(&testdata_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let target_filename = format!("test_screenshot_{}.png", timestamp);
    let target_path = testdata_dir.join(&target_filename);

    std::fs::copy(&screenshot_info.file_path, &target_path)?;

    assert!(
        target_path.exists(),
        "Screenshot should be saved to testdata directory"
    );
    assert!(
        screenshot_info.width > 0,
        "Screenshot should have valid width"
    );
    assert!(
        screenshot_info.height > 0,
        "Screenshot should have valid height"
    );
    assert!(
        !screenshot_info.timestamp.is_empty(),
        "Screenshot should have timestamp"
    );

    println!("Screenshot saved to: {}", target_path.display());
    println!(
        "Original dimensions: {}x{}",
        screenshot_info.width, screenshot_info.height
    );

    Ok(())
}

#[tokio::test]
async fn test_screenshot_and_analyze_to_testdata() -> Result<(), Box<dyn std::error::Error>> {
    let analysis = ScreenshotService::capture_with_ocr().await?;

    let testdata_dir = Path::new("../testdata");
    std::fs::create_dir_all(&testdata_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let target_filename = format!("test_analysis_screenshot_{}.png", timestamp);
    let target_path = testdata_dir.join(&target_filename);

    std::fs::copy(&analysis.screenshot_info.file_path, &target_path)?;

    assert!(
        target_path.exists(),
        "Analyzed screenshot should be saved to testdata directory"
    );
    assert!(
        analysis.ocr_result.confidence >= 0,
        "Analysis should have confidence score"
    );
    assert!(
        !analysis.ocr_result.extracted_text.is_empty(),
        "Analysis should have extracted text"
    );
    assert!(
        analysis.screenshot_info.width > 0,
        "Screenshot should have valid width"
    );
    assert!(
        analysis.screenshot_info.height > 0,
        "Screenshot should have valid height"
    );

    println!("Analyzed screenshot saved to: {}", target_path.display());
    println!("Analysis text: {}", analysis.ocr_result.extracted_text);
    println!("Confidence: {}", analysis.ocr_result.confidence);

    Ok(())
}

#[tokio::test]
async fn test_screenshot_720p_for_llm() -> Result<(), Box<dyn std::error::Error>> {
    let testdata_dir = Path::new("../testdata");
    std::fs::create_dir_all(&testdata_dir)?;

    let screenshot = ScreenshotService::capture_hd().await?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let target_filename = format!("test_720p_{}.png", timestamp);
    let target_path = testdata_dir.join(&target_filename);

    std::fs::copy(&screenshot.file_path, &target_path)?;

    assert!(target_path.exists(), "Screenshot 720p should be saved");
    assert_eq!(screenshot.width, 1280, "720p should have width 1280");
    assert_eq!(screenshot.height, 720, "720p should have height 720");

    // Get file size for analysis
    let file_size = std::fs::metadata(&target_path)?.len();

    println!(
        "720p Screenshot: 1280x720 pixels, File size: {} KB",
        file_size / 1024
    );
    println!("Saved to: {}", target_path.display());

    Ok(())
}

#[tokio::test]
async fn test_ocr_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let testdata_dir = Path::new("../testdata");
    std::fs::create_dir_all(&testdata_dir)?;

    // Use the specific test image
    let test_image_path = testdata_dir.join("high_res_screenshot.png");

    // Verify the test image exists
    assert!(
        test_image_path.exists(),
        "Test image should exist: {}",
        test_image_path.display()
    );

    // Extract text from the specific test image
    let ocr_result = OcrService::extract_text(test_image_path.to_str().unwrap()).await?;

    // Validate the analysis
    assert!(!ocr_result.extracted_text.is_empty(), "OCR should extract some text");
    assert!(ocr_result.confidence >= 0, "Confidence should be non-negative");

    println!("OCR Test Results:");
    println!("Test image: {}", test_image_path.display());
    println!("Extracted text: '{}'", ocr_result.extracted_text);
    println!("Confidence: {}%", ocr_result.confidence);
    println!("Text length: {} characters", ocr_result.extracted_text.len());

    // Check if OCR actually worked
    if ocr_result.confidence > 0 {
        println!("✅ OCR extraction successful!");
    } else {
        println!("⚠️  OCR failed");
    }

    Ok(())
}
