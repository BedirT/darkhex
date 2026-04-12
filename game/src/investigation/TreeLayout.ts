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
 *
 * DAG-safe: shared nodes (transpositions) are visited only once.
 * The first path to reach a shared node determines its position;
 * subsequent edges still render correctly by pointing to that position.
 *
 * Returns the total bounding box size.
 */
export function layoutTree(root: TreeNode, config: LayoutConfig): { width: number; height: number } {
  const { nodeWidth, nodeHeight, levelGap, siblingGap } = config

  // Phase 1: compute subtree widths (post-order)
  const widthOf = new Map<number, number>() // node.id → total subtree width

  function computeWidth(node: TreeNode, visited: Set<number>): number {
    // DAG guard: if already visited via another parent, treat as leaf
    if (visited.has(node.id)) {
      if (!widthOf.has(node.id)) widthOf.set(node.id, nodeWidth)
      return widthOf.get(node.id)!
    }
    visited.add(node.id)

    const visibleChildren = getVisibleChildren(node)
    if (visibleChildren.length === 0) {
      widthOf.set(node.id, nodeWidth)
      return nodeWidth
    }

    let total = 0
    for (let i = 0; i < visibleChildren.length; i++) {
      if (i > 0) total += siblingGap
      total += computeWidth(visibleChildren[i], visited)
    }
    // Ensure parent is at least nodeWidth wide
    const w = Math.max(total, nodeWidth)
    widthOf.set(node.id, w)
    return w
  }

  computeWidth(root, new Set())

  // Phase 2: assign positions (pre-order)
  let maxX = 0
  let maxDepth = 0

  function assignPositions(node: TreeNode, leftX: number, visited: Set<number>): void {
    // DAG guard: skip if already positioned
    if (visited.has(node.id)) return
    visited.add(node.id)

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
      assignPositions(visibleChildren[i], childX, visited)
      childX += cw + siblingGap
    }
  }

  assignPositions(root, 0, new Set())

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
 * DAG-safe: each node appears at most once in the result.
 */
export function collectVisibleNodes(root: TreeNode): TreeNode[] {
  const result: TreeNode[] = []
  const visited = new Set<number>()
  function walk(node: TreeNode): void {
    if (visited.has(node.id)) return
    visited.add(node.id)
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
 * DAG-safe: each edge from a non-collapsed parent is included,
 * even if the child node is shared (edges converge to it).
 */
export function collectVisibleEdges(root: TreeNode): Array<{ parent: TreeNode; edge: import('./types').TreeEdge }> {
  const result: Array<{ parent: TreeNode; edge: import('./types').TreeEdge }> = []
  const visited = new Set<number>()
  function walk(node: TreeNode): void {
    if (visited.has(node.id)) return
    visited.add(node.id)
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

/**
 * Compute the maximum depth in the tree (for depth slider range).
 */
export function computeMaxDepth(root: TreeNode): number {
  let maxDepth = 0
  const visited = new Set<number>()
  function walk(node: TreeNode): void {
    if (visited.has(node.id)) return
    visited.add(node.id)
    maxDepth = Math.max(maxDepth, node.depth)
    for (const edge of node.children) {
      walk(edge.child)
    }
  }
  walk(root)
  return maxDepth
}

/**
 * Set collapsed state for all nodes based on a depth threshold.
 * Nodes at depth >= threshold are collapsed; nodes below are expanded.
 */
export function setCollapseDepth(root: TreeNode, threshold: number): void {
  const visited = new Set<number>()
  function walk(node: TreeNode): void {
    if (visited.has(node.id)) return
    visited.add(node.id)
    node.collapsed = node.children.length > 0 && node.depth >= threshold
    for (const edge of node.children) {
      walk(edge.child)
    }
  }
  walk(root)
}
