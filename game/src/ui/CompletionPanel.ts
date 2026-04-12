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

export type CompletionAction = 'download' | 'new' | 'dismiss' | 'investigate'

/**
 * Completion modal shown when a strategy is fully built.
 * Shows stats and offers download / new strategy options.
 */
export class CompletionPanel {
  private overlay: HTMLDivElement
  private statsEl: HTMLDivElement
  private confirmEl: HTMLDivElement
  private downloaded = false
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
    downloadBtn.addEventListener('click', () => {
      this.downloaded = true
      this._resolve('download')
    })
    btnRow.appendChild(downloadBtn)

    const investigateBtn = this._makeBtn('Investigate', GREEN, '#fff', `
      flex: 1; font-weight: 800; border: none;
      box-shadow: 0 3px 0 #3d7a3f, 0 4px 12px rgba(60, 100, 60, 0.2);
    `)
    investigateBtn.addEventListener('mousedown', () => {
      investigateBtn.style.transform = 'translateY(2px)'
      investigateBtn.style.boxShadow = '0 1px 0 #3d7a3f, 0 2px 6px rgba(60, 100, 60, 0.2)'
    })
    investigateBtn.addEventListener('mouseup', () => {
      investigateBtn.style.transform = ''
      investigateBtn.style.boxShadow = '0 3px 0 #3d7a3f, 0 4px 12px rgba(60, 100, 60, 0.2)'
    })
    investigateBtn.addEventListener('mouseover', () => { investigateBtn.style.background = '#4a8a4e' })
    investigateBtn.addEventListener('mouseout', () => {
      investigateBtn.style.background = GREEN
      investigateBtn.style.transform = ''
      investigateBtn.style.boxShadow = '0 3px 0 #3d7a3f, 0 4px 12px rgba(60, 100, 60, 0.2)'
    })
    investigateBtn.addEventListener('click', () => this._resolve('investigate'))
    btnRow.appendChild(investigateBtn)

    const newBtn = this._makeBtn('New Strategy', CREAM, TEXT, `
      flex: 1; border: 2px solid ${MAUVE_LIGHT};
    `)
    newBtn.addEventListener('mouseover', () => { newBtn.style.background = MAUVE_LIGHT })
    newBtn.addEventListener('mouseout', () => { newBtn.style.background = CREAM })
    newBtn.addEventListener('click', () => {
      if (!this.downloaded) {
        this._showConfirm()
      } else {
        this._resolve('new')
      }
    })
    btnRow.appendChild(newBtn)

    panel.appendChild(btnRow)

    // ── Close / dismiss link ──────────────────────────────────────────
    const closeLink = document.createElement('div')
    closeLink.textContent = 'Keep inspecting the board'
    closeLink.style.cssText = `
      margin-top: 16px; font-size: 13px; font-weight: 600;
      color: ${TEXT_MUTED}; cursor: pointer; text-decoration: underline;
      text-underline-offset: 3px;
    `
    closeLink.addEventListener('mouseover', () => { closeLink.style.color = TEXT })
    closeLink.addEventListener('mouseout', () => { closeLink.style.color = TEXT_MUTED })
    closeLink.addEventListener('click', () => this._resolve('dismiss'))
    panel.appendChild(closeLink)

    // ── Confirmation sub-panel (hidden by default) ──────────────────
    this.confirmEl = document.createElement('div')
    this.confirmEl.style.cssText = `
      display: none; margin-top: 20px; padding: 16px 20px;
      background: #fff0e0; border: 2px solid #ffd5a0; border-radius: 10px;
      text-align: center;
    `
    this.confirmEl.innerHTML = `
      <p style="margin: 0 0 12px; font-size: 15px; font-weight: 700; color: #c75000;">
        You haven't downloaded the strategy yet.
      </p>
      <p style="margin: 0 0 16px; font-size: 14px; color: ${TEXT_MUTED};">
        Starting a new strategy will discard your current work.
      </p>
    `

    const confirmBtnRow = document.createElement('div')
    confirmBtnRow.style.cssText = 'display: flex; gap: 10px; justify-content: center;'

    const discardBtn = this._makeBtn('Discard & Start New', '#c75000', '#fff', `
      border: none; font-weight: 800;
    `)
    discardBtn.addEventListener('mouseover', () => { discardBtn.style.background = '#a84400' })
    discardBtn.addEventListener('mouseout', () => { discardBtn.style.background = '#c75000' })
    discardBtn.addEventListener('click', () => this._resolve('new'))
    confirmBtnRow.appendChild(discardBtn)

    const goBackBtn = this._makeBtn('Go Back', CREAM, TEXT, `border: 2px solid ${MAUVE_LIGHT};`)
    goBackBtn.addEventListener('mouseover', () => { goBackBtn.style.background = MAUVE_LIGHT })
    goBackBtn.addEventListener('mouseout', () => { goBackBtn.style.background = CREAM })
    goBackBtn.addEventListener('click', () => { this.confirmEl.style.display = 'none' })
    confirmBtnRow.appendChild(goBackBtn)

    this.confirmEl.appendChild(confirmBtnRow)
    panel.appendChild(this.confirmEl)

    this.overlay.appendChild(panel)
    parent.appendChild(this.overlay)
  }

  /** Reset download tracking (call once when the strategy first completes). */
  resetDownloaded(): void {
    this.downloaded = false
  }

  show(stats: CompletionStats): Promise<CompletionAction> {
    // Don't reset downloaded here — it persists across re-shows
    this.confirmEl.style.display = 'none'
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

  private _showConfirm(): void {
    this.confirmEl.style.display = 'block'
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
