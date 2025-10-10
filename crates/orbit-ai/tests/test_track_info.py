"""
Unit test for track info functionality including volume data
"""

import unittest
import sys
import os

# Add the orbit-connector python path
orbit_connector_path = os.path.join(os.path.dirname(__file__), '../../orbit-connector/src/python')
sys.path.insert(0, orbit_connector_path)


class TestTrackInfo(unittest.TestCase):
    """Test track info retrieval with volume data"""

    def setUp(self):
        """Set up test fixtures"""
        try:
            from ableton_client import get_ableton_client
            self.client = get_ableton_client()
        except ImportError as e:
            self.skipTest(f"Could not import ableton_client: {e}")

    def test_get_track_names_with_volume(self):
        """Test that get_track_names returns tracks with volume info"""
        tracks = self.client.get_track_names()

        self.assertIsNotNone(tracks, "get_track_names returned None - /live/tracks endpoint not responding")
        self.assertIsInstance(tracks, list)

        if len(tracks) > 0:
            # Check first track has all required fields including volume
            track = tracks[0]
            self.assertIsInstance(track, dict)

            # Required fields
            self.assertIn('index', track)
            self.assertIn('name', track)
            self.assertIn('volume', track)  # This should be present now
            self.assertIn('mute', track)
            self.assertIn('solo', track)
            self.assertIn('arm', track)

            # Volume should be a number between 0 and 1
            volume = track['volume']
            self.assertIsInstance(volume, (int, float))
            self.assertGreaterEqual(volume, 0.0)
            self.assertLessEqual(volume, 1.0)

            print(f"✅ Track 0: {track['name']} - Volume: {track['volume']}")

            # Print all tracks with volume info
            for t in tracks:
                name = t.get('name', 'Unknown')
                vol = t.get('volume', 'N/A')
                mute = '🔇' if t.get('mute', False) else ''
                solo = '🔊' if t.get('solo', False) else ''
                arm = '🔴' if t.get('arm', False) else ''
                print(f"   Track {t.get('index', '?')}: {name} (vol: {vol}) {mute}{solo}{arm}")

    def test_track_volume_range(self):
        """Test that all track volumes are in valid range"""
        tracks = self.client.get_track_names()

        self.assertIsNotNone(tracks, "get_track_names returned None - /live/tracks endpoint not responding")

        for track in tracks:
            if 'volume' in track:
                volume = track['volume']
                self.assertGreaterEqual(volume, 0.0, f"Track {track.get('name')} volume too low: {volume}")
                self.assertLessEqual(volume, 1.0, f"Track {track.get('name')} volume too high: {volume}")

    def test_osc_message_handling(self):
        """Test that OSC messages are being sent and received"""
        # Test basic connection first
        result = self.client.send_message("/live/test")
        self.assertTrue(result, "Basic OSC send failed")

        # Test live info (this should work)
        info = self.client.get_live_set_info()
        if info:
            self.assertIn('track_count', info)
            track_count = info['track_count']
            print(f"✅ Session has {track_count} tracks")

            # Now test track names
            tracks = self.client.get_track_names()
            if tracks:
                self.assertEqual(len(tracks), track_count, "Track count mismatch between live info and track names")
            else:
                self.fail("get_track_names failed but live info shows tracks exist")


if __name__ == '__main__':
    unittest.main(verbosity=2)