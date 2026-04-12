/**
 * Synthesized board-game sounds using the Web Audio API.
 * No external files — all sounds are generated programmatically.
 *
 * AudioContext is created lazily on first user interaction
 * (browsers require a user gesture to start audio).
 */

let _ctx: AudioContext | null = null

function ctx(): AudioContext {
  if (!_ctx) _ctx = new AudioContext()
  return _ctx
}

/** Resume AudioContext after a user gesture (call from first click/key). */
export function ensureAudioReady(): void {
  const c = ctx()
  if (c.state === 'suspended') c.resume()
}

// ── Wooden thock (tile hover) ──────────────────────────────────────────────

/**
 * Short, punchy wooden "thock" — like placing a tile on a board.
 * Built from a filtered noise burst + a low sine knock.
 */
export function playThock(): void {
  const c = ctx()
  if (c.state !== 'running') return
  const now = c.currentTime

  const master = c.createGain()
  master.gain.setValueAtTime(0.13, now)
  master.connect(c.destination)

  // ── Noise burst — warm, round, less clicky ─────────────────────────
  const noiseDur = 0.05
  const noiseLen = Math.ceil(c.sampleRate * noiseDur)
  const noiseBuffer = c.createBuffer(1, noiseLen, c.sampleRate)
  const noiseData = noiseBuffer.getChannelData(0)
  for (let i = 0; i < noiseLen; i++) {
    // Smooth fade-out envelope (quadratic) for rounder tail
    const env = (1 - i / noiseLen) ** 2
    noiseData[i] = (Math.random() * 2 - 1) * env
  }

  const noise = c.createBufferSource()
  noise.buffer = noiseBuffer

  // Lower bandpass with higher Q — more "hollow wood" resonance
  const filter = c.createBiquadFilter()
  filter.type = 'bandpass'
  filter.frequency.setValueAtTime(500, now)
  filter.Q.setValueAtTime(5, now)

  const noiseGain = c.createGain()
  noiseGain.gain.setValueAtTime(0.7, now)
  noiseGain.gain.exponentialRampToValueAtTime(0.001, now + noiseDur)

  noise.connect(filter)
  filter.connect(noiseGain)
  noiseGain.connect(master)
  noise.start(now)
  noise.stop(now + noiseDur)

  // ── Warm sine body — gentle pitch drop gives it "weight" ───────────
  const osc = c.createOscillator()
  osc.type = 'sine'
  osc.frequency.setValueAtTime(120, now)
  osc.frequency.exponentialRampToValueAtTime(70, now + 0.07)

  const oscGain = c.createGain()
  oscGain.gain.setValueAtTime(0.8, now)
  oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.08)

  osc.connect(oscGain)
  oscGain.connect(master)
  osc.start(now)
  osc.stop(now + 0.09)
}

// ── Stone place (heavier thock for clicks) ─────────────────────────────────

/**
 * Deeper, more resonant knock — for placing a stone or confirming an action.
 */
export function playPlace(): void {
  const c = ctx()
  if (c.state !== 'running') return
  const now = c.currentTime

  const master = c.createGain()
  master.gain.setValueAtTime(0.16, now)
  master.connect(c.destination)

  // ── Warm noise body — rounder, longer resonance ────────────────────
  const noiseDur = 0.07
  const noiseLen = Math.ceil(c.sampleRate * noiseDur)
  const noiseBuffer = c.createBuffer(1, noiseLen, c.sampleRate)
  const noiseData = noiseBuffer.getChannelData(0)
  for (let i = 0; i < noiseLen; i++) {
    const env = (1 - i / noiseLen) ** 2
    noiseData[i] = (Math.random() * 2 - 1) * env
  }

  const noise = c.createBufferSource()
  noise.buffer = noiseBuffer

  // Lower, more resonant — like a thick wooden block
  const filter = c.createBiquadFilter()
  filter.type = 'bandpass'
  filter.frequency.setValueAtTime(380, now)
  filter.Q.setValueAtTime(6, now)

  const noiseGain = c.createGain()
  noiseGain.gain.setValueAtTime(0.6, now)
  noiseGain.gain.exponentialRampToValueAtTime(0.001, now + noiseDur)

  noise.connect(filter)
  filter.connect(noiseGain)
  noiseGain.connect(master)
  noise.start(now)
  noise.stop(now + noiseDur)

  // ── Deep sine with slow decay — satisfying thump ───────────────────
  const osc = c.createOscillator()
  osc.type = 'sine'
  osc.frequency.setValueAtTime(95, now)
  osc.frequency.exponentialRampToValueAtTime(50, now + 0.10)

  const oscGain = c.createGain()
  oscGain.gain.setValueAtTime(0.9, now)
  oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.12)

  osc.connect(oscGain)
  oscGain.connect(master)
  osc.start(now)
  osc.stop(now + 0.13)
}

