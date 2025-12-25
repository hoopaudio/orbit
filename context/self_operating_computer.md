# Self-Operating Computer Framework - Research Notes

> Research conducted: 2025-12-25

## Overview

Released in Nov 2023, the **Self-Operating Computer Framework** was one of the first examples of full vision-based computer control for multimodal models. It enables models like GPT-4o, Claude 3, and Gemini to view the screen and decide on mouse/keyboard actions.

- **GitHub**: https://github.com/OthersideAI/self-operating-computer
- **Status**: Legacy/Reference (Newer frameworks like `cua` or `browser-use` are more robust for production).

## Key Concepts

### 1. The Vision Loop
The framework operates on a simple cycle:
1. **View**: Capture a screenshot of the desktop.
2. **Decide**: Pass the screenshot + goal to the VLM.
3. **Act**: Convert VLM output to OS commands (coordinates, clicks, keys).

### 2. Multi-Model Integration
Integrated with:
- GPT-4o
- Gemini Pro Vision
- Claude 3
- Qwen-VL / LLaVa (Open Source)

### 3. Modes of Operation
- **Standard**: Direct vision-to-action coordinate mapping.
- **OCR Mode**: Uses optical character recognition to find text elements before acting.
- **Set-of-Mark (SoM)**: Overlays visual indicators (dots/numbers) on the screen to help the model identify targets.

## Relevance for Orbit

- **Visual Grounding**: The SoM (Set-of-Mark) approach is highly relevant for DAW control. DAWs often have dense, non-standard UIs where direct coordinate mapping can be brittle. SoM helps models "see" which knob or button they are targeting.
- **Cross-Model Benchmarking**: Use it as a reference to compare how different VLMs (e.g., Qwen vs. Claude) interpret a complex Ableton or Logic Pro screenshot.

## Limitations

- **Latency**: Large multimodal models can be slow for real-time DAW control.
- **Coordinate Precision**: Small faders/knobs in DAWs require high-precision clicking, which generic VLMs sometimes struggle with without grounding aids like SoM.

## Lessons for Orbit

1. **Don't rely on raw vision alone**: Grounding aids (SoM, OCR, or Accessibility API metadata) are necessary for complex UIs.
2. **Simplicity exists**: A basic vision-to-action loop can get surprisingly far for high-level tasks ("Add a track").
