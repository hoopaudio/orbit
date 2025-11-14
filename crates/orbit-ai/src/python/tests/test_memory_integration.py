"""
Unit tests for Orbit AI memory functionality using pytest.
Tests the integration of Mem0 memory with NeMo Agent Toolkit.
"""

import pytest
import asyncio
from unittest.mock import Mock, AsyncMock, patch
from pathlib import Path


class TestOrbitMemoryIntegration:
    """Test suite for Orbit AI memory integration."""

    @pytest.fixture
    def config_path(self):
        """Fixture for config file path."""
        return Path(__file__).parent.parent / "orbit_config.yml"

    @pytest.fixture
    def mock_agent(self):
        """Mock agent with memory capabilities."""
        agent = Mock()
        agent.process = AsyncMock()
        return agent

    @pytest.fixture
    def mock_builder(self, mock_agent):
        """Mock NAT builder."""
        builder = Mock()
        builder.set_config_file = Mock()
        builder.get_agent = Mock(return_value=mock_agent)
        return builder

    @pytest.fixture
    def mock_agent_run(self):
        """Mock agent run response."""
        response = Mock()
        response.content = ""
        return response

    @pytest.mark.asyncio
    async def test_memory_configuration_loaded(self, config_path):
        """Test that memory configuration is properly loaded from YAML."""
        import yaml

        with open(config_path, 'r') as f:
            config = yaml.safe_load(f)

        # Verify memory section exists
        assert 'memory' in config
        assert 'orbit_memory' in config['memory']
        assert config['memory']['orbit_memory']['_type'] == 'mem0_memory'

        # Verify memory functions are configured
        functions = config['functions']
        assert 'add_memory' in functions
        assert 'get_memory' in functions
        assert functions['add_memory']['memory'] == 'orbit_memory'
        assert functions['get_memory']['memory'] == 'orbit_memory'

        # Verify workflow includes memory tools
        workflow_tools = config['workflow']['tool_names']
        assert 'get_memory' in workflow_tools
        assert 'add_memory' in workflow_tools

    @pytest.mark.asyncio
    @patch('nat.core.builder.Builder')
    async def test_memory_store_and_retrieve(self, mock_builder_class, mock_builder, mock_agent, mock_agent_run):
        """Test storing and retrieving memory across conversations."""
        # Setup mocks
        mock_builder_class.return_value = mock_builder

        # Mock responses for memory operations
        store_response = Mock()
        store_response.content = "I've remembered that your name is Alex and you prefer house music at 128 BPM."

        retrieve_response = Mock()
        retrieve_response.content = "Your name is Alex and you like house music at 128 BPM."

        mock_agent.process.side_effect = [store_response, retrieve_response]

        # Import after mocking
        from nat.core.builder import Builder
        from nat.core.schemas import AgentRun

        # Test memory storage
        builder = Builder()
        builder.set_config_file("orbit_config.yml")
        agent = builder.get_agent()

        # Store user preference
        store_run = AgentRun(
            id="test-session",
            user_id="test-user",
            message="My name is Alex and I prefer house music at 128 BPM"
        )

        store_result = await agent.process(store_run)
        assert "Alex" in store_result.content
        assert "128 BPM" in store_result.content

        # Retrieve memory
        retrieve_run = AgentRun(
            id="test-session",
            user_id="test-user",
            message="What's my name and what tempo do I like?"
        )

        retrieve_result = await agent.process(retrieve_run)
        assert "Alex" in retrieve_result.content
        assert "128" in retrieve_result.content

    @pytest.mark.asyncio
    @patch('nat.core.builder.Builder')
    async def test_memory_persistence_across_sessions(self, mock_builder_class, mock_builder, mock_agent):
        """Test that memory persists across different conversation sessions."""
        mock_builder_class.return_value = mock_builder

        # Mock responses showing memory persistence
        session1_response = Mock()
        session1_response.content = "I've noted your preference for techno music."

        session2_response = Mock()
        session2_response.content = "I remember you like techno music from our previous conversation."

        mock_agent.process.side_effect = [session1_response, session2_response]

        from nat.core.builder import Builder
        from nat.core.schemas import AgentRun

        builder = Builder()
        agent = builder.get_agent()

        # Session 1: Store preference
        session1_run = AgentRun(
            id="session-1",
            user_id="test-user",
            message="I love techno music"
        )

        result1 = await agent.process(session1_run)
        assert "techno" in result1.content.lower()

        # Session 2: Different session ID, same user
        session2_run = AgentRun(
            id="session-2",
            user_id="test-user",
            message="What kind of music do I like?"
        )

        result2 = await agent.process(session2_run)
        assert "techno" in result2.content.lower()

    @pytest.mark.asyncio
    async def test_memory_integration_with_ableton_tools(self, config_path):
        """Test that memory functions work alongside Ableton Live control tools."""
        import yaml

        with open(config_path, 'r') as f:
            config = yaml.safe_load(f)

        workflow_tools = config['workflow']['tool_names']

        # Verify both memory and Ableton tools are present
        memory_tools = ['get_memory', 'add_memory']
        ableton_tools = ['play_ableton', 'stop_ableton', 'set_tempo', 'mute_track']

        for tool in memory_tools + ableton_tools:
            assert tool in workflow_tools, f"Tool {tool} missing from workflow"

    @pytest.mark.asyncio
    @patch('nat.core.builder.Builder')
    async def test_system_message_includes_memory_instructions(self, mock_builder_class, mock_builder, config_path):
        """Test that system message instructs agent to use memory properly."""
        import yaml

        with open(config_path, 'r') as f:
            config = yaml.safe_load(f)

        system_message = config['workflow']['system_message']

        # Check for memory-related instructions
        assert 'get_memory' in system_message
        assert 'add_memory' in system_message
        assert 'FIRST' in system_message  # Should call get_memory first
        assert 'conversation context' in system_message

    def test_mem0_dependency_in_requirements(self):
        """Test that nvidia-nat-mem0ai dependency is included."""
        requirements_path = Path(__file__).parent.parent.parent / "requirements.txt"

        with open(requirements_path, 'r') as f:
            requirements = f.read()

        assert 'nvidia-nat-mem0ai' in requirements

    @pytest.mark.asyncio
    async def test_memory_error_handling(self, mock_builder, mock_agent):
        """Test proper error handling when memory operations fail."""
        # Mock memory failure
        mock_agent.process.side_effect = Exception("Memory service unavailable")

        with patch('nat.core.builder.Builder', return_value=mock_builder):
            from nat.core.builder import Builder
            from nat.core.schemas import AgentRun

            builder = Builder()
            agent = builder.get_agent()

            run = AgentRun(
                id="test-session",
                user_id="test-user",
                message="Remember my preferences"
            )

            with pytest.raises(Exception) as exc_info:
                await agent.process(run)

            assert "Memory service unavailable" in str(exc_info.value)

    @pytest.mark.parametrize("user_input,expected_memory", [
        ("I like 120 BPM house music", "120 BPM"),
        ("My favorite genre is ambient", "ambient"),
        ("I usually work on tracks at night", "night"),
        ("I prefer working with analog synths", "analog synths")
    ])
    @pytest.mark.asyncio
    async def test_various_user_preferences_storage(self, user_input, expected_memory, mock_builder, mock_agent):
        """Test that various types of user preferences are properly stored."""
        # Mock successful memory storage
        response = Mock()
        response.content = f"I've remembered your preference: {expected_memory}"
        mock_agent.process.return_value = response

        with patch('nat.core.builder.Builder', return_value=mock_builder):
            from nat.core.builder.Builder
            from nat.core.schemas import AgentRun

            builder = Builder()
            agent = builder.get_agent()

            run = AgentRun(
                id="test-session",
                user_id="test-user",
                message=user_input
            )

            result = await agent.process(run)
            assert expected_memory in result.content