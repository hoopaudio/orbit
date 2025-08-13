use crate::services::chatbot::langchain::LangChainChatBot;
use dotenv::dotenv;
use tokio;

#[tokio::test]
async fn test_ask_orbit() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let chatbot = LangChainChatBot::new()?;

    let response1 = chatbot.ask_orbit("I'm from Peru").await?;
    println!("Response1: {}", response1);

    let response2 = chatbot.ask_orbit("What is the typical dish?").await?;
    println!("Response2: {}", response2);

    assert!(!response1.is_empty());
    assert!(!response2.is_empty());

    Ok(())
}

