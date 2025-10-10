#!/bin/bash

echo "Fixing OrbitRemote installation..."

# Kill Ableton if running
pkill -9 -f "Ableton Live"

# Find the actual MIDI Remote Scripts location
APP_DIR="/Applications/Ableton Live 12 Suite.app/Contents/App-Resources/MIDI Remote Scripts/OrbitRemote"
USER_DIR="$HOME/Music/Ableton/User Library/Remote Scripts/OrbitRemote"
SOURCE_DIR="/Users/cuthlehoop/projects/orbit/crates/orbit-connector/scripts/OrbitRemote"

# Remove ALL instances and caches
echo "Removing old installations..."
rm -rf "$APP_DIR/__pycache__" 2>/dev/null
rm -rf "$USER_DIR" 2>/dev/null
rm -rf "$APP_DIR" 2>/dev/null

# Copy fresh version to APPLICATION folder (where Ableton actually looks)
echo "Installing to: $APP_DIR"
mkdir -p "$APP_DIR"
cp "$SOURCE_DIR"/*.py "$APP_DIR/"

# Clear any other Python caches
find ~/Library -name "*OrbitRemote*.pyc" -delete 2>/dev/null
find /private/var -name "*OrbitRemote*.pyc" -delete 2>/dev/null

echo "Starting Ableton..."
open -a "Ableton Live 12 Suite"

echo ""
echo "IMPORTANT: After Ableton opens:"
echo "1. Go to Preferences > Link/Tempo/MIDI"
echo "2. Change Control Surface to 'None'"
echo "3. Change it back to 'OrbitRemote'"
echo "4. The script should now work with responses"