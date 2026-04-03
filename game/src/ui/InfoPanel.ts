// ── Shared style tokens ────────────────────────────────────────────────────
const FONT = "'Nunito', -apple-system, BlinkMacSystemFont, sans-serif"
const CREAM = '#faf5ef'
const MAUVE_DARK = '#995a5a'
const MAUVE_LIGHT = '#e8d5d5'
const TEXT = '#4a3535'
const TEXT_MUTED = '#8a7070'
const SHADOW = '0 4px 16px rgba(100, 60, 60, 0.12)'

/**
 * Compact top bar showing strategy progress.
 * Title + progress bar + collision badge in a single row.
 */
export class InfoPanel {
  private container: HTMLDivElement
  private titleEl: HTMLSpanElement
  private barEl: HTMLDivElement
  private barLabel: HTMLSpanElement
  private collisionEl: HTMLSpanElement
  private historyEl: HTMLDivElement

  constructor(parent: HTMLElement) {
    this.container = document.createElement('div')
    this.container.style.cssText = `
      display: none; position: fixed; top: 12px; left: 12px; right: 12px;
      background: ${CREAM}; border: 2px solid ${MAUVE_LIGHT}; border-radius: 12px;
      padding: 12px 20px; z-index: 50;
      font-family: ${FONT}; color: ${TEXT};
      box-shadow: ${SHADOW};
      align-items: center; gap: 16px; flex-wrap: wrap;
    `

    // ── Title ──────────────────────────────────────────────────────────
    this.titleEl = document.createElement('span')
    this.titleEl.style.cssText = 'font-weight: 800; font-size: 16px; white-space: nowrap;'
    this.container.appendChild(this.titleEl)

    // ── Collision badge ────────────────────────────────────────────────
    this.collisionEl = document.createElement('span')
    this.collisionEl.style.cssText = `
      display: none; font-size: 13px; font-weight: 700;
      color: #c75000; background: #fff0e0; padding: 3px 10px;
      border-radius: 6px; border: 1px solid #ffd5a0; white-space: nowrap;
    `
    this.collisionEl.textContent = 'Collision'
    this.container.appendChild(this.collisionEl)

    // ── Progress bar ──────────────────────────────────────────────────
    const barBg = document.createElement('div')
    barBg.style.cssText = `
      flex: 1; height: 10px; background: ${MAUVE_LIGHT};
      border-radius: 5px; overflow: hidden; min-width: 80px;
    `

    this.barEl = document.createElement('div')
    this.barEl.style.cssText = `
      height: 100%; background: ${MAUVE_DARK};
      border-radius: 5px; transition: width 0.3s ease-out;
    `
    this.barEl.style.width = '0%'
    barBg.appendChild(this.barEl)
    this.container.appendChild(barBg)

    // ── Progress label ─────────────────────────────────────────────────
    this.barLabel = document.createElement('span')
    this.barLabel.style.cssText = `
      white-space: nowrap; font-size: 13px; font-weight: 700;
      color: ${TEXT_MUTED};
    `
    this.container.appendChild(this.barLabel)

    // ── Esc hint ───────────────────────────────────────────────────────
    const exitHint = document.createElement('span')
    exitHint.style.cssText = `
      color: ${TEXT_MUTED}; font-size: 12px; font-weight: 600;
      background: #fff; padding: 3px 10px; border-radius: 6px;
      border: 1px solid ${MAUVE_LIGHT}; white-space: nowrap;
    `
    exitHint.textContent = 'Esc = restart'
    this.container.appendChild(exitHint)

    // ── History line (perfect recall only) ─────────────────────────────
    this.historyEl = document.createElement('div')
    this.historyEl.style.cssText = `
      display: none; width: 100%;
      font-size: 13px; font-weight: 600; color: ${TEXT_MUTED};
      padding-top: 6px; border-top: 1px solid ${MAUVE_LIGHT};
      margin-top: 2px; overflow-x: auto; white-space: nowrap;
    `
    this.container.appendChild(this.historyEl)

    parent.appendChild(this.container)
  }

  update(state: {
    infoState: string
    assigned: number
    remaining: number
    player: number
    isCollision: boolean
    perfectRecall: boolean
  }): void {
    const playerName = state.player === 0 ? 'Black' : 'White'
    const playerColor = state.player === 0 ? '#506080' : MAUVE_DARK
    this.titleEl.textContent = `Strategy: ${playerName}`
    this.titleEl.style.color = playerColor

    const total = state.assigned + state.remaining
    const pct = total > 0 ? (state.assigned / total) * 100 : 0
    this.barEl.style.width = `${pct}%`
    this.barLabel.textContent = `${state.assigned} / ${total}`

    this.collisionEl.style.display = state.isCollision ? 'inline' : 'none'

    // Show action history for perfect recall (third line of info state string)
    if (state.perfectRecall) {
      const lines = state.infoState.split('\n')
      const historyLine = lines.length >= 3 ? lines[2].trim() : ''
      if (historyLine) {
        this.historyEl.textContent = `History: ${historyLine}`
        this.historyEl.style.display = 'block'
      } else {
        this.historyEl.textContent = 'History: (start)'
        this.historyEl.style.display = 'block'
      }
    } else {
      this.historyEl.style.display = 'none'
    }
  }

  show(): void {
    this.container.style.display = 'flex'
  }

  hide(): void {
    this.container.style.display = 'none'
  }
}
