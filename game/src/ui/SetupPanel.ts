import type { StrategyConfig } from '../strategy/types'

/**
 * Setup panel for configuring a new strategy generation session.
 * Shows a modal overlay with board size, player, and perfect-recall options.
 */
export class SetupPanel {
  private container: HTMLDivElement
  private resolve: ((config: StrategyConfig) => void) | null = null

  constructor(parent: HTMLElement) {
    this.container = document.createElement('div')
    this.container.style.cssText = `
      display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      background: rgba(0,0,0,0.7); z-index: 100;
      font-family: 'Courier New', monospace; color: #e0e0e0;
      align-items: center; justify-content: center;
    `

    const panel = document.createElement('div')
    panel.style.cssText = `
      background: #1a1a2e; border: 2px solid #4a4a6a; border-radius: 8px;
      padding: 24px; min-width: 280px;
    `
    panel.innerHTML = `
      <h2 style="margin:0 0 16px; color:#b8becf; font-size:16px;">Strategy Generator</h2>
      <label style="display:block; margin-bottom:8px;">
        Rows: <input id="sg-rows" type="number" min="1" max="5" value="2"
          style="width:48px; background:#2a2a40; color:#e0e0e0; border:1px solid #4a4a6a; padding:4px; font-family:inherit;">
      </label>
      <label style="display:block; margin-bottom:8px;">
        Cols: <input id="sg-cols" type="number" min="1" max="5" value="2"
          style="width:48px; background:#2a2a40; color:#e0e0e0; border:1px solid #4a4a6a; padding:4px; font-family:inherit;">
      </label>
      <div style="margin-bottom:8px;">
        Player:
        <label style="margin-left:8px;"><input type="radio" name="sg-player" value="0" checked> Black</label>
        <label style="margin-left:8px;"><input type="radio" name="sg-player" value="1"> White</label>
      </div>
      <label style="display:block; margin-bottom:16px;">
        <input type="checkbox" id="sg-recall"> Perfect Recall
      </label>
      <div style="display:flex; gap:8px;">
        <button id="sg-start" style="flex:1; padding:8px; background:#5c6bc0; color:#fff; border:none; border-radius:4px; cursor:pointer; font-family:inherit; font-size:13px;">
          Start
        </button>
        <button id="sg-cancel" style="flex:1; padding:8px; background:#4a4a6a; color:#e0e0e0; border:none; border-radius:4px; cursor:pointer; font-family:inherit; font-size:13px;">
          Cancel
        </button>
      </div>
    `
    this.container.appendChild(panel)
    parent.appendChild(this.container)

    panel.querySelector('#sg-start')!.addEventListener('click', () => this.submit())
    panel.querySelector('#sg-cancel')!.addEventListener('click', () => this.cancel())
  }

  /** Show the setup panel. Resolves with config when Start is clicked. */
  show(): Promise<StrategyConfig> {
    this.container.style.display = 'flex'
    return new Promise((resolve) => {
      this.resolve = resolve
    })
  }

  hide(): void {
    this.container.style.display = 'none'
    this.resolve = null
  }

  private submit(): void {
    const rows = parseInt((this.container.querySelector('#sg-rows') as HTMLInputElement).value, 10)
    const cols = parseInt((this.container.querySelector('#sg-cols') as HTMLInputElement).value, 10)
    const playerRadio = this.container.querySelector('input[name="sg-player"]:checked') as HTMLInputElement
    const player = parseInt(playerRadio.value, 10)
    const perfectRecall = (this.container.querySelector('#sg-recall') as HTMLInputElement).checked

    if (rows < 1 || cols < 1 || rows > 5 || cols > 5) return

    const resolve = this.resolve
    this.hide()
    resolve?.({ rows, cols, player, perfectRecall })
  }

  private cancel(): void {
    this.hide()
  }
}
