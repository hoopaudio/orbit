"""
Unit tests for OSC client functionality including concurrency and error handling
"""

import unittest
import sys
import os
import threading
import time
from unittest.mock import patch, MagicMock

# Add the orbit-connector python path
orbit_connector_path = os.path.join(os.path.dirname(__file__), '../../orbit-connector/src/python')
sys.path.insert(0, orbit_connector_path)


class TestOSCClient(unittest.TestCase):
    """Test OSC client functionality"""

    def setUp(self):
        """Set up test fixtures"""
        try:
            from ableton_client import AbletonOSCClient, get_ableton_client
            self.AbletonOSCClient = AbletonOSCClient
            self.get_ableton_client = get_ableton_client
        except ImportError as e:
            self.skipTest(f"Could not import ableton_client: {e}")

    def test_singleton_pattern(self):
        """Test that get_ableton_client returns the same instance"""
        client1 = self.get_ableton_client()
        client2 = self.get_ableton_client()
        self.assertIs(client1, client2, "get_ableton_client should return the same instance")

    def test_concurrent_client_creation(self):
        """Test that concurrent calls to get_ableton_client are thread-safe"""
        clients = []

        def create_client():
            client = self.get_ableton_client()
            clients.append(client)

        threads = []
        for _ in range(5):
            t = threading.Thread(target=create_client)
            threads.append(t)

        for t in threads:
            t.start()

        for t in threads:
            t.join()

        # All clients should be the same instance
        first_client = clients[0]
        for client in clients[1:]:
            self.assertIs(client, first_client, "All concurrent client creations should return same instance")

    def test_concurrent_track_requests(self):
        """Test that concurrent track requests don't interfere with each other"""
        client = self.get_ableton_client()
        results = []

        def request_tracks(request_id):
            try:
                tracks = client.get_track_names()
                result = {
                    'id': request_id,
                    'success': tracks is not None,
                    'track_count': len(tracks) if tracks else 0
                }
                results.append(result)
            except Exception as e:
                results.append({
                    'id': request_id,
                    'success': False,
                    'error': str(e)
                })

        # Start multiple concurrent requests
        threads = []
        for i in range(3):
            t = threading.Thread(target=request_tracks, args=(i,))
            threads.append(t)

        for t in threads:
            t.start()

        for t in threads:
            t.join()

        # All requests should succeed
        self.assertEqual(len(results), 3, "Should have 3 results")
        for result in results:
            self.assertTrue(result['success'], f"Request {result['id']} should succeed: {result}")
            if result['success']:
                self.assertGreater(result['track_count'], 0, f"Request {result['id']} should return tracks")

    def test_track_info_parsing(self):
        """Test that track info is parsed correctly"""
        client = self.get_ableton_client()
        tracks = client.get_track_names()

        if tracks:
            # Test first track has required fields
            track = tracks[0]
            self.assertIsInstance(track, dict)

            required_fields = ['index', 'name', 'volume', 'mute', 'solo', 'arm']
            for field in required_fields:
                self.assertIn(field, track, f"Track should have '{field}' field")

            # Test data types
            self.assertIsInstance(track['index'], int)
            self.assertIsInstance(track['name'], str)
            self.assertIsInstance(track['mute'], bool)
            self.assertIsInstance(track['solo'], bool)
            self.assertIsInstance(track['arm'], bool)

            # Test volume is valid
            if track['volume'] is not None:
                self.assertIsInstance(track['volume'], (int, float))
                self.assertGreaterEqual(track['volume'], 0.0)
                self.assertLessEqual(track['volume'], 1.0)

    def test_osc_message_encoding(self):
        """Test OSC message encoding"""
        client = self.get_ableton_client()

        # Test encoding different message types
        test_cases = [
            ("/live/tracks", []),
            ("/live/tempo", [120.0]),
            ("/live/track/volume", [0, 0.5]),
            ("/live/track/mute", [1, 1]),
        ]

        for address, args in test_cases:
            with self.subTest(address=address, args=args):
                try:
                    encoded = client._encode_osc_message(address, args)
                    self.assertIsInstance(encoded, bytes)
                    self.assertGreater(len(encoded), 0)
                except Exception as e:
                    self.fail(f"Failed to encode message {address} {args}: {e}")

    def test_osc_message_parsing(self):
        """Test OSC message parsing"""
        client = self.get_ableton_client()

        # Test parsing encoded messages
        test_cases = [
            ("/live/tracks", []),
            ("/live/get", []),
            ("/live/tempo", [120.0]),
        ]

        for address, args in test_cases:
            with self.subTest(address=address, args=args):
                try:
                    # Encode then parse to test round-trip
                    encoded = client._encode_osc_message(address, args)
                    parsed_address, parsed_args = client._parse_osc_message(encoded)

                    self.assertEqual(parsed_address, address)
                    self.assertEqual(len(parsed_args), len(args))

                    # Check arg values (allowing for float precision differences)
                    for original, parsed in zip(args, parsed_args):
                        if isinstance(original, float):
                            self.assertAlmostEqual(original, parsed, places=5)
                        else:
                            self.assertEqual(original, parsed)

                except Exception as e:
                    self.fail(f"Failed to parse message {address} {args}: {e}")

    def test_transport_controls(self):
        """Test that transport control methods exist and return boolean"""
        client = self.get_ableton_client()

        transport_methods = ['play', 'stop']
        for method_name in transport_methods:
            with self.subTest(method=method_name):
                method = getattr(client, method_name)
                self.assertTrue(callable(method), f"{method_name} should be callable")

                result = method()
                self.assertIsInstance(result, bool, f"{method_name} should return boolean")

    def test_track_control_methods(self):
        """Test that track control methods exist and return boolean"""
        client = self.get_ableton_client()

        track_methods = [
            ('set_track_volume', [0, 0.5]),
            ('mute_track', [0, True]),
            ('solo_track', [0, True]),
            ('arm_track', [0, True]),
        ]

        for method_name, args in track_methods:
            with self.subTest(method=method_name):
                method = getattr(client, method_name)
                self.assertTrue(callable(method), f"{method_name} should be callable")

                result = method(*args)
                self.assertIsInstance(result, bool, f"{method_name} should return boolean")

    def test_live_session_info(self):
        """Test getting Live session info"""
        client = self.get_ableton_client()
        info = client.get_live_set_info()

        if info:
            self.assertIsInstance(info, dict)
            expected_fields = ['tempo', 'is_playing', 'track_count', 'scene_count']
            for field in expected_fields:
                self.assertIn(field, info, f"Live info should have '{field}' field")

            # Test data types
            self.assertIsInstance(info['tempo'], (int, float))
            self.assertIsInstance(info['is_playing'], bool)
            self.assertIsInstance(info['track_count'], int)
            self.assertIsInstance(info['scene_count'], int)

            # Test reasonable values
            self.assertGreater(info['tempo'], 0)
            self.assertGreaterEqual(info['track_count'], 0)
            self.assertGreaterEqual(info['scene_count'], 0)

    def test_request_locking(self):
        """Test that concurrent requests to the same endpoint don't interfere"""
        client = self.get_ableton_client()

        # Track request results
        results = []
        lock = threading.Lock()

        def make_request(request_id):
            try:
                tracks = client.get_track_names()
                with lock:
                    results.append({
                        'id': request_id,
                        'success': tracks is not None,
                        'track_count': len(tracks) if tracks else 0
                    })
            except Exception as e:
                with lock:
                    results.append({
                        'id': request_id,
                        'success': False,
                        'error': str(e)
                    })

        # Start concurrent requests
        threads = []
        for i in range(5):
            t = threading.Thread(target=make_request, args=(i,))
            threads.append(t)

        for t in threads:
            t.start()

        for t in threads:
            t.join()

        # All requests should succeed
        self.assertEqual(len(results), 5, "Should have 5 results")
        successful_requests = [r for r in results if r['success']]
        self.assertGreaterEqual(len(successful_requests), 4,
                               "At least 4 out of 5 requests should succeed with locking")

        # All successful requests should return the same track count
        if len(successful_requests) > 1:
            first_count = successful_requests[0]['track_count']
            for result in successful_requests[1:]:
                self.assertEqual(result['track_count'], first_count,
                               "All requests should return the same track count")


if __name__ == '__main__':
    unittest.main(verbosity=2)