---
globs: ["darkhex/algorithms/**", "**/algorithms/**", "src/**"]
description: "Rules for game engine and algorithm code"
---

- Implement the simplest correct version first. Optimize only when profiling shows a bottleneck.
- Separate algorithm logic from game logic — algorithms should work on any extensive-form game, not just Dark Hex.
- Configuration over hardcoding: board size, iterations, epsilon thresholds must all be configurable parameters.
- Always document computational complexity (time and space) in the algorithm's companion markdown.
- Tabular algorithms (MCCFR, exploitability, pONE) live in Rust (`src/`). Neural algorithms live in Python. Do not mix.
- Memory is a hard constraint: use f32 for regrets/strategies, integer-encode info state keys in Rust, allocate lazily.
- Run `make check` before marking any algorithm work complete.
- Test all 4 Dark Hex variants (CDH/ADH/NDH/FDH) when modifying game state logic.
- Rust source is organized: `src/game/` (engine, types, board, state) and `src/solver/` (algorithms). Dependency: solver → game, never reverse.
- Rust tests go in separate `_tests.rs` files next to their source: e.g. `src/game/board_tests.rs` for `src/game/board.rs`. Use `#[cfg(test)] #[path = "board_tests.rs"] mod tests;` at the bottom. Never inline tests.
