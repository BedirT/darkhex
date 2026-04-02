import type { Policy, StrategySnapshot } from './types'

/** Deep-clone a Policy map. */
function clonePolicy(policy: Policy): Policy {
  const copy: Policy = new Map()
  for (const [key, val] of policy) {
    copy.set(key, val.map(([a, p]) => [a, p]))
  }
  return copy
}

/** Deep-clone a StrategySnapshot. */
function cloneSnapshot(s: StrategySnapshot): StrategySnapshot {
  return {
    policy: clonePolicy(s.policy),
    currentInfoState: s.currentInfoState,
    actionStack: s.actionStack.map((e) => ({ ...e })),
    queuedStates: new Set(s.queuedStates),
    targetStackState: s.targetStackState,
    lastCollisionIndex: s.lastCollisionIndex,
  }
}

/**
 * History buffer for undo/restart in the strategy generator.
 *
 * Stores deep-cloned snapshots of the generator state at each step.
 * Port of `darkhex/gui/history_buffer.py`.
 */
export class HistoryBuffer {
  private snapshots: StrategySnapshot[] = []

  /** Record the current state. */
  push(snapshot: StrategySnapshot): void {
    this.snapshots.push(cloneSnapshot(snapshot))
  }

  /** Undo the last action. Returns the restored snapshot, or null if at initial. */
  rewind(): StrategySnapshot | null {
    if (this.snapshots.length <= 1) return null
    this.snapshots.pop()
    return cloneSnapshot(this.snapshots[this.snapshots.length - 1])
  }

  /** Reset to the initial state. Returns the initial snapshot, or null if empty. */
  restart(): StrategySnapshot | null {
    if (this.snapshots.length === 0) return null
    this.snapshots.length = 1
    return cloneSnapshot(this.snapshots[0])
  }

  get length(): number {
    return this.snapshots.length
  }

  get canRewind(): boolean {
    return this.snapshots.length > 1
  }
}
