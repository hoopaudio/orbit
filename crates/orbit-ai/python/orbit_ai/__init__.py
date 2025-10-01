"""
Orbit AI - Professional AI Agent with Multi-Provider Support
"""

# Import from Rust bindings (if available)
try:
    from .orbit_ai import (
        PyLangChainBot,
        set_openrouter_api_key,
        get_system_prompt,
    )
except ImportError:
    # Rust bindings not available, use pure Python
    PyLangChainBot = None
    set_openrouter_api_key = None
    get_system_prompt = None

# Import pure Python implementations
from .orbit_pro_agent import OrbitProAgent, create_agent
from .langchain_bot import LangChainBot

__all__ = [
    "OrbitProAgent",
    "LangChainBot",
    "create_agent",
    "PyLangChainBot",
    "set_openrouter_api_key",
    "get_system_prompt",
]