import init, { InfoStateOps as WasmInfoStateOps } from 'darkhex-wasm'

// Reuse the same WASM init cache as GameEngine.
let _initPromise: Promise<void> | null = null

/** TypeScript wrapper for WASM info-state-level operations. */
export class InfoStateOps {
  private inner: WasmInfoStateOps
  readonly rows: number
  readonly cols: number

  private constructor(inner: WasmInfoStateOps, rows: number, cols: number) {
    this.inner = inner
    this.rows = rows
    this.cols = cols
  }

  static async create(rows: number, cols: number): Promise<InfoStateOps> {
    if (!_initPromise) _initPromise = init().then(() => {})
    await _initPromise
    const inner = new WasmInfoStateOps(rows, cols)
    return new InfoStateOps(inner, rows, cols)
  }

  /** Initial info state: "P0\n..\n.." */
  initialInfoState(player: number): string {
    return this.inner.initial_info_state(player)
  }

  /** Legal cell indices (those showing '.' in the view). */
  legalActions(infoState: string): number[] {
    return Array.from(this.inner.legal_actions(infoState))
  }

  /** Can collision occur at this info state? */
  isCollisionPossible(infoState: string): boolean {
    return this.inner.is_collision_possible(infoState)
  }

  /**
   * Compute successor info state after an action.
   * stonePlayer same as info state player → placement.
   * stonePlayer = opponent → collision reveal.
   */
  infoStateAfterAction(
    infoState: string,
    action: number,
    stonePlayer: number,
    perfectRecall: boolean,
  ): string {
    return this.inner.info_state_after_action(infoState, action, stonePlayer, perfectRecall)
  }

  /** Is this info state terminal? */
  isTerminal(infoState: string): boolean {
    return this.inner.is_info_state_terminal(infoState)
  }

  /** Board view as flat array (0=empty, 1=black, 2=white). */
  boardViewFlat(infoState: string): Int8Array {
    return this.inner.board_view_flat(infoState)
  }

  /** Player index from info state string (0 or 1). */
  player(infoState: string): number {
    return this.inner.player(infoState)
  }
}
