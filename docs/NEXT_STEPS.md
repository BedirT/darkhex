# Next Steps

Priority improvements and experiments for the darkhex neural CFR pipeline.

## P0: GPU Acceleration

**Status**: All neural algorithms (Deep CFR, DREAM, ESCHER) currently run on CPU only. No `.to(device)`, no CUDA usage anywhere.

### What Benefits from GPU

| Component | Current | With GPU | Speedup Est. |
|-----------|---------|----------|-------------|
| Network forward passes during traversal | Sequential CPU | Batched GPU | 10-50x |
| SGD training steps (375-2500 per iter) | CPU tensors | GPU tensors | 5-20x |
| ESCHER `_compute_regret` (N fwd per node) | Loop, 1 at a time | Batch all children | 10-50x |
| `encode_info_state` | CPU NumPy→Tensor | Pre-encode to GPU tensor | 2-5x |

### What Stays on CPU

- Game tree traversal (Rust engine, sequential)
- Exploitability computation (Rust DFS)
- Reservoir buffer management (NumPy)

### Implementation Plan

1. Add `device` parameter to all config dataclasses
2. Move networks to device in `__init__`
3. Batch ESCHER's `_compute_regret`: collect all child histories, single `net(batch)` call
4. Keep buffers on CPU (NumPy), transfer to GPU only at training time
5. Cache `encode_info_state` results (same info states visited repeatedly)

### Expected Impact

ESCHER 3x3 currently takes 293s for 50 iterations. With GPU:
- Training SGD: ~5x faster (bulk of non-traversal time)
- Regret computation: ~20x faster (currently the bottleneck in `_gather_regret_data`)
- Estimated total: **60-90s for 50 iterations** (3-5x overall speedup)

For 4x3, GPU acceleration is critical — the value/regret network forward passes dominate runtime.

## P1: NFSP Implementation

**Status**: Planned. User has prior thesis experience with NFSP achieving good results on Dark Hex.

NFSP (Neural Fictitious Self-Play, Heinrich & Silver 2016) is fundamentally different from CFR-based methods:
- Uses RL (DQN) for best response + supervised learning for average strategy
- No regret tracking, no importance sampling, no CFR iteration structure
- Self-play loop: agents play against each other's average strategies

### Why NFSP

- User achieved best thesis results with NFSP
- No IS variance issues (unlike DREAM)
- No value-net accuracy bottleneck (unlike ESCHER)
- Well-suited to GPU training (DQN + supervised learning are standard GPU workloads)
- OpenSpiel has reference implementation

### Architecture

```
NFSP Agent (per player)
├── Best Response Network (DQN)
│   ├── Replay buffer (circular)
│   ├── Target network (periodic sync)
│   └── ε-greedy exploration
├── Average Strategy Network (supervised)
│   ├── Reservoir buffer
│   └── Cross-entropy loss
└── Anticipatory parameter η
    └── Mix: η * best_response + (1-η) * average_strategy
```

### Implementation Plan

1. Port OpenSpiel NFSP to darkhex infrastructure
2. Reuse: `encode_info_state`, `MLP`, canonical actions, exploitability
3. New: DQN with replay buffer, target network, ε-schedule
4. GPU from day 1 (DQN training is the bottleneck)

## P2: Scale ESCHER for 4x3

ESCHER achieves 0.020 exploitability on 3x3 (50 iters, 293s) but only reaches 0.995 on 4x3 (30 iters, 2h). The 184k info states need more coverage.

### Scaling Plan

| Parameter | Current (4x3) | Scaled |
|-----------|---------------|--------|
| value_traversals | 512 | 5000 |
| regret_traversals | 1024 | 5000 |
| num_iters | 30 | 200+ |
| Network size | (256, 256) | (512, 256) |
| GPU | No | Yes (P0 prerequisite) |

With GPU acceleration (P0), a 200-iteration scaled run should complete in ~2-4 hours instead of ~15+ hours on CPU.

## P3: Experiment Pipeline Improvements

- Add solver checkpointing to ESCHER/DREAM (resume interrupted runs)
- Add head-to-head evaluation (play strategies against each other)
- Add wall-clock convergence plots (exploitability vs time, not just iterations)
- Compare neural strategies against tabular MCCFR strategies

## P4: Paper Figures

Once 4x3 results are available:
- Convergence comparison: Tabular MCCFR vs Deep CFR vs DREAM vs ESCHER vs NFSP
- Log-log exploitability vs wall-clock time
- Algorithm scalability figure (performance vs board size)
- Strategy visualization via DSaGe web app
