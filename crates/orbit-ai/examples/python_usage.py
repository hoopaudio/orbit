#!/usr/bin/env python3
"""
Example usage of orbit-ai Python bindings with LangChain

Note: Make sure to build and install the module first:
    cd crates/orbit-ai
    source .venv/bin/activate
    maturin develop --features python
"""

import asyncio
import os
import sys
from pathlib import Path

# Add the parent directory to the path to help with imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from orbit_ai import LangChainBot, get_system_prompt
except ImportError as e:
    print(f"Error importing orbit_ai: {e}")
    print("\nMake sure you've built and installed the module:")
    print("  cd crates/orbit-ai")
    print("  source .venv/bin/activate")
    print("  maturin develop --features python")
    sys.exit(1)


async def main():
    # Set up API key (can also be set as environment variable)
    api_key = os.getenv("OPENROUTER_API_KEY")
    if not api_key:
        print("Please set OPENROUTER_API_KEY environment variable")
        return

    # Initialize the LangChain bot
    bot = LangChainBot(api_key=api_key)

    # Get and print the system prompt
    system_prompt = get_system_prompt()
    print(f"System Prompt Preview: {system_prompt[:100]}...")
    print("-" * 50)

    # Example 1: Simple chat
    print("Example 1: Simple chat")
    response = await bot.chat("What is Orbit and what can it do?")
    print(f"Response: {response}")
    print("-" * 50)

    # Example 2: Chat with context
    print("Example 2: Follow-up question")
    response = await bot.chat("Can you tell me more about its AI features?")
    print(f"Response: {response}")
    print("-" * 50)

    # Example 3: Note about Screenshot service
    print("Example 3: Screenshot service")
    print("Note: Screenshot service requires Tauri AppHandle and is not available in standalone Python")
    print("-" * 50)

    # Example 4: Clear memory and start fresh
    print("Example 4: Clearing memory")
    await bot.clear_memory()
    response = await bot.chat("Hello! Can you introduce yourself?")
    print(f"Response after memory clear: {response}")

    # Example 5: Stream chat (for real-time responses)
    print("\nExample 5: Streaming response")
    stream_response = await bot.stream_chat("Tell me a short story about AI")
    print(f"Streamed response: {stream_response}")


if __name__ == "__main__":
    asyncio.run(main())