import type { TutorialStep } from './types'
import { flattenSteps } from './TutorialData'
import { TutorialOverlay } from './TutorialOverlay'
import { playChime } from '../audio/SoundEngine'

/**
 * Hooks that BoardScene exposes for the tutorial engine to observe user actions.
 */
export interface TutorialHooks {
  onTileSelected?: (cellIndex: number, totalSelected: number) => void
  onActionConfirmed?: () => void
  onDoubleClickTile?: (cellIndex: number) => void
}

/**
 * Interface for BoardScene methods the tutorial engine needs.
 * Keeps the engine decoupled from the full BoardScene class.
 */
export interface TutorialSceneAdapter {
  /** Get DOM rect for the Three.js canvas. */
  getCanvasRect(): DOMRect
  /** Get DOM rect for the action panel container. */
  getActionPanelRect(): DOMRect | null
  /** Get DOM rect for the info panel container. */
  getInfoPanelRect(): DOMRect | null
  /** Project a 3D tile to screen-space DOMRect. */
  getTileScreenRect(cellIndex: number): DOMRect | null
  /** Programmatically start a strategy session. */
  forceStartStrategy(rows: number, cols: number, player: number): Promise<void>
  /** Auto-complete remaining info states with random actions. */
  autoCompleteStrategy(): void
  /** Check if strategy generator has encountered a collision. */
  hasCollision(): boolean
  /** Check if strategy is complete. */
  isStrategyComplete(): boolean
  /** Get the DOM container element. */
  getContainer(): HTMLElement
  /** Run investigation on current policy. */
  runInvestigation(): Promise<void>
  /** Close investigation view. */
  closeInvestigation(): void
  /** Set the tutorial hooks. */
  setTutorialHooks(hooks: TutorialHooks | null): void
  /** Set the tutorial-active flag (changes Esc behavior). */
  setTutorialActive(active: boolean): void
  /** Enable/disable orbit controls. */
  setOrbitEnabled(enabled: boolean): void
  /** Prefill setup form values. */
  prefillSetup(rows: number, cols: number, player: number): void
  /** Show the setup panel (non-blocking, just makes it visible). */
  showSetupPanel(): void
  /** Hide the setup panel. */
  hideSetupPanel(): void
}

type EngineState = 'idle' | 'showing' | 'waiting' | 'advancing' | 'complete'

/**
 * Tutorial state machine — sequences steps, listens for user actions,
 * and coordinates between the overlay and the scene.
 */
export class TutorialEngine {
  private scene: TutorialSceneAdapter
  private overlay: TutorialOverlay
  private state: EngineState = 'idle'
  private steps: ReturnType<typeof flattenSteps> = []
  private currentIdx = 0
  private _sessionToken = 0
  private _resolve: ((result: 'completed' | 'skipped') => void) | null = null
  private _collisionShown = false
  private _resizeHandler: (() => void) | null = null

  constructor(scene: TutorialSceneAdapter, parent: HTMLElement) {
    this.scene = scene
    this.overlay = new TutorialOverlay(parent)

    this.overlay.onNext(() => this._advance())
    this.overlay.onSkip(() => this.skip())
  }

  /** Start the tutorial. Returns when complete or skipped. */
  async run(): Promise<'completed' | 'skipped'> {
    this.state = 'idle'
    this.currentIdx = 0
    this._collisionShown = false
    this._sessionToken++
    const token = this._sessionToken

    this.steps = flattenSteps()
    this.scene.setTutorialActive(true)
    this.scene.setOrbitEnabled(false)

    // Resize handler — update spotlight position
    this._resizeHandler = () => {
      if (this.state === 'showing' || this.state === 'waiting') {
        const step = this.steps[this.currentIdx]?.step
        if (step) {
          const rect = this._resolveTargetRect(step)
          this.overlay.updateSpotlight(rect)
        }
      }
    }
    window.addEventListener('resize', this._resizeHandler)

    this.overlay.show()

    return new Promise((resolve) => {
      this._resolve = resolve
      this._showCurrentStep(token)
    })
  }

  /** Skip/exit the tutorial immediately. */
  skip(): void {
    this._cleanup()
    this._resolve?.('skipped')
    this._resolve = null
  }

  get isActive(): boolean { return this.state !== 'idle' && this.state !== 'complete' }

  // ── Step display ─────────────────────────────────────────────────────────

  private async _showCurrentStep(token: number): Promise<void> {
    if (token !== this._sessionToken) return
    if (this.currentIdx >= this.steps.length) {
      this._finish()
      return
    }

    const { step } = this.steps[this.currentIdx]
    const totalSteps = this.steps.length

    // Execute setup action if specified
    if (step.setup) {
      await this._executeSetup(step.setup, token)
      if (token !== this._sessionToken) return
    }

    // Skip collision section if no collision encountered yet (will be inserted dynamically)
    if (step.phase === 'collision' && !this._collisionShown) {
      if (!this.scene.hasCollision()) {
        // Skip entire collision section
        while (this.currentIdx < this.steps.length && this.steps[this.currentIdx].step.phase === 'collision') {
          this.currentIdx++
        }
        this._showCurrentStep(token)
        return
      }
      this._collisionShown = true
    }

    this.state = 'showing'
    const rect = this._resolveTargetRect(step)
    this.overlay.showStep(step, this.currentIdx, totalSteps, rect)

    // Set up interaction listeners for non-click-next triggers
    if (step.trigger.type !== 'click-next') {
      this.state = 'waiting'
      this._setupInteractionListener(step, token)
    }
  }

