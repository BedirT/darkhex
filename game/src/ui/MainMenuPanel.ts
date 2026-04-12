import { ensureAudioReady, playThock, playPlace } from '../audio/SoundEngine'

// ── Shared style tokens ────────────────────────────────────────────────────
const FONT = "'Nunito', -apple-system, BlinkMacSystemFont, sans-serif"
const CREAM = '#faf5ef'
const MAUVE_DARK = '#995a5a'
const MAUVE_LIGHT = '#e8d5d5'
const TEXT = '#4a3535'
const TEXT_MUTED = '#8a7070'
const SHADOW = '0 8px 32px rgba(100, 60, 60, 0.18), 0 2px 8px rgba(100, 60, 60, 0.10)'
const RADIUS = '14px'

export type MenuChoice = 'strategy-generator' | 'strategy-investigation' | 'tutorial'

interface MenuItem {
  id: MenuChoice | null
  label: string
  enabled: boolean
}

const MENU_ITEMS: MenuItem[] = [
  { id: 'strategy-generator', label: 'Strategy Generator', enabled: true },
  { id: 'strategy-investigation', label: 'Strategy Investigation', enabled: true },
  { id: null, label: 'Strategy Walker', enabled: false },
  { id: null, label: 'Game Tree Explorer', enabled: false },
  { id: null, label: 'Convergence Plots', enabled: false },
]

/**
 * Main menu screen — first thing the user sees on launch.
 * Lists available modes; only Strategy Generator is active for now.
 */
export class MainMenuPanel {
  private overlay: HTMLDivElement
  private resolve: ((choice: MenuChoice) => void) | null = null

