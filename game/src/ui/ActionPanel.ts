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
const SHADOW = '0 -4px 16px rgba(100, 60, 60, 0.10)'

/** Format 1/n so that n copies sum to exactly 1.00 within display tolerance. */
function eqProb(n: number): string {
  const raw = 1 / n
  for (let d = 2; d <= 6; d++) {
    const s = raw.toFixed(d)
    const sum = parseFloat(s) * n
    if (Math.abs(sum - 1) <= 0.01) return s
  }
  return raw.toFixed(6)
}

/**
 * Bottom toolbar for strategy mode.
 * Warm parchment card with action pills, probability editing, and controls.
 */
export class ActionPanel {
  private container: HTMLDivElement
  private selectionEl: HTMLDivElement
  private sumEl: HTMLSpanElement
  private confirmBtn: HTMLButtonElement
  private confirmCallbacks: Array<(actions: number[], probs: number[]) => void> = []
  private undoCallbacks: Array<() => void> = []
  private restartCallbacks: Array<() => void> = []
  private rndCallbacks: Array<() => void> = []
  private probChangeCallbacks: Array<(probs: Map<number, number>) => void> = []
  private probInputs: Map<number, HTMLInputElement> = new Map()

  constructor(parent: HTMLElement) {
    this.container = document.createElement('div')
    this.container.setAttribute('data-tutorial', 'action-panel')
    this.container.style.cssText = `
      display: none; position: fixed; bottom: 12px; left: 12px; right: 12px;
      background: ${CREAM}; border: 2px solid ${MAUVE_LIGHT}; border-radius: 12px;
      padding: 14px 18px; z-index: 50;
      font-family: ${FONT}; color: ${TEXT}; font-size: 15px;
      box-shadow: ${SHADOW};
      align-items: center; gap: 12px;
    `

    // ── Selection area ──────────────────────────────────────────────────
    this.selectionEl = document.createElement('div')
    this.selectionEl.style.cssText = `
      flex: 1; display: flex; gap: 8px; align-items: center;
      flex-wrap: wrap; min-height: 40px;
    `
    this.container.appendChild(this.selectionEl)

    // ── Sum indicator ───────────────────────────────────────────────────
    this.sumEl = document.createElement('span')
    this.sumEl.style.cssText = `
      font-size: 14px; font-weight: 700; min-width: 90px;
      text-align: right; margin-right: 4px;
    `
    this.container.appendChild(this.sumEl)

    // ── Buttons ─────────────────────────────────────────────────────────
    const buttonsEl = document.createElement('div')
    buttonsEl.style.cssText = 'display: flex; gap: 6px; align-items: center;'

    const equalBtn = this._makeBtn('Equal', CREAM, TEXT, `border: 2px solid ${MAUVE_LIGHT}`)
    equalBtn.title = 'Set all probabilities equal'
    equalBtn.addEventListener('click', () => { this.setAllEqual(); this._onProbInput() })
    buttonsEl.appendChild(equalBtn)

    this.confirmBtn = this._makeBtn('Confirm', MAUVE_DARK, '#fff', `
      border: none; font-weight: 800;
      box-shadow: 0 2px 0 #7a4040;
    `)
    this.confirmBtn.addEventListener('click', () => this._fireConfirm())
    this.confirmBtn.addEventListener('mousedown', () => {
      this.confirmBtn.style.transform = 'translateY(1px)'
      this.confirmBtn.style.boxShadow = '0 1px 0 #7a4040'
    })
    this.confirmBtn.addEventListener('mouseup', () => {
      this.confirmBtn.style.transform = ''
      this.confirmBtn.style.boxShadow = '0 2px 0 #7a4040'
    })
    buttonsEl.appendChild(this.confirmBtn)

    const sep = document.createElement('div')
    sep.style.cssText = `width: 2px; height: 24px; background: ${MAUVE_LIGHT}; margin: 0 4px; border-radius: 1px;`
    buttonsEl.appendChild(sep)

    const rndBtn = this._makeBtn('Random', CREAM, TEXT, `border: 2px solid ${MAUVE_LIGHT}`)
    rndBtn.title = 'Assign a random action'
    rndBtn.addEventListener('click', () => this.rndCallbacks.forEach(cb => cb()))
    buttonsEl.appendChild(rndBtn)

    const undoBtn = this._makeBtn('Undo', CREAM, TEXT, `border: 2px solid ${MAUVE_LIGHT}`)
    undoBtn.addEventListener('click', () => this.undoCallbacks.forEach(cb => cb()))
    buttonsEl.appendChild(undoBtn)

    const restartBtn = this._makeBtn('Restart', '#fff0e0', '#c75000', `border: 2px solid #ffd5a0`)
    restartBtn.addEventListener('click', () => this.restartCallbacks.forEach(cb => cb()))
    buttonsEl.appendChild(restartBtn)

    this.container.appendChild(buttonsEl)
    parent.appendChild(this.container)
  }

