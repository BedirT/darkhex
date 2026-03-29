# DarkHex — Agent Progress

## Current Milestone
Modernize codebase and produce publishable paper (target: summer 2026)

## Last Session
- Date: 2026-03-28
- Set up Rust+PyO3 project structure, uv packaging, and implemented Dark Hex game engine
- All tests passing (11 Rust + 16 Python), lint clean

## Completed (recent)
- [x] Rust+PyO3/maturin project structure with feature-gated extension-module
- [x] uv + pyproject.toml packaging (replaces setup.py + requirements.txt)
- [x] Makefile with build/dev/test/lint/check commands
- [x] Rust game engine: HexBoard with union-find win detection
- [x] Rust game engine: DarkHexState with collision handling, per-player views, info states
- [x] 11 Rust unit tests (board, union-find, state mechanics)
- [x] 16 Python integration tests (player, state, collision, info states, copy, 3x3)
- [x] Harness setup: AGENTS.md, CLAUDE.md, hooks, rules, init script, back-pressure
- [x] Tech stack finalized: Rust+PyO3 core, Python 3.12 algorithms, uv, NiceGUI
- [x] Agent team: 4 agents (researcher, planner, implementer, reviewer) in .claude/agents/
- [x] Experiment-loop skill (Karpathy autoresearch pattern for CFR)
- [x] Documentation skeletons: ARCHITECTURE.md, RESEARCH_GUIDELINES.md, templates
- [x] Path-scoped rules: algorithms.md, experiments.md

## Up Next
- [ ] Port MCCFR to Python calling Rust game engine (replace pyspiel dependency)
- [ ] Design experiment suite (Karpathy-style loop for CFR parameter exploration)
- [ ] Build NiceGUI web visualization
- [ ] Write paper (cherry-pick claude-scholar skills for paper writing phase)

## Decisions Made
- Rust + PyO3/maturin for game engine core — 50-100x speedup over Python (2026-03-27)
- Reference repos: erikbrinkman/cfr (trait-based), postflop-solver (production-grade Rust CFR) (2026-03-27)
- Python 3.12 for algorithms, experiments, paper (2026-03-27)
- uv + pyproject.toml for packaging (2026-03-27)
- NiceGUI for web visualization (replacing GTK3/tkinter) (2026-03-27)
- pytest + hypothesis for testing (2026-03-27)
- pyright for type checking, ruff for linting (2026-03-27)
- matplotlib + seaborn for paper figures (2026-03-27)
- AGENTS.md as cross-tool source of truth, CLAUDE.md imports it (2026-03-27)
- 4-agent team: researcher, planner, implementer, reviewer — sequential orchestration (2026-03-27)
- Karpathy autoresearch pattern adapted for CFR experiment loop (2026-03-27)
- Cherry-pick claude-scholar skills for paper writing (later) (2026-03-27)
- No game-theory AI tools exist — this gap is our opportunity (2026-03-27)

## Blockers
- None
