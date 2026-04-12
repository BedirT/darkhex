# Decision Log

Chronological record of design decisions, rejected alternatives, and rationale.

## Format

Each entry: **Date — Decision title**
- **Choice**: What we chose
- **Alternatives considered**: What else we looked at
- **Rationale**: Why this choice
- **Status**: Active / Superseded / Revisit

---

## 2026-03-27 — Game engine language

- **Choice**: Rust + PyO3/maturin for game engine core
- **Alternatives**: Pure Python, C++ with pybind11, Cython, Julia
- **Rationale**: 50-100x speedup over Python for game tree traversal (the CFR bottleneck). PyO3 gives zero-copy interop. Rust's safety guarantees prevent the memory bugs that plague C++ game engines. Reference implementations (erikbrinkman/cfr, postflop-solver) validate Rust for this domain.
- **Status**: Active

## 2026-03-27 — Package manager

- **Choice**: uv + pyproject.toml + maturin build backend
- **Alternatives**: pip + setup.py, poetry, conda
- **Rationale**: uv is fastest resolver, native maturin support, single pyproject.toml for all config. Replaces old setup.py + requirements.txt.
- **Status**: Active

## 2026-03-27 — Visualization framework

- **Choice**: NiceGUI for web-based visualization
- **Alternatives**: GTK3 (old), tkinter/customtkinter (old), Gradio, Streamlit
- **Rationale**: Web-based (shareable), Python-native, good for interactive game tree exploration. Replaces GTK3 and tkinter from thesis code.
- **Status**: Superseded by 2026-03-31 decision

## 2026-03-27 — Testing strategy

- **Choice**: pytest + hypothesis for Python, `#[cfg(test)]` for Rust
- **Alternatives**: unittest, nose2
- **Rationale**: pytest is standard, hypothesis enables property-based testing for game invariants. Rust unit tests catch engine bugs before Python layer.
- **Status**: Active

## 2026-03-28 — Algorithm placement (Rust vs Python)

- **Choice**: Option B — Rust for game engine + tabular algorithms + vectorized env. Python for neural algorithms (via batched Rust env) + experiment management + analysis.
- **Alternatives**:
  - (A) All Rust including NN via tch-rs — fastest but hard to iterate on NN architectures, non-standard for ML reviewers
  - (C) Rust engine only, all algorithms in Python — simplest but PyO3 boundary crossing per tree node kills performance
- **Rationale**: Tabular MCCFR is the bread-and-butter algorithm and MUST be pure Rust (no PyO3 in the hot loop). Neural approaches (NFSP, Deep CFR, ReBeL) need PyTorch, but the game stepping can be batched: Rust VecEnv runs N games in parallel and sends batched state tensors to Python for NN eval, minimizing boundary crossings (1 crossing per batch of 256 vs 1 per step). This is the architecture used by production RL systems (EnvPool, Gymnasium).
- **Status**: Active
- **Supersedes**: Earlier decision to put all algorithms in Python

## 2026-03-28 — Memory as first-class constraint

- **Choice**: Memory optimization is a hard architectural constraint, not an afterthought
- **Rationale**: Thesis was memory-limited. 4x3 perfect recall has ~10^17 info sets (intractable). Even imperfect recall (367,919 info sets) with naive storage is large. Specific measures:
  1. `f32` for regret/strategy values (not `f64`) — halves memory
  2. Integer-encoded info state keys internally (not strings) — ~8 bytes vs ~20+ bytes per key
  3. Imperfect recall by default — 10^11× smaller than perfect recall
  4. pONE pruning — 20% memory savings from sure-win state elimination
  5. Isomorphic reduction — halves effective state space
  6. Pre-allocated arena / pool allocator for tree nodes — no per-node heap fragmentation
  7. Lazy regret tables — only allocate for visited info sets
  8. Compact action arrays — dense `[f32; MAX_ACTIONS]` for small boards, sparse for large
- **Status**: Active — every new data structure must justify its memory cost

## 2026-03-28 — Dark Hex variant support

- **Choice**: Support all 4 variants from the thesis: CDH (Classic), ADH (Abrupt), NDH (Noisy), FDH (Flash). CDH is the default.
- **Alternatives**: Only implement CDH
- **Rationale**: The variants differ only in collision handling and information disclosure. Supporting all four is cheap (enum + match) and enables comparative experiments. The thesis used CDH throughout, so CDH is the default and must be correct first.
- **Status**: Active — CDH collision fix needed (current code implements ADH by mistake)

## 2026-03-28 — Algorithm roadmap

