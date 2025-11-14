use orbit_ai::NemoAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut agent = NemoAgent::new(None, None);

    println!("Testing WebSocket streaming...");

    // Example 1: Streaming with callback
    let result = agent.chat_websocket_stream(
        "what are the current tracks in ableton?",
        Some("demo-session"),
        |chunk| {
            print!("{}", chunk);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    ).await?;

    println!("\nComplete response: {}", result);

    // Example 2: Simple WebSocket chat (collects all chunks)
    let simple_result = agent.chat_websocket(
        "play the first track",
        Some("demo-session")
    ).await?;

    println!("Simple response: {}", simple_result);

    agent.disconnect_websocket().await?;
    Ok(())
}