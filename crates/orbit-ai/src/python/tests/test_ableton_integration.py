"""
Pytest unit tests for NeMo Ableton integration
"""

import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch

from orbit_ai import NeMoAgent
from orbit_ai import (
    linear_to_db,
    db_to_linear,
    VolumeConversionInput,
    VolumeConversionOutput,
    TempoInput,
    TempoOutput,
    TrackVolumeInput,
    TrackVolumeOutput,
    TrackControlInput,
    TrackControlOutput,
    LiveInfoOutput,
    TrackNamesOutput,
)


class TestVolumeConversion:
    """Test volume conversion functions"""

    def test_linear_to_db_normal_values(self):
        """Test linear to dB conversion for normal values"""
        assert abs(linear_to_db(1.0) - 0.0) < 0.001  # 1.0 linear = 0 dB
        assert abs(linear_to_db(0.5) - (-6.021)) < 0.01  # 0.5 linear ≈ -6 dB
        assert abs(linear_to_db(0.1) - (-20.0)) < 0.1  # 0.1 linear = -20 dB

    def test_linear_to_db_edge_cases(self):
        """Test linear to dB conversion for edge cases"""
        assert linear_to_db(0.0) == -70.0  # 0.0 linear = -70 dB (minimum)
        assert linear_to_db(-0.1) == -70.0  # Negative values clamped to -70 dB

    def test_db_to_linear_normal_values(self):
        """Test dB to linear conversion for normal values"""
        assert abs(db_to_linear(0.0) - 1.0) < 0.001  # 0 dB = 1.0 linear
        assert abs(db_to_linear(-6.0) - 0.501) < 0.01  # -6 dB ≈ 0.5 linear
        assert abs(db_to_linear(-20.0) - 0.1) < 0.01  # -20 dB = 0.1 linear

    def test_db_to_linear_edge_cases(self):
        """Test dB to linear conversion for edge cases"""
        assert db_to_linear(-70.0) == 0.0  # -70 dB = 0.0 linear (minimum)
        assert db_to_linear(-100.0) == 0.0  # Below -70 dB = 0.0 linear
        assert db_to_linear(10.0) == 1.0  # Above 0 dB clamped to 1.0

    def test_round_trip_conversion(self):
        """Test that converting linear->dB->linear preserves value"""
        test_values = [0.1, 0.25, 0.5, 0.75, 1.0]
        for value in test_values:
            db_val = linear_to_db(value)
            back_to_linear = db_to_linear(db_val)
            assert abs(back_to_linear - value) < 0.001


