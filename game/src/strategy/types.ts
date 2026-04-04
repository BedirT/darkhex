/** A single action–probability pair. */
export type ActionProb = [action: number, probability: number]

/** Maps info state strings to their assigned action probabilities. */
export type Policy = Map<string, ActionProb[]>

/** Deep-clonable snapshot of the strategy generator state. */
export interface StrategySnapshot {
  policy: Policy
  currentInfoState: string
  actionStack: Array<{ state: string; collisionCell: number | null }>
  queuedStates: Set<string>
  targetStackState: string | null
  lastCollisionIndex: number | null
}

/** Configuration for a new strategy generation session. */
export interface StrategyConfig {
  rows: number
  cols: number
  player: number         // 0 = Black, 1 = White
  perfectRecall: boolean
}

/** JSON format exported by StrategyGenerator.exportPolicy(). */
export interface ExportedPolicy {
  player: number
  rows: number
  cols: number
  perfectRecall: boolean
  policy: Record<string, Record<string, number>>
}
