# Agent Prompts

This directory contains prompts for spawning autonomous agents in separate git worktrees.

## Usage

Each agent operates in:
1. Its own git worktree
2. Its own VS Code window
3. Its own branch

Agents must:
- Follow their scoped responsibilities
- Run validation loops unattended
- Only deliver green branches
- Adhere to contracts in `/spec`

## Agent Types

- [Kernel Agent](kernel_agent.md) — GPU compute, cell simulation
- [Gameplay Agent](gameplay_agent.md) — Entities, inventory, crafting
- [Tools Agent](tools_agent.md) — Dev tools, replay, inspector
- [Infra Agent](infra_agent.md) — CI/CD, toolchains
