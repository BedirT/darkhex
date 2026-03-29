# DarkHex

@AGENTS.md

## Claude-Specific

- @docs/ARCHITECTURE.md
- @docs/RESEARCH_GUIDELINES.md

## Sub-Agents

Use agents sequentially (not in parallel) for scientific correctness:
- `@researcher` — literature review, theory exploration (read-only)
- `@planner` — experiment design, algorithm specs, paper outlines (read-only)
- `@implementer` — code changes, tests, experiments (worktree-isolated)
- `@reviewer` — adversarial review of code, results, paper drafts (read-only)

Flow: researcher → planner → implementer → reviewer → iterate

## Agent Coordination

At session start:
1. Run `./init.sh` to check progress and git state
2. Resume from where the last session left off

Session rules:
- Work on one feature or experiment per session
- Commit incrementally with descriptive messages
- Never remove or edit existing passing tests
- End with production-ready code
- Update `agent-progress.md` before ending the session
- Record failed approaches in progress file (prevent re-attempts)
