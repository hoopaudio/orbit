"""
Dummy LangChain bot implementation for MVP testing.
Returns hardcoded responses without requiring any external dependencies.
"""

import asyncio
import random
from typing import AsyncGenerator


class DummyLangChainBot:
    """A dummy LangChain bot that returns hardcoded responses."""

    def __init__(self, api_key: str = None):
        """Initialize the dummy bot (api_key is ignored in this implementation)."""
        self.api_key = api_key
        self.responses = [
            "This is a dummy response from the Python LangChain bot!",
            "I'm currently running as a placeholder implementation.",
            "The real LangChain integration will be added later.",
            "For now, I return hardcoded responses for testing.",
            "This helps test the Python-Rust integration without dependencies."
        ]
        self.conversation_count = 0

    async def ask(self, question: str) -> str:
        """Return a hardcoded response for any question."""
        # Simulate some async processing time
        await asyncio.sleep(0.1)

        self.conversation_count += 1

        # Return different responses based on the question content
        if "hello" in question.lower() or "hi" in question.lower():
            return "Hello! I'm the dummy Python LangChain bot. How can I pretend to help you today?"
        elif "orbit" in question.lower():
            return "Orbit is an AI-powered productivity tool. (This is a hardcoded response from Python)"
        elif "screenshot" in question.lower():
            return "Screenshot functionality is not available in this dummy implementation."
        elif "test" in question.lower():
            return f"Test successful! This is response #{self.conversation_count} from the Python dummy bot."
        else:
            # Return a random response from the list
            return random.choice(self.responses)

    async def stream(self, question: str) -> AsyncGenerator[str, None]:
        """Stream a response word by word for testing streaming functionality."""
        response = await self.ask(question)
        words = response.split()

        for word in words:
            await asyncio.sleep(0.05)  # Simulate streaming delay
            yield word + " "

    async def ask_with_image(self, question: str, image_path: str) -> str:
        """Return a hardcoded response for image-based questions."""
        await asyncio.sleep(0.1)
        return f"I received your image at '{image_path}' with question '{question}'. (Dummy response - no actual image processing)"

    def clear_memory(self):
        """Reset the conversation counter."""
        self.conversation_count = 0
        return "Memory cleared (dummy implementation)"


# Factory function to create bot instance
def create_bot(api_key: str = None) -> DummyLangChainBot:
    """Create a new instance of the dummy bot."""
    return DummyLangChainBot(api_key)