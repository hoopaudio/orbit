# trycua (Cua) Framework - Research Notes

> Research conducted: 2025-12-25

## Overview

**trycua** (or Cua) is an open-source infrastructure for Computer-Use Agents. It provides sandboxes, SDKs, and benchmarks to train and evaluate AI agents that can control full desktops (macOS, Linux, Windows).

- **GitHub**: https://github.com/trycua/cua
- **Docs**: https://cua.ai/docs
- **Starter Template**: https://github.com/trycua/agent-template

## Key Modules

| Module | Description |
|--------|-------------|
| **Agent SDK** | Python framework for automating tasks with multi-model support (Claude, GPT, Gemini). |
| **Computer SDK** | TypeScript/Python SDK for low-level screen/input control in sandboxed environments. |
| **MCP Server** | Use Cua agents and computers via the Model Context Protocol. |
| **Lume / Lumier** | VM management (Lume) and Docker interface (Lumier) for macOS/Linux. |
| **SOM (Set-of-Mark)** | Library for visual grounding, allowing agents to see interactable elements more clearly. |

## Why Use It for Orbit?

1. **Multi-Model Support**: Easily switch between Claude 3.5 Sonnet (default for computer use) and other models like GPT-4o or Gemini.
2. **Robust Computer Control**: Handles low-level OS interactions (`left_click`, `right_click`, `type_text`, `screenshot`) with a clean API.
3. **Sandboxed Environments**: Critical if Orbit needs to run in a controlled environment to avoid damaging the user's project files during experiments.
4. **macOS Optimization**: Includes specific tools for macOS VM management (`Lume`), which is vital for Logic Pro/Ableton users.

## Sample Usage (Agent SDK)

```python
from agent import ComputerAgent

# Initialize agent with multi-model support
agent = ComputerAgent(
    model="anthropic/claude-sonnet-4-5-20250929", # Uses future-proof model strings
    tools=[computer],
    max_trajectory_budget=5.0
)

messages = [{"role": "user", "content": "Open Ableton and set tempo to 120"}]

async for result in agent.run(messages):
    # Process steps and outputs
    pass
```

## Sample Usage (Computer SDK)

```python
from computer import Computer

computer = Computer(
    os_type="macos",
    provider_type="cloud", # Or "host" for local desktop
    use_host_computer_server=True 
)

await computer.run()
screenshot = await computer.interface.screenshot()
await computer.interface.left_click(500, 500)
await computer.interface.type_text("C-sharp minor chord")
```

## Implementation Recommendation

Orbit should consider using `cua-computer` as the **Interaction Layer** (Hands) instead of building a custom implementation over `pyautogui`/`pyobjc` from scratch. This allows the team to focus on the **Agent Architecture** (Brain) and **Music Domain Logic** rather than low-level desktop automation.
