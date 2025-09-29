"""
Orbit AI - Python bindings for LangChain integration
"""

from .orbit_ai import (
    PyLangChainBot,
    set_openrouter_api_key,
    get_system_prompt,
)

__all__ = [
    "PyLangChainBot",
    "set_openrouter_api_key",
    "get_system_prompt",
    "LangChainBot",
]


class LangChainBot:
    """High-level Python wrapper for the LangChain bot.

    Note: This requires the bot to be created with a Tauri app handle,
    which is only available when running within a Tauri application context.
    For standalone Python usage, you'll need to modify the Rust implementation
    to make the app handle optional.
    """

    def __init__(self, api_key=None):
        """Initialize the LangChain bot.

        Args:
            api_key: Optional OpenRouter API key. If not provided, will use OPENROUTER_API_KEY env var.
        """
        if api_key:
            set_openrouter_api_key(api_key)
        # Note: PyLangChainBot creation needs to be handled differently
        # since the Rust implementation requires a Tauri AppHandle
        raise NotImplementedError(
            "PyLangChainBot requires a Tauri AppHandle. "
            "The Rust implementation needs to be modified to support standalone Python usage."
        )

    async def chat(self, message: str) -> str:
        """Send a chat message and get a response.

        Args:
            message: The message to send to the bot.

        Returns:
            The bot's response.
        """
        return await self._bot.chat(message)

    async def stream_chat(self, message: str) -> str:
        """Send a chat message and stream the response.

        Args:
            message: The message to send to the bot.

        Returns:
            The bot's streaming response.
        """
        return await self._bot.stream_chat(message)

    async def clear_memory(self):
        """Clear the bot's conversation memory."""
        await self._bot.clear_memory()