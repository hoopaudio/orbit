#!/usr/bin/env python3
"""Test script for LangChain implementation"""

import asyncio
import os
import sys

# Add the python directory to the path
sys.path.insert(0, 'python')

# Set environment variables
os.environ["ORBIT_AI_PROVIDER"] = "vertex_ai"
os.environ["VERTEX_PROJECT_ID"] = "hoop-audio-472016"
os.environ["VERTEX_LOCATION"] = "us-central1"
os.environ["VERTEX_MODEL"] = "gemini-1.5-flash"


async def test_langchain():
    """Test the LangChain bot"""
    from orbit_ai.langchain_bot import LangChainBot

    print("Creating LangChain bot...")
    bot = LangChainBot()

    print(f"Provider: {bot.provider}")
    print(f"LLM configured: {bot.llm is not None}")

    # Test basic question
    print("\n--- Testing basic question ---")
    response = await bot.ask("What is the capital of France?")
    print(f"Response: {response}")

    # Test streaming
    print("\n--- Testing streaming ---")
    print("Stream: ", end="")
    async for chunk in bot.stream("Tell me a very short joke"):
        print(chunk, end="", flush=True)
    print()

    # Test memory
    print("\n--- Testing conversation memory ---")
    await bot.ask("My name is Alice")
    response = await bot.ask("What is my name?")
    print(f"Memory test response: {response}")

    # Clear memory
    bot.clear_memory()
    print("Memory cleared")


if __name__ == "__main__":
    asyncio.run(test_langchain())