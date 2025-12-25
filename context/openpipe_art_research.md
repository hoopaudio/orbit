# OpenPipe ART (Agent Reinforcement Trainer) - Research Notes

> Research conducted: 2025-12-25

## Overview

**ART** is an open-source RL framework from OpenPipe that trains LLM-based agents using **GRPO (Group Relative Policy Optimization)**. It's designed specifically for multi-step agent workflows — perfect for LangGraph-based systems like Orbit.

- **GitHub**: https://github.com/OpenPipe/ART
- **Docs**: https://art.openpipe.ai
- **LangGraph Integration**: https://art.openpipe.ai/integrations/langgraph-integration

## Key Features

| Feature | Description |
|---------|-------------|
| **LangGraph Integration** | Drop-in replacement via `art.langgraph.init_chat_model()` |
| **RULER** | General-purpose reward function using LLM-as-judge |
| **Trajectory Tracking** | Auto-captures multi-step tool calls and responses |
| **Serverless Training** | W&B integration for managed GPU infrastructure |
| **GRPO Algorithm** | More stable than PPO for agent training |

## Installation

```bash
pip install openpipe-art[backend,langgraph]>=0.4.9
```

## How It Works

### 1. Trajectory Capture

ART tracks every agent interaction as a **Trajectory**:
- System/user/assistant messages
- Tool calls and results  
- Final reward signal

```python
class ProjectTrajectory(art.Trajectory):
    final_answer: FinalAnswer | None = None
```

### 2. LangGraph Integration

Replace your normal chat model initialization:

```python
from art.langgraph import init_chat_model
from langgraph.prebuilt import create_react_agent

# ART-wrapped model (auto-logs trajectories)
chat_model = init_chat_model(model.name, temperature=1.0)

# Your existing LangGraph setup works unchanged
tools = [search_tool, read_tool, action_tool]
react_agent = create_react_agent(chat_model, tools)
```

### 3. Training Loop

```python
from art.langgraph import wrap_rollout

for scenario in batch:
    groups.append(
        art.TrajectoryGroup(
            wrap_rollout(model, rollout)(model, scenario)
            for _ in range(4)  # 4 rollouts per scenario
        )
    )

finished_groups = await art.gather_trajectory_groups(groups)
await model.train(finished_groups, config=art.TrainConfig(learning_rate=1e-5))
```

### 4. RULER Scoring (Auto Rewards)

No need to hand-craft reward functions:

```python
judged_group = await ruler_score_group(group, "openai/o4-mini")
```

## Orbit Integration Strategy

### Phase 0: Instrument Agent (Now)
Add trajectory logging to capture user sessions and corrections:

```python
# In orbit-agent tools.py
from art.langgraph import init_chat_model

class OrbitAgent:
    def __init__(self, mode="production"):
        if mode == "training":
            self.llm = init_chat_model("your-model", temperature=1.0)
        else:
            self.llm = ChatAnthropic(model="claude-3-5-sonnet")
```

### Phase 1: Define DAW-Specific Rewards

| Task | Reward Signal |
|------|---------------|
| "Add compressor to Track 3" | OSC confirms compressor exists on Track 3 |
| "Set BPM to 120" | Project BPM == 120 |
| "Navigate to Bar 32" | Playhead position == Bar 32 |

```python
async def daw_reward(trajectory: OrbitTrajectory) -> float:
    current_state = await osc_client.get_project_state()
    return 1.0 if trajectory.goal_achieved(current_state) else 0.0
```

### Phase 2: Train with GRPO

```python
scenarios = load_user_interaction_logs()

for scenario in scenarios:
    groups = [
        art.TrajectoryGroup(
            wrap_rollout(model, daw_agent_rollout)(model, scenario)
            for _ in range(4)
        )
    ]
    
    finished = await art.gather_trajectory_groups(groups)
    
    for group in finished:
        for traj in group:
            traj.reward = await daw_reward(traj)
    
    await model.train(finished)
```

## Supported Models

- Qwen 2.5 / Qwen 3
- Llama family
- Most vLLM/HuggingFace-compatible causal LMs
- ⚠️ Gemma 3 not currently supported

## Key Takeaways for Orbit

1. **ART lowers the RL barrier significantly** — No need for ComputerRL's "thousands of VMs"
2. **LangGraph integration is seamless** — Drop-in for existing stack
3. **RULER eliminates reward engineering** — LLM-as-judge handles evaluation
4. **Start collecting data now** — Instrument production agent for future training

## Recommended Action Plan

```
Phase 0 (Now)
├── Add trajectory logging to orbit-agent
└── Capture user corrections and successful sessions

Phase 1 (After ~100 sessions)
└── Try SFT on successful trajectories

Phase 2 (After seeing plateau)
├── Switch to ART/GRPO training
└── Use RULER for automatic reward
```

## Related Resources

- [ART•E: Email Agent That Beats o3](https://openpipe.ai/blog/art-e-mail-agent)
- [RULER: Easy Mode for RL Rewards](https://openpipe.ai/blog/ruler-easy-mode-for-rl-rewards)
- [MCP•RL: Train Models for MCP Servers](https://x.com/corbtt/status/1953171838382817625)
- [AutoRL: Zero-Data Training](https://x.com/mattshumer_/status/1950572449025650733)
