"""
LangChain tools for controlling Ableton Live via OSC and screenshot capabilities.
"""

import sys
import os
import subprocess
import base64
from typing import Annotated
from langchain_core.tools import tool

# Add orbit-connector to path so we can import the OSC client
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../../orbit-connector/src/python'))
from ableton_client import get_ableton_client


# Transport Controls
@tool
def play_ableton() -> str:
    """Start playback in Ableton Live"""
    client = get_ableton_client()
    success = client.play()
    return "Started playback in Ableton Live" if success else "Failed to start playback"


@tool
def stop_ableton() -> str:
    """Stop playback in Ableton Live"""
    client = get_ableton_client()
    success = client.stop()
    return "Stopped playback in Ableton Live" if success else "Failed to stop playback"


@tool
def set_tempo(bpm: Annotated[float, "The tempo in beats per minute (20-999 BPM)"]) -> str:
    """Set the tempo/BPM in Ableton Live"""
    if bpm < 20 or bpm > 999:
        return f"Tempo must be between 20 and 999 BPM (got {bpm})"

    client = get_ableton_client()
    success = client.set_tempo(bpm)
    return f"Set tempo to {bpm} BPM" if success else f"Failed to set tempo to {bpm} BPM"


# Track Controls
@tool
def set_track_volume(
    track_id: Annotated[int, "The track number (0-based index)"],
    volume: Annotated[float, "Volume level from 0.0 (silent) to 1.0 (full volume)"]
) -> str:
    """Set the volume of a specific track in Ableton Live"""
    if volume < 0.0 or volume > 1.0:
        return f"Volume must be between 0.0 and 1.0 (got {volume})"

    client = get_ableton_client()
    success = client.set_track_volume(track_id, volume)
    return f"Set track {track_id} volume to {volume:.2f}" if success else f"Failed to set track {track_id} volume"


@tool
def mute_track(track_id: Annotated[int, "The track number to mute (0-based index)"]) -> str:
    """Mute a specific track in Ableton Live"""
    client = get_ableton_client()
    success = client.mute_track(track_id, mute=True)
    return f"Muted track {track_id}" if success else f"Failed to mute track {track_id}"


@tool
def unmute_track(track_id: Annotated[int, "The track number to unmute (0-based index)"]) -> str:
    """Unmute a specific track in Ableton Live"""
    client = get_ableton_client()
    success = client.mute_track(track_id, mute=False)
    return f"Unmuted track {track_id}" if success else f"Failed to unmute track {track_id}"


@tool
def solo_track(track_id: Annotated[int, "The track number to solo (0-based index)"]) -> str:
    """Solo a specific track in Ableton Live"""
    client = get_ableton_client()
    success = client.solo_track(track_id, solo=True)
    return f"Soloed track {track_id}" if success else f"Failed to solo track {track_id}"


@tool
def unsolo_track(track_id: Annotated[int, "The track number to unsolo (0-based index)"]) -> str:
    """Unsolo a specific track in Ableton Live"""
    client = get_ableton_client()
    success = client.solo_track(track_id, solo=False)
    return f"Unsoloed track {track_id}" if success else f"Failed to unsolo track {track_id}"


@tool
def arm_track(track_id: Annotated[int, "The track number to arm for recording (0-based index)"]) -> str:
    """Arm a track for recording in Ableton Live"""
    client = get_ableton_client()
    success = client.arm_track(track_id, arm=True)
    return f"Armed track {track_id} for recording" if success else f"Failed to arm track {track_id}"


@tool
def disarm_track(track_id: Annotated[int, "The track number to disarm (0-based index)"]) -> str:
    """Disarm a track from recording in Ableton Live"""
    client = get_ableton_client()
    success = client.arm_track(track_id, arm=False)
    return f"Disarmed track {track_id}" if success else f"Failed to disarm track {track_id}"


# Clip and Scene Controls
@tool
def launch_clip(
    track_id: Annotated[int, "The track number (0-based index)"],
    clip_slot: Annotated[int, "The clip slot number (0-based index)"]
) -> str:
    """Launch a specific clip in Ableton Live"""
    client = get_ableton_client()
    success = client.launch_clip(track_id, clip_slot)
    return f"Launched clip in track {track_id}, slot {clip_slot}" if success else f"Failed to launch clip"


@tool
def launch_scene(scene_id: Annotated[int, "The scene number to launch (0-based index)"]) -> str:
    """Launch/trigger a specific scene in Ableton Live"""
    client = get_ableton_client()
    success = client.launch_scene(scene_id)
    return f"Launched scene {scene_id}" if success else f"Failed to launch scene {scene_id}"


