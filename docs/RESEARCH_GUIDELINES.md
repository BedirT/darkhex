# Research Guidelines

## Theoretical Foundation

This project builds on the CFR (Counterfactual Regret Minimization) family of algorithms for solving imperfect-information extensive-form games. Key concepts:

- **Nash equilibrium**: A strategy profile where no player can unilaterally improve their payoff. In two-player zero-sum games, this is the solution concept.
- **ε-Nash equilibrium**: A strategy profile where no player can improve by more than ε. Tabular CFR converges to exact Nash; Deep CFR converges to ε-Nash where ε depends on function approximation error.
- **Exploitability**: Average payoff a best-response opponent could extract. `exploitability(σ) = (BR_Black(σ_White) + BR_White(σ_Black)) / 2`. Zero exploitability = Nash equilibrium.
- **Clairvoyant vs Abstract Best Response**: Our `best_response_values()` computes clairvoyant BR (full game tree, optimal per-state). Thesis used Abstract BR (information-constrained). Clairvoyant BR is an upper bound on true exploitability.

## When Stuck on Theory
- Stop and ask rather than guessing at mathematical/theoretical questions
- Reference the thesis for established results: `internal_docs/thesis_write_up/thesis.pdf`
- Known equilibrium bounds for verification:
  - 2x2 Dark Hex: exploitability ~0.0001 at 100k tabular OS-MCCFR iters; ~0.018 at 100 Deep CFR iters
  - 3x2 Dark Hex: exploitability ~0.027 at 100k tabular OS-MCCFR iters; ~0.045 at 50 Deep CFR iters
  - 4x3 Dark Hex: thesis ε improved 0.156 → 0.002 (Ab-BR metric, 1B iters + SIP+); tabular OS-MCCFR plateaus at ~0.989 (clairvoyant BR, 1B iters)

## Algorithm Documentation Standard
Each algorithm must have a companion markdown doc containing:
1. What the algorithm does (plain language)
2. Theoretical basis (with references)
3. Computational complexity (time and space)
4. How to run it (exact commands)
5. Expected outputs and how to verify correctness

See `docs/algorithms/` for examples: `mccfr.md`, `exploitability.md`, `deep_cfr.md`, etc.

## Citation and References

Key references for this project:

- Zinkevich, Johanson, Bowling, & Piccione (2007). "Regret Minimization in Games with Incomplete Information." NeurIPS.
- Lanctot, Waugh, Zinkevich, & Bowling (2009). "Monte Carlo Sampling for Regret Minimization in Extensive Games." NeurIPS.
- Brown, Lerer, Gross, & Sandholm (2019). "Deep Counterfactual Regret Minimization." ICML.
- Steinberger, Lerer, & Brown (2020). "DREAM: Deep Regret minimization with Advantage baselines and Model-free learning." arXiv:2006.10410.
- Heinrich & Silver (2016). "Deep Reinforcement Learning from Self-Play in Imperfect-Information Games." arXiv:1603.01121.
- Brown & Sandholm (2019). "Solving Imperfect-Information Games via Discounted Regret Minimization." AAAI.
- Tapkan (2024). MSc Thesis — Dark Hex equilibrium bounds.
