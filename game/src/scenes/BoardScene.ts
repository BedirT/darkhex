import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'
import { BoardLayout3D } from '../board/BoardLayout'
import { HexTile3D } from '../board/HexTile'
import { boardCenter, PALETTE } from '../board/IsometricHex'
import { GameEngine, type Player } from '../engine/GameEngine'
import { createOutlineComposer } from '../postprocess/OutlinePostProcess'

const BOARD_ROWS = 4
const BOARD_COLS = 3
const PLAYER_LABEL: Record<Player, string> = { 0: 'Black', 1: 'White' }

/**
 * Main 3D scene: orthographic camera at isometric angle,
 * warm lighting to match the mauve / board-game aesthetic.
 */
export class BoardScene {
  private renderer: THREE.WebGLRenderer
  private scene: THREE.Scene
  private camera: THREE.OrthographicCamera
  private controls: OrbitControls

  private engine!: GameEngine
  private board!: BoardLayout3D

  private outlineRender!: ReturnType<typeof createOutlineComposer>

  private raycaster = new THREE.Raycaster()
  private pointer = new THREE.Vector2()
  private hoveredTile: HexTile3D | null = null
  private lastMoveIndex: number | null = null

  private clock = new THREE.Clock()

  private statusEl!: HTMLDivElement
  private infoEl!: HTMLDivElement

  constructor(private container: HTMLElement) {
    // ── Renderer ──────────────────────────────────────────────────────────
    this.renderer = new THREE.WebGLRenderer({ antialias: true })
    this.renderer.setPixelRatio(window.devicePixelRatio)
    this.renderer.setSize(container.clientWidth, container.clientHeight)
    this.renderer.setClearColor(PALETTE.bg)
    this.renderer.shadowMap.enabled = true
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap
    this.renderer.toneMapping = THREE.NoToneMapping
    container.appendChild(this.renderer.domElement)

    // ── Scene ─────────────────────────────────────────────────────────────
    this.scene = new THREE.Scene()

    // ── Camera ────────────────────────────────────────────────────────────
    const aspect = container.clientWidth / container.clientHeight
    const frustum = 5.5
    this.camera = new THREE.OrthographicCamera(
      -frustum * aspect, frustum * aspect,
      frustum, -frustum,
      0.1, 100,
    )

    const [cx, , cz] = boardCenter(BOARD_ROWS, BOARD_COLS)
    // Isometric view matching the reference image angle
    const dist = 14
    this.camera.position.set(cx + dist * 0.55, dist * 0.7, cz + dist * 0.55)
    this.camera.lookAt(cx, 0, cz)

    // ── Orbit controls ────────────────────────────────────────────────────
    this.controls = new OrbitControls(this.camera, this.renderer.domElement)
    this.controls.target.set(cx, 0, cz)
    this.controls.enablePan = false
    this.controls.enableZoom = true
    this.controls.minZoom = 0.5
    this.controls.maxZoom = 3
    this.controls.minPolarAngle = 0
    this.controls.maxPolarAngle = Math.PI / 2  // allow fully horizontal
    this.controls.enableDamping = true
    this.controls.dampingFactor = 0.08
    this.controls.update()

    // ── Lights (warm, soft) ───────────────────────────────────────────────
    this._setupLights(cx, cz)

    // ── Post-processing (screen-space outlines) ──────────────────────────
    this.outlineRender = createOutlineComposer(this.renderer, this.scene, this.camera, {
      outlineColor: PALETTE.outline,
      depthThreshold: 2.0,     // only big silhouettes (object IDs handle tile boundaries)
      normalThreshold: 0.45,   // top/side crease edges
      thickness: 2,
    })

    // ── HUD ───────────────────────────────────────────────────────────────
    this._buildHud()

    // ── Events ────────────────────────────────────────────────────────────
    this.renderer.domElement.addEventListener('pointermove', this._onPointerMove)
    this.renderer.domElement.addEventListener('pointerdown', this._onPointerDown)
    window.addEventListener('resize', this._onResize)
    window.addEventListener('keydown', this._onKeyDown)
  }

  async init(): Promise<void> {
    this.engine = await GameEngine.create(BOARD_ROWS, BOARD_COLS)
    this.board = new BoardLayout3D(this.scene, BOARD_ROWS, BOARD_COLS)
    this._updateHud()
    this._animate()
  }

  // ── Render loop ─────────────────────────────────────────────────────────

  private _animate = (): void => {
    requestAnimationFrame(this._animate)
    const dt = this.clock.getDelta()
    this.controls.update()
    if (this.board) this.board.tickAnimations(dt)
    this.outlineRender.render()
  }

  // ── Lights ──────────────────────────────────────────────────────────────

  private _setupLights(cx: number, cz: number): void {
    // Strong neutral ambient — dominates so colors render close to their hex values
    this.scene.add(new THREE.AmbientLight(0xffffff, 1.2))

    // Gentle directional — just enough to create soft shadows and slight shading
    const key = new THREE.DirectionalLight(0xffffff, 0.4)
    key.position.set(cx + 6, 14, cz - 4)
    key.target.position.set(cx, 0, cz)
    key.castShadow = true
    key.shadow.mapSize.setScalar(2048)
    key.shadow.camera.left = -10
    key.shadow.camera.right = 10
    key.shadow.camera.top = 10
    key.shadow.camera.bottom = -10
    key.shadow.camera.near = 1
    key.shadow.camera.far = 40
    key.shadow.bias = -0.002
    this.scene.add(key)
    this.scene.add(key.target)
  }

