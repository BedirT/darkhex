import * as THREE from 'three'
import { HEX, hexShape, PALETTE } from './IsometricHex'
import { toonMat } from './ToonMaterials'

export type TileState = 'empty' | 'black' | 'white'

/** How a newly placed stone should animate in. */
export type StoneAnim = 'drop' | 'rise' | 'none'

// ── Shared geometries ───────────────────────────────────────────────────────

let _tileGeo: THREE.ExtrudeGeometry | null = null
function tileGeometry(): THREE.ExtrudeGeometry {
  if (!_tileGeo) {
    const shape = hexShape(HEX.R - HEX.GAP / 2 - HEX.TILE_LIP, HEX.TILE_CORNER)
    _tileGeo = new THREE.ExtrudeGeometry(shape, {
      depth: HEX.HEIGHT,
      bevelEnabled: true,
      bevelThickness: HEX.TILE_LIP,
      bevelSize: HEX.TILE_LIP,
      bevelSegments: HEX.TILE_LIP_SEG,
      curveSegments: HEX.CURVE_SEG,
    })
    _tileGeo.rotateX(-Math.PI / 2)
    _tileGeo.computeVertexNormals()
  }
  return _tileGeo
}

let _stoneGeo: THREE.ExtrudeGeometry | null = null
function stoneGeometry(): THREE.ExtrudeGeometry {
  if (!_stoneGeo) {
    const shape = hexShape(HEX.STONE_R - HEX.STONE_LIP, HEX.STONE_CORNER)
    _stoneGeo = new THREE.ExtrudeGeometry(shape, {
      depth: HEX.STONE_H,
      bevelEnabled: true,
      bevelThickness: HEX.STONE_LIP,
      bevelSize: HEX.STONE_LIP,
      bevelSegments: HEX.STONE_LIP_SEG,
      curveSegments: HEX.CURVE_SEG,
    })
    _stoneGeo.rotateX(-Math.PI / 2)
    _stoneGeo.computeVertexNormals()
  }
  return _stoneGeo
}

// ── Helpers ─────────────────────────────────────────────────────────────────

function disposeMaterials(mat: THREE.Material | THREE.Material[]): void {
  if (Array.isArray(mat)) mat.forEach(m => m.dispose())
  else mat.dispose()
}

// ── HexTile3D ───────────────────────────────────────────────────────────────

export class HexTile3D {
  readonly row: number
  readonly col: number
  readonly cellIndex: number

  readonly mesh: THREE.Mesh
  readonly group: THREE.Group

  private _state: TileState = 'empty'
  private _hovered = false
  private _isLast = false
  private _selected = false

  private topMat: THREE.MeshToonMaterial
  private sideMat: THREE.MeshToonMaterial
  private stoneGroup: THREE.Group | null = null
  private _baseY = 0
  private _collisionTimer = 0
  private _stoneDropTimer = 0        // > 0 while drop/animation is active
  private _stoneRestY = 0            // final resting Y of the stone group
  private _stoneAnim: StoneAnim = 'drop'
  private _dropLanded = false        // true once landing triggers
  private _flipTimer = -1            // > 0 while tile flip is active
  private _flipPivot: THREE.Group | null = null  // pivot for flip rotation

  /** Called when the stone first hits the board (at the bounce point). */
  onDropLand: (() => void) | null = null

  constructor(row: number, col: number, cellIndex: number) {
    this.row = row
    this.col = col
    this.cellIndex = cellIndex

    this.sideMat = toonMat(PALETTE.tileSide)
    this.topMat = toonMat(PALETTE.tileTop)

    this.mesh = new THREE.Mesh(tileGeometry(), [this.sideMat, this.topMat])
    this.mesh.castShadow = true
    this.mesh.receiveShadow = true
    this.mesh.userData['tile'] = this

    this.group = new THREE.Group()
    this.group.add(this.mesh)
  }

  get state(): TileState { return this._state }

  get selected(): boolean { return this._selected }

  get stoneAnim(): StoneAnim { return this._stoneAnim }

  setState(s: TileState, isLast = false, anim: StoneAnim = 'drop'): void {
    const changed = s !== this._state
    this._state = s
    this._isLast = isLast
    this._hovered = false
    this._selected = false
    // Reset tile to base height (e.g. if it was hovered/selected when stone placed)
    this.group.userData['targetY'] = this._baseY
    this._updateColors()
    if (changed) this._updateStone(anim)
  }

