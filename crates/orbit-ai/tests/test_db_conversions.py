"""
Unit tests for dB conversion functions in Orbit AI.
"""

import unittest
import sys
import os

# Add the python package to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../python'))

from orbit_ai.ableton_tools import linear_to_db, db_to_linear


class TestDBConversions(unittest.TestCase):
    """Test cases for dB conversion functions."""

    def test_linear_to_db_basic_values(self):
        """Test linear to dB conversion for common values."""
        # Test exact values
        self.assertAlmostEqual(linear_to_db(1.0), 0.0, places=1)   # Unity gain
        self.assertAlmostEqual(linear_to_db(0.5), -6.0, places=1)  # Half volume
        self.assertAlmostEqual(linear_to_db(0.1), -20.0, places=1)  # Low volume

        # Test edge cases
        self.assertEqual(linear_to_db(0.0), -float('inf'))  # Silence
        self.assertAlmostEqual(linear_to_db(0.01), -40.0, places=1)  # Very low

    def test_linear_to_db_boundary_conditions(self):
        """Test boundary conditions for linear to dB conversion."""
        # Value of 1.0 should be 0dB (unity gain)
        self.assertEqual(linear_to_db(1.0), 0.0)

        # Values above 1.0 should give positive dB but be clamped when converted back
        self.assertGreater(linear_to_db(1.5), 0.0)  # Over unity gives positive dB

        # Values at or below 0.0 should return -inf
        self.assertEqual(linear_to_db(0.0), -float('inf'))
        self.assertEqual(linear_to_db(-0.1), -float('inf'))  # Negative should be -inf

    def test_db_to_linear_basic_values(self):
        """Test dB to linear conversion for common values."""
        # Test exact values
        self.assertAlmostEqual(db_to_linear(0.0), 1.0, places=3)    # Unity gain
        self.assertAlmostEqual(db_to_linear(-6.0), 0.501, places=3) # Half volume (approx)
        self.assertAlmostEqual(db_to_linear(-20.0), 0.1, places=3)  # Low volume
        self.assertAlmostEqual(db_to_linear(6.0), 1.0, places=3)    # +6dB gets clamped to 1.0

    def test_db_to_linear_boundary_conditions(self):
        """Test boundary conditions for dB to linear conversion."""
        # Positive dB values above the 0.0-1.0 range should be clamped to 1.0
        self.assertEqual(db_to_linear(6.0), 1.0)   # +6dB gets clamped
        self.assertEqual(db_to_linear(10.0), 1.0)  # +10dB gets clamped

        # Values at or below -70dB should return 0.0
        self.assertEqual(db_to_linear(-70.0), 0.0)
        self.assertEqual(db_to_linear(-100.0), 0.0)  # Very low should be 0

    def test_round_trip_conversions(self):
        """Test that converting linear->dB->linear gives back original value."""
        test_values = [0.1, 0.25, 0.5, 0.75, 0.9, 1.0]

        for linear_val in test_values:
            db_val = linear_to_db(linear_val)
            if db_val != -float('inf'):  # Skip infinity cases
                converted_back = db_to_linear(db_val)
                self.assertAlmostEqual(linear_val, converted_back, places=2,
                                     msg=f"Round trip failed for {linear_val}")

    def test_round_trip_db_conversions(self):
        """Test that converting dB->linear->dB gives back original value."""
        # Test values within the normal range (positive dB values get clamped)
        test_values = [-40.0, -20.0, -12.0, -6.0, -3.0, 0.0]

        for db_val in test_values:
            linear_val = db_to_linear(db_val)
            converted_back = linear_to_db(linear_val)
            self.assertAlmostEqual(db_val, converted_back, places=1,
                                 msg=f"Round trip failed for {db_val}dB")

        # Test that positive dB values get clamped but behave predictably
        positive_db_vals = [3.0, 6.0]
        for db_val in positive_db_vals:
            linear_val = db_to_linear(db_val)
            self.assertEqual(linear_val, 1.0, msg=f"Positive dB {db_val} should clamp to 1.0")
            converted_back = linear_to_db(linear_val)
            self.assertEqual(converted_back, 0.0, msg=f"1.0 linear should convert back to 0dB")

    def test_realistic_producer_values(self):
        """Test conversion with typical producer volume levels."""
        # Common mixing levels with standard dB conversions
        test_cases = [
            (1.0, 0.0),      # Unity gain
            (0.794, -2.0),   # Hot level (approximately)
            (0.631, -4.0),   # Good level (approximately)
            (0.501, -6.0),   # Standard reference
            (0.316, -10.0),  # Lower mix element (approximately)
            (0.1, -20.0),    # Background element
        ]

        for linear, expected_db in test_cases:
            actual_db = linear_to_db(linear)
            self.assertAlmostEqual(actual_db, expected_db, places=0,
                                 msg=f"Linear {linear} should be ~{expected_db}dB, got {actual_db:.1f}dB")

    def test_producer_workflow(self):
        """Test a typical producer workflow with volume adjustments."""
        # Start with a track at 0.5 (should be -6dB)
        initial_linear = 0.5
        initial_db = linear_to_db(initial_linear)
        self.assertAlmostEqual(initial_db, -6.0, places=1)

        # Producer wants to push it to -3dB
        target_db = -3.0
        new_linear = db_to_linear(target_db)

        # Verify the new linear value makes sense (should be higher than 0.5)
        self.assertGreater(new_linear, initial_linear)

        # Verify round trip
        verified_db = linear_to_db(new_linear)
        self.assertAlmostEqual(verified_db, target_db, places=1)


if __name__ == '__main__':
    unittest.main()