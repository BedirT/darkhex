import type { TutorialStep, TooltipPosition } from './types'

const FONT = "'Nunito', -apple-system, BlinkMacSystemFont, sans-serif"
const CREAM = '#faf5ef'
const MAUVE_DARK = '#995a5a'
const MAUVE_LIGHT = '#e8d5d5'
const TEXT = '#4a3535'
const TEXT_MUTED = '#8a7070'
const SHADOW = '0 8px 32px rgba(100, 60, 60, 0.18), 0 2px 8px rgba(100, 60, 60, 0.10)'
const TRANSITION = 'all 0.25s cubic-bezier(0.22, 1, 0.36, 1)'
const OVERLAY_BG = 'rgba(180, 160, 140, 0.45)'
const TOOLTIP_MAX_W = 380
const TOOLTIP_GAP = 14
const SPOTLIGHT_PAD = 10

/** Render **bold** markers in body text to <strong> tags. */
function renderBody(text: string): string {
  return text.replace(/\*\*(.+?)\*\*/g, '<strong style="color: ' + MAUVE_DARK + '">$1</strong>')
}

/**
 * DOM overlay for the tutorial system.
 * Creates a 4-div spotlight frame with a floating tooltip bubble.
 * z-index 95 (above investigation 90, below modals 100).
 */
export class TutorialOverlay {
  private root: HTMLDivElement
  private top: HTMLDivElement
  private bottom: HTMLDivElement
  private left: HTMLDivElement
  private right: HTMLDivElement
  private tooltip: HTMLDivElement
  private arrow: HTMLDivElement
  private titleEl: HTMLHeadingElement
  private bodyEl: HTMLParagraphElement
  private progressEl: HTMLSpanElement
  private nextBtn: HTMLButtonElement
  private skipBtn: HTMLButtonElement
  private hintEl: HTMLSpanElement

  private _onNext: (() => void) | null = null
  private _onSkip: (() => void) | null = null

