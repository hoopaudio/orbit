#!/usr/bin/osascript

-- Reload OrbitRemote script in Ableton Live
tell application "Ableton Live 12 Suite"
    activate
    delay 0.5

    -- Open Preferences (Cmd+Comma)
    tell application "System Events"
        keystroke "," using command down
        delay 1

        -- Navigate to Link/Tempo/MIDI tab
        keystroke "l" using command down
        delay 0.5

        -- Tab to Control Surface dropdown (adjust number of tabs if needed)
        repeat 8 times
            key code 48 -- Tab key
            delay 0.1
        end repeat

        -- Select "None" first to unload the script
        keystroke "None"
        delay 0.5
        key code 36 -- Return key
        delay 1

        -- Now select OrbitRemote again
        keystroke "OrbitRemote"
        delay 0.5
        key code 36 -- Return key
        delay 1

        -- Close preferences (Escape)
        key code 53 -- Escape key
    end tell
end tell

display notification "OrbitRemote script reloaded" with title "Ableton Script Reload"