- **Choice**: Implement in this order: (1) MCCFR outcome sampling, (2) exploitability/best response, (3) SIP/SIP+, (4) pONE, (5) Deep CFR or ReBeL, (6) NFSP
- **Rationale**: MCCFR is the foundation — everything else builds on or evaluates against it. Exploitability is needed to verify MCCFR correctness. SIP/SIP+ are the thesis's novel contribution. pONE provides memory savings. Neural approaches are the paper's new contribution beyond the thesis.
- **Status**: Active

## 2026-03-31 — Visualization: Three.js/DSaGe replaces NiceGUI

- **Choice**: Three.js + TypeScript + Vite for web visualization (DSaGe — Dark Hex Strategy Generator)
- **Alternatives**: NiceGUI (previous choice), Phaser, Bevy, Godot
- **Rationale**: NiceGUI is a Python GUI framework — insufficient for the 3D isometric board visualization, toon cel-shading, and interactive strategy exploration needed. Three.js gives full 3D control with screen-space post-processing outlines, MeshToonMaterial, and can integrate with Rust via WASM (`crates/wasm/`). The `game/` directory is a standalone Vite project.
- **Status**: Active — board rendering implemented, strategy walker planned
- **Supersedes**: 2026-03-27 NiceGUI decision

## 2026-03-31 — Cargo workspace restructuring

- **Choice**: Restructure monolithic `src/` into a Cargo workspace with three crates: `crates/core/` (game engine + solvers), `crates/python/` (PyO3 bindings), `crates/wasm/` (WebAssembly)
- **Alternatives**: Keep monolithic `src/` with feature flags
- **Rationale**: The WASM target for DSaGe needs different compilation flags than the PyO3 cdylib. A workspace cleanly separates the pure Rust core (no FFI dependencies) from the binding layers. Each crate has its own `Cargo.toml` and can be tested independently.
- **Status**: Active

## 2026-03-28 — Win detection algorithm

- **Choice**: Union-find (disjoint set with path halving + union by rank)
- **Alternatives**: Flood-fill (old implementation), BFS/DFS per move, bitboard
- **Rationale**: O(α(n)) amortized per move vs O(n) for flood-fill. Four virtual edge nodes make the check trivial: `connected(North, South)` for Black. The old flood-fill also tracked connection states (y/z/p/q characters) which added complexity; union-find eliminates that entirely.
- **Status**: Active

## 2026-03-28 — Board representation

- **Choice**: `Vec<Cell>` flat array + union-find for connectivity
- **Alternatives**: String-based board (old code used `str` with special characters), bitboard, 2D array
- **Rationale**: Flat vec is cache-friendly and O(1) access. Old string representation mixed data and display (characters like 'y','z','p','q' for connection state). Separating connectivity tracking (union-find) from cell storage (Vec<Cell>) is cleaner.
- **Status**: Active

## 2026-03-28 — Information state format

- **Choice**: String `"P{player}\n{row1}\n{row2}..."` using `x`/`o`/`.`
- **Alternatives**: Binary encoding, tuple of ints, structured object
- **Rationale**: Matches thesis format (enables result comparison). Strings are hashable dict keys in Python (needed by MCCFR). Human-readable for debugging. Compact enough for boards up to ~6x6.
- **Status**: Active — may add binary encoding later for larger boards

## 2026-03-28 — Imperfect information tracking

- **Choice**: Per-player view arrays (`player_views[2][n]`) updated incrementally
- **Alternatives**: Recompute from action history on demand, dual board copies
- **Rationale**: Incremental updates on each move = O(1). Recomputing from history would be O(moves). Two separate boards would duplicate the union-find. Array of `Option<Cell>` is simple: `None` = appears empty, `Some(cell)` = visible.
- **Status**: Active

## 2026-03-28 — PyO3 feature gating

- **Choice**: `extension-module` as a Cargo feature, not a default dependency flag
- **Alternatives**: Always enable extension-module, separate test crate
- **Rationale**: With feature gating, `cargo test` links against Python and runs Rust tests directly. `maturin develop` passes `--features extension-module` for the cdylib. Without this, `cargo test` fails with undefined Python symbols.
- **Status**: Active

## 2026-04-02 — Deep CFR: External Sampling traversal

- **Choice**: External Sampling (ES) for initial Deep CFR prototype
- **Alternatives**: Outcome Sampling (DREAM variant), full game tree traversal
- **Rationale**: The Deep CFR paper (Brown et al., ICML 2019) uses ES exclusively and provides convergence guarantees for it. ES explores all actions at traverser nodes and samples one at opponent nodes, giving unbiased advantage estimates for all actions simultaneously. This is important because the advantage network needs training data for every action at each info state.
- **Status**: Active for 2x2/3x2; **infeasible for 4x3+** (single traversal >30s due to exponential branching O(|A|^d)). DREAM (Steinberger et al. 2020) needed for larger boards.

