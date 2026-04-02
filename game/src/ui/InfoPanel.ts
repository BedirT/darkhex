/**
 * Top panel showing strategy generator progress and current info state.
 */
export class InfoPanel {
  private container: HTMLDivElement
  private titleEl: HTMLSpanElement
  private progressEl: HTMLDivElement
  private barEl: HTMLDivElement
  private stateEl: HTMLPreElement
  private collisionEl: HTMLSpanElement

  constructor(parent: HTMLElement) {
    this.container = document.createElement('div')
    this.container.style.cssText = `
      display: none; position: fixed; top: 0; left: 0; right: 0;
      background: rgba(26,26,46,0.92); border-bottom: 1px solid #4a4a6a;
      padding: 10px 16px; z-index: 50;
      font-family: 'Courier New', monospace; color: #e0e0e0; font-size: 13px;
    `

    // Title row
    const titleRow = document.createElement('div')
    titleRow.style.cssText = 'display: flex; align-items: center; gap: 12px; margin-bottom: 6px;'

    this.titleEl = document.createElement('span')
    this.titleEl.style.fontWeight = 'bold'
    titleRow.appendChild(this.titleEl)

    this.collisionEl = document.createElement('span')
    this.collisionEl.style.cssText = 'color: #ef9a9a; display: none;'
    this.collisionEl.textContent = '⚡ Collision'
    titleRow.appendChild(this.collisionEl)

    const exitHint = document.createElement('span')
    exitHint.style.cssText = 'margin-left: auto; color: #666; font-size: 11px;'
    exitHint.textContent = '[Esc] exit'
    titleRow.appendChild(exitHint)

    this.container.appendChild(titleRow)

    // Progress bar
    this.progressEl = document.createElement('div')
    this.progressEl.style.cssText = 'margin-bottom: 6px; display: flex; align-items: center; gap: 8px;'

    const barBg = document.createElement('div')
    barBg.style.cssText = 'flex: 1; height: 6px; background: #2a2a40; border-radius: 3px; overflow: hidden;'

    this.barEl = document.createElement('div')
    this.barEl.style.cssText = 'height: 100%; background: #5c6bc0; border-radius: 3px; transition: width 0.2s;'
    this.barEl.style.width = '0%'
    barBg.appendChild(this.barEl)

    this.progressEl.appendChild(barBg)
    this.container.appendChild(this.progressEl)

    // Current info state display
    this.stateEl = document.createElement('pre')
    this.stateEl.style.cssText = `
      margin: 0; padding: 6px 8px; background: #2a2a40; border-radius: 4px;
      font-size: 12px; overflow-x: auto; white-space: pre; color: #b8becf;
      max-height: 60px;
    `
    this.container.appendChild(this.stateEl)

    parent.appendChild(this.container)
  }

  update(state: {
    infoState: string
    assigned: number
    remaining: number
    player: number
    isCollision: boolean
  }): void {
    const playerName = state.player === 0 ? 'Black' : 'White'
    const playerColor = state.player === 0 ? '#4c556b' : '#d6dae4'
    this.titleEl.textContent = `Strategy: ${playerName}`
    this.titleEl.style.color = playerColor

    const total = state.assigned + state.remaining
    const pct = total > 0 ? (state.assigned / total) * 100 : 0
    this.barEl.style.width = `${pct}%`
    this.progressEl.querySelector('span')?.remove()
    const label = document.createElement('span')
    label.style.cssText = 'white-space: nowrap; font-size: 11px; color: #888;'
    label.textContent = `${state.assigned} / ${total} info states`
    this.progressEl.appendChild(label)

    this.stateEl.textContent = state.infoState

    this.collisionEl.style.display = state.isCollision ? 'inline' : 'none'
  }

  show(): void {
    this.container.style.display = 'block'
  }

  hide(): void {
    this.container.style.display = 'none'
  }
}
