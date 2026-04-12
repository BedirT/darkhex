---
globs: ["experiments/**", "**/experiment*"]
---

- Never overwrite experiment results — append with timestamps or version tags.
- Log progress at regular intervals for long-running experiments.
- Generate publication-ready plots (matplotlib/seaborn with proper labels, legends, font sizes).
- Each experiment directory must contain: hypothesis.md, results/, and a summary of findings.