  setSelected(s: boolean): void {
    if (this._state !== 'empty') return
    this._selected = s
    this._updateColors()
    // Raise selected tiles slightly
    this.group.userData['targetY'] = s ? this._baseY + 0.06 : this._baseY
  }

  setHovered(h: boolean): void {
    if (h === this._hovered) return
    if (this._state !== 'empty') return
    this._hovered = h
    this._updateColors()
    if (!this._selected) {
      this.group.userData['targetY'] = h ? this._baseY + 0.10 : this._baseY
    }
  }

  setPosition(x: number, y: number, z: number): void {
    this.group.position.set(x, y, z)
    this._baseY = y
  }

  /** Trigger a collision flash animation (amber flash for ~0.4s). */
  flashCollision(): void {
    this._collisionTimer = 0.4
  }

  tickAnimation(dt: number): void {
    // Hover / position animation
    const target = this.group.userData['targetY'] as number | undefined
    if (target !== undefined) {
      const cur = this.group.position.y
      const diff = target - cur
      if (Math.abs(diff) < 0.001) {
        this.group.position.y = target
        delete this.group.userData['targetY']
      } else {
        this.group.position.y += diff * Math.min(1, dt * 14)
      }
    }

    // Stone placement animation (drop from above or rise from below)
    if (this._stoneDropTimer >= 0 && this.stoneGroup) {
      this._stoneDropTimer += dt

      if (this._stoneAnim === 'drop') {
        // ── Drop from above with bounce ────────────────────────────────
        const dropH = 1.2
        const dur = 0.35
        const t = Math.min(this._stoneDropTimer / dur, 1)

        // Fire landing sound at first impact
        if (t >= 0.5 && !this._dropLanded) {
          this._dropLanded = true
          this.onDropLand?.()
        }

        let y: number
        if (t < 0.5) {
          const ft = t / 0.5
          y = this._stoneRestY + dropH * (1 - ft * ft)
        } else if (t < 0.75) {
          const bt = (t - 0.5) / 0.25
          y = this._stoneRestY + dropH * 0.12 * Math.sin(bt * Math.PI)
        } else {
          const bt = (t - 0.75) / 0.25
          y = this._stoneRestY + dropH * 0.03 * Math.sin(bt * Math.PI)
        }
        this.stoneGroup.position.y = y

        if (t >= 1) {
          this.stoneGroup.position.y = this._stoneRestY
          this._stoneDropTimer = -1
        }

      }
    }

    // Tile flip animation (rise/reveal)
    if (this._flipTimer >= 0 && this._flipPivot) {
      this._flipTimer += dt
      const dur = 0.9
      const t = Math.min(this._flipTimer / dur, 1)

      // Ease-out quart — fast start, smooth settle
      const ease = 1 - (1 - t) ** 4

      // Rotate 0 → PI around X axis (stone swings from below to above)
      this._flipPivot.rotation.x = Math.PI * ease

      // Fire sound at 90°
      if (ease >= 0.5 && !this._dropLanded) {
        this._dropLanded = true
        this.onDropLand?.()
      }

      if (t >= 1) {
        // Flip done — reparent back to group at normal positions
        this._flipPivot.remove(this.mesh)
        this.mesh.position.y = 0
        this.group.add(this.mesh)

        if (this.stoneGroup) {
          this._flipPivot.remove(this.stoneGroup)
          this.stoneGroup.position.set(0, this._stoneRestY, 0)
          this.group.add(this.stoneGroup)
        }

        this.group.remove(this._flipPivot)
        this._flipPivot = null
        this._flipTimer = -1
      }
    }

    // Collision flash
    if (this._collisionTimer > 0) {
      this._collisionTimer -= dt
      // Amber flash that fades out
      const t = Math.max(0, this._collisionTimer / 0.4)
      const r = Math.floor(0xd5 + (0xff - 0xd5) * t)
      const g = Math.floor(0xb7 + (0x8c - 0xb7) * t)
      const b = Math.floor(0xb7 + (0x00 - 0xb7) * t)
      this.topMat.color.setRGB(r / 255, g / 255, b / 255)
      if (this._collisionTimer <= 0) {
        this._updateColors()
      }
    }
  }