@tool
def get_live_info() -> str:
    """Get current information about the Ableton Live session"""
    client = get_ableton_client()
    info = client.get_live_set_info()

    if info:
        # Format the info nicely
        result = "Ableton Live Session Info:\n"
        result += f"  • Tempo: {info.get('tempo', 'Unknown')} BPM\n"
        result += f"  • Playing: {'Yes' if info.get('is_playing', False) else 'No'}\n"
        result += f"  • Tracks: {info.get('track_count', 'Unknown')}\n"
        result += f"  • Scenes: {info.get('scene_count', 'Unknown')}"
        return result
    else:
        return "Failed to get Live session info. Make sure Ableton Live is running and the OrbitRemote script is loaded."


# Screenshot tool for seeing the DAW
@tool
def take_screenshot() -> str:
    """Take a screenshot of the user's current screen to see their Ableton project or any visible content. Use this when the user asks about what's on their screen or refers to visual elements."""
    try:
        import tempfile
        import platform
        from datetime import datetime

        # Create a temporary file for the screenshot
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        temp_dir = tempfile.gettempdir()
        screenshot_path = os.path.join(temp_dir, f"orbit_screenshot_{timestamp}.png")

        system = platform.system()

        if system == "Darwin":  # macOS
            # Use screencapture command
            result = subprocess.run(
                ["screencapture", "-x", screenshot_path],
                capture_output=True,
                text=True
            )
            if result.returncode != 0:
                return f"Failed to capture screenshot: {result.stderr}"

        elif system == "Windows":
            # Use PowerShell to take screenshot
            ps_script = f"""
            Add-Type -AssemblyName System.Windows.Forms
            Add-Type -AssemblyName System.Drawing
            $screen = [System.Windows.Forms.Screen]::PrimaryScreen
            $bitmap = New-Object System.Drawing.Bitmap $screen.Bounds.Width, $screen.Bounds.Height
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            $graphics.CopyFromScreen($screen.Bounds.Location, [System.Drawing.Point]::Empty, $screen.Bounds.Size)
            $bitmap.Save("{screenshot_path}")
            $graphics.Dispose()
            $bitmap.Dispose()
            """
            result = subprocess.run(
                ["powershell", "-Command", ps_script],
                capture_output=True,
                text=True
            )
            if result.returncode != 0:
                return f"Failed to capture screenshot: {result.stderr}"

        elif system == "Linux":
            # Try different screenshot tools in order of preference
            tools = [
                ["gnome-screenshot", "-f", screenshot_path],
                ["scrot", screenshot_path],
                ["import", "-window", "root", screenshot_path],
            ]

            for tool_cmd in tools:
                try:
                    result = subprocess.run(tool_cmd, capture_output=True, text=True)
                    if result.returncode == 0:
                        break
                except FileNotFoundError:
                    continue
            else:
                return "No screenshot tool found. Please install gnome-screenshot, scrot, or imagemagick."
        else:
            return f"Unsupported operating system: {system}"

        # Verify the screenshot was created
        if not os.path.exists(screenshot_path):
            return "Screenshot was not created successfully"

        # Get file size for confirmation
        file_size = os.path.getsize(screenshot_path)

        # Try to extract text using OCR if available
        ocr_text = ""
        try:
            # Try to use pytesseract if available
            import pytesseract
            from PIL import Image

            img = Image.open(screenshot_path)
            ocr_text = pytesseract.image_to_string(img)
            ocr_info = f"\n\nExtracted text from screen:\n{ocr_text[:500]}..." if len(ocr_text) > 500 else f"\n\nExtracted text from screen:\n{ocr_text}"
        except ImportError:
            ocr_info = "\n\n(OCR not available - install pytesseract and Pillow for text extraction)"
        except Exception as e:
            ocr_info = f"\n\n(OCR failed: {str(e)})"

        return f"Screenshot captured successfully at {screenshot_path} ({file_size} bytes). I can now see your screen, including your Ableton Live project if it's visible.{ocr_info}"

    except Exception as e:
        return f"Failed to take screenshot: {str(e)}"


# Collect all tools for easy import
ABLETON_TOOLS = [
    play_ableton,
    stop_ableton,
    set_tempo,
    set_track_volume,
    mute_track,
    unmute_track,
    solo_track,
    unsolo_track,
    arm_track,
    disarm_track,
    launch_clip,
    launch_scene,
    get_live_info,
    take_screenshot,
]