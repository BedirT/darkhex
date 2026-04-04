import type { ActionProb } from '../strategy/types'

export interface TreeNode {
  id: number
  infoState: string
  boardView: Int8Array
  actions: ActionProb[]
  children: TreeEdge[]
  depth: number
  isTerminal: boolean
  /** True if this non-terminal state has no policy entry (partial checkpoint). */
  isMissing: boolean
  // Layout (set by TreeLayout)
  x: number
  y: number
  // UI state
  collapsed: boolean
  subtreeSize: number
}

export interface TreeEdge {
  action: number
  probability: number
  label: string
  isCollision: boolean
  child: TreeNode
}
