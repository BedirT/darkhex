/**
 * Draw miniature pointy-top hex boards on a Canvas 2D context.
 * Uses the thesis-style parallelogram layout where rows shift right,
 * with colored border edges indicating player connection sides.
 *
 * Orientation:
 *   - Pointy-top hexagons (vertex at top, standard for Hex papers)
 *   - Rows stack vertically, each row offset half a cell to the right
 *   - Black connects North (top) ↔ South (bottom)
 *   - White connects West (left) ↔ East (right)
 *
 * Pointy-top vertex numbering (v0 = top, clockwise):
 *   v0 = top          (0, -r)
 *   v1 = upper-right  (+√3/2 r, -r/2)
 *   v2 = lower-right  (+√3/2 r, +r/2)
 *   v3 = bottom       (0, +r)
 *   v4 = lower-left   (-√3/2 r, +r/2)
 *   v5 = upper-left   (-√3/2 r, -r/2)
 */

const COLORS = {
  empty:     '#e0b8b8',
  black:     '#506080',
  white:     '#e8e0d8',
  highlight: '#81c784',
  outline:   '#2a2a30',
  edgeBlack: '#506080',
  edgeWhite: '#c8beb4',  // slightly darker cream for visibility on light bg
}

const SQRT3 = Math.sqrt(3)

/**
 * Grid (row, col) → 2D pixel offset for pointy-top hex.
 * Rows stack vertically; each row shifts half a hex-width right.
 */
function hexCenter(row: number, col: number, r: number): [number, number] {
  const x = SQRT3 * r * col + (SQRT3 / 2) * r * row
  const y = 1.5 * r * row
  return [x, y]
}

/** Vertex position of pointy-top hex. i=0 is top, clockwise. */
function vtx(cx: number, cy: number, r: number, i: number): [number, number] {
  const angle = -Math.PI / 2 + (Math.PI / 3) * i
  return [cx + r * Math.cos(angle), cy + r * Math.sin(angle)]
}

/** Draw a single pointy-top hexagon path. */
function hexPath(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number): void {
  ctx.beginPath()
  for (let i = 0; i < 6; i++) {
    const [px, py] = vtx(cx, cy, r, i)
    if (i === 0) ctx.moveTo(px, py)
    else ctx.lineTo(px, py)
  }
  ctx.closePath()
}

export interface MiniBoardOptions {
  highlightActions?: number[]
}

/**
 * Draw a miniature pointy-top hex board centered at (cx, cy).
 */
export function drawMiniBoard(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  boardView: Int8Array,
  rows: number,
  cols: number,
  cellSize: number,
  options?: MiniBoardOptions,
): void {
  const highlightSet = options?.highlightActions
    ? new Set(options.highlightActions)
    : null

  const r = cellSize * 0.92 // slightly smaller than full cell for gaps

  // Center the board at (cx, cy)
  const [firstX, firstY] = hexCenter(0, 0, cellSize)
  const [lastX, lastY] = hexCenter(rows - 1, cols - 1, cellSize)
  const ox = cx - (firstX + lastX) / 2
  const oy = cy - (firstY + lastY) / 2

  // Draw hex cells
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const idx = row * cols + col
      const [hx, hy] = hexCenter(row, col, cellSize)
      const px = hx + ox
      const py = hy + oy

      const cell = boardView[idx]
      let fill: string
      if (highlightSet?.has(idx)) {
        fill = COLORS.highlight
      } else if (cell === 1) {
        fill = COLORS.black
      } else if (cell === 2) {
        fill = COLORS.white
      } else {
        fill = COLORS.empty
      }

      hexPath(ctx, px, py, r)
      ctx.fillStyle = fill
      ctx.fill()
      ctx.strokeStyle = COLORS.outline
      ctx.lineWidth = Math.max(0.5, cellSize * 0.08)
      ctx.stroke()
    }
  }

  // Draw border edges
  const lw = Math.max(1.5, cellSize * 0.25)

  // North border (row 0): Black — upper-left (v5) → top (v0) → upper-right (v1)
  ctx.strokeStyle = COLORS.edgeBlack
  ctx.lineWidth = lw
  ctx.beginPath()
  for (let col = 0; col < cols; col++) {
    const [hx, hy] = hexCenter(0, col, cellSize)
    const px = hx + ox, py = hy + oy
    const [x5, y5] = vtx(px, py, r, 5)
    const [x0, y0] = vtx(px, py, r, 0)
    const [x1, y1] = vtx(px, py, r, 1)
    ctx.moveTo(x5, y5)
    ctx.lineTo(x0, y0)
    ctx.lineTo(x1, y1)
  }
  ctx.stroke()

  // South border (last row): Black — lower-right (v2) → bottom (v3) → lower-left (v4)
  ctx.beginPath()
  for (let col = 0; col < cols; col++) {
    const [hx, hy] = hexCenter(rows - 1, col, cellSize)
    const px = hx + ox, py = hy + oy
    const [x2, y2] = vtx(px, py, r, 2)
    const [x3, y3] = vtx(px, py, r, 3)
    const [x4, y4] = vtx(px, py, r, 4)
    ctx.moveTo(x2, y2)
    ctx.lineTo(x3, y3)
    ctx.lineTo(x4, y4)
  }
  ctx.stroke()

  // West border (col 0): White — upper-left (v5) → lower-left (v4)
  ctx.strokeStyle = COLORS.edgeWhite
  ctx.beginPath()
  for (let row = 0; row < rows; row++) {
    const [hx, hy] = hexCenter(row, 0, cellSize)
    const px = hx + ox, py = hy + oy
    const [x5, y5] = vtx(px, py, r, 5)
    const [x4, y4] = vtx(px, py, r, 4)
    ctx.moveTo(x5, y5)
    ctx.lineTo(x4, y4)
  }
  ctx.stroke()

  // East border (last col): White — upper-right (v1) → lower-right (v2)
  ctx.beginPath()
  for (let row = 0; row < rows; row++) {
    const [hx, hy] = hexCenter(row, cols - 1, cellSize)
    const px = hx + ox, py = hy + oy
    const [x1, y1] = vtx(px, py, r, 1)
    const [x2, y2] = vtx(px, py, r, 2)
    ctx.moveTo(x1, y1)
    ctx.lineTo(x2, y2)
  }
  ctx.stroke()
}

/** Compute bounding box size of a mini board at given cell size. */
export function miniBoardSize(rows: number, cols: number, cellSize: number): { w: number; h: number } {
  if (rows === 0 || cols === 0) return { w: 0, h: 0 }
  const [x0, y0] = hexCenter(0, 0, cellSize)
  const [x1, y1] = hexCenter(rows - 1, cols - 1, cellSize)
  return {
    w: Math.abs(x1 - x0) + cellSize * SQRT3,
    h: Math.abs(y1 - y0) + cellSize * 2,
  }
}