  // ── Raycaster ─────────────────────────────────────────────────────────

  private _updatePointer(e: PointerEvent): void {
    const rect = this.renderer.domElement.getBoundingClientRect()
    this.pointer.x = ((e.clientX - rect.left) / rect.width) * 2 - 1
    this.pointer.y = -((e.clientY - rect.top) / rect.height) * 2 + 1
  }

  private _raycastTile(): HexTile3D | null {
    if (!this.board) return null
    this.raycaster.setFromCamera(this.pointer, this.camera)
    const hits = this.raycaster.intersectObjects(this.board.meshes, false)
    if (hits.length === 0) return null
    return this.board.tileFromMesh(hits[0].object)
  }

  private _onPointerMove = (e: PointerEvent): void => {
    this._updatePointer(e)
    const tile = this._raycastTile()
    if (tile !== this.hoveredTile) {
      if (this.hoveredTile) this.hoveredTile.setHovered(false)
      if (tile) tile.setHovered(true)
      this.hoveredTile = tile
    }
  }

  private _onPointerDown = (e: PointerEvent): void => {
    this._updatePointer(e)
    const tile = this._raycastTile()
    if (!tile) return
    if (this.engine.isTerminal) return
    const legal = this.engine.legalActions()
    if (!legal.includes(tile.cellIndex)) return

    const result = this.engine.applyAction(tile.cellIndex)
    this.lastMoveIndex = result.placed ? tile.cellIndex : null
    this.board.applyBoard(this.engine.trueBoard(), this.lastMoveIndex)
    this._updateHud()
  }

  private _onKeyDown = (e: KeyboardEvent): void => {
    if (e.key === 'r' || e.key === 'R') this._reset()
  }

  private async _reset(): Promise<void> {
    this.engine = await GameEngine.create(BOARD_ROWS, BOARD_COLS)
    this.lastMoveIndex = null
    this.board.applyBoard(this.engine.trueBoard(), null)
    this._updateHud()
  }

  private _onResize = (): void => {
    const w = this.container.clientWidth
    const h = this.container.clientHeight
    const aspect = w / h
    const frustum = 5.5
    this.camera.left = -frustum * aspect
    this.camera.right = frustum * aspect
    this.camera.top = frustum
    this.camera.bottom = -frustum
    this.camera.updateProjectionMatrix()
    this.renderer.setSize(w, h)
    this.outlineRender.resize(w, h)
  }

  // ── HUD ───────────────────────────────────────────────────────────────

  private _buildHud(): void {
    const hud = document.createElement('div')
    hud.style.cssText = `
      position: absolute; top: 0; left: 0; width: 100%; pointer-events: none;
      font-family: 'Courier New', monospace; color: #4a5568;
      padding: 16px 24px; display: flex; flex-direction: column; gap: 4px;
    `
    this.container.style.position = 'relative'
    this.container.appendChild(hud)

    const title = document.createElement('div')
    title.textContent = 'DSaGe — Dark Hex Strategy Generator'
    title.style.cssText = 'font-size: 13px; color: #64748b;'
    hud.appendChild(title)

    this.statusEl = document.createElement('div')
    this.statusEl.style.cssText = 'font-size: 15px; margin-top: 4px;'
    hud.appendChild(this.statusEl)

    const bottom = document.createElement('div')
    bottom.style.cssText = `
      position: absolute; bottom: 0; left: 0; width: 100%;
      padding: 10px 24px; font-size: 11px; color: #64748b;
      display: flex; justify-content: space-between; pointer-events: none;
    `
    this.container.appendChild(bottom)

    this.infoEl = document.createElement('div')
    bottom.appendChild(this.infoEl)

    const legend = document.createElement('div')
    legend.innerHTML =
      '<span style="color:#5c6bc0">● Black</span> top↔bottom &nbsp; ' +
      '<span style="color:#ef5350">● White</span> left↔right &nbsp; ' +
      '<span style="color:#94a3b8">[R] restart</span>'
    bottom.appendChild(legend)
  }

  private _updateHud(): void {
    if (this.engine.isTerminal) {
      const w = this.engine.winner!
      const color = w === 0 ? '#5c6bc0' : '#ef5350'
      this.statusEl.innerHTML = `<span style="color:${color}">${PLAYER_LABEL[w]} wins!</span>`
      this.infoEl.textContent = ''
    } else {
      const p = this.engine.currentPlayer
      const color = p === 0 ? '#5c6bc0' : '#ef5350'
      const n = this.engine.legalActions().length
      this.statusEl.innerHTML =
        `<span style="color:${color}">${PLAYER_LABEL[p]}'s turn</span>` +
        ` <span style="color:#94a3b8; font-size:13px">(${n} moves)</span>`
      this.infoEl.textContent = `info: ${this.engine.infoStateString(p)}`
    }
  }
}