// ── Stone vanish (soft upward whoosh) ──────────────────────────────────────

/**
 * Airy, breathy "pff" — stone dissolving and floating away.
 * Rising pitch with filtered noise gives an upward, evaporating quality.
 */
export function playVanish(): void {
  const c = ctx()
  if (c.state !== 'running') return
  const now = c.currentTime

  const master = c.createGain()
  master.gain.setValueAtTime(0.08, now)
  master.connect(c.destination)

  // ── Breathy whoosh — highpass noise that fades in then out ──────────
  const dur = 0.18
  const noiseLen = Math.ceil(c.sampleRate * dur)
  const noiseBuf = c.createBuffer(1, noiseLen, c.sampleRate)
  const noiseData = noiseBuf.getChannelData(0)
  for (let i = 0; i < noiseLen; i++) {
    // Bell envelope — fades in then out
    const env = Math.sin((i / noiseLen) * Math.PI)
    noiseData[i] = (Math.random() * 2 - 1) * env
  }

  const noise = c.createBufferSource()
  noise.buffer = noiseBuf

  // Highpass that rises — gives the "upward" sensation
  const filter = c.createBiquadFilter()
  filter.type = 'highpass'
  filter.frequency.setValueAtTime(400, now)
  filter.frequency.exponentialRampToValueAtTime(2000, now + dur)
  filter.Q.setValueAtTime(0.7, now)

  const noiseGain = c.createGain()
  noiseGain.gain.setValueAtTime(1.0, now)
  noiseGain.gain.exponentialRampToValueAtTime(0.001, now + dur)

  noise.connect(filter)
  filter.connect(noiseGain)
  noiseGain.connect(master)
  noise.start(now)
  noise.stop(now + dur)

  // ── Soft rising tone — like air escaping ───────────────────────────
  const osc = c.createOscillator()
  osc.type = 'sine'
  osc.frequency.setValueAtTime(200, now)
  osc.frequency.exponentialRampToValueAtTime(500, now + 0.12)

  const oscGain = c.createGain()
  oscGain.gain.setValueAtTime(0.3, now)
  oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.14)

  osc.connect(oscGain)
  oscGain.connect(master)
  osc.start(now)
  osc.stop(now + 0.15)
}

// ── Stone drop landing (clack on impact) ───────────────────────────────────

/**
 * Satisfying "clack" — stone landing on the board after a drop.
 * Sharper attack than the hover thock, with a resonant wooden tail.
 */
