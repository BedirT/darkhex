# Architecture

## Overview

DarkHex is a research toolkit for solving Dark Hex (imperfect-information Hex) using game-theoretic algorithms. The system has three layers:

- **Rust core** (`crates/core/`): High-performance game engine — board representation, move generation, win detection, information state computation. Tabular solvers (MCCFR, exploitability, pONE).
- **Python layer** (`darkhex/`): Neural algorithms, experiments, analysis, paper figure generation. Calls into the Rust engine via PyO3 (`crates/python/`).
- **Web visualization** (`game/`): DSaGe (Dark Hex Strategy Generator) — Three.js isometric board viewer with toon cel-shading and interactive strategy exploration.

The Rust workspace (`crates/`) produces three targets: `core` (pure engine), `python` (PyO3 bindings), and `wasm` (WebAssembly for the web app).

## Directory Structure

```
darkhex/
├── Cargo.toml              # Workspace root (members = ["crates/*"])
├── pyproject.toml           # Python project (maturin build backend)
├── Makefile                 # Dev commands: build, test, lint, check
├── crates/                  # Rust workspace
│   ├── core/                # Game engine + tabular solvers
│   │   └── src/
│   │       ├── lib.rs       # Crate root (re-exports game + solver)
│   │       ├── error.rs     # Error types
│   │       ├── game/        # Foundation layer
│   │       │   ├── types.rs # Player, Cell, CollisionRule enums
│   │       │   ├── board.rs # HexBoard + union-find win detection
│   │       │   ├── state.rs # DarkHexState (4 Dark Hex variants)
│   │       │   └── enumerate.rs # Memoized info state enumeration
│   │       └── solver/      # Algorithm layer (depends on game/)
│   │           ├── mccfr.rs # External + Outcome Sampling MCCFR
│   │           ├── exploitability.rs # Best response + exploitability
│   │           ├── pone.rs  # pONE belief-space precomputation
│   │           └── sip.rs   # SIP/SIP+ policy simplification
│   ├── python/              # PyO3 bindings (maturin cdylib)
│   │   └── src/lib.rs
│   └── wasm/                # WebAssembly target for DSaGe
│       └── src/lib.rs
├── game/                    # DSaGe web app (Three.js + TypeScript)
│   ├── src/
│   │   ├── main.ts          # App entry point
│   │   ├── board/           # Board visualization
│   │   │   ├── IsometricHex.ts  # Hex geometry constants + palette
│   │   │   ├── HexTile.ts      # Individual tile mesh + state
│   │   │   └── BoardLayout.ts  # Grid layout + edge pieces
│   │   ├── engine/          # Game logic interface
│   │   │   └── GameEngine.ts
│   │   ├── postprocess/     # Screen-space effects
│   │   │   └── OutlinePostProcess.ts  # Cel-shading outlines
│   │   └── scenes/          # Scene composition
│   │       └── BoardScene.ts
│   ├── package.json
│   └── vite.config.ts
├── darkhex/                 # Python package
│   ├── algorithms/          # CFR variants (Python-side)
│   └── utils/               # Shared utilities
├── tests/                   # Python integration tests
├── docs/                    # Documentation
└── internal_docs/           # Thesis reference (gitignored)
```

## Game Engine (`crates/core/`)

### Design Decisions

| Decision | Choice | Why |
|----------|--------|-----|
| Language | Rust + PyO3 | 50-100x faster than Python for game tree traversal |
| Win detection | Union-find | O(α(n)) per move vs O(n²) flood-fill in old code |
| Board repr | `Vec<Cell>` + union-find | Compact, cache-friendly, O(1) cell access |
| Info states | String keys | Compatible with Python dict-based CFR; hashable |
| State cloning | `#[derive(Clone)]` | Required for MCCFR tree traversal (branch at each decision) |
| Feature gating | `extension-module` feature | Enables both `cargo test` and `maturin develop` |

### Core Types

**`Player`** (Python-exposed enum):
- `Player.Black` (0) — moves first, connects North↔South
- `Player.White` (1) — connects West↔East

**`Cell`** (Rust-internal):
- `Empty`, `Black`, `White` — no connection tracking needed (union-find handles it)

### HexBoard (`board.rs`)

Hex board with union-find based win detection.

**Virtual nodes** for edge connectivity:
```
Indices: 0..n-1 = board cells, n = Black North, n+1 = Black South,
         n+2 = White West, n+3 = White East
```

**Win detection**: After each stone placement, union the stone with same-colour neighbours and relevant edge virtual nodes. Win = `find(North) == find(South)` for Black (or West/East for White).

**Hex neighbour topology** (6 neighbours per cell):
```
     NW  NE
   W  ·  E
     SW  SE

For cell at (row, col):
  E  = (row, col+1)
  W  = (row, col-1)
  SW = (row+1, col)
  SE = (row+1, col-1)   ← note: shifted
  NE = (row-1, col)
  NW = (row-1, col+1)   ← note: shifted
```

### DarkHexState (`state.rs`)

