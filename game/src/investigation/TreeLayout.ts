import type { TreeNode } from './types'

export interface LayoutConfig {
  nodeWidth: number
  nodeHeight: number
  levelGap: number
  siblingGap: number
}

export const DEFAULT_LAYOUT: LayoutConfig = {
  nodeWidth: 80,
  nodeHeight: 80,
  levelGap: 60,
  siblingGap: 20,
}

/**
 * Assign (x, y) positions to all visible (non-collapsed) nodes.
 *
 * Uses a simplified layered tree layout:
 * - Post-order: compute subtree widths bottom-up
 * - Pre-order: assign x positions left-to-right, centering parents over children
 * - Y = depth * (nodeHeight + levelGap)
 *
 * Collapsed nodes are treated as leaves (their children are not laid out).
 * Returns the total bounding box size.
 */
export function layoutTree(root: TreeNode, config: LayoutConfig): { width: number; height: number } {
  const { nodeWidth, nodeHeight, levelGap, siblingGap } = config

  // Phase 1: compute subtree widths (post-order)
  const widthOf = new Map<number, number>() // node.id → total subtree width

  function computeWidth(node: TreeNode): number {
    const visibleChildren = getVisibleChildren(node)
    if (visibleChildren.length === 0) {
      widthOf.set(node.id, nodeWidth)
      return nodeWidth
    }

    let total = 0
    for (let i = 0; i < visibleChildren.length; i++) {
      if (i > 0) total += siblingGap
      total += computeWidth(visibleChildren[i])
    }
    // Ensure parent is at least nodeWidth wide
    const w = Math.max(total, nodeWidth)
    widthOf.set(node.id, w)
    return w
  }

  computeWidth(root)

  // Phase 2: assign positions (pre-order)
  let maxX = 0
  let maxDepth = 0

  function assignPositions(node: TreeNode, leftX: number): void {
    const w = widthOf.get(node.id)!
    node.x = leftX + w / 2
    node.y = node.depth * (nodeHeight + levelGap)
    maxX = Math.max(maxX, node.x + nodeWidth / 2)
    maxDepth = Math.max(maxDepth, node.depth)

    const visibleChildren = getVisibleChildren(node)
    if (visibleChildren.length === 0) return

    // Distribute children left-to-right within our subtree width
    let childX = leftX
    // If total children width < parent width, center the children block
    let totalChildWidth = 0
    for (let i = 0; i < visibleChildren.length; i++) {
      if (i > 0) totalChildWidth += siblingGap
      totalChildWidth += widthOf.get(visibleChildren[i].id)!
    }
    if (totalChildWidth < w) {
      childX += (w - totalChildWidth) / 2
    }

    for (let i = 0; i < visibleChildren.length; i++) {
      const cw = widthOf.get(visibleChildren[i].id)!
      assignPositions(visibleChildren[i], childX)
      childX += cw + siblingGap
    }
  }

  assignPositions(root, 0)

  return {
    width: maxX + nodeWidth / 2,
    height: (maxDepth + 1) * nodeHeight + maxDepth * levelGap,
  }
}

/** Get visible children of a node (skip collapsed subtrees). */
function getVisibleChildren(node: TreeNode): TreeNode[] {
  if (node.collapsed || node.children.length === 0) return []
  return node.children.map((edge) => edge.child)
}

/**
 * Collect all visible nodes (for rendering).
 * A node is visible if it is the root or its parent is not collapsed.
 */
export function collectVisibleNodes(root: TreeNode): TreeNode[] {
  const result: TreeNode[] = []
  function walk(node: TreeNode): void {
    result.push(node)
    if (!node.collapsed) {
      for (const edge of node.children) {
        walk(edge.child)
      }
    }
  }
  walk(root)
  return result
}

/**
 * Collect all visible edges (for rendering).
 * An edge is visible if its parent node is not collapsed.
 */
export function collectVisibleEdges(root: TreeNode): Array<{ parent: TreeNode; edge: import('./types').TreeEdge }> {
  const result: Array<{ parent: TreeNode; edge: import('./types').TreeEdge }> = []
  function walk(node: TreeNode): void {
    if (!node.collapsed) {
      for (const edge of node.children) {
        result.push({ parent: node, edge })
        walk(edge.child)
      }
    }
  }
  walk(root)
  return result
}
