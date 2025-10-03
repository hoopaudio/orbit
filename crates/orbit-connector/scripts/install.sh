#!/bin/bash

# Install script for OrbitRemote Ableton Remote Script

echo "Orbit Remote Script Installer"
echo "============================="
echo

# Common Ableton Live installation paths on macOS
ABLETON_PATHS=(
    "/Applications/Ableton Live 11 Suite.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 11 Standard.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 11 Intro.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 11 Trial.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 12 Suite.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 12 Standard.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 12 Intro.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 12 Trial.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 10 Suite.app/Contents/App-Resources/MIDI Remote Scripts"
    "/Applications/Ableton Live 10 Standard.app/Contents/App-Resources/MIDI Remote Scripts"
)

# Find installed Ableton versions
FOUND_PATHS=()
for path in "${ABLETON_PATHS[@]}"; do
    if [ -d "$path" ]; then
        FOUND_PATHS+=("$path")
    fi
done

if [ ${#FOUND_PATHS[@]} -eq 0 ]; then
    echo "❌ No Ableton Live installation found in standard locations."
    echo
    echo "Please manually copy the OrbitRemote folder to your Ableton's MIDI Remote Scripts directory:"
    echo "  cp -r OrbitRemote '/path/to/Ableton Live.app/Contents/App-Resources/MIDI Remote Scripts/'"
    exit 1
fi

# If multiple versions found, let user choose
if [ ${#FOUND_PATHS[@]} -gt 1 ]; then
    echo "Found multiple Ableton Live installations:"
    echo
    for i in "${!FOUND_PATHS[@]}"; do
        # Extract version from path
        VERSION=$(echo "${FOUND_PATHS[$i]}" | sed -n 's/.*Ableton Live \([0-9]* [A-Za-z]*\).*/\1/p')
        echo "  $((i+1)). Ableton Live $VERSION"
    done
    echo
    read -p "Which version would you like to install to? (1-${#FOUND_PATHS[@]}): " choice

    if [[ "$choice" -ge 1 && "$choice" -le ${#FOUND_PATHS[@]} ]]; then
        INSTALL_PATH="${FOUND_PATHS[$((choice-1))]}"
    else
        echo "❌ Invalid choice"
        exit 1
    fi
else
    INSTALL_PATH="${FOUND_PATHS[0]}"
fi

# Get the script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Check if OrbitRemote exists
if [ ! -d "$SCRIPT_DIR/OrbitRemote" ]; then
    echo "❌ OrbitRemote folder not found in $SCRIPT_DIR"
    exit 1
fi

# Install the remote script
echo "Installing OrbitRemote to:"
echo "  $INSTALL_PATH"
echo

# Check if already installed
if [ -d "$INSTALL_PATH/OrbitRemote" ]; then
    read -p "OrbitRemote is already installed. Overwrite? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
    rm -rf "$INSTALL_PATH/OrbitRemote"
fi

# Copy the remote script
cp -r "$SCRIPT_DIR/OrbitRemote" "$INSTALL_PATH/"

if [ $? -eq 0 ]; then
    echo "✅ OrbitRemote installed successfully!"
    echo
    echo "Next steps:"
    echo "1. Restart Ableton Live"
    echo "2. Go to Preferences > Link/Tempo/MIDI"
    echo "3. Under Control Surface, select 'OrbitRemote'"
    echo "4. No MIDI input/output selection needed (uses OSC)"
    echo
    echo "Test the connection with:"
    echo "  cargo run --example ableton_test"
else
    echo "❌ Failed to install OrbitRemote"
    exit 1
fi