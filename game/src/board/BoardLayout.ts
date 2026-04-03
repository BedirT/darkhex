import * as THREE from 'three'
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js'
import { HexTile3D, type TileState, type StoneAnim } from './HexTile'
import { gridToWorld, HEX, PALETTE } from './IsometricHex'
import { toonMat } from './ToonMaterials'

// ── Edge piece neighbor offsets per edge index ────────────────────────────────
// Edge i of a flat-top hex faces the neighbor at EDGE_NEIGHBOR[i].
const EDGE_NEIGHBOR: [number, number][] = [
  [0, 1],   // edge 0 → E
  [-1, 1],  // edge 1 → NW
  [-1, 0],  // edge 2 → NE
  [0, -1],  // edge 3 → W
  [1, -1],  // edge 4 → SE
  [1, 0],   // edge 5 → SW
]

// ── Edge grouping ───────────────────────────────────────────────────────────
// Groups exposed edges of a border hex into consecutive runs by colour.

interface EdgeGroup {
  color: 'black' | 'white'
  edges: number[]
}

function groupExposedEdges(
  r: number, c: number, rows: number, cols: number,
): EdgeGroup[] {
  // Classify each exposed edge by colour
  const byColor = new Map<string, Set<number>>()
  for (let ei = 0; ei < 6; ei++) {
    const [dr, dc] = EDGE_NEIGHBOR[ei]
    const nr = r + dr, nc = c + dc
    if (nr >= 0 && nr < rows && nc >= 0 && nc < cols) continue
    const color = (nr < 0 || nr >= rows) ? 'black' : 'white'
    if (!byColor.has(color)) byColor.set(color, new Set())
    byColor.get(color)!.add(ei)
  }

  // For each colour, find consecutive runs (with 5→0 wrap-around)
  const groups: EdgeGroup[] = []
  for (const [color, edgeSet] of byColor) {
    // Find a gap (index NOT in the set) to start scanning from
    let gap = -1
    for (let i = 0; i < 6; i++) {
      if (!edgeSet.has(i)) { gap = i; break }
    }
    if (gap === -1) {
      // All 6 edges same colour (shouldn't happen on a real board)
      groups.push({ color: color as 'black' | 'white', edges: [0, 1, 2, 3, 4, 5] })
      continue
    }
    let run: number[] = []
    for (let step = 1; step <= 6; step++) {
      const idx = (gap + step) % 6
      if (edgeSet.has(idx)) {
        run.push(idx)
      } else if (run.length > 0) {
        groups.push({ color: color as 'black' | 'white', edges: [...run] })
        run = []
      }
    }
    if (run.length > 0) {
      groups.push({ color: color as 'black' | 'white', edges: [...run] })
    }
  }
  return groups
}

// ── Chevron shape builder ───────────────────────────────────────────────────
// Creates a V-shaped band that follows the hex contour along consecutive edges.
//
// IMPORTANT: rotateX(-π/2) flips Y→-Z, mirroring the hex.  World edge i
// corresponds to shape-space edge (5-i).  We traverse the mapped vertices
// in reverse so the resulting Shape has CCW winding (required by Three.js).

function buildChevronShape(
  worldEdges: number[],
  verts: [number, number][],
  R: number,
  edgeW: number,
): THREE.Shape {
  const scale = (R + edgeW) / R
  const b = worldEdges[worldEdges.length - 1]
  const numVerts = worldEdges.length + 1

  // Shape-space vertices covered by these world edges (forward order)
  const fwd: [number, number][] = []
  for (let i = 0; i < numVerts; i++) {
    fwd.push(verts[((5 - b + i) % 6 + 6) % 6])
  }

  // Inner path: reversed for CCW winding
  const inner = [...fwd].reverse()
  // Outer path: forward order, scaled outward
  const outer = fwd.map(([x, y]) => [x * scale, y * scale] as [number, number])

  const shape = new THREE.Shape()
  shape.moveTo(inner[0][0], inner[0][1])
  for (let i = 1; i < inner.length; i++) {
    shape.lineTo(inner[i][0], inner[i][1])
  }
  for (const [x, y] of outer) {
    shape.lineTo(x, y)
  }
  shape.closePath()
  return shape
}

/**
 * Creates and manages the full 3D hex board:
 *  - Positions all HexTile3D objects in the scene
 *  - Adds coloured 3D chevron edge pieces for win-condition borders
 *  - Builds the cream platform base beneath the board
 */
export class BoardLayout3D {
  readonly rows: number
  readonly cols: number
  readonly tiles: HexTile3D[][]
  readonly meshes: THREE.Mesh[]

  constructor(scene: THREE.Scene, rows: number, cols: number) {
    this.rows = rows
    this.cols = cols
    this.tiles = []
    this.meshes = []

    // Create tiles
    for (let r = 0; r < rows; r++) {
      this.tiles[r] = []
      for (let c = 0; c < cols; c++) {
        const idx = r * cols + c
        const tile = new HexTile3D(r, c, idx)
        const [wx, wz] = gridToWorld(r, c)
        tile.setPosition(wx, 0, wz)
        scene.add(tile.group)
        this.tiles[r][c] = tile
        this.meshes.push(tile.mesh)
      }
    }

    // Win-condition edge pieces (chevron bands along each border)
    this._buildEdgePieces(scene)
  }

