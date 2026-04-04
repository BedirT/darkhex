# Visual Verification — DSaGe Web App

When modifying files under `game/`, verify changes visually before marking work complete.

## Design Quality

Use the Impeccable design skills for any UI changes:
- Before implementing: use `critique` to evaluate the current design
- During implementation: use `audit` to check quality across accessibility, performance, and consistency
- After implementation: use `polish` for a final quality pass
- Other Impeccable skills as needed: `animate`, `arrange`, `colorize`, `typeset`, `harden`, `normalize`

## Prerequisites

Before visual testing, ensure:
1. WASM pkg is built: `make game` or `cd crates/wasm && wasm-pack build --target web --out-dir ../../game/pkg`
2. npm deps installed: `cd game && npm install`

## Verification Steps

After completing game/ changes:

1. **Start dev server** in background: `cd game && npx vite --host &`
2. **Open in browser** via chrome-devtools-mcp: navigate to `http://localhost:5173`
3. **Take screenshot** and verify the visual state matches expectations
4. **Test interactions**: click buttons, verify state transitions, check animations
5. **Check console**: use `list_console_messages` to catch runtime errors
6. **Screenshot after each key interaction** to document the visual state
7. **Stop dev server** when done: kill the background process

## What to Verify

- Layout renders correctly (no blank screens, no overlapping elements)
- Colors and typography match the existing warm board-game palette
- Interactive elements respond to clicks and hovers
- Animations play smoothly (stone drop, tile flip, etc.)
- Three.js scene renders (board visible, outlines working)
- No console errors or warnings
- Responsive: resize the browser window to check layout adaptation

## When to Skip

- Pure TypeScript refactors with no visual impact (rename, type changes)
- Changes only to strategy logic that don't affect rendering
- `npm run build` / type-check-only verification is sufficient for non-visual changes
