# DarkHex

Research toolkit for solving Dark Hex (imperfect-information Hex) using game-theoretic algorithms, targeting publishable Nash equilibrium bounds.

## Tech Stack

- Language: Python 3.12 + Rust (game engine core)
- Game engine: Rust + PyO3/maturin (board, state, moves, game tree traversal)
- Algorithms & experiments: Python (NumPy, hypothesis)
- Package manager: uv (pyproject.toml + uv.lock)
- Testing: pytest + hypothesis (property-based)
- Type checking: pyright
- Linting: ruff
- Visualization: matplotlib + seaborn (paper), Three.js/DSaGe (web viz)
- DL interop: PyTorch/JAX via Python interface to Rust game states

## Architecture

- Rust workspace (`crates/`): `core` (game engine + tabular solvers), `python` (PyO3 bindings), `wasm` (WebAssembly for DSaGe)
- Python (`darkhex/`): neural algorithms (NFSP, Deep CFR, ReBeL via PyTorch) + SIP/SIP+ + experiments + analysis
- Web (`game/`): DSaGe — Three.js isometric board viewer with toon cel-shading, TypeScript, Vite
- Tabular MCCFR MUST be pure Rust (no PyO3 in hot loop). Neural approaches use Rust VecEnv with batched tensor exchange.
- Memory optimization is a hard constraint — f32 regrets, integer info state keys, lazy allocation. See docs/DECISIONS.md.

## Game Variants

Four Dark Hex variants supported via CollisionRule x CollisionInfo:
- CDH (Classic): player retries after collision, opponent unaware — thesis default
- ADH (Abrupt): collision wastes turn
- NDH (Noisy): opponent told collision happened
- FDH (Flash): opponent told where collision was

All variants must be tested. CDH is the default for all experiments.

## Git Workflow

- `re-dev` is the development branch (treat like `develop` in gitflow)
- Never push directly to `re-dev` — branch off it, then PR back
- Branch naming: `feature/<name>`, `fix/<name>`, `experiment/<name>`
- Example: `git checkout -b feature/outcome-sampling re-dev`
- PRs merge into `re-dev` with `--no-ff`
- `main` is the stable branch — only merge from `re-dev` at milestones

## Commands

- Install: `uv sync`
- Build Rust (debug): `make dev`
- Build Rust (release): `make build`
- Test all: `make test`
- Test Rust only: `make test-rust`
- Test Python only: `make test-python`
- Lint: `make lint`
- Full check: `make check`

## Workflow

- Document the reasoning chain: when a result reveals a limitation, state what was observed, why it's a problem, and how the next step addresses it. Commit messages and docs should read like a research narrative, not a changelog.
- Prefer the simplest code and simplest solution. Less code, more results, better performance.
- Each algorithm lives in a single Python file. Functions used 3+ times get elevated to shared modules.
- Every experiment needs a hypothesis, evaluation plan, verification plan, and write-up plan in markdown before code is written.
- All experiments must be reproducible: seed RNGs, log all parameters, checkpoint state for resumption.
- Verify algorithm correctness on smallest board (2x2) before scaling. Compare against known equilibrium bounds.
- Every algorithm has a companion markdown doc: what it does, theoretical basis, computational complexity, how to run.
- Every code change should serve the paper — don't refactor for its own sake.

## Project Docs

- docs/ARCHITECTURE.md
- docs/RESEARCH_GUIDELINES.md
- docs/DECISIONS.md
- docs/EXPERIMENT_TEMPLATE.md
- docs/ALGORITHM_TEMPLATE.md

## Verification

Before completing any task:
- All tests pass: `make check`
- Changes address the original request
- No unrelated files modified
- No debug prints or TODO comments left behind
- New functionality has tests

For algorithm changes:
- Verified on smallest board (2x2) first
- Output compared against known equilibrium bounds

For visualization/plot changes:
- Renders without errors
- Publication-quality (labels, legends, proper font sizes)

Loop detection: if you've edited the same file 3+ times for the same issue, stop, summarize what you've tried, and ask for guidance.

Stuck detection: after 3 failed fix attempts, present what you've tried, show the current error, propose alternatives, and ask which to try.

If stuck on a theoretical/mathematical question, stop and ask rather than guessing.
