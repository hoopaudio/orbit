use anyhow::Result;
use dotenv::dotenv;
use tokio;

use crate::services::chatbot::langchain::LangChainChatBot;
use crate::services::chatbot::service::ChatBotTrait;

#[tokio::test]
async fn test_ask_orbit() -> Result<()> {
    dotenv().ok();

    let chatbot = LangChainChatBot::new()?;

    let response1 = ChatBotTrait::ask_orbit(&chatbot, "I'm from Peru").await?;
    println!("Response1: {}", response1);

    let response2 = ChatBotTrait::ask_orbit(&chatbot, "What is the typical dish?").await?;
    println!("Response2: {}", response2);

    assert!(!response1.is_empty());
    assert!(!response2.is_empty());

    Ok(())
}
