# Orbit Remote Script for Ableton Live

This minimal Python Remote Script enables OSC communication between Orbit and Ableton Live.

## Installation

1. Locate your Ableton Live MIDI Remote Scripts folder:
   - **Mac**: `/Applications/Ableton Live [Version].app/Contents/App-Resources/MIDI Remote Scripts/`
   - **Windows**: `C:\ProgramData\Ableton\Live [Version]\Resources\MIDI Remote Scripts\`
   - **Linux**: Check your Live installation directory

2. Copy the `OrbitRemote` folder to the MIDI Remote Scripts directory:
   ```bash
   # Mac example
   cp -r OrbitRemote "/Applications/Ableton Live 11 Suite.app/Contents/App-Resources/MIDI Remote Scripts/"
   ```

3. Restart Ableton Live

4. In Live's Preferences:
   - Go to Link/Tempo/MIDI tab
   - Under Control Surface, select "OrbitRemote" from the dropdown
   - No MIDI input/output needs to be selected (we use OSC)

## OSC Communication

The script listens on port 11000 (UDP) and responds to these OSC messages:

### Transport Controls
- `/live/play` - Start playback
- `/live/stop` - Stop playback
- `/live/tempo [float]` - Set BPM (20-999)

### Track Controls
- `/live/track/volume [track_id] [volume]` - Set track volume (0.0-1.0)
- `/live/track/mute [track_id] [0/1]` - Mute/unmute track
- `/live/track/solo [track_id] [0/1]` - Solo/unsolo track
- `/live/track/arm [track_id] [0/1]` - Arm/disarm track for recording

### Clip & Scene Controls
- `/live/clip/launch [track_id] [clip_slot]` - Launch a clip
- `/live/scene/launch [scene_id]` - Launch a scene

### Info Queries
- `/live/get` - Get current Live set information

All commands send a response back to the sender with status and current values.

## Testing from Rust

Use the test example in `crates/orbit-connector/examples/ableton_test.rs`:
```bash
cargo run --example ableton_test
```