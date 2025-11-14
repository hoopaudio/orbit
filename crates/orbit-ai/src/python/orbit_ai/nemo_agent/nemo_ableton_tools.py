"""
NeMo Agent Toolkit functions for controlling Ableton Live via OSC.
Following official NeMo documentation pattern with proper function registration.
"""

import logging
import math
import os
import sys

# Apply LiteLLM patch for model attribute access
try:
    current_dir = os.path.dirname(os.path.abspath(__file__))
    litellm_fix_path = os.path.join(current_dir, "..", "litellm_fix.py")
    if os.path.exists(litellm_fix_path):
        import importlib.util

        spec = importlib.util.spec_from_file_location("litellm_fix", litellm_fix_path)
        litellm_fix = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(litellm_fix)
except Exception as e:
    print(f"Warning: Could not apply LiteLLM patch: {e}")

from nat.builder.builder import Builder
from nat.builder.function_info import FunctionInfo
from nat.cli.register_workflow import register_function
from nat.data_models.function import FunctionBaseConfig
from pydantic import BaseModel, Field

# Add orbit-connector to path so we can import the OSC client
# Get the absolute path to orbit-connector
current_dir = os.path.dirname(os.path.abspath(__file__))
orbit_connector_path = os.path.join(current_dir, "..", "..", "..", "..", "orbit-connector", "src", "python")
orbit_connector_path = os.path.abspath(orbit_connector_path)

if orbit_connector_path not in sys.path:
    sys.path.insert(0, orbit_connector_path)

try:
    from ableton_client import get_ableton_client
except ImportError as e:
    print(f"Failed to import ableton_client from {orbit_connector_path}")
    print(f"Error: {e}")
    raise

logger = logging.getLogger(__name__)


# Conversion functions for volume levels
def linear_to_db(linear_value: float) -> float:
    """Convert linear volume (0.0-1.0) to dB. Uses standard 20*log10 conversion."""
    if linear_value <= 0.0:
        return -70.0  # Minimum practical dB instead of -infinity

    # Standard conversion: 20*log10(linear_value)
    db = 20.0 * math.log10(linear_value)
    return max(-70.0, db)  # Clamp to minimum of -70dB for practical purposes


def db_to_linear(db_value: float) -> float:
    """Convert dB to linear volume (0.0-1.0) for setting track volumes."""
    if db_value <= -70.0:
        return 0.0  # Anything below -70dB is effectively silence

    # Standard conversion: 10^(dB/20)
    linear = 10.0 ** (db_value / 20.0)
    return min(1.0, linear)  # Clamp to maximum of 1.0 for the 0.0-1.0 range


# Input/Output Models for NeMo functions
class EmptyInput(BaseModel):
    """Empty input for functions that don't require parameters"""
    pass


class VolumeConversionInput(BaseModel):
    linear_volume: float = Field(description="Linear volume value (0.0-1.0)")


class VolumeConversionOutput(BaseModel):
    db_value: float = Field(description="Volume in decibels")


class DbConversionInput(BaseModel):
    db_value: float = Field(description="Decibel value (extremities -70dB to +6dB)")


class DbConversionOutput(BaseModel):
    linear_value: float = Field(description="Linear volume (0.0-1.0)")


class TempoInput(BaseModel):
    bpm: float = Field(description="The tempo in beats per minute (20-999 BPM)")


class TrackVolumeInput(BaseModel):
    track_id: int = Field(description="The track number (0-based index)")
    volume: float = Field(description="Volume level from 0.0 (silent) to 1.0 (full volume)")


class TrackControlInput(BaseModel):
    track_id: int = Field(description="The track number (0-based index)")


# Individual function configs - each tool gets its own registration
class GetTrackNamesConfig(FunctionBaseConfig, name="get_track_names"):
    """Get track names and info"""
    pass


class GetLiveInfoConfig(FunctionBaseConfig, name="get_live_info"):
    """Get Live session info"""
    pass


class PlayAbletonConfig(FunctionBaseConfig, name="play_ableton"):
    """Start playback"""
    pass


class StopAbletonConfig(FunctionBaseConfig, name="stop_ableton"):
    """Stop playback"""
    pass


