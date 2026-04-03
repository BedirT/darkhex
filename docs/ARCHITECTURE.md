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
├── Cargo.toml              # Workspace root (members = core/python/wasm)
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
│   │       │   ├── enumerate.rs # Memoized info state enumeration
│   │       │   └── info_state_ops.rs # Info state string operations (parse, successor, terminal)
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
│   │   │   ├── GameEngine.ts
│   │   │   └── InfoStateOps.ts  # WASM wrapper for info state operations
│   │   ├── strategy/        # Strategy generator (PolGen port)
│   │   │   ├── types.ts         # Policy, StrategyConfig, StrategySnapshot
│   │   │   ├── HistoryBuffer.ts # Deep-clone snapshot undo/redo
│   │   │   └── StrategyGenerator.ts # Core state machine (action stack, collision branching)
│   │   ├── ui/              # Strategy mode UI panels
│   │   │   ├── SetupPanel.ts    # Modal for board size, player, recall config
│   │   │   ├── ActionPanel.ts   # Bottom toolbar (prob editing, sum validation, confirm/undo)
│   │   │   └── InfoPanel.ts     # Top panel (progress bar, info state display)
│   │   ├── postprocess/     # Screen-space effects
│   │   │   └── OutlinePostProcess.ts  # Cel-shading outlines
│   │   └── scenes/          # Scene composition
│   │       └── BoardScene.ts
│   ├── package.json
│   └── vite.config.ts
├── darkhex/                 # Python package
│   ├── algorithms/          # CFR variants (Python-side)
│   │   ├── deep_cfr.py      # Deep CFR (External Sampling + NNs)
│   │   └── dream.py         # DREAM (Outcome Sampling + NNs + optional Q-baseline)
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

### InfoStateOps (`info_state_ops.rs`)

Stateless operations on info state strings — parsing, legal action extraction, collision detection, successor computation, and terminal detection. This module enables working with info states without constructing a full `DarkHexState`, which is critical for the strategy generator where the game tree is walked from the perspective of a single player's information sets.

**Key operations**:
- `parse_info_state(s)` — Extract player, board view, and action history from an info state string
- `legal_actions(s)` — Return indices of empty cells (`.`) in the board view
- `is_collision_cell(s, action)` — Check if a cell shows an opponent stone (collision occurred)
- `successor(s, action, is_collision)` — Compute the next info state string after an action
- `is_terminal(s)` — Reuse `HexBoard` + union-find to detect if either player has won

**Design**: Stateless — parses the info state string on every call, only needs `(rows, cols)` as context. This makes it safe for WASM where persistent Rust state is awkward.

**WASM exposure**: `InfoStateOps` struct in `crates/wasm/` exposes 7 `wasm_bindgen` methods wrapping these operations for client-side use in DSaGe.

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
state.canonical_info_state(player)   # (canonical_key, is_canonical) for isomorphic reduction
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
| SIP (policy simplification) | **Implemented** | `crates/core/src/solver/sip.rs` | Thesis §4.4 |
| SIP+ (fractionized) | **Implemented** | `crates/core/src/solver/sip.rs` | Thesis §4.5 |
| Deep CFR | **Implemented** | `darkhex/algorithms/deep_cfr.py` | Brown et al. ICML 2019 |
| DREAM (Outcome Sampling Deep CFR) | **Implemented** | `darkhex/algorithms/dream.py` | Steinberger et al. 2020 |
| NFSP | Planned | — | Heinrich & Silver 2016 |

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

DSaGe (Dark Hex Strategy Generator) is a Three.js web app for interactive strategy exploration and manual strategy construction.

**Tech stack**: TypeScript, Three.js, Vite, toon cel-shading, WASM (via `crates/wasm/`)

**Visual features**:
- Isometric hex board with flat-top tiles (MeshToonMaterial)
- Screen-space post-processing outlines (depth + normal + object ID edge detection)
- 3D chevron edge pieces forming zigzag border bands (colors match stone colors)
- Interactive stone placement with hover raise animation
- Selected tile highlight (green) with probability overlay text
- Collision flash animation on opponent-occupied cells

### Strategy Generator (PolGen)

Ported from the old Python/Tkinter `darkhex/gui/` PolGen tool. Lets researchers manually build complete strategies by walking through every reachable info state for one player, assigning action probabilities at each step.

**Architecture**: The generator operates on **info state strings**, not full game states. This is critical for handling collision branching — when a player's action collides with a hidden opponent stone, the info state forks into a collision successor without needing to track the full game tree.

**Workflow**:
1. Press `[S]` to open setup panel (board size, player, perfect recall toggle)
2. Board rebuilds to selected dimensions, showing the chosen player's imperfect-information view
3. Click empty tiles to select actions — tiles turn green with probability overlays
4. Multiple tile selections default to equiprobable; editable via toolbar inputs
5. Sum validation: green checkmark when probabilities sum to 1, amber warning otherwise; Confirm blocked if invalid
6. Double-click for instant deterministic action (probability 1.0)
7. Confirm advances the generator — collision branching creates successor info states automatically
8. Undo/Restart/Rnd buttons for navigation
9. On completion, the full policy exports as JSON (`Dict[info_state, Dict[action, probability]]`)
10. Press `[Esc]` to exit strategy mode and restore the play-mode board

**Key components**:
- `StrategyGenerator` (`strategy/StrategyGenerator.ts`) — Core state machine managing the action stack, collision branching, and policy accumulation
- `HistoryBuffer` (`strategy/HistoryBuffer.ts`) — Deep-clone snapshot buffer for undo/redo
- `SetupPanel` (`ui/SetupPanel.ts`) — Configuration modal
- `ActionPanel` (`ui/ActionPanel.ts`) — Bottom toolbar with probability editing and sum validation
- `InfoPanel` (`ui/InfoPanel.ts`) — Top panel showing progress bar and current info state
- `InfoStateOps` (`engine/InfoStateOps.ts`) — WASM wrapper calling into Rust for info state operations

**Design decisions**:

| Decision | Choice | Why |
|----------|--------|-----|
| Operate on info state strings | Not full game states | Collision branching requires forking at the info set level; full states would need exponential tracking |
| WASM InfoStateOps is stateless | Parses string each call | Avoids persistent Rust state in WASM; simpler lifetime management |
| Terminal detection via HexBoard | Reuse union-find | Consistent win detection with the game engine; no duplicate logic |
| Board-centric UX | Click tiles, overlays on 3D board | Researchers see the board as the player sees it, not an abstract tree |

**Planned features**:
- Strategy walker (step through MCCFR policies)
- Game tree exploration
- Live MCCFR convergence plots

## Data Flow

```
DarkHexState (Rust, crates/core/)
    ↓ PyO3 (crates/python/)          ↓ WASM (crates/wasm/)
Tabular: MCCFR (Rust)             DSaGe web app (game/)
Neural: Deep CFR / DREAM (Python+PyTorch) ↓                     ↓
    ↓                              Interactive play mode     Strategy generator
Policy (Dict[str, List[(int, f)]])                             ↓
    ↓                                                    InfoStateOps (WASM)
Experiment runner (Python)                                     ↓
    ↓                                                    Policy (JSON export)
Results (JSON/CSV)
    ↓
matplotlib/seaborn → Paper figures
```
