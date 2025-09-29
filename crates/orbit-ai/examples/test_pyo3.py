#!/usr/bin/env python3
"""
Test PyO3 bindings for orbit-ai
"""

import os
import orbit_ai


def main():
    print("Testing orbit-ai PyO3 bindings...")
    print("-" * 50)

    # Test getting the system prompt
    system_prompt = orbit_ai.get_system_prompt()
    print(f"System prompt (first 200 chars):")
    print(f"{system_prompt[:200]}...")
    print("-" * 50)

    # Test setting API key
    test_key = "test-api-key-123"
    orbit_ai.set_openrouter_api_key(test_key)
    print(f"Set OpenRouter API key: {test_key}")

    # Verify it was set
    api_key = os.environ.get("OPENROUTER_API_KEY")
    print(f"Verified API key in environment: {api_key}")
    print("-" * 50)

    # Note about PyLangChainBot
    print("Note: PyLangChainBot requires a Tauri AppHandle and cannot be")
    print("instantiated in standalone Python. To use it, you would need to")
    print("modify the Rust implementation to make AppHandle optional.")
    print()
    print("Available functions and classes:")
    print("  - orbit_ai.get_system_prompt()")
    print("  - orbit_ai.set_openrouter_api_key(api_key)")
    print("  - orbit_ai.PyLangChainBot (requires Tauri context)")


if __name__ == "__main__":
    main()