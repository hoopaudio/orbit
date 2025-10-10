#!/usr/bin/env python3
"""
Install/Update OrbitRemote script in Ableton Live's MIDI Remote Scripts folder
"""

import os
import shutil
import subprocess
import platform
from pathlib import Path

def find_ableton_scripts_folder():
    """Find Ableton's MIDI Remote Scripts folder based on OS"""
    system = platform.system()

    if system == "Darwin":  # macOS
        # Common paths for Ableton Live
        possible_paths = [
            Path.home() / "Music/Ableton/User Library/Remote Scripts",
            Path("/Applications/Ableton Live 12 Suite.app/Contents/App-Resources/MIDI Remote Scripts"),
            Path("/Applications/Ableton Live 11 Suite.app/Contents/App-Resources/MIDI Remote Scripts"),
            Path("/Applications/Ableton Live 10 Suite.app/Contents/App-Resources/MIDI Remote Scripts"),
        ]
    elif system == "Windows":
        possible_paths = [
            Path.home() / "Documents/Ableton/User Library/Remote Scripts",
            Path("C:/ProgramData/Ableton/Live 12 Suite/Resources/MIDI Remote Scripts"),
            Path("C:/ProgramData/Ableton/Live 11 Suite/Resources/MIDI Remote Scripts"),
        ]
    else:  # Linux
        possible_paths = [
            Path.home() / "Ableton/User Library/Remote Scripts",
        ]

    # Check user library first (preferred location for custom scripts)
    for path in possible_paths:
        if path.exists():
            # For system paths, use user library instead
            if "App-Resources" in str(path) or "ProgramData" in str(path):
                user_path = Path.home() / "Music/Ableton/User Library/Remote Scripts"
                user_path.mkdir(parents=True, exist_ok=True)
                return user_path
            return path

    # If no path exists, create the user library path
    user_path = Path.home() / "Music/Ableton/User Library/Remote Scripts"
    user_path.mkdir(parents=True, exist_ok=True)
    return user_path

def install_orbitremote():
    """Copy OrbitRemote script files to Ableton's scripts folder"""

    # Source directory
    source_dir = Path(__file__).parent.parent / "crates/orbit-connector/scripts/OrbitRemote"

    if not source_dir.exists():
        print(f"Error: Source directory not found: {source_dir}")
        return False

    # Find destination
    dest_base = find_ableton_scripts_folder()
    dest_dir = dest_base / "OrbitRemote"

    print(f"Source: {source_dir}")
    print(f"Destination: {dest_dir}")

    # Remove old version if it exists
    if dest_dir.exists():
        print("Removing old OrbitRemote script...")
        shutil.rmtree(dest_dir)

    # Copy new version
    print("Installing OrbitRemote script...")
    shutil.copytree(source_dir, dest_dir)

    # Verify installation
    required_files = ["__init__.py", "OrbitRemote.py", "OSCServer.py"]
    for file in required_files:
        if not (dest_dir / file).exists():
            print(f"Error: Failed to copy {file}")
            return False

    print("OrbitRemote script installed successfully!")
    print("\nNext steps:")
    print("1. Restart Ableton Live (or open it if not running)")
    print("2. Go to Preferences > Link/Tempo/MIDI")
    print("3. Select 'OrbitRemote' in the Control Surface dropdown")
    print("4. The script should now be active with the latest changes")

    return True

def restart_ableton():
    """Attempt to restart Ableton Live (macOS only)"""
    if platform.system() != "Darwin":
        return

    try:
        # Check if Ableton is running
        result = subprocess.run(
            ["pgrep", "-f", "Ableton Live"],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            print("\nAttempting to restart Ableton Live...")
            # Kill Ableton
            subprocess.run(["pkill", "-f", "Ableton Live"])
            print("Ableton Live closed.")

            # Wait a moment
            import time
            time.sleep(2)

            # Reopen Ableton
            subprocess.run(["open", "-a", "Ableton Live 12 Suite"])
            print("Ableton Live reopened. Please reconfigure OrbitRemote in preferences.")
    except Exception as e:
        print(f"Could not restart Ableton automatically: {e}")
        print("Please restart Ableton Live manually.")

if __name__ == "__main__":
    if install_orbitremote():
        response = input("\nWould you like to restart Ableton Live now? (y/n): ")
        if response.lower() == 'y':
            restart_ableton()