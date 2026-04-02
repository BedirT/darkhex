/**
 * Screen-space outline post-processing.
 *
 * Four render passes:
 *   1. Normal pre-pass   → MeshNormalMaterial override
 *   2. Depth pre-pass    → default materials (captures depth texture)
 *   3. Object ID pre-pass→ each mesh gets a unique flat color
 *   4. EffectComposer     → RenderPass + EdgePass + OutputPass
 *
 * Edge detection uses THREE signals:
 *   - Depth discontinuity  → silhouettes against background
 *   - Normal discontinuity → top/side face creases
 *   - Object ID change     → tile-to-tile boundaries (gap-independent!)
 *
 * The object ID approach means tile outlines work at ANY gap size
 * and are not affected by hover raise.
 */
import * as THREE from 'three'
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js'
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js'
import { ShaderPass } from 'three/addons/postprocessing/ShaderPass.js'
import { OutputPass } from 'three/addons/postprocessing/OutputPass.js'

// ── Override materials ─────────────────────────────────────────────────────

const normalMat = new THREE.MeshNormalMaterial()

// ── Object ID material cache ───────────────────────────────────────────────
// Each mesh gets a unique flat color so the edge detector can distinguish
// adjacent objects regardless of gap size or depth.

// WeakMap lets GC reclaim materials when meshes are removed (no leaks).
const _idMaterials = new WeakMap<THREE.Object3D, THREE.MeshBasicMaterial>()
let _nextId = 1

function getIdMaterial(obj: THREE.Object3D): THREE.MeshBasicMaterial {
  let mat = _idMaterials.get(obj)
  if (!mat) {
    const id = _nextId++
    // Encode ID as RGB (supports up to 16M unique objects)
    const r = (id & 0xff) / 255
    const g = ((id >> 8) & 0xff) / 255
    const b = ((id >> 16) & 0xff) / 255
    mat = new THREE.MeshBasicMaterial({ color: new THREE.Color(r, g, b) })
    _idMaterials.set(obj, mat)
  }
  return mat
}

// ── Edge detection shader ──────────────────────────────────────────────────