  private _updateColors(): void {
    if (this._selected) {
      this.topMat.color.setHex(PALETTE.selectedTop)
      this.sideMat.color.setHex(PALETTE.selectedSide)
    } else if (this._hovered) {
      this.topMat.color.setHex(PALETTE.hoverTop)
      this.sideMat.color.setHex(PALETTE.tileSide)
    } else if (this._isLast) {
      this.topMat.color.setHex(PALETTE.lastTop)
      this.sideMat.color.setHex(PALETTE.tileSide)
    } else {
      this.topMat.color.setHex(PALETTE.tileTop)
      this.sideMat.color.setHex(PALETTE.tileSide)
    }
  }

  private _updateStone(anim: StoneAnim = 'drop'): void {
    // Clean up any in-progress flip
    if (this._flipPivot) {
      this._flipPivot.remove(this.mesh)
      if (this.stoneGroup) this._flipPivot.remove(this.stoneGroup)
      this.group.remove(this._flipPivot)
      this.group.add(this.mesh)
      this._flipPivot = null
      this._flipTimer = -1
    }

    if (this.stoneGroup) {
      this.stoneGroup.traverse(obj => {
        if (obj instanceof THREE.Mesh) {
          disposeMaterials(obj.material)
        }
      })
      this.group.remove(this.stoneGroup)
      this.stoneGroup = null
    }

    if (this._state === 'empty') return

    const isBlack = this._state === 'black'
    const topColor = isBlack ? PALETTE.blackTop : PALETTE.whiteTop
    const sideColor = isBlack ? PALETTE.blackSide : PALETTE.whiteSide

    const stoneMesh = new THREE.Mesh(stoneGeometry(), [toonMat(sideColor), toonMat(topColor)])
    stoneMesh.castShadow = true
    stoneMesh.receiveShadow = true

    this.stoneGroup = new THREE.Group()
    this.stoneGroup.add(stoneMesh)

    // Final resting Y within the tile group
    this._stoneRestY = HEX.STONE_H + HEX.STONE_LIP + HEX.TILE_LIP + 0.08

    if (anim === 'drop') {
      // ── Drop from above — player's action ────────────────────────────
      this.stoneGroup.position.set(0, this._stoneRestY + 1.2, 0)
      this.stoneGroup.scale.setScalar(1)
      this._stoneDropTimer = 0.001
      this._stoneAnim = 'drop'
      this._dropLanded = false
      this.group.add(this.stoneGroup)

    } else if (anim === 'rise') {
      // ── Tile flip — stone attached underneath, both rotate together ──
      //
      // How it works:
      //   1. Stone is placed at -stoneRestY (below tile) in the pivot
      //   2. Pivot starts at rotation.x = 0 (stone is underneath, hidden)
      //   3. Pivot rotates from 0 → PI around X axis
      //   4. At PI, local -Y maps to world +Y = stone ends up on top
      //   5. Tile is upside-down but looks identical (symmetric hex prism)
      //
      this._stoneAnim = 'rise'
      this._dropLanded = false

      this.stoneGroup.scale.setScalar(1)

      // Pivot at the vertical center of the tile so it rotates in place.
      const pivotY = HEX.HEIGHT / 2
      this._flipPivot = new THREE.Group()
      this._flipPivot.position.y = pivotY

      this.group.remove(this.mesh)
      this.mesh.position.y = -pivotY
      // Stone underneath: top of stone must be at or below tile bottom (-pivotY).
      // Stone mesh top is at pos + STONE_H + STONE_LIP relative to stoneGroup origin.
      // So pos = -pivotY - (STONE_H + STONE_LIP)
      this.stoneGroup.position.set(0, -pivotY - HEX.STONE_H - HEX.STONE_LIP, 0)

      this._flipPivot.add(this.mesh)
      this._flipPivot.add(this.stoneGroup)
      this._flipPivot.rotation.x = 0
      this.group.add(this._flipPivot)

      this._flipTimer = 0.001
      this._stoneDropTimer = -1

    } else {
      // ── No animation — snap to position ──────────────────────────────
      this.stoneGroup.position.set(0, this._stoneRestY, 0)
      this.stoneGroup.scale.setScalar(1)
      this._stoneDropTimer = -1
      this.group.add(this.stoneGroup)
    }
    // NOTE: each branch adds stoneGroup to the correct parent.
    // Do NOT add this.group.add(this.stoneGroup) here.
  }
}
