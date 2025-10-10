#!/usr/bin/env python3
"""Unit tests for the Ableton OSC client"""

import unittest
import threading
import socket
import time
from unittest.mock import patch, MagicMock
from ableton_client import AbletonOSCClient


class TestAbletonOSCClient(unittest.TestCase):
    """Test the OSC client with response handling"""

    def setUp(self):
        """Set up test client"""
        self.client = AbletonOSCClient(response_port=11002)  # Use different port for testing

    def tearDown(self):
        """Clean up"""
        self.client.running = False
        time.sleep(0.2)  # Let listener thread stop

    def test_response_listener_thread_starts(self):
        """Test that the response listener thread is running"""
        self.assertTrue(self.client.listener_thread.is_alive())
        self.assertTrue(self.client.running)

    def test_send_and_wait_for_response(self):
        """Test sending a message and waiting for response"""
        # Simulate sending a response to the client
        def send_mock_response():
            time.sleep(0.1)  # Small delay
            # Send a fake response
            message = self.client._encode_osc_message("/live/get/response", ["{'tempo': 120}"])
            mock_sender = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            mock_sender.sendto(message, ('127.0.0.1', 11002))
            mock_sender.close()

        # Start mock response in background
        response_thread = threading.Thread(target=send_mock_response)
        response_thread.start()

        # Test the send_and_wait_for_response method
        with patch.object(self.client, 'send_message', return_value=True):
            response = self.client.send_and_wait_for_response("/live/get", timeout=1.0)

        # Verify we got the response
        self.assertIsNotNone(response)
        self.assertEqual(response[0], "{'tempo': 120}")

        response_thread.join()

    def test_get_live_set_info_parses_response(self):
        """Test that get_live_set_info properly parses the response"""
        # Mock the send_and_wait_for_response to return a test response
        with patch.object(self.client, 'send_and_wait_for_response',
                          return_value=["{'tempo': 154.0, 'is_playing': True, 'track_count': 8, 'scene_count': 10}"]):

            info = self.client.get_live_set_info()

            self.assertIsNotNone(info)
            self.assertEqual(info['tempo'], 154.0)
            self.assertEqual(info['is_playing'], True)
            self.assertEqual(info['track_count'], 8)
            self.assertEqual(info['scene_count'], 10)

    def test_timeout_returns_none(self):
        """Test that timeout properly returns None"""
        with patch.object(self.client, 'send_message', return_value=True):
            # Don't send any response, should timeout
            response = self.client.send_and_wait_for_response("/live/test", timeout=0.1)
            self.assertIsNone(response)

    def test_parse_osc_message(self):
        """Test OSC message parsing"""
        # Create a test OSC message
        test_message = self.client._encode_osc_message("/test/address", [42, 3.14, "hello"])

        # Parse it back
        address, values = self.client._parse_osc_message(test_message)

        self.assertEqual(address, "/test/address")
        self.assertEqual(values[0], 42)
        self.assertAlmostEqual(values[1], 3.14, places=5)
        self.assertEqual(values[2], "hello")

    def test_response_socket_binding(self):
        """Test that response socket is properly bound"""
        # Socket should be bound to response_port
        self.assertEqual(self.client.response_port, 11002)

        # Test we can't bind another socket to same port (proves it's bound)
        test_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        with self.assertRaises(OSError):
            test_socket.bind(('127.0.0.1', 11002))
        test_socket.close()


if __name__ == '__main__':
    unittest.main()