export function playDrop(): void {
  const c = ctx()
  if (c.state !== 'running') return
  const now = c.currentTime

  const master = c.createGain()
  master.gain.setValueAtTime(0.18, now)
  master.connect(c.destination)

  // ── Sharp attack — short bright noise burst ────────────────────────
  const attackDur = 0.02
  const attackLen = Math.ceil(c.sampleRate * attackDur)
  const attackBuf = c.createBuffer(1, attackLen, c.sampleRate)
  const attackData = attackBuf.getChannelData(0)
  for (let i = 0; i < attackLen; i++) {
    attackData[i] = (Math.random() * 2 - 1) * (1 - i / attackLen)
  }

  const attack = c.createBufferSource()
  attack.buffer = attackBuf

  const hiFilter = c.createBiquadFilter()
  hiFilter.type = 'bandpass'
  hiFilter.frequency.setValueAtTime(1200, now)
  hiFilter.Q.setValueAtTime(2, now)

  const attackGain = c.createGain()
  attackGain.gain.setValueAtTime(0.6, now)
  attackGain.gain.exponentialRampToValueAtTime(0.001, now + attackDur)

  attack.connect(hiFilter)
  hiFilter.connect(attackGain)
  attackGain.connect(master)
  attack.start(now)
  attack.stop(now + attackDur)

  // ── Resonant body — lower, longer, wooden ring ─────────────────────
  const bodyDur = 0.08
  const bodyLen = Math.ceil(c.sampleRate * bodyDur)
  const bodyBuf = c.createBuffer(1, bodyLen, c.sampleRate)
  const bodyData = bodyBuf.getChannelData(0)
  for (let i = 0; i < bodyLen; i++) {
    const env = (1 - i / bodyLen) ** 2
    bodyData[i] = (Math.random() * 2 - 1) * env
  }

  const body = c.createBufferSource()
  body.buffer = bodyBuf

  const loFilter = c.createBiquadFilter()
  loFilter.type = 'bandpass'
  loFilter.frequency.setValueAtTime(400, now)
  loFilter.Q.setValueAtTime(6, now)

  const bodyGain = c.createGain()
  bodyGain.gain.setValueAtTime(0.8, now)
  bodyGain.gain.exponentialRampToValueAtTime(0.001, now + bodyDur)

  body.connect(loFilter)
  loFilter.connect(bodyGain)
  bodyGain.connect(master)
  body.start(now)
  body.stop(now + bodyDur)

  // ── Deep thump — impact weight ─────────────────────────────────────
  const osc = c.createOscillator()
  osc.type = 'sine'
  osc.frequency.setValueAtTime(100, now)
  osc.frequency.exponentialRampToValueAtTime(45, now + 0.10)

  const oscGain = c.createGain()
  oscGain.gain.setValueAtTime(1.0, now)
  oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.12)

  osc.connect(oscGain)
  oscGain.connect(master)
  osc.start(now)
  osc.stop(now + 0.13)
}

// ── Stone reveal (rise from below) ─────────────────────────────────────────

/**
 * Soft, muffled emergence — stone rising from beneath the tile.
 * Quieter and more muted than the drop, with a "sliding" quality.
 */
export function playReveal(): void {
  const c = ctx()
  if (c.state !== 'running') return
  const now = c.currentTime

  const master = c.createGain()
  master.gain.setValueAtTime(0.09, now)
  master.connect(c.destination)

  // Soft filtered noise — like something sliding into place
  const noiseDur = 0.10
  const noiseLen = Math.ceil(c.sampleRate * noiseDur)
  const noiseBuffer = c.createBuffer(1, noiseLen, c.sampleRate)
  const noiseData = noiseBuffer.getChannelData(0)
  for (let i = 0; i < noiseLen; i++) {
    const env = Math.sin((i / noiseLen) * Math.PI) // bell-shaped: fade in then out
    noiseData[i] = (Math.random() * 2 - 1) * env
  }

  const noise = c.createBufferSource()
  noise.buffer = noiseBuffer

  // Low-pass — muffled, underground feel
  const filter = c.createBiquadFilter()
  filter.type = 'lowpass'
  filter.frequency.setValueAtTime(300, now)
  filter.frequency.linearRampToValueAtTime(600, now + noiseDur)
  filter.Q.setValueAtTime(1, now)

  const noiseGain = c.createGain()
  noiseGain.gain.setValueAtTime(0.8, now)
  noiseGain.gain.exponentialRampToValueAtTime(0.001, now + noiseDur)

  noise.connect(filter)
  filter.connect(noiseGain)
  noiseGain.connect(master)
  noise.start(now)
  noise.stop(now + noiseDur)

  // Gentle rising tone
  const osc = c.createOscillator()
  osc.type = 'sine'
  osc.frequency.setValueAtTime(60, now)
  osc.frequency.linearRampToValueAtTime(90, now + 0.08)

  const oscGain = c.createGain()
  oscGain.gain.setValueAtTime(0.5, now)
  oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.10)

  osc.connect(oscGain)
  oscGain.connect(master)
  osc.start(now)
  osc.stop(now + 0.11)
}

