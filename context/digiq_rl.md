# Digi-Q: Offline RL for Device Control - Research Notes

> Research conducted: 2025-12-25

## Overview

**Digi-Q** is a scalable value-based offline reinforcement learning (RL) approach designed to train Vision-Language Model (VLM) agents for device control using static, non-interactive data. It addresses the instability and cost of online RL training.

- **Official Title**: Digi-Q: Learning Q-Value Functions for Training Device-Control Agents
- **Conference**: ICLR 2025
- **GitHub**: https://github.com/DigiRL-agent/digiq
- **Project Page**: https://digirl-agent.github.io/digiq/

## The Problem

Standard RL (Online) requires the agent to interact with the environment (e.g., a live Ableton session) thousands of times during training. This is:
- **Slow**: Real-world apps have latency.
- **Unstable**: Complex apps can crash or enter weird states.
- **Expensive**: High GPU/Compute costs for interactive rollouts.

## The Digi-Q Solution: Offline Q-Learning

Digi-Q learns a **Value Function (Q-Value)** from a static dataset of screenshots, actions, and rewards. It then uses this Q-value function to guide a VLM policy.

### Key Innovations

1. **Conservative Q-Learning (CQL)**: Prevents the agent from choosing out-of-distribution (unseen) actions that it thinks have high reward but are actually catastrophic.
2. **Scaled VLM Training**: Demonstrates that VLMs can be effectively fine-tuned into high-performance controllers using only offline data.
3. **State-Action Representation**: Maps visual states (screenshots) directly to discrete or continuous device actions (clicks, swipes, types).

## Relevance for Orbit

Orbit has access to two huge potential data sources:
1. **User Correction Data**: When a user corrects the agent, that's a negative reward transition.
2. **Reference Projects / Tutorials**: Pre-recorded sessions of experts using DAWs.

### Orbit Application

- **Bootstrapping**: Instead of starting with a "blank" agent, use Digi-Q to train on pre-recorded Ableton/Logic tutorials or sessions.
- **Reward Modeling**: The learned Q-function can act as an automated "judge" for other agent implementations.
- **No-Live-Ableton Training**: You can train Orbit on your laptop using static data logs without having Ableton open or running.

## Comparison with ART (OpenPipe)

| Feature | OpenPipe ART | Digi-Q |
|---------|--------------|--------|
| **Approach** | GRPO (Online/Hybrid RL) | Value-Based Offline RL |
| **Interactive?** | Yes, needs environment rollouts | No, uses static logs |
| **Model Focus** | LLM Agents (reasoning) | VLM Agents (perception-action) |
| **Best For** | Improving multi-step logic | Mastering precise UI control |

## Recommendation

Use **ART** for the "Orchestrator" (high-level reasoning) and **Digi-Q** concepts for the "Executor" (low-level knob/fader control) once you have a dataset of successful DAW interactions.
