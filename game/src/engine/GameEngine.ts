import init, { GameState } from 'darkhex-wasm'

export { GameState }

export type Player = 0 | 1  // 0 = Black, 1 = White

export interface MoveResult {
  placed: boolean   // true = stone placed, false = collision (CDH retry)
}

// Cache WASM init so it only runs once regardless of how many engines are created.
let _initPromise: Promise<void> | null = null

/** Async-init wrapper around the WASM GameState. */
export class GameEngine {
  private state: GameState
  readonly rows: number
  readonly cols: number

  private constructor(state: GameState, rows: number, cols: number) {
    this.state = state
    this.rows = rows
    this.cols = cols
  }

  static async create(rows: number, cols: number): Promise<GameEngine> {
    if (!_initPromise) _initPromise = init().then(() => {})
    await _initPromise
    const state = new GameState(rows, cols)
    return new GameEngine(state, rows, cols)
  }

  get currentPlayer(): Player {
    return this.state.current_player() as Player
  }

  get isTerminal(): boolean {
    return this.state.is_terminal()
  }

  get winner(): Player | null {
    const w = this.state.winner()
    return w === -1 ? null : (w as Player)
  }

  legalActions(): number[] {
    return Array.from(this.state.legal_actions())
  }

  applyAction(cellIndex: number): MoveResult {
    const placed = this.state.apply_action(cellIndex)
    return { placed }
  }

  /** True board: 0=empty, 1=black, 2=white. Length = rows*cols. */
  trueBoard(): Int8Array {
    return this.state.true_board()
  }

  /** Player's view of the board (opponent stones hidden). */
  playerView(player: Player): Int8Array {
    return this.state.player_view(player)
  }

  infoStateString(player: Player, perfectRecall = false): string {
    return perfectRecall
      ? this.state.info_state_string_perfect_recall(player)
      : this.state.info_state_string(player)
  }

  /** Payoffs: [black, white]. Only valid at terminal states. */
  returns(): [number, number] {
    const r = this.state.returns()
    return [r[0], r[1]]
  }

  clone(): GameEngine {
    return new GameEngine(this.state.copy(), this.rows, this.cols)
  }

  /** Cell index from (row, col). */
  cellIndex(row: number, col: number): number {
    return row * this.cols + col
  }

  /** (row, col) from cell index. */
  cellCoords(index: number): [number, number] {
    return [Math.floor(index / this.cols), index % this.cols]
  }
}
