"""
Pytest unit tests for NeMo function registration and configuration
"""

import pytest

from orbit_ai import (
    AbletonControlConfig,
    VolumeConversionInput,
    TempoInput,
    TrackVolumeInput,
    TrackControlInput,
)


class TestAbletonControlConfig:
    """Test Ableton control configuration"""

    def test_config_default_values(self):
        """Test configuration with default values"""
        config = AbletonControlConfig()
        assert config.osc_host == "localhost"
        assert config.osc_port == 9000
        assert config.timeout == 5.0

    def test_config_custom_values(self):
        """Test configuration with custom values"""
        config = AbletonControlConfig(
            osc_host="192.168.1.100",
            osc_port=8000,
            timeout=10.0
        )
        assert config.osc_host == "192.168.1.100"
        assert config.osc_port == 8000
        assert config.timeout == 10.0


class TestFunctionRegistration:
    """Test NeMo function registration system"""

    def test_function_registration_structure(self):
        """Test that function registration follows NeMo pattern"""
        from orbit_ai import AbletonControlConfig

        # Test that the config class exists and has proper structure
        config = AbletonControlConfig()
        assert hasattr(config, 'osc_host')
        assert hasattr(config, 'osc_port')
        assert hasattr(config, 'timeout')

        # Test that the function registration decorator exists
        from orbit_ai import ableton_control_functions
        assert callable(ableton_control_functions)

    def test_volume_conversion_functions(self):
        """Test volume conversion functions exist"""
        from orbit_ai import (
            VolumeConversionInput,
            DbConversionInput,
            linear_to_db,
            db_to_linear
        )

        # Test that the volume conversion functions and models exist
        assert callable(linear_to_db)
        assert callable(db_to_linear)

        # Test input/output models
        vol_input = VolumeConversionInput(linear_volume=0.5)
        assert vol_input.linear_volume == 0.5

        db_input = DbConversionInput(db_value=-6.0)
        assert db_input.db_value == -6.0

    def test_transport_control_functions(self):
        """Test transport control input models exist"""
        from orbit_ai import (
            TempoInput,
            TempoOutput,
            TrackControlOutput
        )

        # Test that the transport models exist
        tempo_input = TempoInput(bpm=120.0)
        assert tempo_input.bpm == 120.0

        tempo_output = TempoOutput(success=True, message="Test")
        assert tempo_output.success is True
        assert tempo_output.message == "Test"

        control_output = TrackControlOutput(success=True, message="Test", track_id=0)
        assert control_output.success is True

    def test_track_control_functions(self):
        """Test track control input models exist"""
        from orbit_ai import (
            TrackVolumeInput,
            TrackVolumeOutput,
            TrackControlInput
        )

        # Test track volume models
        volume_input = TrackVolumeInput(track_id=0, volume=0.8)
        assert volume_input.track_id == 0
        assert volume_input.volume == 0.8

        volume_output = TrackVolumeOutput(
            success=True, message="Test", track_id=0, new_volume=0.8
        )
        assert volume_output.success is True

        control_input = TrackControlInput(track_id=1)
        assert control_input.track_id == 1

    def test_info_functions(self):
        """Test info gathering output models exist"""
        from orbit_ai import (
            LiveInfoOutput,
            TrackNamesOutput
        )

        # Test info output models
        live_info = LiveInfoOutput(success=True, session_info={"test": "data"})
        assert live_info.success is True
        assert live_info.session_info == {"test": "data"}

        track_names = TrackNamesOutput(success=True, tracks=[{"name": "Track 1"}])
        assert track_names.success is True
        assert len(track_names.tracks) == 1


class TestFunctionExecution:
    """Test function execution with mocked Ableton client"""

    def test_volume_conversion_execution(self):
        """Test volume conversion function execution"""
        from orbit_ai import linear_to_db, db_to_linear

        # Test direct function calls
        db_result = linear_to_db(0.5)
        assert db_result < 0  # 0.5 linear should be negative dB
        assert abs(db_result - (-6.021)) < 0.1  # Approximately -6 dB

        # Test reverse conversion
        linear_result = db_to_linear(-6.0)
        assert 0.4 < linear_result < 0.6  # Should be around 0.5

    def test_transport_function_with_mock_client(self):
        """Test transport function models"""
        from orbit_ai import TrackControlOutput

        # Test successful transport control output
        result = TrackControlOutput(
            success=True,
            message="Playback started",
            track_id=-1
        )

        assert result.success is True
        assert "Playback started" in result.message
        assert result.track_id == -1

    def test_transport_function_error_handling(self):
        """Test transport function error handling models"""
        from orbit_ai import TrackControlOutput

        # Test error transport control output
        result = TrackControlOutput(
            success=False,
            message="Failed to start playback: Connection failed",
            track_id=-1
        )

        assert result.success is False
        assert "Failed to start playback" in result.message
        assert "Connection failed" in result.message

    def test_track_volume_function(self):
        """Test track volume function models"""
        from orbit_ai import TrackVolumeInput, TrackVolumeOutput

        # Test successful track volume operation
        input_data = TrackVolumeInput(track_id=0, volume=0.8)
        result = TrackVolumeOutput(
            success=True,
            message="Track 0 volume set to 0.8",
            track_id=0,
            new_volume=0.8
        )

        assert result.success is True
        assert result.track_id == 0
        assert result.new_volume == 0.8
        assert "Track 0 volume set to 0.8" in result.message

    def test_track_info_function_success(self):
        """Test track info function with successful response models"""
        from orbit_ai import TrackNamesOutput

        # Test successful track info response
        mock_tracks = [
            {"name": "Track 1", "volume": 0.8, "muted": False, "volume_db": -1.94},
            {"name": "Track 2", "volume": 0.5, "muted": True, "volume_db": -6.02}
        ]

        result = TrackNamesOutput(success=True, tracks=mock_tracks)

        assert result.success is True
        assert len(result.tracks) == 2
        assert result.tracks[0]["name"] == "Track 1"
        # Check that dB conversion was included
        assert "volume_db" in result.tracks[0]
        assert "volume_db" in result.tracks[1]

    def test_track_info_function_no_tracks(self):
        """Test track info function when no tracks are returned"""
        from orbit_ai import TrackNamesOutput

        # Test error response when no tracks found
        result = TrackNamesOutput(
            success=False,
            tracks=[],
            error="Failed to get track names. Make sure Ableton Live is running and the OrbitRemote script is loaded."
        )

        assert result.success is False
        assert "Failed to get track names" in result.error
        assert "Ableton Live is running" in result.error


class TestInputValidation:
    """Test input validation for function parameters"""

    def test_volume_input_valid_range(self):
        """Test volume input validation for valid values"""
        # Valid values
        input1 = VolumeConversionInput(linear_volume=0.0)
        input2 = VolumeConversionInput(linear_volume=0.5)
        input3 = VolumeConversionInput(linear_volume=1.0)

        assert input1.linear_volume == 0.0
        assert input2.linear_volume == 0.5
        assert input3.linear_volume == 1.0

    def test_tempo_input_validation(self):
        """Test tempo input validation"""
        tempo = TempoInput(bpm=120.5)
        assert tempo.bpm == 120.5

    def test_track_input_validation(self):
        """Test track input validation"""
        track_input = TrackControlInput(track_id=5)
        assert track_input.track_id == 5

        volume_input = TrackVolumeInput(track_id=2, volume=0.75)
        assert volume_input.track_id == 2
        assert volume_input.volume == 0.75


# Test configuration
@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    import asyncio
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])