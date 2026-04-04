import type { InfoStateOps } from '../engine/InfoStateOps'
import type { Policy, StrategyConfig } from '../strategy/types'
import type { TreeNode } from './types'

const DEFAULT_COLLAPSE_DEPTH = 3

/** Cell index → human-readable label: col letter (a-z) + row (1-based). */
export function cellLabel(index: number, cols: number): string {
  const row = Math.floor(index / cols)
  const col = index % cols
  return String.fromCharCode(97 + col) + (row + 1)
}

/**
 * Build the game tree from a completed policy.
 *
 * Walks the policy starting from the initial info state, creating a TreeNode
 * for each reachable state. Collision-possible actions produce two children:
 * one for placement (stonePlayer = policy player) and one for collision
 * (stonePlayer = opponent, edge marked isCollision).
 *
 * Terminal info states become leaf nodes with no actions.
 * Info states not in the policy (but reachable) also become leaf nodes.
 *
 * Memoizes by info state: when the same info state is reached via multiple
 * paths, subsequent occurrences reuse the first-built node (shared subtree).
 * This prevents exponential blowup from transpositions in realistic policies.
 */
export function buildTree(
  policy: Policy,
  infoOps: InfoStateOps,
  config: StrategyConfig,
): TreeNode {
  let nextId = 0
  const memo = new Map<string, TreeNode>() // infoState → already-built node

  function build(infoState: string, depth: number): TreeNode {
    // Reuse memoized node for transpositions (shared info states via different paths)
    const existing = memo.get(infoState)
    if (existing) return existing

    const id = nextId++
    const boardView = infoOps.boardViewFlat(infoState)
    const isTerminal = infoOps.isTerminal(infoState)
    const actions = policy.get(infoState) ?? []

    const isMissing = !isTerminal && actions.length === 0

    const node: TreeNode = {
      id,
      infoState,
      boardView,
      actions,
      children: [],
      depth,
      isTerminal,
      isMissing,
      x: 0,
      y: 0,
      collapsed: depth >= DEFAULT_COLLAPSE_DEPTH,
      subtreeSize: 0,
    }

    // Memoize before recursing to handle cycles
    memo.set(infoState, node)

    if (isTerminal || actions.length === 0) return node

    const { player, perfectRecall } = config
    const opponent = 1 - player
    const collisionPossible = infoOps.isCollisionPossible(infoState)

    for (const [action, probability] of actions) {
      if (probability <= 0) continue

      // Placement successor (stone belongs to the policy's player)
      const placementState = infoOps.infoStateAfterAction(infoState, action, player, perfectRecall)
      const placementChild = build(placementState, depth + 1)
      node.children.push({
        action,
        probability,
        label: cellLabel(action, config.cols),
        isCollision: false,
        child: placementChild,
      })

      // Collision successor (stone belongs to opponent — reveals opponent stone)
      if (collisionPossible) {
        const collisionState = infoOps.infoStateAfterAction(infoState, action, opponent, perfectRecall)
        // Only add if the collision state differs from placement
        if (collisionState !== placementState) {
          const collisionChild = build(collisionState, depth + 1)
          node.children.push({
            action,
            probability,
            label: cellLabel(action, config.cols),
            isCollision: true,
            child: collisionChild,
          })
        }
      }
    }

    return node
  }

  let initialState = infoOps.initialInfoState(config.player)
  if (config.perfectRecall) initialState += '\n'

  const root = build(initialState, 0)
  computeSubtreeSize(root)
  return root
}

/** Compute subtreeSize for each node (total descendants). */
function computeSubtreeSize(node: TreeNode, visited = new Set<number>()): number {
  if (visited.has(node.id)) {
    node.subtreeSize = 0
    return 0 // already counted in another branch
  }
  visited.add(node.id)

  if (node.children.length === 0) {
    node.subtreeSize = 0
    return 1
  }
  let total = 0
  for (const edge of node.children) {
    total += computeSubtreeSize(edge.child, visited)
  }
  node.subtreeSize = total
  return total + 1
}
