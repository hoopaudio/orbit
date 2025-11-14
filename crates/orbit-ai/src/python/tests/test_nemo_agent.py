#!/usr/bin/env python3
"""
Unit tests for NeMo Agent integration
"""

import unittest
import asyncio
import os
import sys
from unittest.mock import patch, MagicMock, AsyncMock, mock_open

# Add the project path
sys.path.insert(0, os.path.dirname(__file__))

from orbit_ai import NeMoAgent


class TestNeMoAgent(unittest.TestCase):
    """Unit tests for NeMoAgent class"""

    def setUp(self):
        """Set up test fixtures"""
        self.agent = NeMoAgent(api_key="test_key")
        self.agent.base_url = "http://localhost:8001"

    def test_initialization(self):
        """Test agent initialization"""
        self.assertEqual(self.agent.api_key, "test_key")
        self.assertEqual(self.agent.base_url, "http://localhost:8001")
        self.assertEqual(len(self.agent.conversation_history), 0)

    def test_initialization_with_env_var(self):
        """Test agent initialization with environment variable"""
        with patch.dict(os.environ, {"NEMO_SERVER_URL": "http://custom:9000"}):
            agent = NeMoAgent()
            self.assertEqual(agent.base_url, "http://custom:9000")

    @patch('aiohttp.ClientSession.post')
    async def test_ask_success(self, mock_post):
        """Test successful ask request"""
        # Mock successful response
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "choices": [{"message": {"content": "Test response"}}]
        })
        mock_post.return_value.__aenter__.return_value = mock_response

        result = await self.agent.ask("Test question")

        self.assertEqual(result, "Test response")
        self.assertEqual(len(self.agent.conversation_history), 2)  # Question + response

    @patch('aiohttp.ClientSession.post')
    async def test_ask_server_error(self, mock_post):
        """Test ask request with server error"""
        # Mock error response
        mock_response = MagicMock()
        mock_response.status = 500
        mock_response.text = AsyncMock(return_value="Internal Server Error")
        mock_post.return_value.__aenter__.return_value = mock_response

        result = await self.agent.ask("Test question")

        self.assertIn("Error: NeMo server returned 500", result)

    @patch('aiohttp.ClientSession.post')
    async def test_ask_invalid_response_format(self, mock_post):
        """Test ask request with invalid response format"""
        # Mock response with invalid format
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={"invalid": "format"})
        mock_post.return_value.__aenter__.return_value = mock_response

        result = await self.agent.ask("Test question")

        self.assertIn("Error: Unexpected response format", result)

    @patch('aiohttp.ClientSession.post')
    async def test_stream_success(self, mock_post):
        """Test successful streaming request"""
        # Mock streaming response
        mock_response = MagicMock()
        mock_response.status = 200

        # Mock streaming data
        stream_data = [
            b'data: {"choices": [{"delta": {"content": "Hello"}}]}\n',
            b'data: {"choices": [{"delta": {"content": " world"}}]}\n',
            b'data: [DONE]\n'
        ]
        mock_response.content.__aiter__.return_value = stream_data
        mock_post.return_value.__aenter__.return_value = mock_response

        chunks = []
        async for chunk in self.agent.stream("Test question"):
            chunks.append(chunk)

        self.assertEqual(chunks, ["Hello", " world"])
        self.assertEqual(len(self.agent.conversation_history), 2)

    @patch('aiohttp.ClientSession.post')
    async def test_ask_with_image_success(self, mock_post):
        """Test successful image request"""
        # Mock file reading
        with patch('builtins.open', mock_open(read_data=b'fake_image_data')):
            with patch('base64.b64encode', return_value=b'ZmFrZV9pbWFnZV9kYXRh'):
                # Mock successful response
                mock_response = MagicMock()
                mock_response.status = 200
                mock_response.json = AsyncMock(return_value={
                    "choices": [{"message": {"content": "Image analysis result"}}]
                })
                mock_post.return_value.__aenter__.return_value = mock_response

                result = await self.agent.ask_with_image("Analyze this image", "/fake/path.jpg")

                self.assertEqual(result, "Image analysis result")

    @patch('aiohttp.ClientSession.get')
    async def test_health_check_success(self, mock_get):
        """Test successful health check"""
        mock_response = MagicMock()
        mock_response.status = 200
        mock_get.return_value.__aenter__.return_value = mock_response

        result = await self.agent.health_check()
        self.assertTrue(result)

    @patch('aiohttp.ClientSession.get')
    async def test_health_check_failure(self, mock_get):
        """Test failed health check"""
        mock_get.side_effect = Exception("Connection failed")

        result = await self.agent.health_check()
        self.assertFalse(result)

    def test_clear_memory(self):
        """Test memory clearing"""
        # Add some fake history
        from langchain_core.messages import HumanMessage, AIMessage
        self.agent.conversation_history = [
            HumanMessage(content="Test"),
            AIMessage(content="Response")
        ]

        result = self.agent.clear_memory()

        self.assertEqual(len(self.agent.conversation_history), 0)
        self.assertEqual(result, "Memory cleared")


class TestIntegration(unittest.TestCase):
    """Integration tests for the complete system"""

    @patch('aiohttp.ClientSession.post')
    async def test_request_payload_format(self, mock_post):
        """Test that request payload is correctly formatted"""
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            "choices": [{"message": {"content": "Test response"}}]
        })
        mock_post.return_value.__aenter__.return_value = mock_response

        agent = NeMoAgent()
        await agent.ask("Test question", "Custom system prompt")

        # Verify the request was made with correct payload
        mock_post.assert_called_once()
        call_args = mock_post.call_args

        # Check URL
        self.assertIn("/v1/chat/completions", call_args[0][0])

        # Check payload structure
        payload = call_args[1]['json']
        self.assertEqual(payload['model'], 'orbit_llm')
        self.assertIn('messages', payload)
        self.assertEqual(payload['temperature'], 0.1)
        self.assertEqual(payload['max_tokens'], 1000)
        self.assertFalse(payload['stream'])

        # Check message format
        messages = payload['messages']
        self.assertEqual(messages[0]['role'], 'system')
        self.assertEqual(messages[0]['content'], 'Custom system prompt')
        self.assertEqual(messages[1]['role'], 'user')
        self.assertEqual(messages[1]['content'], 'Test question')


def run_async_test(coro):
    """Helper to run async tests"""
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    try:
        return loop.run_until_complete(coro)
    finally:
        loop.close()


# Make async tests work with unittest
for test_class in [TestNeMoAgent, TestIntegration]:
    for method_name in dir(test_class):
        if method_name.startswith('test_') and asyncio.iscoroutinefunction(getattr(test_class, method_name)):
            method = getattr(test_class, method_name)
            setattr(test_class, method_name, lambda self, m=method: run_async_test(m(self)))


if __name__ == '__main__':
    unittest.main(verbosity=2)