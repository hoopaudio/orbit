#!/usr/bin/env python3
"""
Complete setup for OrbitRemote - install files and configure in Ableton
"""

import os
import shutil
import subprocess
import time
import platform
from pathlib import Path

def find_ableton_scripts_folder():
    """Find Ableton's MIDI Remote Scripts folder based on OS"""
    system = platform.system()

    if system == "Darwin":  # macOS
        # Check user library first (preferred location for custom scripts)
        user_path = Path.home() / "Music/Ableton/User Library/Remote Scripts"
        user_path.mkdir(parents=True, exist_ok=True)
        return user_path

    return None

def install_orbitremote():
    """Copy OrbitRemote script files to Ableton's scripts folder"""

    # Source directory
    source_dir = Path(__file__).parent.parent / "crates/orbit-connector/scripts/OrbitRemote"

    if not source_dir.exists():
        print(f"Error: Source directory not found: {source_dir}")
        return False

    # Find destination
    dest_base = find_ableton_scripts_folder()
    if not dest_base:
        print("Error: Could not find Ableton scripts folder")
        return False

    dest_dir = dest_base / "OrbitRemote"

    print(f"Installing OrbitRemote to: {dest_dir}")

    # Remove old version if it exists
    if dest_dir.exists():
        shutil.rmtree(dest_dir)

    # Copy new version
    shutil.copytree(source_dir, dest_dir)

    # Verify installation
    required_files = ["__init__.py", "OrbitRemote.py", "OSCServer.py"]
    for file in required_files:
        if not (dest_dir / file).exists():
            print(f"Error: Failed to copy {file}")
            return False

    print("✓ OrbitRemote script files installed successfully!")
    return True

def configure_in_ableton():
    """Use AppleScript to configure OrbitRemote in Ableton preferences"""

    if platform.system() != "Darwin":
        print("Auto-configuration only available on macOS")
        return False

    applescript = '''
    tell application "Ableton Live 12 Suite"
        activate
        delay 2
    end tell

    tell application "System Events"
        tell process "Live"
            -- Open Preferences
            keystroke "," using command down
            delay 2

            -- Click on Link/Tempo/MIDI tab (usually the 4th tab)
            try
                click button 4 of toolbar 1 of window 1
            end try
            delay 1

            -- Find and click the Control Surface dropdown
            -- First, try to find it by searching for the label
            try
                -- Look for the Control Surface popup button
                set controlSurfacePopup to first pop up button of window 1 whose description contains "Control Surface" or value contains "None" or value contains "OrbitRemote"

                -- Click to open the dropdown
                click controlSurfacePopup
                delay 0.5

                -- Select OrbitRemote from the menu
                click menu item "OrbitRemote" of menu 1 of controlSurfacePopup
                delay 1

            on error
                -- Alternative method: use keyboard navigation
                -- Tab to the Control Surface dropdown (adjust number if needed)
                repeat 10 times
                    key code 48 -- Tab key
                    delay 0.1
                end repeat

                -- Type to select
                keystroke "OrbitRemote"
                delay 0.5
                key code 36 -- Return key
            end try

            delay 1

            -- Close preferences
            key code 53 -- Escape key
        end tell
    end tell

    display notification "OrbitRemote configured successfully" with title "Setup Complete"
    '''

    try:
        # Run the AppleScript
        result = subprocess.run(
            ['osascript', '-e', applescript],
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode == 0:
            print("✓ OrbitRemote configured in Ableton preferences!")
            return True
        else:
            print(f"AppleScript error: {result.stderr}")
            return False

    except subprocess.TimeoutExpired:
        print("Configuration timed out - you may need to configure manually")
        return False
    except Exception as e:
        print(f"Error running AppleScript: {e}")
        return False

def main():
    """Main setup function"""
    print("=== OrbitRemote Complete Setup ===\n")

    # Step 1: Install files
    if not install_orbitremote():
        print("\n❌ Installation failed")
        return False

    # Step 2: Check if Ableton is running
    result = subprocess.run(
        ["pgrep", "-f", "Ableton Live"],
        capture_output=True
    )

    if result.returncode != 0:
        print("\nStarting Ableton Live...")
        subprocess.run(["open", "-a", "Ableton Live 12 Suite"])
        time.sleep(5)  # Give Ableton time to start

    # Step 3: Configure in Ableton
    print("\nConfiguring OrbitRemote in Ableton preferences...")
    if configure_in_ableton():
        print("\n✅ Setup complete! OrbitRemote is ready to use.")
        print("\nYou can now use the Orbit agent to control Ableton Live!")
        return True
    else:
        print("\n⚠️ Automatic configuration failed.")
        print("\nPlease configure manually:")
        print("1. In Ableton, go to Preferences > Link/Tempo/MIDI")
        print("2. Select 'OrbitRemote' in the Control Surface dropdown")
        print("3. Leave Input/Output as 'None'")
        return False

if __name__ == "__main__":
    main()