## 2026-04-02 — Deep CFR: Neural architecture

- **Choice**: Simple MLP with LayerNorm: `input → [Linear→ReLU]* → LayerNorm → Linear → [Softmax]`
- **Alternatives**: CNN on board grid, GNN on hex graph, Transformer
- **Rationale**: Paper and OpenSpiel reference both use MLP with LayerNorm on last hidden layer. 4x3 Dark Hex (~184k canonical info states) is tiny by DL standards — a 64-128 hidden unit MLP with 2 layers is sufficient. Architecture can be upgraded later if needed.
- **Status**: Active — verified on 2x2 (expl 0.04) and 3x2 (expl 0.08)

## 2026-04-02 — Deep CFR: Isomorphic reduction for neural

- **Choice**: Canonicalize info states via 180° rotation before encoding for neural networks
- **Alternatives**: Let NN learn symmetry implicitly, data augmentation (train on both orientations)
- **Rationale**: Halves effective state space (~184k→~92k for 4x3). The Rust engine already has `canonical_info_state()` (used by tabular MCCFR). Matching canonical key format simplifies strategy extraction and exploitability comparison. Action rotation: `action → n-1-action`.
- **Status**: Active — exposed `canonical_info_state()` in PyO3 bindings

## 2026-04-02 — Deep CFR: Key algorithmic choices

- **Choice**: (1) Advantage net reinitialized from scratch each CFR iteration, (2) Argmax tiebreaker when all regrets ≤ 0, (3) LCFR weighting via sqrt(t) trick, (4) Reservoir sampling (not sliding window), (5) Strategy samples at opponent nodes during traversal
- **Alternatives**: Fine-tune advantage net, uniform tiebreaker, vanilla CFR weighting, sliding window buffer, separate strategy collection pass
- **Rationale**: All choices validated by paper ablations (Figure 4): reinitialization gives ~50% lower exploitability vs fine-tuning; argmax gives ~50% lower vs uniform; reservoir sampling converges indefinitely while sliding window stalls at buffer capacity. LCFR weighting follows Brown & Sandholm (2019). Strategy collection at opponent nodes follows paper Algorithm 2 exactly.
- **Status**: Active

## 2026-04-02 — Deep CFR implementation

- **Choice**: Implement Deep CFR (Brown et al., ICML 2019) with External Sampling traversal, isomorphic reduction, and LCFR weighting. PyTorch as optional dependency.
- **Alternatives**: (A) ReBeL — more complex, requires value network + subgame solving. (B) NFSP — RL-based, different convergence guarantees. (C) Single Deep CFR — simpler but less tested.
- **Rationale**: Deep CFR is the simplest neural CFR variant with strong theoretical backing (ε-Nash convergence). External Sampling matches the paper exactly. LCFR weighting, advantage net reinitialization, and argmax tiebreaker are all validated by paper ablations.
- **Status**: Active — ES works for ≤3x2. DREAM (Outcome Sampling variant) needed for 4x3+.

## 2026-04-02 — External Sampling infeasible for 4x3+

- **Choice**: Accept ES infeasibility for boards ≥3x3 in Deep CFR. Plan DREAM (Steinberger et al., 2020) as follow-up.
- **Alternatives**: (A) Batched ES — still exponential per traversal. (B) Depth-limited ES — biased. (C) Move to Rust for traversal — marginally faster but still exponential.
- **Rationale**: ES explores ALL actions at traverser nodes, giving O(|A|^d) per traversal. Measured: 2x2=0.00s, 3x2=0.07s, 3x3=14.4s, 4x3=∞. The exponential scaling is fundamental to ES, not a Python overhead issue. DREAM uses Outcome Sampling (O(depth) per traversal) with importance-weighted advantages.
- **Status**: Active — blocking 4x3 neural experiments

## 2026-04-02 — Isomorphic reduction in neural algorithms

- **Choice**: Use canonical info states (180° rotation) in Deep CFR. Exposed `canonical_info_state()` via PyO3.
- **Alternatives**: (A) Skip — let NN learn symmetry implicitly. (B) Data augmentation — train on both original and rotated.
- **Rationale**: Halves effective state space (~184k→~92k for 4x3). Matching the tabular MCCFR's canonical key format simplifies exploitability comparison. The Rust method already existed; just needed a one-line PyO3 binding.
- **Status**: Active
