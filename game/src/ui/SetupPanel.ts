import type { StrategyConfig } from '../strategy/types'

// ── Shared style tokens ────────────────────────────────────────────────────
const FONT = "'Nunito', -apple-system, BlinkMacSystemFont, sans-serif"
const CREAM = '#faf5ef'
const MAUVE = '#c29797'
const MAUVE_DARK = '#995a5a'
const MAUVE_LIGHT = '#e8d5d5'
const TEXT = '#4a3535'
const TEXT_MUTED = '#8a7070'
const SHADOW = '0 8px 32px rgba(100, 60, 60, 0.18), 0 2px 8px rgba(100, 60, 60, 0.10)'
const RADIUS = '14px'

/**
 * Setup panel for configuring a new strategy generation session.
 * Warm board-game styled modal with visual controls.
 */
export class SetupPanel {
  private overlay: HTMLDivElement
  private resolve: ((config: StrategyConfig | null) => void) | null = null

  constructor(parent: HTMLElement) {
    this.overlay = document.createElement('div')
    this.overlay.style.cssText = `
      display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      background: rgba(180, 160, 140, 0.45); backdrop-filter: blur(4px);
      z-index: 100; align-items: center; justify-content: center;
    `

    const panel = document.createElement('div')
    panel.style.cssText = `
      background: ${CREAM}; border-radius: ${RADIUS}; padding: 32px 36px;
      min-width: 340px; max-width: 400px;
      box-shadow: ${SHADOW}; border: 2px solid ${MAUVE_LIGHT};
      font-family: ${FONT}; color: ${TEXT};
    `
    panel.innerHTML = `
      <h2 style="margin: 0 0 6px; color: ${MAUVE_DARK}; font-size: 22px; font-weight: 800; letter-spacing: -0.3px;">
        New Strategy
      </h2>
      <p style="margin: 0 0 24px; color: ${TEXT_MUTED}; font-size: 14px; font-weight: 400;">
        Configure the board and choose a player to build a strategy for.
      </p>

      <div style="display: flex; gap: 16px; margin-bottom: 20px;">
        <div style="flex: 1;">
          <label style="display: block; font-size: 13px; font-weight: 700; color: ${TEXT_MUTED}; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">
            Rows
          </label>
          <input id="sg-rows" type="number" min="1" max="5" value="2" style="
            width: 100%; padding: 10px 14px; font-size: 18px; font-weight: 700;
            font-family: ${FONT}; color: ${TEXT};
            background: #fff; border: 2px solid ${MAUVE_LIGHT}; border-radius: 10px;
            text-align: center; outline: none; transition: border-color 0.15s;
          " onfocus="this.style.borderColor='${MAUVE}'" onblur="this.style.borderColor='${MAUVE_LIGHT}'">
        </div>
        <div style="display: flex; align-items: flex-end; padding-bottom: 10px; color: ${TEXT_MUTED}; font-size: 20px; font-weight: 700;">
          &times;
        </div>
        <div style="flex: 1;">
          <label style="display: block; font-size: 13px; font-weight: 700; color: ${TEXT_MUTED}; margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.5px;">
            Cols
          </label>
          <input id="sg-cols" type="number" min="1" max="5" value="2" style="
            width: 100%; padding: 10px 14px; font-size: 18px; font-weight: 700;
            font-family: ${FONT}; color: ${TEXT};
            background: #fff; border: 2px solid ${MAUVE_LIGHT}; border-radius: 10px;
            text-align: center; outline: none; transition: border-color 0.15s;
          " onfocus="this.style.borderColor='${MAUVE}'" onblur="this.style.borderColor='${MAUVE_LIGHT}'">
        </div>
      </div>

      <div style="margin-bottom: 20px;">
        <label style="display: block; font-size: 13px; font-weight: 700; color: ${TEXT_MUTED}; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.5px;">
          Player
        </label>
        <div id="sg-player-group" style="display: flex; gap: 10px;">
          <label id="sg-player-black" style="
            flex: 1; display: flex; align-items: center; justify-content: center; gap: 8px;
            padding: 10px 16px; border-radius: 10px; cursor: pointer;
            font-size: 15px; font-weight: 700; transition: all 0.15s;
            background: ${TEXT}; color: #fff; border: 2px solid ${TEXT};
          ">
            <span style="width: 14px; height: 14px; border-radius: 50%; background: #4c556b; display: inline-block; border: 2px solid #373d4d;"></span>
            <input type="radio" name="sg-player" value="0" checked style="display: none;">
            Black
          </label>
          <label id="sg-player-white" style="
            flex: 1; display: flex; align-items: center; justify-content: center; gap: 8px;
            padding: 10px 16px; border-radius: 10px; cursor: pointer;
            font-size: 15px; font-weight: 700; transition: all 0.15s;
            background: #fff; color: ${TEXT}; border: 2px solid ${MAUVE_LIGHT};
          ">
            <span style="width: 14px; height: 14px; border-radius: 50%; background: #d6dae4; display: inline-block; border: 2px solid #b8becf;"></span>
            <input type="radio" name="sg-player" value="1" style="display: none;">
            White
          </label>
        </div>
      </div>

      <label style="
        display: flex; align-items: center; gap: 10px; margin-bottom: 28px;
        cursor: pointer; font-size: 15px; color: ${TEXT}; font-weight: 600;
      ">
        <input type="checkbox" id="sg-recall" style="
          width: 20px; height: 20px; accent-color: ${MAUVE_DARK}; cursor: pointer;
        ">
        Perfect Recall
      </label>

      <div style="display: flex; gap: 10px;">
        <button id="sg-start" style="
          flex: 2; padding: 12px 20px; font-size: 16px; font-weight: 800;
          font-family: ${FONT}; color: #fff; background: ${MAUVE_DARK};
          border: none; border-radius: 10px; cursor: pointer;
          transition: background 0.15s, transform 0.1s;
          box-shadow: 0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2);
        ">
          Start Building
        </button>
        <button id="sg-cancel" style="
          flex: 1; padding: 12px 16px; font-size: 15px; font-weight: 700;
          font-family: ${FONT}; color: ${TEXT_MUTED}; background: #fff;
          border: 2px solid ${MAUVE_LIGHT}; border-radius: 10px; cursor: pointer;
          transition: background 0.15s;
        ">
          Cancel
        </button>
      </div>
    `
    this.overlay.appendChild(panel)
    parent.appendChild(this.overlay)

    // ── Player toggle logic ─────────────────────────────────────────────
    const blackLabel = panel.querySelector('#sg-player-black') as HTMLLabelElement
    const whiteLabel = panel.querySelector('#sg-player-white') as HTMLLabelElement
    const blackRadio = blackLabel.querySelector('input') as HTMLInputElement
    const whiteRadio = whiteLabel.querySelector('input') as HTMLInputElement

    const updatePlayerStyles = () => {
      if (blackRadio.checked) {
        blackLabel.style.background = TEXT
        blackLabel.style.color = '#fff'
        blackLabel.style.borderColor = TEXT
        whiteLabel.style.background = '#fff'
        whiteLabel.style.color = TEXT
        whiteLabel.style.borderColor = MAUVE_LIGHT
      } else {
        whiteLabel.style.background = TEXT
        whiteLabel.style.color = '#fff'
        whiteLabel.style.borderColor = TEXT
        blackLabel.style.background = '#fff'
        blackLabel.style.color = TEXT
        blackLabel.style.borderColor = MAUVE_LIGHT
      }
    }

    blackLabel.addEventListener('click', () => { blackRadio.checked = true; updatePlayerStyles() })
    whiteLabel.addEventListener('click', () => { whiteRadio.checked = true; updatePlayerStyles() })

    // ── Button interactions ──────────────────────────────────────────────
    const startBtn = panel.querySelector('#sg-start') as HTMLButtonElement
    const cancelBtn = panel.querySelector('#sg-cancel') as HTMLButtonElement

    startBtn.addEventListener('mousedown', () => {
      startBtn.style.transform = 'translateY(2px)'
      startBtn.style.boxShadow = `0 1px 0 #7a4040, 0 2px 6px rgba(100, 60, 60, 0.2)`
    })
    startBtn.addEventListener('mouseup', () => {
      startBtn.style.transform = ''
      startBtn.style.boxShadow = `0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2)`
    })
    startBtn.addEventListener('mouseover', () => { startBtn.style.background = '#a84848' })
    startBtn.addEventListener('mouseout', () => {
      startBtn.style.background = MAUVE_DARK
      startBtn.style.transform = ''
      startBtn.style.boxShadow = `0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2)`
    })
    cancelBtn.addEventListener('mouseover', () => { cancelBtn.style.background = MAUVE_LIGHT })
    cancelBtn.addEventListener('mouseout', () => { cancelBtn.style.background = '#fff' })

    startBtn.addEventListener('click', () => this._submit())
    cancelBtn.addEventListener('click', () => this._cancel())
  }

  show(): Promise<StrategyConfig | null> {
    this.overlay.style.display = 'flex'
    return new Promise((resolve) => { this.resolve = resolve })
  }

  hide(): void {
    this.overlay.style.display = 'none'
    this.resolve = null
  }

  private _submit(): void {
    const rows = parseInt((this.overlay.querySelector('#sg-rows') as HTMLInputElement).value, 10)
    const cols = parseInt((this.overlay.querySelector('#sg-cols') as HTMLInputElement).value, 10)
    const playerRadio = this.overlay.querySelector('input[name="sg-player"]:checked') as HTMLInputElement
    const player = parseInt(playerRadio.value, 10)
    const perfectRecall = (this.overlay.querySelector('#sg-recall') as HTMLInputElement).checked

    if (rows < 1 || cols < 1 || rows > 5 || cols > 5) return

    const resolve = this.resolve
    this.hide()
    resolve?.({ rows, cols, player, perfectRecall })
  }

  private _cancel(): void {
    const resolve = this.resolve
    this.hide()
    resolve?.(null)
  }
}