// ── Collision (short buzz) ─────────────────────────────────────────────────

/**
 * Quick buzzy tap — collision feedback (stone already there).
 */
export function playCollision(): void {
  const c = ctx()
  if (c.state !== 'running') return
  const now = c.currentTime

  const master = c.createGain()
  master.gain.setValueAtTime(0.10, now)
  master.connect(c.destination)

  const osc = c.createOscillator()
  osc.type = 'square'
  osc.frequency.setValueAtTime(220, now)
  osc.frequency.exponentialRampToValueAtTime(110, now + 0.08)

  const oscGain = c.createGain()
  oscGain.gain.setValueAtTime(0.5, now)
  oscGain.gain.exponentialRampToValueAtTime(0.001, now + 0.08)

  osc.connect(oscGain)
  oscGain.connect(master)
  osc.start(now)
  osc.stop(now + 0.08)
}

// ── Page turn (tutorial step advance) ─────────────────────────────────────

/**
 * Soft, breathy swish — like turning a page in a book.
 * Very quiet to avoid competing with board sounds.
 */
export function playPageTurn(): void {
  const c = ctx()
  if (c.state !== 'running') return
  const now = c.currentTime

  const master = c.createGain()
  master.gain.setValueAtTime(0.06, now)
  master.connect(c.destination)

  const dur = 0.08
  const noiseLen = Math.ceil(c.sampleRate * dur)
  const noiseBuf = c.createBuffer(1, noiseLen, c.sampleRate)
  const noiseData = noiseBuf.getChannelData(0)
  for (let i = 0; i < noiseLen; i++) {
    const env = Math.sin((i / noiseLen) * Math.PI)
    noiseData[i] = (Math.random() * 2 - 1) * env
  }

  const noise = c.createBufferSource()
  noise.buffer = noiseBuf

  const filter = c.createBiquadFilter()
  filter.type = 'highpass'
  filter.frequency.setValueAtTime(800, now)
  filter.frequency.exponentialRampToValueAtTime(2500, now + dur)
  filter.Q.setValueAtTime(0.5, now)

  const noiseGain = c.createGain()
  noiseGain.gain.setValueAtTime(0.8, now)
  noiseGain.gain.exponentialRampToValueAtTime(0.001, now + dur)

  noise.connect(filter)
  filter.connect(noiseGain)
  noiseGain.connect(master)
  noise.start(now)
  noise.stop(now + dur)
}

// ── Celebration chime (strategy complete) ──────────────────────────────────

/**
 * Warm, ascending three-note chime — "you did it!"
 * Major triad arpeggio with gentle sine tones.
 */
export function playChime(): void {
  const c = ctx()
  if (c.state !== 'running') return
  const now = c.currentTime

  const master = c.createGain()
  master.gain.setValueAtTime(0.12, now)
  master.connect(c.destination)

  // Three ascending notes: C5 → E5 → G5 (major triad)
  const notes = [523.25, 659.25, 783.99]
  const noteDelay = 0.12  // stagger between notes
  const noteDur = 0.4

  for (let i = 0; i < notes.length; i++) {
    const t = now + i * noteDelay

    const osc = c.createOscillator()
    osc.type = 'sine'
    osc.frequency.setValueAtTime(notes[i], t)

    const gain = c.createGain()
    gain.gain.setValueAtTime(0, t)
    gain.gain.linearRampToValueAtTime(0.8, t + 0.03)       // quick attack
    gain.gain.exponentialRampToValueAtTime(0.001, t + noteDur)  // gentle decay

    osc.connect(gain)
    gain.connect(master)
    osc.start(t)
    osc.stop(t + noteDur)
  }
}
