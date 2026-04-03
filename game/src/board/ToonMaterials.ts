import * as THREE from 'three'

/** Shared 3-step toon gradient map (cel-shading ramp). */
let _gradientMap: THREE.DataTexture | null = null
export function gradientMap(): THREE.DataTexture {
  if (!_gradientMap) {
    const colors = new Uint8Array([100, 200, 255])
    _gradientMap = new THREE.DataTexture(colors, 3, 1, THREE.RedFormat)
    _gradientMap.needsUpdate = true
    _gradientMap.minFilter = THREE.NearestFilter
    _gradientMap.magFilter = THREE.NearestFilter
  }
  return _gradientMap
}

/** Create a MeshToonMaterial with the shared gradient map. */
export function toonMat(color: number): THREE.MeshToonMaterial {
  return new THREE.MeshToonMaterial({ color, gradientMap: gradientMap() })
}
