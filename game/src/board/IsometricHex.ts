/**
 * Geometry constants for the 3D hex board.
 *
 * Flat-top hexagon orientation.
 * Hex shapes use arc-rounded corners for smooth edges.
 * Colours extracted directly from the reference SVG.
 */
import * as THREE from 'three'

export const HEX = {
  R: 1.0,
  HEIGHT: 0.30,
  GAP: 0.01,
  STONE_R: 0.72,
  STONE_H: 0.22,
  TILE_CORNER: 0.05,        // tile corner rounding (subtle)
  TILE_LIP: 0.0,           // tile top/bottom edge rounding
  TILE_LIP_SEG: 0,          // smooth lip curve
  STONE_CORNER: 0.12,       // stone corner rounding (rounder)
  STONE_LIP: 0.09,          // stone top/bottom edge rounding (plump, pillowy)
  STONE_LIP_SEG: 8,         // very smooth stone lip
  CURVE_SEG: 12,            // segments per rounded corner arc
} as const

export const HEX_W = 2 * HEX.R
export const HEX_H = Math.sqrt(3) * HEX.R

/**
 * Create a flat-top hexagonal Shape with rounded corners.
 *
 * Each corner is replaced by a quadratic Bézier curve that smoothly
 * rounds the vertex. The `cornerR` parameter controls how far the
 * rounding extends along each edge.
 */
export function hexShape(r: number, cornerR = 0): THREE.Shape {
  const shape = new THREE.Shape()
  const verts: [number, number][] = []
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 3) * i
    verts.push([r * Math.cos(angle), r * Math.sin(angle)])
  }

  if (cornerR <= 0.001) {
    // Sharp corners (fallback)
    shape.moveTo(verts[0][0], verts[0][1])
    for (let i = 1; i < 6; i++) shape.lineTo(verts[i][0], verts[i][1])
    shape.closePath()
    return shape
  }

  // For each corner: start the line `cornerR` before the vertex,
  // quadratic-curve through the vertex, end `cornerR` after.
  for (let i = 0; i < 6; i++) {
    const prev = verts[(i + 5) % 6]
    const curr = verts[i]
    const next = verts[(i + 1) % 6]

    // Point on edge (prev→curr), cornerR before curr
    const dx0 = prev[0] - curr[0]
    const dy0 = prev[1] - curr[1]
    const len0 = Math.sqrt(dx0 * dx0 + dy0 * dy0)
    const t0 = cornerR / len0
    const p0x = curr[0] + dx0 * t0
    const p0y = curr[1] + dy0 * t0

    // Point on edge (curr→next), cornerR after curr
    const dx1 = next[0] - curr[0]
    const dy1 = next[1] - curr[1]
    const len1 = Math.sqrt(dx1 * dx1 + dy1 * dy1)
    const t1 = cornerR / len1
    const p1x = curr[0] + dx1 * t1
    const p1y = curr[1] + dy1 * t1

    if (i === 0) {
      shape.moveTo(p0x, p0y)
    } else {
      shape.lineTo(p0x, p0y)
    }
    // Quadratic curve through the original vertex (control point)
    shape.quadraticCurveTo(curr[0], curr[1], p1x, p1y)
  }
  shape.closePath()
  return shape
}

/** Grid (row, col) → world-space (x, z). */
export function gridToWorld(row: number, col: number): [number, number] {
  const s = HEX.R + HEX.GAP / 2
  const x = s * 1.5 * col + s * 1.5 * row
  const z = s * Math.sqrt(3) * (col * 0.5 - row * 0.5)
  return [x, z]
}

/** World-space centre of the entire board. */
export function boardCenter(rows: number, cols: number): [number, number, number] {
  const [x0, z0] = gridToWorld(0, 0)
  const [x1, z1] = gridToWorld(rows - 1, cols - 1)
  return [(x0 + x1) / 2, 0, (z0 + z1) / 2]
}

// ── Palette — warm board-game colours with more saturation ──────────────────

export const PALETTE = {
  bg:          0xb8daf5,       // slightly deeper sky blue

  tileTop:     0xe0a8a0,       // warmer terracotta-rose (was grayish-pink)
  tileSide:    0xc48878,       // richer clay side
  tileDark:    0xa05848,       // deeper terracotta shadow

  hoverTop:    0xecc0b8,       // warm highlight on hover

  blackTop:    0x3a4460,       // richer navy-slate
  blackSide:   0x2a3148,       // deeper navy side
  blackEdge:   0x0c0e14,

  whiteTop:    0xf0ece4,       // warm ivory (was cold gray-blue)
  whiteSide:   0xd8d0c4,       // warm linen side
  whiteHi:     0x8a8070,

  edgeBlackTop:  0x3a4460,
  edgeBlackSide: 0x2a3148,
  edgeWhiteTop:  0xf0ece4,
  edgeWhiteSide: 0xd8d0c4,

  lastTop:     0xd49888,       // warm recent-move highlight
  selectedTop: 0x7cc47e,       // fresh green — selected tile
  selectedSide:0x5ca860,

  platform:    0xfaf5ef,       // cream platform (matches UI)
  platformEdge:0xd5c8b8,      // warm edge

  outline:     0x3a2820,       // warm dark brown outlines (was cold gray)
} as const