  constructor(parent: HTMLElement) {
    // Root container — covers full screen, pointer-events:none so it doesn't block by default
    // z-index 105: must be above all app modals (z-index 100) and investigation (90)
    this.root = document.createElement('div')
    this.root.style.cssText = `
      position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      z-index: 105; pointer-events: none;
    `

    // ── 4-div spotlight frame ──────────────────────────────────────────
    const makePane = (): HTMLDivElement => {
      const d = document.createElement('div')
      d.style.cssText = `
        position: fixed; background: ${OVERLAY_BG}; backdrop-filter: blur(2px);
        pointer-events: auto; transition: ${TRANSITION};
      `
      this.root.appendChild(d)
      return d
    }
    this.top = makePane()
    this.bottom = makePane()
    this.left = makePane()
    this.right = makePane()

    // ── Tooltip bubble ─────────────────────────────────────────────────
    this.tooltip = document.createElement('div')
    this.tooltip.style.cssText = `
      position: fixed; max-width: ${TOOLTIP_MAX_W}px; background: ${CREAM};
      border: 2px solid ${MAUVE_LIGHT}; border-radius: 14px;
      padding: 20px 24px 16px; box-shadow: ${SHADOW};
      font-family: ${FONT}; color: ${TEXT}; pointer-events: auto;
      transition: opacity 0.2s, transform 0.25s cubic-bezier(0.22, 1, 0.36, 1);
      z-index: 106;
    `

    this.titleEl = document.createElement('h3')
    this.titleEl.style.cssText = `
      margin: 0 0 8px; font-size: 18px; font-weight: 800;
      color: ${MAUVE_DARK}; letter-spacing: -0.3px;
    `
    this.tooltip.appendChild(this.titleEl)

    this.bodyEl = document.createElement('p')
    this.bodyEl.style.cssText = `
      margin: 0 0 16px; font-size: 15px; line-height: 1.55;
      color: ${TEXT}; font-weight: 400;
    `
    this.tooltip.appendChild(this.bodyEl)

    // Bottom row: skip — progress — next/hint
    const bottomRow = document.createElement('div')
    bottomRow.style.cssText = `
      display: flex; align-items: center; justify-content: space-between; gap: 8px;
    `

    this.skipBtn = document.createElement('button')
    this.skipBtn.textContent = 'Skip Tutorial'
    this.skipBtn.style.cssText = `
      background: none; border: none; cursor: pointer;
      font-family: ${FONT}; font-size: 13px; font-weight: 600;
      color: ${TEXT_MUTED}; padding: 4px 0;
      transition: color 0.12s;
    `
    this.skipBtn.addEventListener('mouseover', () => { this.skipBtn.style.color = TEXT })
    this.skipBtn.addEventListener('mouseout', () => { this.skipBtn.style.color = TEXT_MUTED })
    this.skipBtn.addEventListener('click', () => this._onSkip?.())
    bottomRow.appendChild(this.skipBtn)

    this.progressEl = document.createElement('span')
    this.progressEl.style.cssText = `
      font-size: 12px; font-weight: 700; color: ${TEXT_MUTED};
      min-width: 40px; text-align: center;
    `
    bottomRow.appendChild(this.progressEl)

    // Next button
    this.nextBtn = document.createElement('button')
    this.nextBtn.textContent = 'Next'
    this.nextBtn.style.cssText = `
      padding: 8px 20px; border-radius: 8px; border: none; cursor: pointer;
      font-family: ${FONT}; font-size: 14px; font-weight: 800;
      background: ${MAUVE_DARK}; color: #fff;
      box-shadow: 0 2px 0 #7a4040;
      transition: filter 0.12s, transform 0.1s;
    `
    this.nextBtn.addEventListener('mousedown', () => {
      this.nextBtn.style.transform = 'translateY(1px)'
      this.nextBtn.style.boxShadow = '0 1px 0 #7a4040'
    })
    this.nextBtn.addEventListener('mouseup', () => {
      this.nextBtn.style.transform = ''
      this.nextBtn.style.boxShadow = '0 2px 0 #7a4040'
    })
    this.nextBtn.addEventListener('mouseover', () => { this.nextBtn.style.filter = 'brightness(0.92)' })
    this.nextBtn.addEventListener('mouseout', () => {
      this.nextBtn.style.filter = ''
      this.nextBtn.style.transform = ''
      this.nextBtn.style.boxShadow = '0 2px 0 #7a4040'
    })
    this.nextBtn.addEventListener('click', () => this._onNext?.())
    bottomRow.appendChild(this.nextBtn)

    // Hint text (replaces next button for interactive steps)
    this.hintEl = document.createElement('span')
    this.hintEl.style.cssText = `
      font-size: 13px; font-weight: 600; font-style: italic;
      color: ${TEXT_MUTED}; display: none;
    `
    bottomRow.appendChild(this.hintEl)

    this.tooltip.appendChild(bottomRow)

    // ── Arrow ──────────────────────────────────────────────────────────
    this.arrow = document.createElement('div')
    this.arrow.style.cssText = `
      position: fixed; width: 0; height: 0; z-index: 106;
      pointer-events: none; transition: ${TRANSITION};
    `
    this.root.appendChild(this.arrow)
    this.root.appendChild(this.tooltip)
    parent.appendChild(this.root)

    this.root.style.display = 'none'
  }

  /** Set callback for the Next button. */
  onNext(cb: () => void): void { this._onNext = cb }
  /** Set callback for the Skip button. */
  onSkip(cb: () => void): void { this._onSkip = cb }

  /** Show the overlay. */
  show(): void { this.root.style.display = '' }
  /** Hide the overlay. */
  hide(): void { this.root.style.display = 'none' }

