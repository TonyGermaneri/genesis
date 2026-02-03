# Git Worktree Setup for Multi-Agent Development

This document explains how to set up git worktrees for parallel agent development.

## Overview

Each agent works in an isolated git worktree with its own branch. This allows:
- True parallel development
- No merge conflicts during work
- Separate VS Code windows per agent
- Clean integration via orchestrator

## Initial Setup

From the main repository:

```bash
cd /Users/tonygermaneri/gh/genesis

# Create agent branches
git branch kernel-agent
git branch gameplay-agent
git branch tools-agent
git branch infra-agent

# Create worktrees in sibling directories
git worktree add ../genesis-kernel kernel-agent
git worktree add ../genesis-gameplay gameplay-agent
git worktree add ../genesis-tools tools-agent
git worktree add ../genesis-infra infra-agent
```

## Directory Structure After Setup

```
/Users/tonygermaneri/gh/
├── genesis/           # Main repo (orchestrator)
├── genesis-kernel/    # Kernel agent worktree
├── genesis-gameplay/  # Gameplay agent worktree
├── genesis-tools/     # Tools agent worktree
└── genesis-infra/     # Infra agent worktree
```

## VS Code Multi-Window

Open each worktree in its own VS Code window:

```bash
# Terminal 1: Orchestrator
code /Users/tonygermaneri/gh/genesis

# Terminal 2: Kernel Agent
code /Users/tonygermaneri/gh/genesis-kernel

# Terminal 3: Gameplay Agent
code /Users/tonygermaneri/gh/genesis-gameplay

# Terminal 4: Tools Agent
code /Users/tonygermaneri/gh/genesis-tools

# Terminal 5: Infra Agent (optional)
code /Users/tonygermaneri/gh/genesis-infra
```

## Agent Workflow

### 1. Start Work

```bash
cd ../genesis-kernel  # or appropriate worktree
git pull origin main
git rebase main
```

### 2. Validation Loop (MANDATORY)

Each agent MUST run this loop until green:

```bash
# Run the full validation
just validate

# Or manually:
cargo fmt --check || cargo fmt
cargo clippy -- -D warnings
cargo test --workspace
```

### 3. Commit (only when green)

```bash
git add -A
git commit -m "[kernel] feat: implement cell simulation shader"
```

### 4. Push for Integration

```bash
git push origin kernel-agent
```

### 5. Integration (Orchestrator Only)

The orchestrator merges validated branches:

```bash
cd /Users/tonygermaneri/gh/genesis
git fetch --all
git merge kernel-agent --no-ff -m "Integrate kernel agent work"
```

## Worktree Management

### List Worktrees

```bash
git worktree list
```

### Remove Worktree

```bash
git worktree remove ../genesis-kernel
```

### Prune Stale Worktrees

```bash
git worktree prune
```

## Troubleshooting

### "Branch already checked out"

This happens if you try to checkout a branch that's in another worktree.
Solution: Use a different branch or remove the other worktree.

### Merge Conflicts

1. Agent does NOT resolve conflicts
2. Agent reports to orchestrator
3. Orchestrator resolves and re-merges

### Stale Worktree

If a worktree's directory was deleted without `git worktree remove`:

```bash
git worktree prune
```
