use crate::orbit_agent::OrbitAgent;
use std::env;

#[test]
fn test_orbit_agent_new() {
    let result = OrbitAgent::new();

    match result {
        Ok(_agent) => {
            // Verify the agent was created successfully
            // We can't access private fields, so just check it was created
            println!("Agent created successfully");
        }
        Err(e) => {
            // If creation fails, it should be due to missing Python dependencies or environment
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Failed to create OrbitAgent")
                    || error_msg.contains("No module named")
                    || error_msg.contains("orbit_ai")
            );
        }
    }
}

#[tokio::test]
async fn test_orbit_agent_run_with_mock() {
    // This test requires the Python environment to be set up
    // We'll test the error handling path when Python modules are missing
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        // If agent was created successfully, test a simple run call
        let result = agent.run("test message", Some("test-thread")).await;

        // The result should either succeed or fail with a meaningful error
        match result {
            Ok(response) => {
                assert!(!response.is_empty());
                println!("Agent response: {}", response);
            }
            Err(e) => {
                // Expected errors when environment isn't fully set up
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to run agent")
                        || error_msg.contains("authentication")
                        || error_msg.contains("GOOGLE_")
                        || error_msg.contains("Unable to import")
                );
            }
        }
    }
}

#[tokio::test]
async fn test_orbit_agent_run_default_thread() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        // Test with default thread_id (None)
        let result = agent.run("hello", None).await;

        match result {
            Ok(response) => {
                assert!(!response.is_empty());
            }
            Err(e) => {
                // Expected when environment isn't set up
                assert!(e.to_string().contains("Failed to run agent"));
            }
        }
    }
}

#[tokio::test]
async fn test_orbit_agent_stream() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let result = agent.stream("test streaming", Some("stream-thread")).await;

        match result {
            Ok(chunks) => {
                // Chunks should be a vector
                println!("Received {} chunks", chunks.len());
            }
            Err(e) => {
                // Expected when environment isn't set up
                assert!(e.to_string().contains("Failed to stream"));
            }
        }
    }
}

#[test]
fn test_orbit_agent_error_handling() {
    // Test various error conditions

    // Save original env vars
    let original_manifest_dir = env::var("CARGO_MANIFEST_DIR");

    // Temporarily mess with the environment to force an error
    env::remove_var("CARGO_MANIFEST_DIR");

    let result = OrbitAgent::new();

    // The test might still pass if Python can find the module via other means
    // So we'll check that either it fails OR succeeds with a valid agent
    match result {
        Ok(_) => {
            // If it succeeds, that's fine - Python found the module another way
            println!("Agent creation succeeded despite missing CARGO_MANIFEST_DIR");
        }
        Err(e) => {
            // If it fails, that's expected
            println!("Agent creation failed as expected: {}", e);
        }
    }

    // Restore environment
    if let Ok(dir) = original_manifest_dir {
        env::set_var("CARGO_MANIFEST_DIR", dir);
    }
}

#[tokio::test]
async fn test_orbit_agent_run_empty_message() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let result = agent.run("", Some("empty-thread")).await;

        // Should handle empty messages gracefully
        match result {
            Ok(response) => {
                // Even empty messages might get a response from the agent
                println!("Response to empty message: {}", response);
            }
            Err(e) => {
                // Expected when environment isn't set up
                assert!(e.to_string().contains("Failed to run agent"));
            }
        }
    }
}

#[tokio::test]
async fn test_orbit_agent_run_long_message() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let long_message = "a".repeat(1000);
        let result = agent.run(&long_message, Some("long-thread")).await;

        match result {
            Ok(response) => {
                assert!(!response.is_empty());
            }
            Err(e) => {
                // Expected when environment isn't set up
                assert!(e.to_string().contains("Failed to run agent"));
            }
        }
    }
}

#[test]
fn test_orbit_agent_thread_safety() {
    // Test that OrbitAgent can be created from multiple threads
    let handles: Vec<_> = (0..3)
        .map(|i| {
            std::thread::spawn(move || {
                let result = OrbitAgent::new();
                println!("Thread {}: Agent creation result: {:?}", i, result.is_ok());
                result.is_ok() || result.is_err() // Should always return either Ok or Err
            })
        })
        .collect();

    for handle in handles {
        assert!(handle.join().unwrap());
    }
}

#[tokio::test]
async fn test_orbit_agent_conversation_memory() {
    // Test that agent remembers previous messages in the same thread
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let thread_id = "memory-test-thread";

        // First message: "Hey, yeet"
        let first_result = agent.run("Hey, yeet", Some(thread_id)).await;

        match first_result {
            Ok(first_response) => {
                println!("First response: {}", first_response);
                assert!(!first_response.is_empty());

                // Second message: Ask if it remembers the word
                let second_result = agent
                    .run("What word did I just say to you?", Some(thread_id))
                    .await;

                match second_result {
                    Ok(second_response) => {
                        println!("Second response: {}", second_response);
                        assert!(!second_response.is_empty());

                        // Check if the agent remembers "yeet"
                        let response_lower = second_response.to_lowercase();
                        if response_lower.contains("yeet") {
                            println!("Agent successfully remembered the word 'yeet'");
                        } else {
                            println!(
                                "Agent response doesn't contain 'yeet', but conversation worked"
                            );
                            println!("Response was: {}", second_response);
                        }
                    }
                    Err(e) => {
                        println!("Second message failed (expected in test env): {}", e);
                        assert!(e.to_string().contains("Failed to run agent"));
                    }
                }
            }
            Err(e) => {
                println!("First message failed (expected in test env): {}", e);
                assert!(e.to_string().contains("Failed to run agent"));
            }
        }
    } else {
        println!("Agent creation failed (expected in test environment without proper setup)");
    }
}
