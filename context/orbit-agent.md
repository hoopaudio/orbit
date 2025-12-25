# Orbit Agent Architecture & Library Recommendations

## Overview
This document outlines the recommended architecture and library choices for Orbit's agent capabilities, specifically tailored for DAW control (Ableton Live, FL Studio, Logic Pro) and general computer use.

## Recommended Stack: Hybrid Architecture

For a robust DAW-control agent, a single library is insufficient. The optimal approach combines three distinct layers:

### 1. Agent Framework (The Brain)
**Selection: LangGraph**
*   **Status:** Currently implemented in `packages/orbit-agent`.
*   **Why:** Music production workflows are stateful and multi-step (e.g., "Add compressor" -> "Set Ratio" -> "Check Output"). LangGraph excels at managing this state and cyclical workflows compared to linear chains.
*   **Recommendation:** Continue using LangGraph as the core orchestration engine.

### 2. Interaction Layer (The Hands)
This layer executes commands. It requires two distinct modes:

**A. API-Based Control (Primary for Supported DAWs)**
*   **Selection: OSC (Open Sound Control)**
*   **Libraries:** `python-osc` / `rosc` (Rust).
*   **Target:** Ableton Live (via OrbitRemote script), Reaper.
*   **Why:** Low latency, high reliability, runs in background without stealing focus.

**B. Visual/GUI Control (Secondary for FL Studio/Logic/Plugins)**
*   **Selection: PyAutoGUI + PyObjC (macOS)**
*   **Target:** FL Studio, Logic Pro, 3rd Party VSTs.
*   **Why:** Many DAWs lack full APIs. The agent must physically simulate mouse clicks and keystrokes.
*   **Implementation:** 
    *   `pyautogui` for cross-platform coordinate clicking.
    *   `pyobjc` (specifically `ApplicationServices`) for robust macOS accessibility actions.

### 3. Vision Layer (The Eyes)
**Selection: Claude 3.5 Sonnet (Computer Use) + XCap**
*   **Status:** `xcap` implemented in `crates/orbit-ai`.
*   **The "Best" Modern Approach:** Feed screenshots to Claude 3.5 Sonnet with a `computer` tool definition.
*   **Why:** It is currently the state-of-the-art model for interpreting complex UI interfaces like DAWs without requiring custom-trained object detection models.

## Implementation Roadmap

To extend the current Orbit Agent to support generic "Computer Use" or FL Studio:

1.  **Add `ComputerControlTool`:**
    Create a new tool in `packages/orbit-agent/src/tools.py` that utilizes `pyautogui` or `pyobjc`.
    ```python
    class ComputerControlTool(BaseTool):
        name = "computer_control"
        def _run(self, action: str, coordinates: list[int]):
            # Implementation for clicking/typing
    ```

2.  **Enhance Vision:**
    Update `crates/orbit-ai` to pass screenshots directly to the LLM context rather than just OCR text, allowing the agent to "see" buttons and knobs.

3.  **Hybrid Routing:**
    Update the LangGraph router to decide:
    *   "Can I do this via OSC?" -> Use Ableton Tools.
    *   "Do I need to click a plugin UI?" -> Use Computer Control Tools.
