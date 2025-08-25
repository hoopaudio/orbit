use crate::langchain::LangChainChatBot;
use dotenv::dotenv;
use std::error::Error as StdError;
use tokio;

#[tokio::test]
async fn test_ask_orbit() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let chatbot = LangChainChatBot::new_for_test()?;

    let response1 = chatbot.ask_orbit("I'm from Peru").await?;
    println!("Response1: {}", response1);

    let response2 = chatbot.ask_orbit("What is the typical dish?").await?;
    println!("Response2: {}", response2);

    assert!(!response1.is_empty());
    assert!(!response2.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_ask_orbit_with_testdata_image() -> Result<(), Box<dyn StdError>> {
    dotenv().ok();
    let chatbot = LangChainChatBot::new_for_test()?;
    let test_image_path = "/Users/cuthlehoop/projects/orbit/testdata/high_res_screenshot.png";
    let question = "what's on my screen right now";

    let response = chatbot
        .ask_orbit_with_image(question, test_image_path)
        .await?;
    println!("Response with testdata image: {}", response);

    assert!(!response.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_ask_orbit_screenshot_tool() -> Result<(), Box<dyn StdError>> {
    dotenv().ok();
    let chatbot = LangChainChatBot::new_for_test()?;
    let question = "what's on my screen right now";

    // This should trigger the screenshot tool automatically
    let response = chatbot.ask_orbit(question).await?;
    println!("Response with screenshot tool: {}", response);

    assert!(!response.is_empty());

    Ok(())
}
