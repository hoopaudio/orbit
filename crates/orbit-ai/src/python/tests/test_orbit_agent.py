"""
Unit tests for OrbitAgent.
Tests the agent's ability to handle streaming, tool calling, and error recovery.
"""

import pytest
from unittest.mock import AsyncMock, MagicMock

from langchain_core.messages import (
    HumanMessage,
    AIMessage,
    SystemMessage,
    AIMessageChunk,
    ToolMessage,
)
from langchain_core.tools import BaseTool, tool
from langchain_litellm import ChatLiteLLM
from langgraph.graph.state import CompiledStateGraph

# Import the agent
import sys
import os

from orbit_ai.orbit_agent.prompt import ORBIT_SYSTEM_PROMPT

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from orbit_ai.orbit_agent import OrbitAgent, OrbitState, OrbitContext


class MockTool(BaseTool):
    """Mock tool for testing."""
    name: str = "mock_tool"
    description: str = "A mock tool for testing"

    def _run(self, *args, **kwargs):
        return "Mock tool response"

    async def _arun(self, *args, **kwargs):
        return "Mock tool async response"


@tool
def mock_play_tool() -> str:
    """Start playback in Ableton Live."""
    return "Playback started"


@tool
def mock_stop_tool() -> str:
    """Stop playback in Ableton Live."""
    return "Playback stopped"


class TestOrbitAgent:
    """Test suite for OrbitAgent."""

    @pytest.fixture
    def mock_llm(self):
        """Create a mock LLM."""
        llm = MagicMock(spec=ChatLiteLLM)
        llm.bind_tools = MagicMock(return_value=llm)
        return llm

    @pytest.fixture
    def mock_tools(self):
        """Create mock tools for testing."""
        return [MockTool(), mock_play_tool, mock_stop_tool]

    @pytest.fixture
    def agent(self):
        """Create an OrbitAgent instance for testing."""
        context = OrbitContext(
            system_prompt="You are a test agent.",
            detailed_logs=True
        )
        return OrbitAgent(context=context)

    @pytest.mark.asyncio
    async def test_agent_initialization(self, agent):
        """Test that agent initializes correctly."""
        assert agent.context is not None
        assert agent.graph is not None
        assert agent.context.system_prompt == "You are a test agent."
        assert agent.context.detailed_logs is True

    @pytest.mark.asyncio
    async def test_agent_has_tools(self, agent):
        """Test that agent has access to Ableton tools."""
        from orbit_ai.orbit_agent.tools import ABLETON_TOOLS
        # Verify tools are available
        assert len(ABLETON_TOOLS) > 0
        # Agent should be initialized successfully with tools
        assert agent.graph is not None

    @pytest.mark.asyncio
    async def test_orbit_agent_yeet(self):
        agent = OrbitAgent(OrbitContext())

        # Test streaming for first message
        print("\nFirst message streaming:")
        chunks = []
        async for chunk in agent.stream("yeet", thread_id="test-conversation"):
            print(chunk, end="", flush=True)
            chunks.append(chunk)
        first_response = "".join(chunks)
        print(f"\nComplete first response: {first_response}")

        # Test streaming for second message (should remember "yeet")
        print("\nSecond message streaming:")
        chunks = []
        async for chunk in agent.stream(
            "what was the weird word that I had sent you earlier? say it without saying anything else.",
            thread_id="test-conversation"
        ):
            print(chunk, end="", flush=True)
            chunks.append(chunk)
        second_response = "".join(chunks)
        print(f"\nComplete second response: {second_response}")


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
