/** Which part of the app this step targets. */
export type TutorialPhase =
  | 'concepts'
  | 'main-menu'
  | 'setup'
  | 'info-states'
  | 'building'
  | 'collision'
  | 'shortcuts'
  | 'completion'
  | 'investigation'
  | 'wrapup'

/** How the user advances past this step. */
export type StepTrigger =
  | { type: 'click-next' }
  | { type: 'click-target' }
  | { type: 'tile-select'; count: number }
  | { type: 'confirm-action' }
  | { type: 'double-click-tile' }
  | { type: 'custom'; id: string }

/** Where to position the tooltip relative to the spotlight. */
export type TooltipPosition = 'top' | 'bottom' | 'left' | 'right' | 'center'

/** A single tutorial step. */
export interface TutorialStep {
  id: string
  phase: TutorialPhase
  /** CSS selector OR special token for the spotlight target.
   *  '@board'       — the Three.js canvas
   *  '@action-panel'— bottom toolbar container
   *  '@info-panel'  — top bar container
   *  '@tile:N'      — project 3D tile N to screen rect
   *  '@none'        — no spotlight (centered message)
   */
  target: string
  title: string
  body: string
  tooltipPosition: TooltipPosition
  trigger: StepTrigger
  /** If set, engine executes this named setup action before showing the step. */
  setup?: string
}

/** A section groups steps under a heading for the progress indicator. */
export interface TutorialSection {
  id: string
  title: string
  steps: TutorialStep[]
}
