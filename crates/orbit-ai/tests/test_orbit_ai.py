"""
Tests for orbit-ai Python bindings
"""

import pytest
import orbit_ai


def test_import():
    """Test that the module can be imported."""
    assert orbit_ai is not None
    assert hasattr(orbit_ai, 'get_system_prompt')
    assert hasattr(orbit_ai, 'set_openrouter_api_key')
    assert hasattr(orbit_ai, 'PyLangChainBot')


def test_get_system_prompt():
    """Test getting the system prompt."""
    prompt = orbit_ai.get_system_prompt()
    assert isinstance(prompt, str)
    assert len(prompt) > 0
    assert "Orbit" in prompt
    assert "music producer" in prompt


def test_set_openrouter_api_key():
    """Test setting the OpenRouter API key."""
    test_key = "test-key-12345"
    orbit_ai.set_openrouter_api_key(test_key)
    # Note: The env var is set in Rust context, not Python
    # So we can't verify it directly from Python's os.environ


def test_module_contents():
    """Test that expected items are in the module."""
    expected_items = [
        'PyLangChainBot',
        'get_system_prompt',
        'set_openrouter_api_key',
        'LangChainBot',
    ]

    module_dir = dir(orbit_ai)
    for item in expected_items:
        assert item in module_dir, f"{item} not found in module"


def test_langchain_bot_class_exists():
    """Test that LangChainBot class exists and has expected structure."""
    assert hasattr(orbit_ai, 'LangChainBot')
    bot_class = orbit_ai.LangChainBot

    # Check that it's a class
    assert isinstance(bot_class, type)

    # Check expected methods exist
    expected_methods = ['__init__', 'chat', 'stream_chat', 'clear_memory']
    for method in expected_methods:
        assert hasattr(bot_class, method), f"Method {method} not found"


def test_pylangchainbot_instantiation_fails_without_handle():
    """Test that PyLangChainBot cannot be instantiated without Tauri handle."""
    # This should raise an error since we don't have a Tauri AppHandle
    with pytest.raises(NotImplementedError) as excinfo:
        bot = orbit_ai.LangChainBot()

    assert "Tauri AppHandle" in str(excinfo.value)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])