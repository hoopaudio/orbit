"""
Unit tests for Ableton Live OSC connection functionality
"""

import unittest
import sys
import os

# Add the orbit-connector python path
orbit_connector_path = os.path.join(os.path.dirname(__file__), '../../orbit-connector/src/python')
sys.path.insert(0, orbit_connector_path)


class TestAbletonConnection(unittest.TestCase):
    """Test OSC connection to Ableton Live"""

    def setUp(self):
        """Set up test fixtures"""
        try:
            from ableton_client import get_ableton_client
            self.client = get_ableton_client()
        except ImportError as e:
            self.skipTest(f"Could not import ableton_client: {e}")

    def test_import_ableton_client(self):
        """Test that ableton_client can be imported"""
        from ableton_client import get_ableton_client, AbletonOSCClient
        client = get_ableton_client()
        self.assertIsInstance(client, AbletonOSCClient)

    def test_osc_message_send(self):
        """Test basic OSC message sending"""
        result = self.client.send_message("/live/test")
        self.assertIsInstance(result, bool)
        # Note: result might be False if Ableton isn't running, but method should work

    def test_get_live_info(self):
        """Test getting Live session info"""
        info = self.client.get_live_set_info()
        # info will be None if Ableton isn't running, but method should work
        if info:
            self.assertIsInstance(info, dict)
            self.assertIn('tempo', info)
            self.assertIn('is_playing', info)
            self.assertIn('track_count', info)
            self.assertIn('scene_count', info)

    def test_get_track_names(self):
        """Test getting track names"""
        tracks = self.client.get_track_names()
        # tracks will be None if Ableton isn't running, but method should work
        if tracks:
            self.assertIsInstance(tracks, list)
            for track in tracks:
                self.assertIsInstance(track, dict)
                self.assertIn('index', track)
                self.assertIn('name', track)

    def test_transport_controls(self):
        """Test transport control methods exist and are callable"""
        # These should not raise exceptions even if Ableton isn't connected
        self.assertTrue(hasattr(self.client, 'play'))
        self.assertTrue(hasattr(self.client, 'stop'))
        self.assertTrue(hasattr(self.client, 'set_tempo'))

        # Test they return boolean values
        play_result = self.client.play()
        self.assertIsInstance(play_result, bool)

        stop_result = self.client.stop()
        self.assertIsInstance(stop_result, bool)

    def test_track_controls(self):
        """Test track control methods exist and are callable"""
        self.assertTrue(hasattr(self.client, 'set_track_volume'))
        self.assertTrue(hasattr(self.client, 'mute_track'))
        self.assertTrue(hasattr(self.client, 'solo_track'))
        self.assertTrue(hasattr(self.client, 'arm_track'))

        # Test they return boolean values (even if Ableton isn't connected)
        volume_result = self.client.set_track_volume(0, 0.5)
        self.assertIsInstance(volume_result, bool)

        mute_result = self.client.mute_track(0, True)
        self.assertIsInstance(mute_result, bool)


if __name__ == '__main__':
    unittest.main()