  // ── Tile accessors ────────────────────────────────────────────────────────

  tileByIndex(index: number): HexTile3D {
    return this.tiles[Math.floor(index / this.cols)][index % this.cols]
  }

  allTiles(): HexTile3D[] {
    return this.tiles.flat()
  }

  tileFromMesh(mesh: THREE.Object3D): HexTile3D | null {
    let obj: THREE.Object3D | null = mesh
    while (obj) {
      if (obj.userData['tile']) return obj.userData['tile'] as HexTile3D
      obj = obj.parent
    }
    return null
  }

  applyBoard(board: Int8Array, lastIndex: number | null = null): void {
    for (let r = 0; r < this.rows; r++) {
      for (let c = 0; c < this.cols; c++) {
        const idx = r * this.cols + c
        const cell = board[idx]
        const state: TileState = cell === 1 ? 'black' : cell === 2 ? 'white' : 'empty'
        this.tiles[r][c].setState(state, idx === lastIndex)
      }
    }
  }

  /**
   * Render from an imperfect-information view array (strategy mode).
   * Same format as applyBoard but with optional collision highlight.
   */
  applyView(
    view: Int8Array,
    collisionIndex: number | null = null,
    defaultAnim: StoneAnim = 'rise',
    dropOverrides?: Set<number>,
  ): void {
    for (let r = 0; r < this.rows; r++) {
      for (let c = 0; c < this.cols; c++) {
        const idx = r * this.cols + c
        const cell = view[idx]
        const state: TileState = cell === 1 ? 'black' : cell === 2 ? 'white' : 'empty'
        const anim = dropOverrides?.has(idx) ? 'drop' as StoneAnim : defaultAnim
        this.tiles[r][c].setState(state, false, anim)
        if (idx === collisionIndex) {
          this.tiles[r][c].flashCollision()
        }
      }
    }
  }

  tickAnimations(dt: number): void {
    for (const tile of this.allTiles()) {
      tile.tickAnimation(dt)
    }
  }

  // ── Edge pieces ───────────────────────────────────────────────────────────
  //
  // For each border hex, group consecutive exposed edges by colour and create
  // a single chevron (V-shaped band) per group.  This forms a continuous
  // zigzag border along each side of the board.
  //
  //   • Row-border edges (top / bottom of board) → Black
  //   • Col-border edges (left / right of board) → White
  //   • Corner edges where both → Black (first-player corner advantage)

  private _buildEdgePieces(scene: THREE.Scene): void {
    const R = HEX.R
    const edgeW = 0.35
    const edgeH = HEX.HEIGHT

    // Use R + GAP/2 so adjacent inner vertices meet at the cell boundary.
    const innerR = R + HEX.GAP / 2
    const verts: [number, number][] = []
    for (let i = 0; i < 6; i++) {
      const a = (Math.PI / 3) * i
      verts.push([innerR * Math.cos(a), innerR * Math.sin(a)])
    }

    // Collect geometries per colour, then merge into one mesh each.
    // This eliminates outline artifacts at junctions between adjacent chevrons.
    const blackGeos: THREE.BufferGeometry[] = []
    const whiteGeos: THREE.BufferGeometry[] = []
    const geoCache = new Map<string, THREE.ExtrudeGeometry>()

    for (let r = 0; r < this.rows; r++) {
      for (let c = 0; c < this.cols; c++) {
        const groups = groupExposedEdges(r, c, this.rows, this.cols)
        if (groups.length === 0) continue

        const [wx, wz] = gridToWorld(r, c)

        for (const group of groups) {
          const key = group.edges.join(',')
          let baseGeo = geoCache.get(key)
          if (!baseGeo) {
            const shape = buildChevronShape(group.edges, verts, innerR, edgeW)
            baseGeo = new THREE.ExtrudeGeometry(shape, {
              depth: edgeH,
              bevelEnabled: false,
            })
            baseGeo.rotateX(-Math.PI / 2)
            baseGeo.computeVertexNormals()
            geoCache.set(key, baseGeo)
          }

          // Clone and translate to world position for merging
          const geo = baseGeo.clone()
          geo.translate(wx, 0, wz)
          ;(group.color === 'black' ? blackGeos : whiteGeos).push(geo)
        }
      }
    }

    // Merge and add one mesh per colour.
    // useGroups=false: flatten all groups to material index 0 (single material).
    // We use the "top" colour as a single MeshToonMaterial for simplicity.
    for (const [geos, mat] of [
      [blackGeos, toonMat(PALETTE.edgeBlackTop)] as const,
      [whiteGeos, toonMat(PALETTE.edgeWhiteTop)] as const,
    ]) {
      if (geos.length === 0) continue
      // Strip groups so mergeGeometries works cleanly
      for (const g of geos) g.clearGroups()
      const merged = mergeGeometries(geos as THREE.BufferGeometry[])
      if (!merged) continue
      const mesh = new THREE.Mesh(merged, mat)
      mesh.castShadow = true
      mesh.receiveShadow = true
      scene.add(mesh)
    }
  }

}
