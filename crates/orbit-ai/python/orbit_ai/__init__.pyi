"""
Type stubs for orbit_ai module
"""

from typing import Optional

class PyLangChainBot:
    """Direct wrapper for Rust LangChain bot (requires Tauri AppHandle)."""
    def ask(self, question: str) -> str: ...
    def ask_with_image(self, question: str, image_path: str) -> str: ...

class LangChainBot:
    """High-level Python wrapper for the LangChain bot."""
    def __init__(self, api_key: Optional[str] = None) -> None: ...
    async def chat(self, message: str) -> str: ...
    async def stream_chat(self, message: str) -> str: ...
    async def clear_memory(self) -> None: ...

def set_openrouter_api_key(api_key: str) -> None:
    """Set the OpenRouter API key."""
    ...

def get_system_prompt() -> str:
    """Get the system prompt for Orbit."""
    ...

__all__ = [
    "PyLangChainBot",
    "LangChainBot",
    "set_openrouter_api_key",
    "get_system_prompt",
]