  private _advance(): void {
    if (this.state !== 'showing' && this.state !== 'waiting') return

    // Clean up any pending interaction listener
    this._teardownInteractionListeners()

    this.currentIdx++

    // Dynamic collision insertion: after confirm-action in building phase,
    // check if a collision just happened and we haven't shown collision steps yet
    if (!this._collisionShown && this.scene.hasCollision()) {
      const nextStep = this.steps[this.currentIdx]?.step
      if (nextStep && nextStep.phase !== 'collision') {
        // Find the collision section and jump to it
        const collisionIdx = this.steps.findIndex(s => s.step.phase === 'collision')
        if (collisionIdx !== -1) {
          this._collisionShown = true
          this.currentIdx = collisionIdx
        }
      }
    }

    this.state = 'advancing'
    // Brief pause for transition
    setTimeout(() => {
      this._showCurrentStep(this._sessionToken)
    }, 100)
  }

  private _finish(): void {
    this.state = 'complete'
    this._cleanup()
    this._resolve?.('completed')
    this._resolve = null
  }

  // ── Interaction listeners ──────────────────────────────────────────────

  private _setupInteractionListener(step: TutorialStep, token: number): void {
    const trigger = step.trigger

    if (trigger.type === 'click-target') {
      // Listen for click on the target element
      const el = this._resolveTargetElement(step)
      if (el) {
        const handler = () => {
          el.removeEventListener('click', handler)
          if (token === this._sessionToken) this._advance()
        }
        el.addEventListener('click', handler)
      }
      return
    }

    if (trigger.type === 'tile-select') {
      this.scene.setTutorialHooks({
        onTileSelected: (_cellIndex, totalSelected) => {
          if (token !== this._sessionToken) return
          if (totalSelected >= trigger.count) {
            this.scene.setTutorialHooks(null)
            this._advance()
          }
        },
      })
      return
    }

    if (trigger.type === 'confirm-action') {
      this.scene.setTutorialHooks({
        onActionConfirmed: () => {
          if (token !== this._sessionToken) return
          this.scene.setTutorialHooks(null)
          this._advance()
        },
        // Double-click also confirms (deterministic action)
        onDoubleClickTile: () => {
          if (token !== this._sessionToken) return
          this.scene.setTutorialHooks(null)
          this._advance()
        },
      })

      return
    }

    if (trigger.type === 'double-click-tile') {
      this.scene.setTutorialHooks({
        onDoubleClickTile: () => {
          if (token !== this._sessionToken) return
          this.scene.setTutorialHooks(null)
          this._advance()
        },
      })
      return
    }

    if (trigger.type === 'custom' && trigger.id === 'node-selected') {
      // For node-selected, listen for click events on the investigation overlay
      const checkForClick = () => {
        if (token !== this._sessionToken) return
        // Any click in investigation mode counts as node selection attempt
        // The actual node selection happens in TreeRenderer; we just advance
        setTimeout(() => {
          document.removeEventListener('click', checkForClick, true)
          if (token === this._sessionToken) this._advance()
        }, 300) // Small delay to let TreeRenderer process the click first
      }
      setTimeout(() => {
        document.addEventListener('click', checkForClick, true)
      }, 200)
      return
    }
  }

  private _teardownInteractionListeners(): void {
    this.scene.setTutorialHooks(null)
  }

  // ── Setup actions ──────────────────────────────────────────────────────

  private async _executeSetup(action: string, token: number): Promise<void> {
    switch (action) {
      case 'open-setup':
        this.scene.showSetupPanel()
        this.scene.prefillSetup(2, 2, 0)
        break

      case 'prefill-setup':
        this.scene.prefillSetup(2, 2, 0)
        break

      case 'start-strategy':
        this.scene.hideSetupPanel()
        await this.scene.forceStartStrategy(2, 2, 0)
        await this._delay(300)
        break

      case 'auto-complete':
        this.scene.autoCompleteStrategy()
        // Small delay for visual feedback
        await this._delay(300)
        break

      case 'enter-investigation':
        // Slight delay, then enter investigation
        await this._delay(200)
        if (token !== this._sessionToken) return
        // Run investigation in background — the tutorial overlay stays on top
        this.scene.runInvestigation().catch(() => {})
        await this._delay(500) // Wait for investigation to render
        break

      case 'play-chime':
        playChime()
        break
    }
  }

  // ── Target resolution ──────────────────────────────────────────────────

  private _resolveTargetRect(step: TutorialStep): DOMRect | null {
    const target = step.target

    if (target === '@none') return null

    if (target === '@board') return this.scene.getCanvasRect()

    if (target === '@action-panel') return this.scene.getActionPanelRect()

    if (target === '@info-panel') return this.scene.getInfoPanelRect()

    if (target.startsWith('@tile:')) {
      const idx = parseInt(target.slice(6), 10)
      return this.scene.getTileScreenRect(idx)
    }

    // CSS selector
    const el = document.querySelector(target)
    return el ? el.getBoundingClientRect() : null
  }

  private _resolveTargetElement(step: TutorialStep): HTMLElement | null {
    const target = step.target
    if (target.startsWith('@') || target === '@none') return null
    return document.querySelector(target) as HTMLElement | null
  }

  // ── Helpers ────────────────────────────────────────────────────────────

  private _delay(ms: number): Promise<void> {
    return new Promise(r => setTimeout(r, ms))
  }

  private _cleanup(): void {
    this.state = 'idle'
    this._teardownInteractionListeners()
    this.scene.setTutorialActive(false)
    this.scene.setTutorialHooks(null)
    this.scene.setOrbitEnabled(true)
    this.overlay.hide()
    this.overlay.dispose()
    if (this._resizeHandler) {
      window.removeEventListener('resize', this._resizeHandler)
      this._resizeHandler = null
    }
  }
}
