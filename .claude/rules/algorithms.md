---
globs: ["darkhex/algorithms/**", "**/algorithms/**", "src/**"]
---

- Implement the simplest correct version first. Optimize only when profiling shows a bottleneck.
- Separate algorithm logic from game logic — algorithms should work on any extensive-form game, not just Dark Hex.
- Configuration over hardcoding: board size, iterations, epsilon thresholds must all be configurable parameters.
- Always document computational complexity (time and space) in the algorithm's companion markdown.
- Tabular algorithms (MCCFR, exploitability, pONE) live in Rust (`src/`). Neural algorithms live in Python. Do not mix.
- Memory is a hard constraint: use f32 for regrets/strategies, integer-encode info state keys in Rust, allocate lazily.
- Run `make check` before marking any algorithm work complete.
- Test all 4 Dark Hex variants (CDH/ADH/NDH/FDH) when modifying game state logic.