class TestNeMoAgent:
    """Test NeMo Agent functionality"""

    def test_agent_initialization(self):
        """Test agent initialization with default values"""
        agent = NeMoAgent()
        assert agent.base_url == "http://localhost:8001"
        assert len(agent.conversation_history) == 0

    @patch.dict("os.environ", {"NEMO_SERVER_URL": "http://localhost:8004"})
    def test_agent_initialization_with_custom_url(self):
        """Test agent initialization with custom URL"""
        agent = NeMoAgent()
        assert agent.base_url == "http://localhost:8004"

    @patch.dict("os.environ", {"NEMO_SERVER_URL": "http://custom:9000"})
    def test_agent_initialization_with_env_var(self):
        """Test agent initialization with environment variable"""
        agent = NeMoAgent()
        assert agent.base_url == "http://custom:9000"

    @pytest.mark.asyncio
    async def test_health_check_success(self):
        """Test successful health check"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.get') as mock_get:
            mock_response = MagicMock()
            mock_response.status = 200
            mock_get.return_value.__aenter__.return_value = mock_response

            result = await agent.health_check()
            assert result is True

    @pytest.mark.asyncio
    async def test_health_check_failure(self):
        """Test failed health check"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.get') as mock_get:
            mock_get.side_effect = Exception("Connection failed")

            result = await agent.health_check()
            assert result is False

    @pytest.mark.asyncio
    async def test_ask_success(self):
        """Test successful ask request"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.post') as mock_post:
            mock_response = MagicMock()
            mock_response.status = 200
            mock_response.json = AsyncMock(return_value={
                "choices": [{"message": {"content": "Test response"}}]
            })
            mock_post.return_value.__aenter__.return_value = mock_response

            result = await agent.ask("Test question")

            assert result == "Test response"
            assert len(agent.conversation_history) == 2  # Question + response

    @pytest.mark.asyncio
    async def test_ask_server_error(self):
        """Test ask request with server error"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.post') as mock_post:
            mock_response = MagicMock()
            mock_response.status = 500
            mock_response.text = AsyncMock(return_value="Internal Server Error")
            mock_post.return_value.__aenter__.return_value = mock_response

            result = await agent.ask("Test question")

            assert "Error: NeMo server returned 500" in result

    @pytest.mark.asyncio
    async def test_ask_invalid_response_format(self):
        """Test ask request with invalid response format"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.post') as mock_post:
            mock_response = MagicMock()
            mock_response.status = 200
            mock_response.json = AsyncMock(return_value={"invalid": "format"})
            mock_post.return_value.__aenter__.return_value = mock_response

            result = await agent.ask("Test question")

            assert "Error: Unexpected response format" in result

    @pytest.mark.asyncio
    async def test_stream_success(self):
        """Test successful streaming request"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.post') as mock_post:
            mock_response = MagicMock()
            mock_response.status = 200

            stream_data = [
                b'data: {"choices": [{"delta": {"content": "Hello"}}]}\n',
                b'data: {"choices": [{"delta": {"content": " world"}}]}\n',
                b'data: [DONE]\n'
            ]
            mock_response.content.__aiter__.return_value = stream_data
            mock_post.return_value.__aenter__.return_value = mock_response

            chunks = []
            async for chunk in agent.stream("Test question"):
                chunks.append(chunk)

            assert chunks == ["Hello", " world"]
            assert len(agent.conversation_history) == 2

    @pytest.mark.asyncio
    async def test_ask_with_system_prompt(self):
        """Test ask request with custom system prompt"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.post') as mock_post:
            mock_response = MagicMock()
            mock_response.status = 200
            mock_response.json = AsyncMock(return_value={
                "choices": [{"message": {"content": "Custom response"}}]
            })
            mock_post.return_value.__aenter__.return_value = mock_response

            result = await agent.ask("Test question", "Custom system prompt")

            # Verify the request was made with correct payload
            mock_post.assert_called_once()
            call_args = mock_post.call_args
            payload = call_args[1]['json']

            assert payload['messages'][0]['role'] == 'system'
            assert payload['messages'][0]['content'] == 'Custom system prompt'
            assert payload['messages'][1]['role'] == 'user'
            assert payload['messages'][1]['content'] == 'Test question'

    def test_clear_memory(self):
        """Test memory clearing functionality"""
        from langchain_core.messages import HumanMessage, AIMessage

        agent = NeMoAgent()
        agent.conversation_history = [
            HumanMessage(content="Test"),
            AIMessage(content="Response")
        ]

        result = agent.clear_memory()

        assert len(agent.conversation_history) == 0
        assert result == "Memory cleared"


class TestAbletonToolInputs:
    """Test Ableton tool input/output models"""

    def test_volume_conversion_input_validation(self):
        """Test VolumeConversionInput validation"""
        # Valid input
        valid_input = VolumeConversionInput(linear_volume=0.5)
        assert valid_input.linear_volume == 0.5

    def test_volume_conversion_output_creation(self):
        """Test VolumeConversionOutput creation"""
        output = VolumeConversionOutput(db_value=-6.0)
        assert output.db_value == -6.0

    def test_tempo_input_validation(self):
        """Test TempoInput validation"""
        tempo_input = TempoInput(bpm=120.0)
        assert tempo_input.bpm == 120.0

    def test_track_volume_input_validation(self):
        """Test TrackVolumeInput validation"""
        track_input = TrackVolumeInput(track_id=0, volume=0.8)
        assert track_input.track_id == 0
        assert track_input.volume == 0.8

    def test_track_control_input_validation(self):
        """Test TrackControlInput validation"""
        control_input = TrackControlInput(track_id=2)
        assert control_input.track_id == 2


class TestAbletonIntegration:
    """Integration tests for Ableton functionality"""

    @patch.dict("os.environ", {"NEMO_SERVER_URL": "http://localhost:8004"})
    @pytest.mark.asyncio
    async def test_nemo_agent_with_ableton_server(self):
        """Test NeMo agent connecting to Ableton-enabled server"""
        agent = NeMoAgent()

        # Test health check
        with patch('aiohttp.ClientSession.get') as mock_get:
            mock_response = MagicMock()
            mock_response.status = 200
            mock_get.return_value.__aenter__.return_value = mock_response

            health = await agent.health_check()
            assert health is True

    @patch.dict("os.environ", {"NEMO_SERVER_URL": "http://localhost:8004"})
    @pytest.mark.asyncio
    async def test_tool_calling_request_format(self):
        """Test that tool calling requests are formatted correctly"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.post') as mock_post:
            mock_response = MagicMock()
            mock_response.status = 200
            mock_response.json = AsyncMock(return_value={
                "choices": [{"message": {"content": "Tool executed successfully"}}]
            })
            mock_post.return_value.__aenter__.return_value = mock_response

            await agent.ask("Convert 0.5 linear volume to decibels")

            # Verify the request was made
            mock_post.assert_called_once()
            call_args = mock_post.call_args

            # Check URL
            assert "/v1/chat/completions" in call_args[0][0]

            # Check payload structure
            payload = call_args[1]['json']
            assert payload['model'] == 'orbit_llm'
            assert 'messages' in payload
            assert payload['temperature'] == 0.1
            assert payload['max_tokens'] == 1000

    @pytest.mark.asyncio
    async def test_conversation_context_preservation(self):
        """Test that conversation context is preserved across requests"""
        agent = NeMoAgent()

        with patch('aiohttp.ClientSession.post') as mock_post:
            mock_response = MagicMock()
            mock_response.status = 200
            mock_response.json = AsyncMock(return_value={
                "choices": [{"message": {"content": "Response"}}]
            })
            mock_post.return_value.__aenter__.return_value = mock_response

            # First question
            await agent.ask("First question")
            assert len(agent.conversation_history) == 2

            # Second question should include context
            await agent.ask("Second question")
            assert len(agent.conversation_history) == 4

            # Verify second request includes all messages
            call_args = mock_post.call_args
            payload = call_args[1]['json']
            assert len(payload['messages']) == 4  # 2 questions + 2 responses


# Pytest configuration for async tests
@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


# Test runners
if __name__ == "__main__":
    pytest.main([__file__, "-v"])