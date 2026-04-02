/** Format 1/n so that n copies sum to exactly 1.00 within display tolerance.
 *  Uses enough decimal places (up to 6) to avoid rounding rejection. */
function eqProb(n: number): string {
  const raw = 1 / n
  // Try increasing precision until n * rounded == 1 within tolerance
  for (let d = 2; d <= 6; d++) {
    const s = raw.toFixed(d)
    const sum = parseFloat(s) * n
    if (Math.abs(sum - 1) <= 0.01) return s
  }
  return raw.toFixed(6)
}

/**
 * Thin bottom toolbar for strategy mode.
 * Shows selected actions with editable probabilities + control buttons.
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
    this.container.style.cssText = `
      display: none; position: fixed; bottom: 0; left: 0; right: 0;
      background: rgba(26,26,46,0.92); border-top: 1px solid #4a4a6a;
      padding: 8px 16px; z-index: 50;
      font-family: 'Courier New', monospace; color: #e0e0e0; font-size: 13px;
      align-items: center; gap: 8px;
    `

    // Selection area (shows selected actions with prob inputs)
    this.selectionEl = document.createElement('div')
    this.selectionEl.style.cssText = 'flex: 1; display: flex; gap: 6px; align-items: center; flex-wrap: wrap; min-height: 32px;'
    this.container.appendChild(this.selectionEl)

    // Sum indicator
    this.sumEl = document.createElement('span')
    this.sumEl.style.cssText = 'font-size: 12px; min-width: 70px; text-align: right; margin-right: 4px;'
    this.container.appendChild(this.sumEl)

    // Buttons
    const buttonsEl = document.createElement('div')
    buttonsEl.style.cssText = 'display: flex; gap: 4px; align-items: center;'

    const btnBase = `
      padding: 6px 10px; border: none; border-radius: 4px; cursor: pointer;
      font-family: inherit; font-size: 12px;
    `

    const equalBtn = this.makeBtn('=', `${btnBase} background: #4a6a5c; color: #fff;`)
    equalBtn.title = 'Set all equal'
    equalBtn.addEventListener('click', () => { this.setAllEqual(); this.onProbInput() })
    buttonsEl.appendChild(equalBtn)

    this.confirmBtn = this.makeBtn('Confirm', `${btnBase} background: #5c6bc0; color: #fff; font-weight: bold;`)
    this.confirmBtn.addEventListener('click', () => this.fireConfirm())
    buttonsEl.appendChild(this.confirmBtn)

    const sep = document.createElement('div')
    sep.style.cssText = 'width: 1px; height: 20px; background: #4a4a6a; margin: 0 4px;'
    buttonsEl.appendChild(sep)

    const rndBtn = this.makeBtn('Rnd', `${btnBase} background: #6a5c4a; color: #fff;`)
    rndBtn.title = 'Random action'
    rndBtn.addEventListener('click', () => this.rndCallbacks.forEach(cb => cb()))
    buttonsEl.appendChild(rndBtn)

    const undoBtn = this.makeBtn('Undo', `${btnBase} background: #4a4a6a; color: #e0e0e0;`)
    undoBtn.addEventListener('click', () => this.undoCallbacks.forEach(cb => cb()))
    buttonsEl.appendChild(undoBtn)

    const restartBtn = this.makeBtn('Rst', `${btnBase} background: #6a4a4a; color: #e0e0e0;`)
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
      hint.style.color = '#666'
      this.selectionEl.appendChild(hint)
      this.sumEl.textContent = ''
      this.confirmBtn.style.opacity = '0.4'
      return
    }

    for (const [cellIdx, _] of selected) {
      const col = cellIdx % cols
      const row = Math.floor(cellIdx / cols)
      const name = String.fromCharCode('a'.charCodeAt(0) + col) + (row + 1)

      const pill = document.createElement('div')
      pill.style.cssText = `
        display: flex; align-items: center; gap: 3px;
        background: #2a3a2a; border: 1px solid #4a6a4a; border-radius: 4px;
        padding: 2px 6px;
      `

      const label = document.createElement('span')
      label.textContent = name
      label.style.cssText = 'color: #81c784; font-weight: bold; font-size: 13px;'
      pill.appendChild(label)

      const input = document.createElement('input')
      input.type = 'text'
      input.value = eqProb(selected.size)
      input.style.cssText = `
        width: 40px; background: #1a2a1a; color: #e0e0e0; border: 1px solid #4a6a4a;
        padding: 2px 4px; font-family: inherit; font-size: 12px; border-radius: 3px;
        text-align: center;
      `
      input.addEventListener('input', () => this.onProbInput())
      pill.appendChild(input)
      this.probInputs.set(cellIdx, input)

      this.selectionEl.appendChild(pill)
    }

    this.onProbInput()
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
  private onProbInput(): void {
    const sum = this.computeSum()
    const valid = Math.abs(sum - 1) <= 0.01
    const remaining = 1 - sum

    if (this.probInputs.size === 0) {
      this.sumEl.textContent = ''
      this.confirmBtn.style.opacity = '0.4'
      return
    }

    // Sum indicator with color
    if (valid) {
      this.sumEl.innerHTML = `<span style="color:#81c784;">&#10003; = 1.00</span>`
      this.confirmBtn.style.opacity = '1'
    } else {
      const color = sum > 1 ? '#ef9a9a' : '#ffcc80'
      const sign = remaining >= 0 ? '+' : ''
      this.sumEl.innerHTML = `<span style="color:${color};">&Sigma; ${sum.toFixed(2)} (${sign}${remaining.toFixed(2)})</span>`
      this.confirmBtn.style.opacity = '0.4'
    }

    // Notify listeners (for board overlay sync)
    const probMap = new Map<number, number>()
    for (const [cellIdx, input] of this.probInputs) {
      const p = parseFloat(input.value)
      probMap.set(cellIdx, isNaN(p) ? 0 : p)
    }
    this.probChangeCallbacks.forEach(cb => cb(probMap))
  }

  private computeSum(): number {
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
    // Normalize to exactly 1
    for (let i = 0; i < probs.length; i++) probs[i] /= sum
    return { actions, probs }
  }

  onConfirm(cb: (actions: number[], probs: number[]) => void): void {
    this.confirmCallbacks.push(cb)
  }

  onUndo(cb: () => void): void {
    this.undoCallbacks.push(cb)
  }

  onRestart(cb: () => void): void {
    this.restartCallbacks.push(cb)
  }

  onRnd(cb: () => void): void {
    this.rndCallbacks.push(cb)
  }

  /** Called when probability inputs change — use to sync board overlays. */
  onProbChange(cb: (probs: Map<number, number>) => void): void {
    this.probChangeCallbacks.push(cb)
  }

  show(): void {
    this.container.style.display = 'flex'
  }

  hide(): void {
    this.container.style.display = 'none'
  }

  private fireConfirm(): void {
    const result = this.getActionProbs()
    if (!result) return // blocked: sum != 1
    this.confirmCallbacks.forEach(cb => cb(result.actions, result.probs))
  }

  private makeBtn(text: string, style: string): HTMLButtonElement {
    const btn = document.createElement('button')
    btn.textContent = text
    btn.style.cssText = style
    return btn
  }
}