  /** Display a tutorial step. */
  showStep(step: TutorialStep, globalIdx: number, totalSteps: number, targetRect: DOMRect | null): void {
    this.titleEl.textContent = step.title
    this.bodyEl.innerHTML = renderBody(step.body)
    this.progressEl.textContent = `${globalIdx + 1} / ${totalSteps}`

    const isInteractive = step.trigger.type !== 'click-next' && step.trigger.type !== 'click-target'
    const isClickTarget = step.trigger.type === 'click-target'
    const isLastStep = globalIdx === totalSteps - 1

    // Show next button for click-next steps, hint for interactive
    if (isInteractive) {
      this.nextBtn.style.display = 'none'
      this.hintEl.style.display = ''
      this.hintEl.textContent = this._hintText(step)
    } else {
      this.nextBtn.style.display = ''
      this.hintEl.style.display = 'none'
      this.nextBtn.textContent = isLastStep ? 'Finish' : 'Next'
    }

    // For click-target: show next hidden, but target area is clickable
    if (isClickTarget) {
      this.nextBtn.style.display = 'none'
      this.hintEl.style.display = ''
      this.hintEl.textContent = 'Click the highlighted element'
    }

    if (targetRect && step.target !== '@none') {
      this._positionSpotlight(targetRect)
      this._positionTooltip(targetRect, step.tooltipPosition)
      this.arrow.style.display = ''
    } else {
      // @none — full overlay, centered tooltip
      this._positionFullOverlay()
      this._positionCenteredTooltip()
      this.arrow.style.display = 'none'
    }
  }

  /** Update spotlight position (e.g., on resize). */
  updateSpotlight(targetRect: DOMRect | null): void {
    if (targetRect) {
      this._positionSpotlight(targetRect)
    } else {
      this._positionFullOverlay()
    }
  }

  /** Clean up DOM elements. */
  dispose(): void {
    this.root.remove()
  }

  // ── Private: Spotlight positioning ─────────────────────────────────────

  private _positionSpotlight(rect: DOMRect): void {
    const vw = window.innerWidth
    const vh = window.innerHeight
    const pad = SPOTLIGHT_PAD
    const x = Math.max(0, rect.left - pad)
    const y = Math.max(0, rect.top - pad)
    const w = rect.width + pad * 2
    const h = rect.height + pad * 2

    // Top: full width, from top to spotlight top
    this.top.style.cssText += `; top: 0; left: 0; width: ${vw}px; height: ${y}px;`
    // Bottom: full width, from spotlight bottom to viewport bottom
    this.bottom.style.cssText += `; top: ${y + h}px; left: 0; width: ${vw}px; height: ${vh - y - h}px;`
    // Left: from spotlight top to bottom, left edge to spotlight left
    this.left.style.cssText += `; top: ${y}px; left: 0; width: ${x}px; height: ${h}px;`
    // Right: from spotlight top to bottom, spotlight right to viewport right
    this.right.style.cssText += `; top: ${y}px; left: ${x + w}px; width: ${vw - x - w}px; height: ${h}px;`
  }

  private _positionFullOverlay(): void {
    const vw = window.innerWidth
    const vh = window.innerHeight
    // Make top pane cover everything
    this.top.style.cssText += `; top: 0; left: 0; width: ${vw}px; height: ${vh}px;`
    this.bottom.style.cssText += `; top: 0; left: 0; width: 0; height: 0;`
    this.left.style.cssText += `; top: 0; left: 0; width: 0; height: 0;`
    this.right.style.cssText += `; top: 0; left: 0; width: 0; height: 0;`
  }

  // ── Private: Tooltip positioning ───────────────────────────────────────