Full game state for Dark Hex (imperfect information variant of Hex).

**Key difference from Hex**: Players cannot see opponent's stones.

**Collision mechanic**:
1. Player selects a cell that *appears* empty to them
2. If truly empty → stone placed, turn passes
3. If opponent's stone is there → collision: no stone placed, player now *sees* the opponent stone, turn passes

**Per-player views** (`player_views: [Vec<Option<Cell>>; 2]`):
- `None` = cell appears empty to that player
- `Some(Black/White)` = player sees a stone there

A player sees a stone when:
- They placed it themselves, OR
- They collided there (discovering opponent's stone)

**Information state format**:
- Imperfect recall: `"P{player}\n{board_view}"` where board uses `x`/`o`/`.`
- Perfect recall: appends `"\n{player},{action} {player},{action} ..."`

**Terminal conditions**: A player connects their edges (detected by union-find after each successful placement). Hex guarantees a winner on any full board.

### Tested Properties (2x2 board)

| Scenario | Expected | Status |
|----------|----------|--------|
| Black plays column 0 (cells 0,2) | Black wins (N-S) | ✅ |
| White plays row 1 (cells 2,3) | White wins (W-E) | ✅ |
| Collision on occupied cell | No stone placed, turn wasted | ✅ |
| Info state hides opponent | Opponent stones shown as `.` | ✅ |
| Collision reveals in info state | Discovered stone shown as `x`/`o` | ✅ |
| State copy is independent | Mutations don't affect original | ✅ |
| 3x3 Black column win | Black wins with 3 stones | ✅ |

## Algorithms

### Interface Contract

All algorithms operate on `DarkHexState` via this interface:

```python
state = DarkHexState(rows, cols)    # new game
state.current_player()               # whose turn
state.legal_actions()                # available moves
state.apply_action(action)           # make a move
state.is_terminal()                  # game over?
state.returns()                      # [black_payoff, white_payoff]
state.info_state_string(player)      # info set key for CFR
state.copy()                         # branch for tree traversal
```

This replaces the old `pyspiel.Game` / `pyspiel.State` interface.

### Algorithms

| Algorithm | Status | Location | Reference |
|-----------|--------|----------|-----------|
| Outcome Sampling MCCFR | **Implemented** | `crates/core/src/solver/mccfr.rs` | Lanctot et al. 2009 |
| External Sampling MCCFR | **Implemented** | `crates/core/src/solver/mccfr.rs` | Lanctot et al. 2009 |
| Game tree enumeration | **Implemented** | `crates/core/src/game/enumerate.rs` | — |
| Best Response / Exploitability | **Implemented** | `crates/core/src/solver/exploitability.rs` | Zinkevich et al. 2007 |
| Isomorphic state reduction | **Implemented** | `crates/core/src/game/state.rs` | 180° rotation symmetry |
| pONE (probability-1 win states) | **Implemented** | `crates/core/src/solver/pone.rs` | Bonnet 2018 / Thesis §4.2 |
| SimPly (policy simplification) | Planned (port) | — | Thesis |
| SimPly+ (fractionized) | Planned (port) | — | Thesis |
| pONE (sure-win pruning) | Planned | — | Thesis |

## Experiment Pipeline

### Workflow

```
Hypothesis → Config → Run → Results → Analysis → Paper figure
```

Each experiment follows `docs/EXPERIMENT_TEMPLATE.md`:
1. State hypothesis and evaluation plan *before* writing code
2. Verify on smallest board (2x2) first
3. Compare against known equilibrium bounds
4. Log all parameters, seed RNG, checkpoint state
5. Generate publication-ready figures

### Known Equilibrium Bounds

| Board | Result | Source |
|-------|--------|--------|
| 2x2 Dark Hex | TBD | — |
| 3x2 Dark Hex | TBD | — |
| 4x3 Dark Hex | ε improved 0.156 → 0.002 | Thesis |

## Web Visualization — DSaGe (`game/`)

DSaGe (Dark Hex Strategy Generator) is a Three.js web app for interactive strategy exploration.

**Tech stack**: TypeScript, Three.js, Vite, toon cel-shading

**Visual features**:
- Isometric hex board with flat-top tiles (MeshToonMaterial)
- Screen-space post-processing outlines (depth + normal + object ID edge detection)
- 3D chevron edge pieces forming zigzag border bands (Blue=Black, Red=White)
- Interactive stone placement with hover raise animation

**Planned features**:
- Strategy walker (step through MCCFR policies)
- Game tree exploration
- Live MCCFR convergence plots
- WASM integration with `crates/wasm/` for client-side game logic

## Data Flow

```
DarkHexState (Rust, crates/core/)
    ↓ PyO3 (crates/python/)          ↓ WASM (crates/wasm/)
MCCFR algorithms (Python)         DSaGe web app (game/)
    ↓                                 ↓
Policy (Dict[str, Dict[int, f]])   Interactive strategy viewer
    ↓
Experiment runner (Python)
    ↓
Results (JSON/pickle)
    ↓
matplotlib/seaborn → Paper figures
```