  constructor(parent: HTMLElement) {
    this.overlay = document.createElement('div')
    this.overlay.style.cssText = `
      display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      background: rgba(180, 160, 140, 0.35); backdrop-filter: blur(3px);
      z-index: 100; align-items: center; justify-content: center;
    `

    const panel = document.createElement('div')
    panel.setAttribute('data-tutorial', 'menu-panel')
    panel.style.cssText = `
      background: ${CREAM}; border-radius: ${RADIUS}; padding: 36px 40px;
      min-width: 360px; max-width: 420px;
      box-shadow: ${SHADOW}; border: 2px solid ${MAUVE_LIGHT};
      font-family: ${FONT}; color: ${TEXT};
    `

    // ── Title ──────────────────────────────────────────────────────────
    const title = document.createElement('h1')
    title.textContent = 'DSaGe'
    title.style.cssText = `
      margin: 0 0 4px; color: ${MAUVE_DARK}; font-size: 36px;
      font-weight: 800; letter-spacing: -0.5px;
    `
    panel.appendChild(title)

    const subtitle = document.createElement('p')
    subtitle.textContent = 'Dark Hex Strategy Generator'
    subtitle.style.cssText = `margin: 0 0 24px; color: ${TEXT_MUTED}; font-size: 15px; font-weight: 400;`
    panel.appendChild(subtitle)

    // ── Tutorial button ───────────────────────────────────────────────
    const tutBtn = document.createElement('button')
    tutBtn.setAttribute('data-tutorial', 'btn-tutorial')
    tutBtn.style.cssText = `
      display: flex; flex-direction: column; align-items: flex-start;
      width: 100%; padding: 14px 20px; margin-bottom: 16px;
      font-family: ${FONT}; background: #fff8ee;
      border: 2px dashed ${MAUVE_LIGHT}; border-radius: 10px;
      cursor: pointer; transition: background 0.15s, border-color 0.15s;
    `
    const tutTitle = document.createElement('span')
    tutTitle.style.cssText = `
      font-size: 15px; font-weight: 800; color: ${MAUVE_DARK};
      display: flex; align-items: center; gap: 6px; width: 100%;
      justify-content: space-between;
    `
    tutTitle.innerHTML = 'Take the Guided Tutorial <span style="font-size: 16px; opacity: 0.6;">\u2192</span>'
    tutBtn.appendChild(tutTitle)
    const tutSub = document.createElement('span')
    tutSub.textContent = 'Learn the basics in 5 minutes'
    tutSub.style.cssText = `font-size: 13px; font-weight: 400; color: ${TEXT_MUTED}; margin-top: 2px;`
    tutBtn.appendChild(tutSub)
    tutBtn.addEventListener('mouseover', () => {
      tutBtn.style.background = '#fff0d8'
      tutBtn.style.borderColor = MAUVE_DARK
      playThock()
    })
    tutBtn.addEventListener('mouseout', () => {
      tutBtn.style.background = '#fff8ee'
      tutBtn.style.borderColor = MAUVE_LIGHT
    })
    tutBtn.addEventListener('click', () => {
      ensureAudioReady()
      playPlace()
      const resolve = this.resolve
      this.hide()
      resolve?.('tutorial')
    })
    panel.appendChild(tutBtn)

    // ── Divider ────────────────────────��───────────────────────────────
    const hr = document.createElement('hr')
    hr.style.cssText = `border: none; border-top: 1px solid ${MAUVE_LIGHT}; margin: 0 0 20px;`
    panel.appendChild(hr)

    // ── Menu items ─────────────────────────────────────────────────────
    const list = document.createElement('div')
    list.style.cssText = 'display: flex; flex-direction: column; gap: 10px;'

    for (const item of MENU_ITEMS) {
      const btn = document.createElement('button')
      if (item.id) btn.setAttribute('data-tutorial', `btn-${item.id}`)

      if (item.enabled) {
        btn.style.cssText = `
          display: flex; align-items: center; justify-content: space-between;
          width: 100%; padding: 14px 20px; font-size: 16px; font-weight: 800;
          font-family: ${FONT}; color: #fff; background: ${MAUVE_DARK};
          border: none; border-radius: 10px; cursor: pointer;
          transition: background 0.15s, transform 0.1s;
          box-shadow: 0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2);
        `

        // Chevron
        const chevron = document.createElement('span')
        chevron.textContent = '\u2192'
        chevron.style.cssText = 'font-size: 18px; opacity: 0.7;'
        btn.appendChild(document.createTextNode(item.label))
        btn.appendChild(chevron)

        // Press animation
        btn.addEventListener('mousedown', () => {
          btn.style.transform = 'translateY(2px)'
          btn.style.boxShadow = '0 1px 0 #7a4040, 0 2px 6px rgba(100, 60, 60, 0.2)'
        })
        btn.addEventListener('mouseup', () => {
          btn.style.transform = ''
          btn.style.boxShadow = '0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2)'
        })
        btn.addEventListener('mouseover', () => {
          btn.style.background = '#a84848'
          playThock()
        })
        btn.addEventListener('mouseout', () => {
          btn.style.background = MAUVE_DARK
          btn.style.transform = ''
          btn.style.boxShadow = '0 3px 0 #7a4040, 0 4px 12px rgba(100, 60, 60, 0.2)'
        })
        btn.addEventListener('click', () => {
          ensureAudioReady()
          playPlace()
          const resolve = this.resolve
          this.hide()
          resolve?.(item.id as MenuChoice)
        })
      } else {
        btn.style.cssText = `
          display: flex; align-items: center; justify-content: space-between;
          width: 100%; padding: 14px 20px; font-size: 15px; font-weight: 700;
          font-family: ${FONT}; color: ${TEXT_MUTED}; background: #fff;
          border: 2px solid ${MAUVE_LIGHT}; border-radius: 10px;
          cursor: default; opacity: 0.55;
        `
        btn.appendChild(document.createTextNode(item.label))

        const badge = document.createElement('span')
        badge.textContent = 'COMING SOON'
        badge.style.cssText = `
          font-size: 11px; font-weight: 800; letter-spacing: 0.5px;
          color: ${TEXT_MUTED}; background: ${MAUVE_LIGHT}; padding: 3px 8px;
          border-radius: 4px; text-transform: uppercase;
        `
        btn.appendChild(badge)
      }

      list.appendChild(btn)
    }

    panel.appendChild(list)
    this.overlay.appendChild(panel)
    parent.appendChild(this.overlay)
  }

  show(): Promise<MenuChoice> {
    this.overlay.style.display = 'flex'
    return new Promise((resolve) => { this.resolve = resolve })
  }

  hide(): void {
    this.overlay.style.display = 'none'
    this.resolve = null
  }
}
