import type { ActionProb, Policy, StrategyConfig, ExportedPolicy } from '../strategy/types'

/** Convert exported dict-of-dicts JSON to internal Policy Map. */
export function parseExportedPolicy(data: unknown): { config: StrategyConfig; policy: Policy } {
  const obj = data as Record<string, unknown>
  if (typeof obj !== 'object' || obj === null) throw new Error('Invalid policy: not an object')
  if (typeof obj.player !== 'number') throw new Error('Invalid policy: missing player')
  if (typeof obj.rows !== 'number') throw new Error('Invalid policy: missing rows')
  if (typeof obj.cols !== 'number') throw new Error('Invalid policy: missing cols')
  if (typeof obj.policy !== 'object' || obj.policy === null) throw new Error('Invalid policy: missing policy map')

  const exported = obj as unknown as ExportedPolicy

  // Strict range validation
  if (exported.player !== 0 && exported.player !== 1) throw new Error(`Invalid player: ${exported.player} (must be 0 or 1)`)
  if (!Number.isInteger(exported.rows) || exported.rows < 1 || exported.rows > 19) throw new Error(`Invalid rows: ${exported.rows}`)
  if (!Number.isInteger(exported.cols) || exported.cols < 1 || exported.cols > 19) throw new Error(`Invalid cols: ${exported.cols}`)

  const config: StrategyConfig = {
    player: exported.player,
    rows: exported.rows,
    cols: exported.cols,
    perfectRecall: exported.perfectRecall ?? false,
  }

  const totalCells = exported.rows * exported.cols
  const policy: Policy = new Map()
  for (const [infoState, actionDict] of Object.entries(exported.policy)) {
    const actions: ActionProb[] = []
    for (const [actionStr, prob] of Object.entries(actionDict)) {
      const action = Number(actionStr)
      const probability = Number(prob)
      if (!Number.isFinite(action) || action < 0 || action >= totalCells || action !== Math.floor(action)) {
        throw new Error(`Invalid action "${actionStr}" in info state`)
      }
      if (!Number.isFinite(probability) || probability < 0 || probability > 1) {
        throw new Error(`Invalid probability ${prob} for action ${actionStr}`)
      }
      actions.push([action, probability])
    }
    if (actions.length > 0) {
      policy.set(infoState, actions)
    }
  }

  if (policy.size === 0) throw new Error('Invalid policy: no info states')
  return { config, policy }
}

/** Open file picker, load + parse a policy JSON. Null if cancelled. */
export function loadPolicyFromFile(): Promise<{ config: StrategyConfig; policy: Policy } | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.json'
    input.style.display = 'none'
    document.body.appendChild(input)

    let resolved = false
    let fileChosen = false

    input.addEventListener('change', () => {
      fileChosen = true
      const file = input.files?.[0]
      input.remove()
      if (!file) { resolved = true; resolve(null); return }

      const reader = new FileReader()
      reader.onload = () => {
        try {
          const data = JSON.parse(reader.result as string)
          resolved = true
          resolve(parseExportedPolicy(data))
        } catch (err) {
          console.error('Failed to parse policy JSON:', err)
          resolved = true
          resolve(null)
        }
      }
      reader.onerror = () => { resolved = true; resolve(null) }
      reader.readAsText(file)
    })

    // Handle cancel: file picker closed without selecting
    // Focus returns to window after picker closes
    window.addEventListener('focus', () => {
      setTimeout(() => {
        if (!resolved && !fileChosen) {
          input.remove()
          resolved = true
          resolve(null)
        }
      }, 300)
    }, { once: true })

    input.click()
  })
}