  private _positionTooltip(rect: DOMRect, pos: TooltipPosition): void {
    const vw = window.innerWidth
    const vh = window.innerHeight
    const pad = SPOTLIGHT_PAD
    const gap = TOOLTIP_GAP

    // Measure tooltip
    this.tooltip.style.left = '-9999px'
    this.tooltip.style.top = '-9999px'
    const tw = this.tooltip.offsetWidth
    const th = this.tooltip.offsetHeight

    const cx = rect.left + rect.width / 2
    const cy = rect.top + rect.height / 2

    let tx: number, ty: number
    let arrowSide: 'top' | 'bottom' | 'left' | 'right'

    if (pos === 'bottom' || (pos === 'center' && cy < vh / 2)) {
      tx = cx - tw / 2
      ty = rect.bottom + pad + gap
      arrowSide = 'top'
      if (ty + th > vh - 16) { ty = rect.top - pad - gap - th; arrowSide = 'bottom' }
    } else if (pos === 'top') {
      tx = cx - tw / 2
      ty = rect.top - pad - gap - th
      arrowSide = 'bottom'
      if (ty < 16) { ty = rect.bottom + pad + gap; arrowSide = 'top' }
    } else if (pos === 'right') {
      tx = rect.right + pad + gap
      ty = cy - th / 2
      arrowSide = 'left'
      if (tx + tw > vw - 16) { tx = rect.left - pad - gap - tw; arrowSide = 'right' }
    } else {
      // left
      tx = rect.left - pad - gap - tw
      ty = cy - th / 2
      arrowSide = 'right'
      if (tx < 16) { tx = rect.right + pad + gap; arrowSide = 'left' }
    }

    // Clamp to viewport
    tx = Math.max(16, Math.min(vw - tw - 16, tx))
    ty = Math.max(16, Math.min(vh - th - 16, ty))

    this.tooltip.style.left = `${tx}px`
    this.tooltip.style.top = `${ty}px`

    this._positionArrow(arrowSide, tx, ty, tw, th, rect)
  }

  private _positionCenteredTooltip(): void {
    this.tooltip.style.left = '-9999px'
    this.tooltip.style.top = '-9999px'
    const tw = this.tooltip.offsetWidth
    const th = this.tooltip.offsetHeight
    const vw = window.innerWidth
    const vh = window.innerHeight
    this.tooltip.style.left = `${(vw - tw) / 2}px`
    this.tooltip.style.top = `${(vh - th) / 2}px`
  }

  private _positionArrow(
    side: 'top' | 'bottom' | 'left' | 'right',
    tx: number, ty: number, tw: number, th: number,
    spotRect: DOMRect,
  ): void {
    const sz = 8
    const cx = spotRect.left + spotRect.width / 2
    const cy = spotRect.top + spotRect.height / 2

    let ax: number, ay: number, border: string

    if (side === 'top') {
      ax = Math.max(tx + 20, Math.min(tx + tw - 20, cx)) - sz
      ay = ty - sz * 2
      border = `border-left: ${sz}px solid transparent; border-right: ${sz}px solid transparent; border-bottom: ${sz * 2}px solid ${MAUVE_LIGHT};`
    } else if (side === 'bottom') {
      ax = Math.max(tx + 20, Math.min(tx + tw - 20, cx)) - sz
      ay = ty + th
      border = `border-left: ${sz}px solid transparent; border-right: ${sz}px solid transparent; border-top: ${sz * 2}px solid ${MAUVE_LIGHT};`
    } else if (side === 'left') {
      ax = tx - sz * 2
      ay = Math.max(ty + 20, Math.min(ty + th - 20, cy)) - sz
      border = `border-top: ${sz}px solid transparent; border-bottom: ${sz}px solid transparent; border-right: ${sz * 2}px solid ${MAUVE_LIGHT};`
    } else {
      ax = tx + tw
      ay = Math.max(ty + 20, Math.min(ty + th - 20, cy)) - sz
      border = `border-top: ${sz}px solid transparent; border-bottom: ${sz}px solid transparent; border-left: ${sz * 2}px solid ${MAUVE_LIGHT};`
    }

    this.arrow.style.cssText = `
      position: fixed; width: 0; height: 0; z-index: 106;
      pointer-events: none; ${border}
      left: ${ax}px; top: ${ay}px;
    `
  }

  private _hintText(step: TutorialStep): string {
    switch (step.trigger.type) {
      case 'tile-select': return 'Click a tile to continue'
      case 'confirm-action': return 'Select tiles, then Confirm or double-click'
      case 'double-click-tile': return 'Double-click a tile to continue'
      case 'custom': return 'Complete the action to continue'
      default: return ''
    }
  }
}
