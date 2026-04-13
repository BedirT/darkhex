# DarkHex

Research toolkit for solving **Dark Hex** — the imperfect-information variant of the board game [Hex](https://en.wikipedia.org/wiki/Hex_(board_game)) — using game-theoretic algorithms. Targets publishable Nash equilibrium bounds.

In Dark Hex, players cannot see their opponent's stones. When a player attempts to place a stone on a cell already occupied by the opponent, a *collision* occurs — the stone is not placed, but the player discovers the opponent's hidden stone. Four collision variants are supported (Classic, Abrupt, Noisy, Flash).

## Architecture

The system has three layers:

- **Rust core** (`crates/core/`) — High-performance game engine with union-find win detection, move generation, and tabular solvers (MCCFR, exploitability, pONE, SIP/SIP+).
- **Python layer** (`darkhex/`) — Neural algorithms (Deep CFR, DREAM, ESCHER), experiments, and paper figure generation. Calls into Rust via [PyO3](https://pyo3.rs/) / [maturin](https://www.maturin.rs/).
- **Web visualization** (`game/`) — DSaGe (Dark Hex Strategy Generator), a Three.js isometric board viewer with toon cel-shading for interactive strategy exploration.

## DSaGe — Web Visualization

DSaGe is an interactive web tool for building and investigating Dark Hex strategies. It runs entirely in the browser using WebAssembly compiled from the Rust game engine.

<p align="center">
  <video src="docs/screenshots/dsage-demo.mp4" width="600" autoplay loop muted playsinline></video>
</p>

**Features:**
- Isometric hex board with toon cel-shading and post-processing outlines
- **Strategy Generator** — manually build complete strategies by walking through every reachable info state, assigning action probabilities
- **Strategy Investigation** — browse completed strategies, step through info states, view assigned probabilities on the board
- Guided tutorial, SVG tree export, MCCFR policy import, keyboard navigation

## Algorithms

### Tabular (Rust)

| Algorithm | Description |
|-----------|-------------|
| External Sampling MCCFR | Monte Carlo CFR with external sampling |
| Outcome Sampling MCCFR | Monte Carlo CFR with outcome sampling |
| Best Response / Exploitability | Exact best response computation for exploitability measurement |
| pONE | Belief-space precomputation of probability-1 win states |
| SIP / SIP+ | Policy simplification (deterministic / fractionized) |

### Neural (Python + PyTorch)

| Algorithm | Description |
|-----------|-------------|
| Deep CFR | External sampling CFR with neural value networks |
| DREAM | Outcome sampling CFR with neural networks and optional Q-baseline |
| ESCHER | Information-set-free neural CFR (value net + regret net + avg policy) |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Python 3.12+](https://www.python.org/)
- [uv](https://docs.astral.sh/uv/) (Python package manager)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) (for DSaGe web app)
- [Node.js](https://nodejs.org/) (for DSaGe web app)

## Getting Started

```bash
# Install Python dependencies
uv sync

# Build Rust extension (debug mode, fast compile)
make dev

# Run all tests
make test
```

## Commands

| Command | Description |
|---------|-------------|
| `make install` | Install Python dependencies via uv |
| `make dev` | Build Rust extension (debug mode) |
| `make build` | Build Rust extension (release mode) |
| `make test` | Run all tests (Rust + Python) |
| `make test-rust` | Run Rust unit tests only |
| `make test-python` | Run Python integration tests only |
| `make lint` | Lint Python code (ruff) |
| `make typecheck` | Type check Python code (pyright) |
| `make check` | Full verification (lint + all tests) |
| `make game` | Build WASM + launch DSaGe web app |
| `make fmt` | Format Rust code |

## Running DSaGe

```bash
# Build WASM and start dev server
make game

# Or step by step:
make wasm              # Build WASM package
cd game && npm run dev # Start Vite dev server
```

Then open `http://localhost:5173` in your browser.

## Project Structure

```
├── crates/
│   ├── core/          # Game engine + tabular solvers (Rust)
│   ├── python/        # PyO3 bindings (maturin cdylib)
│   └── wasm/          # WebAssembly target for DSaGe
├── darkhex/           # Neural algorithms + experiments (Python)
│   └── algorithms/    # Deep CFR, DREAM, ESCHER
├── game/              # DSaGe web app (Three.js + TypeScript + Vite)
├── tests/             # Python integration tests
├── docs/              # Documentation
└── results/           # Experiment outputs
```

## References

- Zinkevich, Johanson, Bowling, & Piccione (2007). "Regret Minimization in Games with Incomplete Information." *NeurIPS*.
- Lanctot, Waugh, Zinkevich, & Bowling (2009). "Monte Carlo Sampling for Regret Minimization in Extensive Games." *NeurIPS*.
- Brown, Lerer, Gross, & Sandholm (2019). "Deep Counterfactual Regret Minimization." *ICML*.
- Steinberger, Lerer, & Brown (2020). "DREAM: Deep Regret minimization with Advantage baselines and Model-free learning."
- McAleer, Wang, Lanier, Baldi, & Fox (2023). "ESCHER: Eschewing Importance Sampling in Games by Computing Histories of Expected Rewards." *ICLR*.

## License

MIT
