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
- **Status**: Active (not yet implemented)

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
