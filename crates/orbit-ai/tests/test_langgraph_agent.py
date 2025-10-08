"""
Unit tests for LangGraph agent implementation
"""

import pytest
import pytest_asyncio
import asyncio
from unittest.mock import Mock, patch, AsyncMock, MagicMock
from langchain_core.messages import HumanMessage, SystemMessage, AIMessage

# Import the module to test
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../python'))
from orbit_ai.langgraph_agent import LangGraphAgent


class TestLangGraphAgent:
    """Test suite for LangGraph agent implementation"""

    @pytest.fixture
    def mock_llm(self):
        """Create a mock LLM for testing"""
        mock = MagicMock()
        mock.astream = AsyncMock()
        mock.ainvoke = AsyncMock()
        return mock

    @pytest.fixture
    def mock_tools(self):
        """Create mock Ableton tools"""
        from langchain_core.tools import tool

        @tool
        def mock_play():
            """Mock play tool"""
            return "Playing"

        @tool
        def mock_stop():
            """Mock stop tool"""
            return "Stopped"

        return [mock_play, mock_stop]

    @pytest.fixture
    def bot_with_mock_llm(self, mock_llm):
        """Create a bot instance with mocked LLM"""
        with patch('langchain_google_vertexai.ChatVertexAI', return_value=mock_llm):
            bot = LangGraphAgent()
            bot.llm = mock_llm
            return bot

    @pytest.mark.asyncio
    async def test_initialization(self):
        """Test bot initialization"""
        bot = LangGraphAgent()
        assert bot.provider in ["vertex_ai", "dummy"]
        assert bot.conversation_history == []
        assert bot.memory is not None

    @pytest.mark.asyncio
    async def test_ask_without_agent(self, bot_with_mock_llm, mock_llm):
        """Test asking a question without agent (no tools)"""
        bot = bot_with_mock_llm
        bot.agent = None

        # Mock LLM response
        mock_response = AIMessage(content="Test response")
        mock_llm.ainvoke.return_value = mock_response

        result = await bot.ask("Test question")

        assert result == "Test response"
        assert len(bot.conversation_history) == 2
        assert isinstance(bot.conversation_history[0], HumanMessage)
        assert isinstance(bot.conversation_history[1], AIMessage)
        mock_llm.ainvoke.assert_called_once()

    @pytest.mark.asyncio
    async def test_ask_with_agent(self, bot_with_mock_llm, mock_tools):
        """Test asking a question with LangGraph agent"""
        bot = bot_with_mock_llm

        # Mock agent executor
        mock_agent = AsyncMock()
        mock_agent.ainvoke = AsyncMock(return_value={
            "messages": [
                HumanMessage(content="Test question"),
                AIMessage(content="Agent response with tools")
            ]
        })
        bot.agent = mock_agent

        result = await bot.ask("Test question")

        assert result == "Agent response with tools"
        assert len(bot.conversation_history) == 2
        mock_agent.ainvoke.assert_called_once()

    @pytest.mark.asyncio
    async def test_streaming_with_agent(self, bot_with_mock_llm):
        """Test streaming responses with LangGraph agent"""
        bot = bot_with_mock_llm

        # Mock agent streaming
        async def mock_stream(*args, **kwargs):
            yield {"agent": {"messages": [AIMessage(content="Streaming ")]}}
            yield {"agent": {"messages": [AIMessage(content="response")]}}

        mock_agent = MagicMock()
        mock_agent.astream = mock_stream
        bot.agent = mock_agent

        # Collect streamed response
        response = ""
        async for chunk in bot.stream("Test question"):
            response += chunk

        assert "Streaming" in response or "response" in response
        assert len(bot.conversation_history) >= 2

    @pytest.mark.asyncio
    async def test_streaming_with_tools(self, bot_with_mock_llm):
        """Test streaming with tool calls"""
        bot = bot_with_mock_llm

        # Mock agent streaming with tool calls
        async def mock_stream(*args, **kwargs):
            # Tool call
            yield {
                "agent": {
                    "messages": [
                        MagicMock(
                            tool_calls=[{"name": "play_ableton"}],
                            content=""
                        )
                    ]
                }
            }
            # Tool response
            yield {
                "tools": {
                    "messages": [
                        MagicMock(content="Started playback")
                    ]
                }
            }
            # Final response
            yield {
                "agent": {
                    "messages": [
                        AIMessage(content="I've started playback in Ableton.")
                    ]
                }
            }

        mock_agent = MagicMock()
        mock_agent.astream = mock_stream
        bot.agent = mock_agent

        # Collect streamed response
        response = ""
        async for chunk in bot.stream("Play the track"):
            response += chunk

        assert "play_ableton" in response or "Started playback" in response
        assert "started playback" in response.lower() or "Using" in response

    @pytest.mark.asyncio
    async def test_clear_memory(self, bot_with_mock_llm):
        """Test clearing conversation memory"""
        bot = bot_with_mock_llm

        # Add some conversation history
        bot.conversation_history = [
            HumanMessage(content="Question 1"),
            AIMessage(content="Answer 1")
        ]

        result = bot.clear_memory()

        assert result == "Memory cleared"
        assert bot.conversation_history == []

    @pytest.mark.asyncio
    async def test_multimodal_input(self, bot_with_mock_llm, mock_llm):
        """Test handling image input with Vertex AI"""
        bot = bot_with_mock_llm
        bot.provider = "vertex_ai"

        # Mock LLM response for multimodal
        mock_response = AIMessage(content="I can see the image")
        mock_llm.ainvoke.return_value = mock_response

        # Create a temporary test image
        import tempfile
        with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as tmp:
            tmp.write(b'fake image data')
            tmp_path = tmp.name

        try:
            result = await bot.ask_with_image("What's in this image?", tmp_path)
            assert "I can see the image" in result or "Error" in result
        finally:
            os.unlink(tmp_path)

    @pytest.mark.asyncio
    async def test_error_handling_in_ask(self, bot_with_mock_llm, mock_llm):
        """Test error handling when agent fails"""
        bot = bot_with_mock_llm

        # Mock agent that raises an error
        mock_agent = AsyncMock()
        mock_agent.ainvoke.side_effect = Exception("Agent error")
        bot.agent = mock_agent

        # Mock fallback LLM response
        mock_llm.ainvoke.return_value = AIMessage(content="Fallback response")

        result = await bot.ask("Test question")

        assert result == "Fallback response"
        mock_agent.ainvoke.assert_called_once()
        mock_llm.ainvoke.assert_called_once()

    @pytest.mark.asyncio
    async def test_streaming_error_handling(self, bot_with_mock_llm, mock_llm):
        """Test error handling during streaming"""
        bot = bot_with_mock_llm

        # Mock agent that raises an error during streaming
        async def mock_stream(*args, **kwargs):
            yield {"agent": {"messages": [AIMessage(content="Start")]}}
            raise Exception("Streaming error")

        mock_agent = MagicMock()
        mock_agent.astream = mock_stream
        bot.agent = mock_agent

        # Mock fallback streaming
        async def fallback_stream(*args, **kwargs):
            yield AIMessage(content="Fallback ")
            yield AIMessage(content="stream")

        mock_llm.astream = fallback_stream

        # Collect streamed response
        response = ""
        async for chunk in bot.stream("Test question"):
            response += chunk

        assert "Error" in response or "Fallback" in response

    @pytest.mark.asyncio
    async def test_dummy_provider(self):
        """Test dummy provider behavior"""
        with patch.dict(os.environ, {"ORBIT_AI_PROVIDER": "dummy"}):
            bot = LangGraphAgent()

            assert bot.provider == "dummy"
            assert bot.llm is None
            assert bot.agent is None

            # Test dummy responses
            response = await bot.ask("hello")
            assert "dummy mode" in response.lower()

            response = await bot.ask("test orbit")
            assert "orbit" in response.lower()

    @pytest.mark.asyncio
    async def test_conversation_history_limit(self, bot_with_mock_llm, mock_llm):
        """Test that conversation history is limited to last 10 messages"""
        bot = bot_with_mock_llm

        # Add 20 messages to history
        for i in range(20):
            bot.conversation_history.append(HumanMessage(content=f"Question {i}"))
            bot.conversation_history.append(AIMessage(content=f"Answer {i}"))

        mock_llm.ainvoke.return_value = AIMessage(content="New response")

        await bot.ask("New question")

        # Check that only last 10 messages were included in the call
        call_args = mock_llm.ainvoke.call_args[0][0]
        # Should be: system message + last 10 history messages + new question
        assert len(call_args) == 12  # 1 system + 10 history + 1 new

    @pytest.mark.asyncio
    async def test_system_prompt_usage(self, bot_with_mock_llm, mock_llm):
        """Test that system prompts are properly used"""
        bot = bot_with_mock_llm
        bot.agent = None  # Use direct LLM

        mock_llm.ainvoke.return_value = AIMessage(content="Response")

        # Test with custom system prompt
        custom_prompt = "You are a music expert"
        await bot.ask("Test question", system_prompt=custom_prompt)

        call_args = mock_llm.ainvoke.call_args[0][0]
        assert isinstance(call_args[0], SystemMessage)
        assert call_args[0].content == custom_prompt

        # Test with default system prompt
        await bot.ask("Another question")

        call_args = mock_llm.ainvoke.call_args[0][0]
        assert isinstance(call_args[0], SystemMessage)
        assert "Orbit" in call_args[0].content  # Check it uses ORBIT_SYSTEM_PROMPT


if __name__ == "__main__":
    pytest.main([__file__, "-v"])