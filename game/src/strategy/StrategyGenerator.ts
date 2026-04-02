import type { InfoStateOps } from '../engine/InfoStateOps'
import { HistoryBuffer } from './HistoryBuffer'
import type { ActionProb, Policy, StrategyConfig, StrategySnapshot } from './types'

/**
 * Strategy generator state machine.
 *
 * Walks through every reachable info state for one player, letting the user
 * assign action probabilities at each step to build a complete policy.
 *
 * Port of `darkhex/gui/strategy_generator.py`.
 */
export class StrategyGenerator {
  private infoOps: InfoStateOps
  private config: StrategyConfig

  // Mutable state
  currentInfoState: string
  private actionStack: Array<{ state: string; collisionCell: number | null }>
  private queuedStates: Set<string> = new Set() // prevents duplicate stack entries
  policy: Policy
  private targetStackState: string | null = null
  private history: HistoryBuffer

  /** Last collision cell index (for visual feedback), or null. */
  lastCollisionIndex: number | null = null

  constructor(infoOps: InfoStateOps, config: StrategyConfig) {
    this.infoOps = infoOps
    this.config = config
    // Perfect recall initial state needs trailing newline for empty action history
    let initialState = infoOps.initialInfoState(config.player)
    if (config.perfectRecall) initialState += '\n'
    this.currentInfoState = initialState
    this.actionStack = []
    this.policy = new Map()
    this.history = new HistoryBuffer()

    // Record initial state
    this.history.push(this.snapshot())
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  /**
   * Process user input. Returns true if all info states have been assigned
   * (strategy complete).
   *
   * Input formats:
   * - "r"          → random action (deterministic, prob 1.0)
   * - "3"          → single action by cell index
   * - "a2"         → single action by alphanumeric
   * - "= 0 3 5"    → equiprobable over listed actions
   * - "0 0.5 3 0.5" → explicit action–probability pairs
   */
  iterateBoard(input: string): boolean {
    const [actions, probs, addition] = this.parseInput(input)
    return this._advance(actions, probs, addition, input.trim() === 'r')
  }

  /**
   * Submit actions with probabilities directly (no string serialization).
   * Used by the board-centric UI to avoid rounding errors.
   */
  submitActions(actions: number[], probs: number[]): boolean {
    // Validate
    if (actions.length === 0) throw new Error('No actions')
    const sum = probs.reduce((a, b) => a + b, 0)
    if (Math.abs(sum - 1) > 0.01) throw new Error(`Probabilities sum to ${sum}`)
    const legal = new Set(this.legalActions)
    for (const a of actions) {
      if (!legal.has(a)) throw new Error(`Action ${a} is not legal`)
    }
    // Normalize
    for (let i = 0; i < probs.length; i++) probs[i] /= sum

    // Enumerate successors
    const addition = this._enumerateSuccessors(actions)
    return this._advance(actions, probs, addition, false)
  }

  private _advance(actions: number[], probs: number[], addition: number, isRandom: boolean): boolean {
    // Store policy for current info state
    const actionProbs: ActionProb[] = actions.map((a, i) => [a, probs[i]])
    this.policy.set(this.currentInfoState, actionProbs)

    // Check if action stack is exhausted
    if (this.actionStack.length === 0) {
      this.history.push(this.snapshot())
      return true
    }

    // Pop next unvisited info state
    const next = this.actionStack.pop()!
    this.currentInfoState = next.state
    this.lastCollisionIndex = next.collisionCell
    this.history.push(this.snapshot())

    // Handle random-action chaining
    if (isRandom && this.targetStackState === null) {
      if (addition > 0 && this.actionStack.length >= addition) {
        this.targetStackState =
          this.actionStack[this.actionStack.length - addition].state
      } else {
        this.targetStackState = null
      }
    }
    if (this.targetStackState !== null) {
      if (this.targetStackState === this.currentInfoState) {
        this.targetStackState = null
      } else {
        return this.iterateBoard('r')
      }
    }

    return false
  }

  /** Enumerate successor info states with collision branching. Returns count added to stack. */
  private _enumerateSuccessors(actions: number[]): number {
    const { player, perfectRecall } = this.config
    const opponent = 1 - player
    const collisionPossible = this.infoOps.isCollisionPossible(this.currentInfoState)
    const stonePlayersToTry = collisionPossible ? [player, opponent] : [player]

    let addition = 0
    for (const action of actions) {
      for (const stonePlayer of stonePlayersToTry) {
        const nextState = this.infoOps.infoStateAfterAction(
          this.currentInfoState, action, stonePlayer, perfectRecall,
        )
        if (this.infoOps.isTerminal(nextState)) continue
        if (!this.policy.has(nextState) && !this.queuedStates.has(nextState)) {
          const isCollision = stonePlayer !== player
          this.actionStack.push({
            state: nextState,
            collisionCell: isCollision ? action : null,
          })
          this.queuedStates.add(nextState)
          addition++
        }
      }
    }
    return addition
  }

  /** Legal actions at the current info state. */
  get legalActions(): number[] {
    return this.infoOps.legalActions(this.currentInfoState)
  }

  /** Whether all reachable info states have been assigned. */
  get isComplete(): boolean {
    return this.actionStack.length === 0 && this.policy.has(this.currentInfoState)
  }

  /** Progress: how many info states assigned vs remaining in stack. */
  get progress(): { assigned: number; remaining: number } {
    return {
      assigned: this.policy.size,
      remaining: this.actionStack.length + (this.policy.has(this.currentInfoState) ? 0 : 1),
    }
  }

  /** Board view for the current info state (flat i8 array). */
  get boardView(): Int8Array {
    return this.infoOps.boardViewFlat(this.currentInfoState)
  }

  get canRewind(): boolean {
    return this.history.canRewind
  }

  get player(): number {
    return this.config.player
  }

  get rows(): number {
    return this.config.rows
  }

  get cols(): number {
    return this.config.cols
  }

  get perfectRecall(): boolean {
    return this.config.perfectRecall
  }

  /** Undo the last action. Returns false if at initial state. */
  rewind(): boolean {
    const restored = this.history.rewind()
    if (!restored) return false
    this.restoreSnapshot(restored)
    return true
  }

  /** Reset to the initial state. */
  restart(): void {
    const restored = this.history.restart()
    if (restored) this.restoreSnapshot(restored)
  }

  /** Export the built policy as a JSON-serializable object.
   *  Policy format: { [infoState]: { [action]: probability } }
   *  Matches the Python SinglePlayerTabularPolicy dict format. */
  exportPolicy(): object {
    const entries: Record<string, Record<number, number>> = {}
    for (const [infoState, actionProbs] of this.policy) {
      const actionDict: Record<number, number> = {}
      for (const [a, p] of actionProbs) actionDict[a] = p
      entries[infoState] = actionDict
    }
    return {
      player: this.config.player,
      rows: this.config.rows,
      cols: this.config.cols,
      perfectRecall: this.config.perfectRecall,
      policy: entries,
    }
  }

  // ---------------------------------------------------------------------------
  // Private
  // ---------------------------------------------------------------------------

  /**
   * Parse and validate user input.
   * Returns [actions, probabilities, stackAdditions].
   */
  private parseInput(input: string): [number[], number[], number] {
    const trimmed = input.trim()
    if (trimmed.length === 0) throw new Error('No input given')

    let actions: number[] = []
    let probs: number[] = []
    const tokens = trimmed.split(/\s+/)

    if (tokens[0] === 'r') {
      // Random action
      const legal = this.legalActions
      if (legal.length === 0) throw new Error('No legal actions available')
      const action = legal[Math.floor(Math.random() * legal.length)]
      actions = [action]
      probs = [1.0]
    } else if (tokens[0] === '=') {
      // Equiprobable: "= 0 3 5"
      actions = tokens.slice(1).map((t) => this.parseAction(t))
      probs = actions.map(() => 1.0 / actions.length)
    } else if (tokens.length === 1) {
      // Single action: "3" or "a2"
      actions = [this.parseAction(tokens[0])]
      probs = [1.0]
    } else {
      // Explicit pairs: "0 0.5 3 0.5"
      for (let i = 0; i < tokens.length; i += 2) {
        actions.push(this.parseAction(tokens[i]))
        if (i + 1 >= tokens.length) throw new Error(`Missing probability for action ${tokens[i]}`)
        const p = parseFloat(tokens[i + 1])
        if (isNaN(p)) throw new Error(`Invalid probability: ${tokens[i + 1]}`)
        probs.push(p)
      }
    }

    if (actions.length === 0) throw new Error('No valid actions found')

    // Validate probabilities sum to 1
    const sum = probs.reduce((a, b) => a + b, 0)
    if (Math.abs(sum - 1.0) > 1e-6) {
      throw new Error(`Probabilities sum to ${sum}, expected 1.0`)
    }

    // Validate all actions are legal
    const legal = new Set(this.legalActions)
    for (const a of actions) {
      if (!legal.has(a)) throw new Error(`Action ${a} is not legal`)
    }

    const addition = this._enumerateSuccessors(actions)
    return [actions, probs, addition]
  }

  /** Parse a cell reference: numeric index or alphanumeric (e.g., "a2"). */
  private parseAction(token: string): number {
    // Try numeric first
    const n = parseInt(token, 10)
    if (!isNaN(n) && String(n) === token) return n

    // Alphanumeric: "a1" → col=0, row=0 → cell 0
    if (/^[a-z]\d+$/i.test(token)) {
      const col = token.charCodeAt(0) - 'a'.charCodeAt(0)
      const row = parseInt(token.slice(1), 10) - 1
      return row * this.config.cols + col
    }

    throw new Error(`Invalid action: ${token}`)
  }

  private snapshot(): StrategySnapshot {
    return {
      policy: this.policy,
      currentInfoState: this.currentInfoState,
      actionStack: this.actionStack,
      queuedStates: this.queuedStates,
      targetStackState: this.targetStackState,
      lastCollisionIndex: this.lastCollisionIndex,
    }
  }

  private restoreSnapshot(s: StrategySnapshot): void {
    this.policy = s.policy
    this.currentInfoState = s.currentInfoState
    this.actionStack = s.actionStack
    this.queuedStates = s.queuedStates
    this.targetStackState = s.targetStackState
    this.lastCollisionIndex = s.lastCollisionIndex
  }
}
