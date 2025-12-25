# Agent Workflow Memory (AWM) - Research Notes

> Research conducted: 2025-12-25

## Overview

**Agent Workflow Memory (AWM)** is a method to improve agent performance on long-horizon tasks by inducing, integrating, and utilizing commonly reused sub-routines ("workflows") from past experiences.

- **Paper**: [Agent Workflow Memory](https://arxiv.org/abs/2409.07429) (Sep 2024)
- **GitHub**: https://github.com/zorazrw/agent-workflow-memory

## The Core Concept

A **Workflow** is a common sub-routine (sequence of actions) where example-specific contexts are abstracted out. For example, instead of "Click Track 1", a workflow might be "Add Instrument Track -> Select Serum -> Open Plugin".

### Workflow Modes

| Mode | Description |
|------|-------------|
| **Offline** | Induces workflows from a set of high-quality training examples/demonstrations beforehand. |
| **Online** | Learns and induces workflows from the agent's own past experiences on the fly (Self-Correction/Memory). |

## Why Orbit Needs AWM

Music production is highly repetitive and pattern-based. Users often perform the same multi-step sequences:
- **Mixing Template**: EQ -> Compressor -> Limiter.
- **Sound Design**: Load Preset -> Adjust Oscillator -> Apply Filter.
- **Arrangement**: Select Clip -> Duplicate -> Move to Bar 17.

### Benefits for Orbit

1. **Success Rate Boost**: AWM improved success rates on benchmarks by 25-51% by reducing long-horizon planning errors.
2. **Efficiency**: Reduces the number of steps/tokens needed as the agent can call a "stored workflow" instead of re-reasoning from scratch.
3. **Personalization**: Online AWM can learn the *specific user's* production workflow over time.

## Implementation Path for Orbit

1. **Phase 1 (Offline)**: Pre-train common "DAW Workflows" (e.g., "Chain Sidechain Compression") based on standard production techniques.
2. **Phase 2 (Experience Memory)**: Store successful agent trajectories in a vector database.
3. **Phase 3 (Workflow Induction)**: Use an LLM to periodically analyze stored trajectories and extract reusable sub-routines (workflows).
4. **Phase 4 (Retrieval)**: At inference time, the agent retrieves relevant workflows based on the current goal and state.

## Related Findings

- **Cross-Domain Generalization**: AWM robustly generalizes as train-test distribution gaps widen (e.g., learning on Ableton and applying to Logic Pro).
- **Reduced Steps**: Agents with AWM consistently take fewer actions to solve complex tasks.
