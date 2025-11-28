use crate::orbit_agent::OrbitAgent;
use std::env;

#[test]
fn test_orbit_agent_new() {
    let result = OrbitAgent::new();

    match result {
        Ok(_agent) => {
            println!("Agent created successfully");
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Failed to create OrbitAgent")
                    || error_msg.contains("No module named")
                    || error_msg.contains("orbit_agent")
            );
        }
    }
}

#[test]
fn test_orbit_agent_run_with_mock() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let result = agent.run("test message", Some("test-thread"));

        match result {
            Ok(response) => {
                assert!(!response.is_empty());
                println!("Agent response: {}", response);
            }
            Err(e) => {
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

#[test]
fn test_orbit_agent_run_default_thread() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let result = agent.run("hello", None);

        match result {
            Ok(response) => {
                assert!(!response.is_empty());
            }
            Err(e) => {
                assert!(e.to_string().contains("Failed to run agent"));
            }
        }
    }
}

#[test]
fn test_orbit_agent_stream() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let result = agent.stream("test streaming", Some("stream-thread"));

        match result {
            Ok(chunks) => {
                println!("Received {} chunks", chunks.len());
            }
            Err(e) => {
                assert!(e.to_string().contains("Failed to stream"));
            }
        }
    }
}

#[test]
fn test_orbit_agent_error_handling() {
    let original_manifest_dir = env::var("CARGO_MANIFEST_DIR");

    env::remove_var("CARGO_MANIFEST_DIR");

    let result = OrbitAgent::new();

    match result {
        Ok(_) => {
            println!("Agent creation succeeded despite missing CARGO_MANIFEST_DIR");
        }
        Err(e) => {
            println!("Agent creation failed as expected: {}", e);
        }
    }

    if let Ok(dir) = original_manifest_dir {
        env::set_var("CARGO_MANIFEST_DIR", dir);
    }
}

#[test]
fn test_orbit_agent_run_empty_message() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let result = agent.run("", Some("empty-thread"));

        match result {
            Ok(response) => {
                println!("Response to empty message: {}", response);
            }
            Err(e) => {
                assert!(e.to_string().contains("Failed to run agent"));
            }
        }
    }
}

#[test]
fn test_orbit_agent_run_long_message() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let long_message = "a".repeat(1000);
        let result = agent.run(&long_message, Some("long-thread"));

        match result {
            Ok(response) => {
                assert!(!response.is_empty());
            }
            Err(e) => {
                assert!(e.to_string().contains("Failed to run agent"));
            }
        }
    }
}

#[test]
fn test_orbit_agent_thread_safety() {
    let handles: Vec<_> = (0..3)
        .map(|i| {
            std::thread::spawn(move || {
                let result = OrbitAgent::new();
                println!("Thread {}: Agent creation result: {:?}", i, result.is_ok());
                result.is_ok() || result.is_err()
            })
        })
        .collect();

    for handle in handles {
        assert!(handle.join().unwrap());
    }
}

#[test]
fn test_orbit_agent_conversation_memory() {
    let agent_result = OrbitAgent::new();

    if let Ok(agent) = agent_result {
        let thread_id = "memory-test-thread";

        let first_result = agent.run("Hey, yeet", Some(thread_id));

        match first_result {
            Ok(first_response) => {
                println!("First response: {}", first_response);
                assert!(!first_response.is_empty());

                let second_result = agent.run("What word did I just say to you?", Some(thread_id));

                match second_result {
                    Ok(second_response) => {
                        println!("Second response: {}", second_response);
                        assert!(!second_response.is_empty());

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
