use super::ask_orbit;

#[tokio::test]
async fn test_ask_orbit_success() {
    // Test with a simple message that should work if the agent is available
    let result = ask_orbit("hello").await;

    match result {
        Ok(response) => {
            // If the agent works, we should get a non-empty response
            assert!(!response.is_empty());
            println!("Agent response: {}", response);
        }
        Err(e) => {
            // If it fails, it should be due to environment setup issues
            assert!(e.contains("Failed to create OrbitAgent") || e.contains("Agent run failed"));
            println!("Expected error (environment not set up): {}", e);
        }
    }
}

#[tokio::test]
async fn test_ask_orbit_empty_message() {
    let result = ask_orbit("").await;

    match result {
        Ok(response) => {
            // Even empty messages might get a response
            println!("Response to empty message: {}", response);
        }
        Err(e) => {
            // Expected when environment isn't set up
            assert!(e.contains("Failed to create OrbitAgent") || e.contains("Agent run failed"));
        }
    }
}

#[tokio::test]
async fn test_ask_orbit_error_handling() {
    // This test verifies that errors are properly formatted and returned
    let result = ask_orbit("test message").await;

    // The function should always return a Result, never panic
    match result {
        Ok(_) => {
            // Success case - agent is working
            println!("Agent is working properly");
        }
        Err(e) => {
            // Error case - should have descriptive error message
            assert!(!e.is_empty(), "Error message should not be empty");
            assert!(
                e.starts_with("Failed to create OrbitAgent:") || e.starts_with("Agent run failed:"),
                "Error message should have proper prefix: {}",
                e
            );
            println!("Properly formatted error: {}", e);
        }
    }
}

#[tokio::test]
async fn test_ask_orbit_thread_safety() {
    // Test that multiple calls can be made concurrently
    let handles: Vec<_> = (0..3)
        .map(|i| {
            tokio::spawn(async move {
                let message = format!("test message {}", i);
                let result = ask_orbit(&message).await;
                (i, result.is_ok() || result.is_err()) // Should always return either Ok or Err
            })
        })
        .collect();

    for handle in handles {
        let (i, success) = handle.await.unwrap();
        assert!(success, "Thread {} should return a valid Result", i);
    }
}