const EdgeDetectShader = {
  uniforms: {
    tDiffuse:        { value: null as THREE.Texture | null },
    tNormal:         { value: null as THREE.Texture | null },
    tDepth:          { value: null as THREE.Texture | null },
    tObjectId:       { value: null as THREE.Texture | null },
    resolution:      { value: new THREE.Vector2(1, 1) },
    outlineColor:    { value: new THREE.Color(0x2a2a30) },
    depthThreshold:  { value: 0.5 },
    normalThreshold: { value: 0.3 },
    thickness:       { value: 1.0 },
    cameraNear:      { value: 0.1 },
    cameraFar:       { value: 100.0 },
    orthoCamera:     { value: 0.0 },
  },

  vertexShader: /* glsl */ `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,

  fragmentShader: /* glsl */ `
    uniform sampler2D tDiffuse;
    uniform sampler2D tNormal;
    uniform sampler2D tDepth;
    uniform sampler2D tObjectId;
    uniform vec2 resolution;
    uniform vec3 outlineColor;
    uniform float depthThreshold;
    uniform float normalThreshold;
    uniform float thickness;
    uniform float cameraNear;
    uniform float cameraFar;
    uniform float orthoCamera;

    varying vec2 vUv;

    float getDepth(float d) {
      if (orthoCamera > 0.5) {
        return cameraNear + d * (cameraFar - cameraNear);
      }
      return cameraNear * cameraFar / (cameraFar - d * (cameraFar - cameraNear));
    }

    // Detect edge at a single sample point using 1px neighbours.
    float detectEdge(vec2 uv, vec2 px) {
      // -- Depth --
      float d  = getDepth(texture2D(tDepth, uv).r);
      float dU = getDepth(texture2D(tDepth, uv + vec2(0.0, px.y)).r);
      float dD = getDepth(texture2D(tDepth, uv - vec2(0.0, px.y)).r);
      float dL = getDepth(texture2D(tDepth, uv - vec2(px.x, 0.0)).r);
      float dR = getDepth(texture2D(tDepth, uv + vec2(px.x, 0.0)).r);
      float depthEdge = max(
        max(abs(d - dU), abs(d - dD)),
        max(abs(d - dL), abs(d - dR))
      );

      // -- Normals --
      vec3 n  = texture2D(tNormal, uv).rgb;
      vec3 nU = texture2D(tNormal, uv + vec2(0.0, px.y)).rgb;
      vec3 nD = texture2D(tNormal, uv - vec2(0.0, px.y)).rgb;
      vec3 nL = texture2D(tNormal, uv - vec2(px.x, 0.0)).rgb;
      vec3 nR = texture2D(tNormal, uv + vec2(px.x, 0.0)).rgb;
      float normalEdge = max(
        max(length(n - nU), length(n - nD)),
        max(length(n - nL), length(n - nR))
      );

      // -- Object ID --
      vec3 id  = texture2D(tObjectId, uv).rgb;
      vec3 idU = texture2D(tObjectId, uv + vec2(0.0, px.y)).rgb;
      vec3 idD = texture2D(tObjectId, uv - vec2(0.0, px.y)).rgb;
      vec3 idL = texture2D(tObjectId, uv - vec2(px.x, 0.0)).rgb;
      vec3 idR = texture2D(tObjectId, uv + vec2(px.x, 0.0)).rgb;
      // Any difference in ID = hard edge (IDs are flat colors, no interpolation)
      float idEdge = max(
        max(length(id - idU), length(id - idD)),
        max(length(id - idL), length(id - idR))
      );
      // Threshold very low: any ID difference > 0 means different object
      float idLine = step(0.01, idEdge);

      float dLine = smoothstep(depthThreshold * 0.5, depthThreshold, depthEdge);
      float nLine = smoothstep(normalThreshold * 0.7, normalThreshold, normalEdge);
      return max(max(dLine, nLine), idLine);
    }

    void main() {
      vec2 px = 1.0 / resolution;

      float edge = detectEdge(vUv, px);

      // Dilate along 8 directions for thickness
      if (edge < 1.0 && thickness > 0.5) {
        float t = thickness;
        edge = max(edge, detectEdge(vUv + vec2( t,  0.0) * px, px));
        edge = max(edge, detectEdge(vUv + vec2(-t,  0.0) * px, px));
        edge = max(edge, detectEdge(vUv + vec2( 0.0,  t) * px, px));
        edge = max(edge, detectEdge(vUv + vec2( 0.0, -t) * px, px));
        float diag = t * 0.707;
        edge = max(edge, detectEdge(vUv + vec2( diag,  diag) * px, px));
        edge = max(edge, detectEdge(vUv + vec2(-diag,  diag) * px, px));
        edge = max(edge, detectEdge(vUv + vec2( diag, -diag) * px, px));
        edge = max(edge, detectEdge(vUv + vec2(-diag, -diag) * px, px));
      }

      vec4 color = texture2D(tDiffuse, vUv);
      gl_FragColor = vec4(mix(color.rgb, outlineColor, edge), 1.0);
    }
  `,
}

// ── Public interface ───────────────────────────────────────────────────────

export interface OutlineOptions {
  outlineColor?: number
  depthThreshold?: number
  normalThreshold?: number
  thickness?: number
}

export function createOutlineComposer(
  renderer: THREE.WebGLRenderer,
  scene: THREE.Scene,
  camera: THREE.Camera,
  options: OutlineOptions = {},
): {
  composer: EffectComposer
  resize: (w: number, h: number) => void
  render: () => void
  edgeUniforms: Record<string, THREE.IUniform>
} {
  const pixelRatio = renderer.getPixelRatio()
  const w = renderer.domElement.clientWidth
  const h = renderer.domElement.clientHeight
  const pw = Math.floor(w * pixelRatio)
  const ph = Math.floor(h * pixelRatio)

  // ── Offscreen targets ──────────────────────────────────────────────────
  const normalTarget = new THREE.WebGLRenderTarget(pw, ph, {
    minFilter: THREE.NearestFilter,
    magFilter: THREE.NearestFilter,
    type: THREE.HalfFloatType,
  })

  const depthTarget = new THREE.WebGLRenderTarget(pw, ph, {
    minFilter: THREE.NearestFilter,
    magFilter: THREE.NearestFilter,
    depthTexture: new THREE.DepthTexture(pw, ph),
  })
  depthTarget.depthTexture!.type = THREE.FloatType

  const objectIdTarget = new THREE.WebGLRenderTarget(pw, ph, {
    minFilter: THREE.NearestFilter,
    magFilter: THREE.NearestFilter,
  })

  // ── EffectComposer ─────────────────────────────────────────────────────
  const composer = new EffectComposer(renderer)
  composer.setSize(w, h)

  const renderPass = new RenderPass(scene, camera)
  composer.addPass(renderPass)

  const edgePass = new ShaderPass(EdgeDetectShader)
  const eu = edgePass.uniforms
  eu['tNormal'].value = normalTarget.texture
  eu['tDepth'].value = depthTarget.depthTexture
  eu['tObjectId'].value = objectIdTarget.texture
  eu['resolution'].value.set(pw, ph)

  if (options.outlineColor !== undefined) eu['outlineColor'].value.setHex(options.outlineColor)
  if (options.depthThreshold !== undefined) eu['depthThreshold'].value = options.depthThreshold
  if (options.normalThreshold !== undefined) eu['normalThreshold'].value = options.normalThreshold
  if (options.thickness !== undefined) eu['thickness'].value = options.thickness

  if (camera instanceof THREE.OrthographicCamera || camera instanceof THREE.PerspectiveCamera) {
    eu['cameraNear'].value = camera.near
    eu['cameraFar'].value = camera.far
  }
  if (camera instanceof THREE.OrthographicCamera) {
    eu['orthoCamera'].value = 1.0
  }

  composer.addPass(edgePass)
  composer.addPass(new OutputPass())

  // ── Per-frame render ───────────────────────────────────────────────────
  const _clearCol = new THREE.Color()

  // Pre-allocate arrays for object ID pass to avoid per-frame Map/GC churn.
  let _meshList: THREE.Mesh[] = []
  let _savedMats: (THREE.Material | THREE.Material[])[] = []
  let _meshListDirty = true

  /** Call this after adding/removing meshes from the scene. */
  function invalidateMeshList(): void { _meshListDirty = true }

  function rebuildMeshList(): void {
    _meshList = []
    scene.traverse((obj) => {
      if (obj instanceof THREE.Mesh) _meshList.push(obj)
    })
    _savedMats = new Array(_meshList.length)
    _meshListDirty = false
  }

  function render(): void {
    const prevClear = renderer.getClearColor(_clearCol).clone()
    const prevAlpha = renderer.getClearAlpha()

    // Pre-pass 1: normals
    scene.overrideMaterial = normalMat
    renderer.setRenderTarget(normalTarget)
    renderer.setClearColor(0x8080ff, 1.0)
    renderer.clear()
    renderer.render(scene, camera)

    // Pre-pass 2: depth
    scene.overrideMaterial = null
    renderer.setRenderTarget(depthTarget)
    renderer.setClearColor(prevClear, prevAlpha)
    renderer.clear()
    renderer.render(scene, camera)

    // Pre-pass 3: object IDs (each mesh gets a unique flat color)
    // Uses pre-allocated arrays instead of per-frame Map to avoid GC churn.
    if (_meshListDirty) rebuildMeshList()
    for (let i = 0; i < _meshList.length; i++) {
      _savedMats[i] = _meshList[i].material
      _meshList[i].material = getIdMaterial(_meshList[i])
    }
    renderer.setRenderTarget(objectIdTarget)
    renderer.setClearColor(0x000000, 1.0)
    renderer.clear()
    renderer.render(scene, camera)
    // Restore original materials
    for (let i = 0; i < _meshList.length; i++) {
      _meshList[i].material = _savedMats[i]
    }

    // Restore
    scene.overrideMaterial = null
    renderer.setRenderTarget(null)
    renderer.setClearColor(prevClear, prevAlpha)

    // Main pass: composer renders scene (color) -> edge detect -> sRGB
    composer.render()
  }

  // ── Resize ─────────────────────────────────────────────────────────────
  function resize(newW: number, newH: number): void {
    const pr = renderer.getPixelRatio()
    const npw = Math.floor(newW * pr)
    const nph = Math.floor(newH * pr)
    normalTarget.setSize(npw, nph)
    depthTarget.setSize(npw, nph)
    objectIdTarget.setSize(npw, nph)
    composer.setSize(newW, newH)
    eu['resolution'].value.set(npw, nph)
  }

  return { composer, resize, render, invalidateMeshList, edgeUniforms: eu }
}
