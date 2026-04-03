import * as THREE from 'three'
import { HEX, hexShape, PALETTE } from './IsometricHex'
import { toonMat } from './ToonMaterials'

export type TileState = 'empty' | 'black' | 'white'

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

  setState(s: TileState, isLast = false): void {
    this._state = s
    this._isLast = isLast
    this._hovered = false
    this._selected = false
    // Reset tile to base height (e.g. if it was hovered/selected when stone placed)
    this.group.userData['targetY'] = this._baseY
    this._updateColors()
    this._updateStone()
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

  private _updateStone(): void {
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
    this.stoneGroup.position.set(0, HEX.STONE_H + HEX.STONE_LIP + HEX.TILE_LIP + 0.08, 0)
    this.group.add(this.stoneGroup)
  }
}
