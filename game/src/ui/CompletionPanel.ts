// ── Shared style tokens ────────────────────────────────────────────────────
const FONT = "'Nunito', -apple-system, BlinkMacSystemFont, sans-serif"
const CREAM = '#faf5ef'
const MAUVE_DARK = '#995a5a'
const MAUVE_LIGHT = '#e8d5d5'
const TEXT = '#4a3535'
const TEXT_MUTED = '#8a7070'
const GREEN = '#5a9a5e'
const GREEN_LIGHT = '#e8f5e9'
const GREEN_BORDER = '#b5d8b7'
const SHADOW = '0 8px 32px rgba(100, 60, 60, 0.18), 0 2px 8px rgba(100, 60, 60, 0.10)'

export interface CompletionStats {
  player: number
  rows: number
  cols: number
  infoStates: number
}

export type CompletionAction = 'download' | 'new' | 'dismiss'

/**
 * Completion modal shown when a strategy is fully built.
 * Shows stats and offers download / new strategy options.
 */
export class CompletionPanel {
  private overlay: HTMLDivElement
  private statsEl: HTMLDivElement
  private resolve: ((action: CompletionAction) => void) | null = null

  constructor(parent: HTMLElement) {
    this.overlay = document.createElement('div')
    this.overlay.style.cssText = `
      display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      background: rgba(180, 160, 140, 0.45); backdrop-filter: blur(4px);
      z-index: 100; align-items: center; justify-content: center;
    `

    const panel = document.createElement('div')
    panel.style.cssText = `
      background: ${CREAM}; border-radius: 14px; padding: 36px 40px;
      min-width: 360px; max-width: 440px; text-align: center;
      box-shadow: ${SHADOW}; border: 2px solid ${GREEN_BORDER};
      font-family: ${FONT}; color: ${TEXT};
    `

    // ── Title ──────────────────────────────────────────────────────────
    const title = document.createElement('h2')
    title.textContent = 'Strategy Complete!'
    title.style.cssText = `
      margin: 0 0 8px; font-size: 24px; font-weight: 800;
      color: ${GREEN}; letter-spacing: -0.3px;
    `
    panel.appendChild(title)

    const subtitle = document.createElement('p')
    subtitle.textContent = 'All information states have been assigned.'
    subtitle.style.cssText = `margin: 0 0 24px; color: ${TEXT_MUTED}; font-size: 15px;`
    panel.appendChild(subtitle)

    // ── Stats ──────────────────────────────────────────────────────────
    this.statsEl = document.createElement('div')
    this.statsEl.style.cssText = `
      display: flex; justify-content: center; gap: 24px;
      margin-bottom: 28px; padding: 16px 20px;
      background: ${GREEN_LIGHT}; border: 1px solid ${GREEN_BORDER};
      border-radius: 10px;
    `
    panel.appendChild(this.statsEl)

    // ── Buttons ────────────────────────────────────────────────────────
    const btnRow = document.createElement('div')
    btnRow.style.cssText = 'display: flex; gap: 10px;'

    const downloadBtn = this._makeBtn('Download JSON', MAUVE_DARK, '#fff', `
      flex: 2; font-weight: 800;
      box-shadow: 0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2);
    `)
    downloadBtn.addEventListener('mousedown', () => {
      downloadBtn.style.transform = 'translateY(2px)'
      downloadBtn.style.boxShadow = '0 1px 0 #7a4040, 0 2px 6px rgba(100, 60, 60, 0.2)'
    })
    downloadBtn.addEventListener('mouseup', () => {
      downloadBtn.style.transform = ''
      downloadBtn.style.boxShadow = '0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2)'
    })
    downloadBtn.addEventListener('mouseover', () => { downloadBtn.style.background = '#a84848' })
    downloadBtn.addEventListener('mouseout', () => {
      downloadBtn.style.background = MAUVE_DARK
      downloadBtn.style.transform = ''
      downloadBtn.style.boxShadow = '0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2)'
    })
    downloadBtn.addEventListener('click', () => this._resolve('download'))
    btnRow.appendChild(downloadBtn)

    const newBtn = this._makeBtn('New Strategy', CREAM, TEXT, `
      flex: 1; border: 2px solid ${MAUVE_LIGHT};
    `)
    newBtn.addEventListener('mouseover', () => { newBtn.style.background = MAUVE_LIGHT })
    newBtn.addEventListener('mouseout', () => { newBtn.style.background = CREAM })
    newBtn.addEventListener('click', () => this._resolve('new'))
    btnRow.appendChild(newBtn)

    panel.appendChild(btnRow)

    this.overlay.appendChild(panel)
    parent.appendChild(this.overlay)
  }

  show(stats: CompletionStats): Promise<CompletionAction> {
    // Update stats display
    const playerName = stats.player === 0 ? 'Black' : 'White'
    const playerColor = stats.player === 0 ? '#506080' : MAUVE_DARK
    this.statsEl.innerHTML = `
      <div style="text-align: center;">
        <div style="font-size: 22px; font-weight: 800; color: ${playerColor};">${playerName}</div>
        <div style="font-size: 12px; color: ${TEXT_MUTED}; font-weight: 600;">Player</div>
      </div>
      <div style="text-align: center;">
        <div style="font-size: 22px; font-weight: 800; color: ${TEXT};">${stats.rows}x${stats.cols}</div>
        <div style="font-size: 12px; color: ${TEXT_MUTED}; font-weight: 600;">Board</div>
      </div>
      <div style="text-align: center;">
        <div style="font-size: 22px; font-weight: 800; color: ${GREEN};">${stats.infoStates}</div>
        <div style="font-size: 12px; color: ${TEXT_MUTED}; font-weight: 600;">Info States</div>
      </div>
    `

    this.overlay.style.display = 'flex'
    return new Promise((resolve) => { this.resolve = resolve })
  }

  hide(): void {
    this.overlay.style.display = 'none'
    this.resolve = null
  }

  private _resolve(action: CompletionAction): void {
    const resolve = this.resolve
    this.hide()
    resolve?.(action)
  }

  private _makeBtn(text: string, bg: string, fg: string, extra = ''): HTMLButtonElement {
    const btn = document.createElement('button')
    btn.textContent = text
    btn.style.cssText = `
      padding: 12px 20px; border-radius: 10px; cursor: pointer;
      font-family: ${FONT}; font-size: 15px; font-weight: 700;
      background: ${bg}; color: ${fg}; border: none;
      transition: filter 0.12s, transform 0.1s;
      ${extra}
    `
    return btn
  }
}
