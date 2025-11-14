"""
LangChain tools for controlling Ableton Live via OSC.
Converts the Ableton client functions into LangChain tool format.
"""

import logging
import math
import os
import sys
from typing import List, Any, Optional

from langchain_core.tools import BaseTool
from pydantic import BaseModel, Field

# Add orbit-connector to path so we can import the OSC client
current_dir = os.path.dirname(os.path.abspath(__file__))
orbit_connector_path = os.path.join(current_dir, "..", "..", "..", "..", "..", "orbit-connector", "src", "python")
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
    """Convert linear volume (0.0-1.0) to dB."""
    if linear_value <= 0.0:
        return -70.0

    db = 20.0 * math.log10(linear_value)
    return max(-70.0, db)


def db_to_linear(db_value: float) -> float:
    """Convert dB to linear volume (0.0-1.0)."""
    if db_value <= -70.0:
        return 0.0

    linear = 10.0 ** (db_value / 20.0)
    return min(1.0, linear)


# Input models for tools
class TempoInput(BaseModel):
    """Input for setting tempo."""
    bpm: float = Field(description="The tempo in beats per minute (20-999 BPM)")


class TrackVolumeInput(BaseModel):
    """Input for setting track volume."""
    track_id: int = Field(description="The track number (0-based index)")
    volume: float = Field(description="Volume level from 0.0 (silent) to 1.0 (full volume)")


class TrackControlInput(BaseModel):
    """Input for track control operations."""
    track_id: int = Field(description="The track number (0-based index)")


# LangChain tools
class GetTrackNamesTool(BaseTool):
    """Tool to get track names and current states."""

    name: str = "get_track_names"
    description: str = "Get the names and current states of all tracks in Ableton Live"

    def _run(self) -> List[dict]:
        """Get track names and current states."""
        client = get_ableton_client()
        tracks = client.get_track_names()

        if not tracks:
            raise Exception(
                "Failed to get track names. Make sure Ableton Live is running and the OrbitRemote script is loaded."
            )

        # Add dB volume info to each track
        for track in tracks:
            linear_vol = track.get("volume")
            if linear_vol is not None and isinstance(linear_vol, (int, float)):
                if linear_vol <= 0.0:
                    track["volume_db"] = -70.0
                else:
                    track["volume_db"] = linear_to_db(float(linear_vol))

        return tracks


class GetLiveInfoTool(BaseTool):
    """Tool to get current Live session information."""

    name: str = "get_live_info"
    description: str = "Get current Live session information including tempo, playing state, etc."

    def _run(self) -> dict:
        """Get current Live session information."""
        client = get_ableton_client()
        info = client.get_live_set_info()

        if not info:
            raise Exception(
                "Failed to get Live session info. Make sure Ableton Live is running and the OrbitRemote script is loaded."
            )

        return info


class PlayAbletonTool(BaseTool):
    """Tool to start playback in Ableton Live."""

    name: str = "play_ableton"
    description: str = "Start playback in Ableton Live"

    def _run(self) -> str:
        """Start playback in Ableton Live."""
        client = get_ableton_client()
        client.play()
        return "Playback started"


class StopAbletonTool(BaseTool):
    """Tool to stop playback in Ableton Live."""

    name: str = "stop_ableton"
    description: str = "Stop playback in Ableton Live"

    def _run(self) -> str:
        """Stop playback in Ableton Live."""
        client = get_ableton_client()
        client.stop()
        return "Playback stopped"


class SetTempoTool(BaseTool):
    """Tool to set the tempo in Ableton Live."""

    name: str = "set_tempo"
    description: str = "Set the tempo in Ableton Live (BPM)"
    args_schema: type[BaseModel] = TempoInput

    def _run(self, bpm: float) -> str:
        """Set the tempo in Ableton Live."""
        client = get_ableton_client()
        client.set_tempo(bpm)
        return f"Tempo set to {bpm} BPM"


class SetTrackVolumeTool(BaseTool):
    """Tool to set the volume of a specific track."""

    name: str = "set_track_volume"
    description: str = "Set the volume of a specific track in Ableton Live"
    args_schema: type[BaseModel] = TrackVolumeInput

    def _run(self, track_id: int, volume: float) -> str:
        """Set the volume of a specific track."""
        client = get_ableton_client()
        client.set_track_volume(track_id, volume)
        return f"Track {track_id} volume set to {volume}"


class MuteTrackTool(BaseTool):
    """Tool to mute a specific track."""

    name: str = "mute_track"
    description: str = "Mute a specific track in Ableton Live"
    args_schema: type[BaseModel] = TrackControlInput

    def _run(self, track_id: int) -> str:
        """Mute a specific track."""
        client = get_ableton_client()
        client.mute_track(track_id)
        return f"Track {track_id} muted"


# List of all available tools
ABLETON_TOOLS: List[BaseTool] = [
    GetTrackNamesTool(),
    GetLiveInfoTool(),
    PlayAbletonTool(),
    StopAbletonTool(),
    SetTempoTool(),
    SetTrackVolumeTool(),
    MuteTrackTool(),
]
