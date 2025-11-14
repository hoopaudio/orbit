"""
Unit tests for OrbitAgent with NeMo Agent Toolkit integration.
Tests the agent's ability to handle streaming, tool calling, and error recovery.
"""

import pytest
from unittest.mock import AsyncMock, MagicMock

from langchain_aws.chains.graph_qa.neptune_cypher import get_prompt
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
from orbit_ai import OrbitAgent, OrbitState, OrbitContext


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
    def agent(self, mock_llm, mock_tools):
        """Create an OrbitAgent instance for testing."""
        return OrbitAgent(
            llm=mock_llm,
            tools=mock_tools,
            system_prompt="You are a test agent.",
            detailed_logs=True,
            max_retries=2
        )

    @pytest.mark.asyncio
    async def test_agent_initialization(self, agent, mock_tools):
        """Test that agent initializes correctly."""
        assert agent.llm is not None
        assert len(agent.tools) == 3
        assert agent.system_prompt == "You are a test agent."
        assert agent.detailed_logs is True
        assert agent.max_retries == 2
        assert agent.graph is None

    @pytest.mark.asyncio
    async def test_build_graph(self, agent):
        """Test graph building and compilation."""
        graph = await agent.build_graph()

        assert graph is not None
        assert isinstance(graph, CompiledStateGraph)
        assert agent.graph == graph

    @pytest.mark.asyncio
    async def test_should_continue_with_tool_calls(self, agent):
        """Test the should_continue method with tool calls."""
        # Create a mock message with tool calls
        message_with_tools = AIMessage(content="Using tools...")
        message_with_tools.tool_calls = [{"name": "mock_tool", "args": {}}]

        state = OrbitAgentState(
            messages=[HumanMessage(content="Test"), message_with_tools],
            retry_count=0,
            user_query="Test"
        )

        result = agent.should_continue(state)
        assert result == "continue"

    @pytest.mark.asyncio
    async def test_should_continue_without_tool_calls(self, agent):
        """Test the should_continue method without tool calls."""
        message_without_tools = AIMessage(content="No tools needed.")

        state = OrbitAgentState(
            messages=[HumanMessage(content="Test"), message_without_tools],
            retry_count=0,
            user_query="Test"
        )

        result = agent.should_continue(state)
        assert result == "end"

    @pytest.mark.asyncio
    async def test_agent_node_streaming(self, agent, mock_llm):
        """Test the agent_node with streaming response."""

        # Create mock streaming response
        async def mock_astream(*args, **kwargs):
            chunks = [
                AIMessageChunk(content="Hello "),
                AIMessageChunk(content="from "),
                AIMessageChunk(content="streaming!"),
            ]
            for chunk in chunks:
                yield chunk

        mock_llm.astream = mock_astream
        agent.llm_with_tools = mock_llm

        state = OrbitAgentState(
            messages=[HumanMessage(content="Test message")],
            retry_count=0,
            user_query="Test message"
        )

        result = await agent.agent_node(state)

        assert "messages" in result
        # Should have original message, system message, and AI response
        assert len(result["messages"]) == 3
        assert isinstance(result["messages"][0], SystemMessage)
        assert isinstance(result["messages"][1], HumanMessage)
        assert isinstance(result["messages"][2], AIMessageChunk)

    @pytest.mark.asyncio
    async def test_agent_node_error_handling(self, agent, mock_llm):
        """Test error handling and retries in agent_node."""
        # Make astream raise an exception
        mock_llm.astream = AsyncMock(side_effect=Exception("Test error"))
        agent.llm_with_tools = mock_llm

        state = OrbitAgentState(
            messages=[HumanMessage(content="Test message")],
            retry_count=0,
            user_query="Test message"
        )

        result = await agent.agent_node(state)

        assert "messages" in result
        assert "retry_count" in result
        assert result["retry_count"] == 1
        # Check that error message was added
        last_message = result["messages"][-1]
        assert isinstance(last_message, AIMessage)
        assert "Error occurred" in last_message.content

    @pytest.mark.asyncio
    async def test_agent_node_max_retries(self, agent, mock_llm):
        """Test that agent stops after max retries."""
        mock_llm.astream = AsyncMock(side_effect=Exception("Test error"))
        agent.llm_with_tools = mock_llm

        state = OrbitAgentState(
            messages=[HumanMessage(content="Test message")],
            retry_count=2,  # Already at max retries
            user_query="Test message"
        )

        result = await agent.agent_node(state)

        assert "messages" in result
        last_message = result["messages"][-1]
        assert isinstance(last_message, AIMessage)
        assert "Failed after 2 retries" in last_message.content

    @pytest.mark.asyncio
    async def test_stream_method(self, agent):
        """Test the stream method for async iteration."""

        # Mock the graph's astream method
        async def mock_graph_astream(*args, **kwargs):
            yield {
                "agent": {
                    "messages": [
                        HumanMessage(content="Test"),
                        AIMessage(content="Response chunk 1")
                    ]
                }
            }
            yield {
                "tools": {
                    "messages": [
                        HumanMessage(content="Test"),
                        AIMessage(content="Response chunk 1"),
                        ToolMessage(content="Tool result", tool_call_id="123")
                    ]
                }
            }
            yield {
                "agent": {
                    "messages": [
                        HumanMessage(content="Test"),
                        AIMessage(content="Response chunk 1"),
                        ToolMessage(content="Tool result", tool_call_id="123"),
                        AIMessage(content="Final response")
                    ]
                }
            }

        mock_graph = MagicMock()
        mock_graph.astream = mock_graph_astream
        agent.graph = mock_graph

        chunks = []
        async for chunk in agent.stream("Test query"):
            chunks.append(chunk)

        assert len(chunks) == 2
        assert chunks[0] == "Response chunk 1"
        assert chunks[1] == "Final response"

    @pytest.mark.asyncio
    async def test_stream_error_handling(self, agent):
        """Test error handling in the stream method."""

        # Create an async generator that raises an error
        async def error_generator(*args, **kwargs):
            raise Exception("Stream error")
            yield  # Make it a generator

        mock_graph = MagicMock()
        mock_graph.astream = error_generator
        agent.graph = mock_graph

        chunks = []
        async for chunk in agent.stream("Test query"):
            chunks.append(chunk)

        assert len(chunks) == 1
        assert "Error: Stream error" in chunks[0]

    @pytest.mark.asyncio
    async def test_run_method(self, agent):
        """Test the run method that collects all stream chunks."""

        # Mock the stream method
        async def mock_stream(user_input):
            yield "Hello "
            yield "world!"

        agent.stream = mock_stream

        result = await agent.run("Test query")

        assert result == "Hello world!"

    @pytest.mark.asyncio
    async def test_run_method_no_response(self, agent):
        """Test run method when no response is generated."""

        # Mock stream to return nothing
        async def mock_stream(user_input):
            return
            yield  # Make it a generator but yield nothing

        agent.stream = mock_stream

        result = await agent.run("Test query")

        assert result == "No response generated."

    @pytest.mark.asyncio
    async def test_integration_with_litellm_patch(self):
        """Test that the LiteLLM patch is applied correctly."""
        # Import the litellm_fix module
        import importlib.util
        current_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        litellm_fix_path = os.path.join(current_dir, "..", "litellm_fix.py")

        if os.path.exists(litellm_fix_path):
            spec = importlib.util.spec_from_file_location("litellm_fix", litellm_fix_path)
            litellm_fix = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(litellm_fix)

            # After patch, ChatLiteLLM should have model attributes
            from langchain_litellm import ChatLiteLLM

            # Create an instance with model parameter
            llm = ChatLiteLLM(model="test-model")

            # Check that model attributes exist
            assert hasattr(llm, "model")
            assert hasattr(llm, "model_name")
            assert llm.model == "test-model"
            assert llm.model_name == "test-model"

    @pytest.mark.asyncio
    async def test_tool_binding(self, mock_llm, mock_tools):
        """Test that tools are properly bound to the LLM."""
        agent = OrbitAgent(
            llm=mock_llm,
            tools=mock_tools,
            system_prompt="Test",
        )

        # Verify bind_tools was called with the tools
        mock_llm.bind_tools.assert_called_once_with(mock_tools)
        assert agent.llm_with_tools == mock_llm

    @pytest.mark.asyncio
    async def test_no_tools_initialization(self, mock_llm):
        """Test agent initialization without tools."""
        agent = OrbitAgent(
            llm=mock_llm,
            tools=[],
            system_prompt="Test",
        )

        # Should not call bind_tools if no tools
        mock_llm.bind_tools.assert_not_called()
        assert agent.llm_with_tools == mock_llm

    def test_orbit_agent_yeet(self):
        agent = OrbitAgent(
            [],
            OrbitContext(ORBIT_SYSTEM_PROMPT)
        )

        result = agent.run("yeet")
        print_line(result)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
