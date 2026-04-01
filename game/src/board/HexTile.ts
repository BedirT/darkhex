import * as THREE from 'three'
import { HEX, hexShape, PALETTE } from './IsometricHex'

export type TileState = 'empty' | 'black' | 'white'

// ── Toon gradient map ───────────────────────────────────────────────────────

let _gradientMap: THREE.DataTexture | null = null
function gradientMap(): THREE.DataTexture {
  if (!_gradientMap) {
    const colors = new Uint8Array([100, 200, 255])
    _gradientMap = new THREE.DataTexture(colors, 3, 1, THREE.RedFormat)
    _gradientMap.needsUpdate = true
    _gradientMap.minFilter = THREE.NearestFilter
    _gradientMap.magFilter = THREE.NearestFilter
  }
  return _gradientMap
}

function toonMat(color: number): THREE.MeshToonMaterial {
  return new THREE.MeshToonMaterial({ color, gradientMap: gradientMap() })
}

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

  private topMat: THREE.MeshToonMaterial
  private sideMat: THREE.MeshToonMaterial
  private stoneGroup: THREE.Group | null = null
  private _baseY = 0

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

  setState(s: TileState, isLast = false): void {
    this._state = s
    this._isLast = isLast
    this._hovered = false
    this._updateColors()
    this._updateStone()
  }

  setHovered(h: boolean): void {
    if (h === this._hovered) return
    if (this._state !== 'empty') return
    this._hovered = h
    this._updateColors()
    this.group.userData['targetY'] = h ? this._baseY + 0.10 : this._baseY
  }

  setPosition(x: number, y: number, z: number): void {
    this.group.position.set(x, y, z)
    this._baseY = y
  }

  tickAnimation(dt: number): void {
    const target = this.group.userData['targetY'] as number | undefined
    if (target === undefined) return
    const cur = this.group.position.y
    const diff = target - cur
    if (Math.abs(diff) < 0.001) {
      this.group.position.y = target
      delete this.group.userData['targetY']
    } else {
      this.group.position.y += diff * Math.min(1, dt * 14)
    }
  }

  private _updateColors(): void {
    if (this._hovered) {
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