class SetTempoConfig(FunctionBaseConfig, name="set_tempo"):
    """Set tempo"""
    pass


class SetTrackVolumeConfig(FunctionBaseConfig, name="set_track_volume"):
    """Set track volume"""
    pass


class MuteTrackConfig(FunctionBaseConfig, name="mute_track"):
    """Mute track"""
    pass


# Register each function as a separate tool
@register_function(config_type=GetTrackNamesConfig)
async def get_track_names(config: GetTrackNamesConfig, builder: Builder):
    """Get track names and current states"""

    async def _get_track_names(input_data: EmptyInput) -> list:
        client = get_ableton_client()
        tracks = client.get_track_names()

        if not tracks:
            raise Exception(
                "Failed to get track names. Make sure Ableton Live is running and the OrbitRemote script is loaded.")

        # Add dB volume info to each track
        for track in tracks:
            linear_vol = track.get("volume")
            if linear_vol is not None and isinstance(linear_vol, (int, float)):
                if linear_vol <= 0.0:
                    track["volume_db"] = -70.0
                else:
                    track["volume_db"] = linear_to_db(float(linear_vol))

        return tracks

    yield FunctionInfo.create(
        single_fn=_get_track_names,
        description="Get the names and current states of all tracks"
    )


@register_function(config_type=GetLiveInfoConfig)
async def get_live_info(config: GetLiveInfoConfig, builder: Builder):
    """Get current Live session information"""

    async def _get_live_info(input_data: EmptyInput) -> dict:
        client = get_ableton_client()
        info = client.get_live_set_info()

        if not info:
            raise Exception(
                "Failed to get Live session info. Make sure Ableton Live is running and the OrbitRemote script is loaded.")

        return info

    yield FunctionInfo.create(
        single_fn=_get_live_info,
        description="Get current Live session information"
    )


@register_function(config_type=PlayAbletonConfig)
async def play_ableton(config: PlayAbletonConfig, builder: Builder):
    """Start playback in Ableton Live"""

    async def _play_ableton(input_data: EmptyInput) -> str:
        client = get_ableton_client()
        client.play()
        return "Playback started"

    yield FunctionInfo.create(
        single_fn=_play_ableton,
        description="Start playback in Ableton Live"
    )


@register_function(config_type=StopAbletonConfig)
async def stop_ableton(config: StopAbletonConfig, builder: Builder):
    """Stop playback in Ableton Live"""

    async def _stop_ableton(input_data: EmptyInput) -> str:
        client = get_ableton_client()
        client.stop()
        return "Playback stopped"

    yield FunctionInfo.create(
        single_fn=_stop_ableton,
        description="Stop playback in Ableton Live"
    )


@register_function(config_type=SetTempoConfig)
async def set_tempo(config: SetTempoConfig, builder: Builder):
    """Set the tempo in Ableton Live"""

    async def _set_tempo(input_data: TempoInput) -> str:
        client = get_ableton_client()
        client.set_tempo(input_data.bpm)
        return f"Tempo set to {input_data.bpm}"

    yield FunctionInfo.create(
        single_fn=_set_tempo,
        description="Set the tempo in Ableton Live (BPM)"
    )


@register_function(config_type=SetTrackVolumeConfig)
async def set_track_volume(config: SetTrackVolumeConfig, builder: Builder):
    """Set the volume of a specific track"""

    async def _set_track_volume(input_data: TrackVolumeInput) -> str:
        client = get_ableton_client()
        client.set_track_volume(input_data.track_id, input_data.volume)
        return f"Track {input_data.track_id} volume set to {input_data.volume}"

    yield FunctionInfo.create(
        single_fn=_set_track_volume,
        description="Set the volume of a specific track"
    )


@register_function(config_type=MuteTrackConfig)
async def mute_track(config: MuteTrackConfig, builder: Builder):
    """Mute a specific track"""

    async def _mute_track(input_data: TrackControlInput) -> str:
        client = get_ableton_client()
        client.mute_track(input_data.track_id)
        return f"Track {input_data.track_id} muted"

    yield FunctionInfo.create(
        single_fn=_mute_track,
        description="Mute a specific track in Ableton Live"
    )
