"""
Singleton manager for maintaining a persistent bot instance.
This solves the lifetime issues with PyO3 by keeping the Python object alive.
"""

import asyncio
from typing import Optional
from orbit_ai import NeMoAgent


class BotSingleton:
    """Manages a singleton instance of the NeMoAgent"""

    _instance: Optional[NeMoAgent] = None
    _loop: Optional[asyncio.AbstractEventLoop] = None

    @classmethod
    def get_instance(cls, api_key: Optional[str] = None) -> NeMoAgent:
        """Get or create the singleton bot instance"""
        if cls._instance is None:
            # Create the event loop first, before creating the bot
            cls.get_or_create_loop()
            cls._instance = NeMoAgent(api_key)
            print("BotSingleton: Created new NeMoAgent instance")
        else:
            print("BotSingleton: Reusing existing NeMoAgent instance")
        return cls._instance

    @classmethod
    def get_or_create_loop(cls) -> asyncio.AbstractEventLoop:
        """Get or create a persistent event loop"""
        if cls._loop is None or cls._loop.is_closed():
            cls._loop = asyncio.new_event_loop()
            asyncio.set_event_loop(cls._loop)
            print("BotSingleton: Created new event loop")
        return cls._loop

    @classmethod
    def reset_instance(cls) -> None:
        """Reset the singleton instance, forcing recreation on next access"""
        print("BotSingleton: Resetting instance")
        if cls._instance:
            # Clear memory if the bot has that method
            if hasattr(cls._instance, 'clear_memory'):
                try:
                    # Run clear_memory synchronously
                    cls._instance.clear_memory()
                except:
                    pass  # Ignore errors during cleanup
        cls._instance = None
        # Also close the event loop
        if cls._loop and not cls._loop.is_closed():
            cls._loop.close()
        cls._loop = None

    @classmethod
    def has_instance(cls) -> bool:
        """Check if an instance exists"""
        return cls._instance is not None


# Global singleton manager
_bot_singleton = BotSingleton()


def get_bot_instance(api_key: Optional[str] = None) -> NeMoAgent:
    """Get the singleton bot instance"""
    return _bot_singleton.get_instance(api_key)


def get_event_loop() -> asyncio.AbstractEventLoop:
    """Get the persistent event loop"""
    return _bot_singleton.get_or_create_loop()


def reset_bot_instance() -> None:
    """Reset the singleton bot instance"""
    _bot_singleton.reset_instance()


def has_bot_instance() -> bool:
    """Check if a bot instance exists"""
    return _bot_singleton.has_instance()


async def ask_bot_async(question: str, api_key: Optional[str] = None) -> str:
    """Async wrapper for asking the bot a question"""
    bot = get_bot_instance(api_key)
    return await bot.ask(question)


async def ask_bot_with_image_async(question: str, image_path: str, api_key: Optional[str] = None) -> str:
    """Async wrapper for asking the bot with an image"""
    bot = get_bot_instance(api_key)
    return await bot.ask_with_image(question, image_path)


async def ask_bot_stream_async(question: str, api_key: Optional[str] = None):
    """Async wrapper for streaming bot responses"""
    bot = get_bot_instance(api_key)
    async for chunk in bot.stream(question):
        yield chunk