  /** Update the selection display with current selected tiles. */
  updateSelection(selected: Map<number, number>, cols: number): void {
    this.selectionEl.innerHTML = ''
    this.probInputs.clear()

    if (selected.size === 0) {
      const hint = document.createElement('span')
      hint.textContent = 'Click tiles to select actions'
      hint.style.cssText = `color: ${TEXT_MUTED}; font-size: 15px; font-weight: 600; font-style: italic;`
      this.selectionEl.appendChild(hint)
      this.sumEl.textContent = ''
      this.confirmBtn.style.opacity = '0.35'
      this.confirmBtn.style.pointerEvents = 'none'
      return
    }

    for (const [cellIdx, _] of selected) {
      const col = cellIdx % cols
      const row = Math.floor(cellIdx / cols)
      const name = String.fromCharCode('a'.charCodeAt(0) + col) + (row + 1)

      const pill = document.createElement('div')
      pill.style.cssText = `
        display: flex; align-items: center; gap: 6px;
        background: ${GREEN_LIGHT}; border: 2px solid ${GREEN_BORDER};
        border-radius: 8px; padding: 4px 10px;
      `

      const label = document.createElement('span')
      label.textContent = name
      label.style.cssText = `color: ${GREEN}; font-weight: 800; font-size: 15px;`
      pill.appendChild(label)

      const input = document.createElement('input')
      input.type = 'text'
      input.value = eqProb(selected.size)
      input.style.cssText = `
        width: 56px; background: #fff; color: ${TEXT}; border: 2px solid ${GREEN_BORDER};
        padding: 4px 6px; font-family: ${FONT}; font-size: 15px; font-weight: 700;
        border-radius: 6px; text-align: center; outline: none;
        transition: border-color 0.15s;
      `
      input.addEventListener('focus', () => { input.style.borderColor = GREEN })
      input.addEventListener('blur', () => { input.style.borderColor = GREEN_BORDER })
      input.addEventListener('input', () => this._onProbInput())
      pill.appendChild(input)
      this.probInputs.set(cellIdx, input)

      this.selectionEl.appendChild(pill)
    }

    this._onProbInput()
  }

  /** Set all probability inputs to equal values. */
  setAllEqual(): void {
    const n = this.probInputs.size
    if (n === 0) return
    const eq = eqProb(n)
    for (const input of this.probInputs.values()) {
      input.value = eq
    }
  }

  /** Compute sum, update indicator, enable/disable confirm, notify board overlays. */
  private _onProbInput(): void {
    const sum = this._computeSum()
    const valid = Math.abs(sum - 1) <= 0.01
    const remaining = 1 - sum

    if (this.probInputs.size === 0) {
      this.sumEl.textContent = ''
      this.confirmBtn.style.opacity = '0.35'
      this.confirmBtn.style.pointerEvents = 'none'
      return
    }

    if (valid) {
      this.sumEl.innerHTML = `<span style="color: ${GREEN}; font-size: 15px;">&#10003; = 1.00</span>`
      this.confirmBtn.style.opacity = '1'
      this.confirmBtn.style.pointerEvents = ''
    } else {
      const color = sum > 1 ? '#c75000' : '#b8860b'
      const sign = remaining >= 0 ? '+' : ''
      this.sumEl.innerHTML = `<span style="color: ${color};">&Sigma; ${sum.toFixed(2)} <span style="font-size:13px;">(${sign}${remaining.toFixed(2)})</span></span>`
      this.confirmBtn.style.opacity = '0.35'
      this.confirmBtn.style.pointerEvents = 'none'
    }

    // Notify listeners (for board overlay sync)
    const probMap = new Map<number, number>()
    for (const [cellIdx, input] of this.probInputs) {
      const p = parseFloat(input.value)
      probMap.set(cellIdx, isNaN(p) ? 0 : p)
    }
    this.probChangeCallbacks.forEach(cb => cb(probMap))
  }

  private _computeSum(): number {
    let sum = 0
    for (const input of this.probInputs.values()) {
      const p = parseFloat(input.value)
      if (!isNaN(p)) sum += p
    }
    return sum
  }

  /** Read current action-probability pairs from the UI. Returns null if invalid. */
  getActionProbs(): { actions: number[]; probs: number[] } | null {
    const actions: number[] = []
    const probs: number[] = []
    for (const [cellIdx, input] of this.probInputs) {
      const p = parseFloat(input.value)
      if (isNaN(p) || p < 0) return null
      actions.push(cellIdx)
      probs.push(p)
    }
    const sum = probs.reduce((a, b) => a + b, 0)
    if (Math.abs(sum - 1) > 0.01) return null
    for (let i = 0; i < probs.length; i++) probs[i] /= sum
    return { actions, probs }
  }

  onConfirm(cb: (actions: number[], probs: number[]) => void): void { this.confirmCallbacks.push(cb) }
  onUndo(cb: () => void): void { this.undoCallbacks.push(cb) }
  onRestart(cb: () => void): void { this.restartCallbacks.push(cb) }
  onRnd(cb: () => void): void { this.rndCallbacks.push(cb) }
  onProbChange(cb: (probs: Map<number, number>) => void): void { this.probChangeCallbacks.push(cb) }

  show(): void { this.container.style.display = 'flex' }
  hide(): void { this.container.style.display = 'none' }

  private _fireConfirm(): void {
    const result = this.getActionProbs()
    if (!result) return
    this.confirmCallbacks.forEach(cb => cb(result.actions, result.probs))
  }

  private _makeBtn(text: string, bg: string, fg: string, extra = ''): HTMLButtonElement {
    const btn = document.createElement('button')
    btn.textContent = text
    btn.style.cssText = `
      padding: 8px 14px; border-radius: 8px; cursor: pointer;
      font-family: ${FONT}; font-size: 14px; font-weight: 700;
      background: ${bg}; color: ${fg};
      transition: filter 0.12s, transform 0.1s;
      ${extra}
    `
    btn.addEventListener('mouseover', () => { btn.style.filter = 'brightness(0.94)' })
    btn.addEventListener('mouseout', () => { btn.style.filter = ''; btn.style.transform = '' })
    return btn
  }
}
