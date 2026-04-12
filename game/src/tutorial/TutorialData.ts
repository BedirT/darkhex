import type { TutorialSection } from './types'

/**
 * Tutorial step definitions — pure data, no DOM or logic.
 * Body text supports **bold** markers (rendered by TutorialOverlay).
 */
export const TUTORIAL_SECTIONS: TutorialSection[] = [
  // ── Section 1: Dark Hex Concepts ──────────────────────────────────────────
  {
    id: 'concepts',
    title: 'Dark Hex Concepts',
    steps: [
      {
        id: 'welcome',
        phase: 'concepts',
        target: '@none',
        title: 'Welcome to DSaGe',
        body: 'DSaGe helps you build and explore strategies for **Dark Hex** — a variant of the board game Hex where you **can\'t see your opponent\'s stones**. This tutorial takes about 5 minutes.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
      {
        id: 'what-is-dark-hex',
        phase: 'concepts',
        target: '@none',
        title: 'What is Dark Hex?',
        body: 'In regular Hex, both players see the whole board. In **Dark Hex**, each player only sees their own stones. When you try to place a stone where your opponent already has one, that\'s called a **collision** — your move fails, but you discover they were there.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
      {
        id: 'what-is-strategy',
        phase: 'concepts',
        target: '@none',
        title: 'What is a Strategy?',
        body: 'A **strategy** (or policy) is a complete plan: at every possible situation you could face, it says what action to take and with what probability. **Mixed strategies** (using probabilities) are important — if you always do the same thing, your opponent can predict and exploit you.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
    ],
  },

  // ── Section 2: Main Menu ──────────────────────────────────────────────────
  {
    id: 'main-menu',
    title: 'Main Menu',
    steps: [
      {
        id: 'menu-overview',
        phase: 'main-menu',
        target: '@none',
        title: 'The Main Menu',
        body: 'You just saw the main menu. From there you can **build** new strategies or **investigate** existing ones. The **Strategy Investigation** mode lets you load and explore saved policies as tree diagrams.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
      {
        id: 'menu-lets-build',
        phase: 'main-menu',
        target: '@none',
        title: "Let's Build a Strategy",
        body: "We'll now walk through building a complete strategy from scratch. First, let's configure the board.",
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
        setup: 'open-setup',
      },
    ],
  },

  // ── Section 3: Setup ──────────────────────────────────────────────────────
  {
    id: 'setup',
    title: 'Setup',
    steps: [
      {
        id: 'setup-board-size',
        phase: 'setup',
        target: '#sg-rows',
        title: 'Board Size',
        body: 'Choose the board dimensions. Larger boards = exponentially more situations to handle. We\'ll use a small **2×2 board** for learning.',
        tooltipPosition: 'right',
        trigger: { type: 'click-next' },
      },
      {
        id: 'setup-player',
        phase: 'setup',
        target: '#sg-player-group',
        title: 'Choose Your Player',
        body: 'You build a strategy for **one player**. **Black** goes first and connects top↔bottom. **White** connects left↔right. Each sees only their own stones.',
        tooltipPosition: 'bottom',
        trigger: { type: 'click-next' },
      },
      {
        id: 'setup-start',
        phase: 'setup',
        target: '@none',
        title: 'Start Building',
        body: "Everything's set! Let's start building the strategy.",
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
        setup: 'start-strategy',
      },
    ],
  },

  // ── Section 4: Information States ─────────────────────────────────────────
  {
    id: 'info-states',
    title: 'Information States',
    steps: [
      {
        id: 'board-view',
        phase: 'info-states',
        target: '@board',
        title: "Your Player's View",
        body: 'This board shows what **your player knows** right now — their own stones and any discovered collisions. Empty cells could be truly empty or hiding an opponent stone.',
        tooltipPosition: 'right',
        trigger: { type: 'click-next' },
      },
      {
        id: 'info-panel-intro',
        phase: 'info-states',
        target: '@info-panel',
        title: 'Information State',
        body: 'The top bar shows the current **information state** — a text encoding of everything your player knows. The progress bar tracks how many states you\'ve assigned actions to.',
        tooltipPosition: 'bottom',
        trigger: { type: 'click-next' },
      },
      {
        id: 'what-is-info-state',
        phase: 'info-states',
        target: '@none',
        title: "What's an Info State?",
        body: 'An **information state** captures all the knowledge a player has at a decision point: which cells they placed stones on, and which cells they discovered opponent stones on. Different game histories can lead to the **same** information state if the player can\'t tell them apart.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
      {
        id: 'action-panel-intro',
        phase: 'info-states',
        target: '@action-panel',
        title: 'Action Toolbar',
        body: 'This toolbar is where you\'ll assign actions. Selected tiles and their probabilities appear here.',
        tooltipPosition: 'top',
        trigger: { type: 'click-next' },
      },
    ],
  },

  // ── Section 5: Building a Move ────────────────────────────────────────────
  {
    id: 'building',
    title: 'Building a Move',
    steps: [
      {
        id: 'select-first-tile',
        phase: 'building',
        target: '@board',
        title: 'Select a Tile',
        body: 'Click any tile to select it as a possible action. The tile turns **green** and a probability label appears.',
        tooltipPosition: 'left',
        trigger: { type: 'tile-select', count: 1 },
      },
      {
        id: 'select-second-tile',
        phase: 'building',
        target: '@board',
        title: 'Mixed Strategy',
        body: 'Select another tile. Now you have a **mixed action** — your player randomizes between these cells. This unpredictability is what makes strategies hard to exploit.',
        tooltipPosition: 'left',
        trigger: { type: 'tile-select', count: 2 },
      },
      {
        id: 'probabilities',
        phase: 'building',
        target: '@action-panel',
        title: 'Probabilities',
        body: 'Each selected tile gets a probability. They must **sum to 1.00**. The **Equal** button distributes them evenly. You can also type exact values.',
        tooltipPosition: 'top',
        trigger: { type: 'click-next' },
      },
      {
        id: 'sum-validation',
        phase: 'building',
        target: '@action-panel',
        title: 'Sum Validation',
        body: 'The **∑** indicator shows if your probabilities are valid: **green ✓** means you\'re good, **amber** means they don\'t sum to 1 yet.',
        tooltipPosition: 'top',
        trigger: { type: 'click-next' },
      },
      {
        id: 'confirm-move',
        phase: 'building',
        target: '@action-panel',
        title: 'Confirm Your Move',
        body: 'When probabilities are valid, click **Confirm** (or press **Enter**) to lock in this decision and move to the next information state.',
        tooltipPosition: 'top',
        trigger: { type: 'confirm-action' },
      },
    ],
  },

  // ── Section 6: Collision ──────────────────────────────────────────────────
  {
    id: 'collision',
    title: 'Collision',
    steps: [
      {
        id: 'collision-explain',
        phase: 'collision',
        target: '@info-panel',
        title: 'Collision!',
        body: 'The orange **Collision** badge means your player tried to place a stone but hit an opponent\'s hidden stone. The move failed, but now your player **knows** the opponent is there.',
        tooltipPosition: 'bottom',
        trigger: { type: 'click-next' },
      },
      {
        id: 'collision-branching',
        phase: 'collision',
        target: '@board',
        title: 'Collision Branching',
        body: 'When you assign an action, the generator considers **both possibilities**: the cell was empty (stone placed) OR the cell had an opponent stone (collision). Each branch creates a different information state needing its own action.',
        tooltipPosition: 'right',
        trigger: { type: 'click-next' },
      },
      {
        id: 'collision-continue',
        phase: 'collision',
        target: '@board',
        title: 'Continue Building',
        body: 'Assign an action for this collision state. You can **double-click** a tile for a quick deterministic action (100% probability on one tile).',
        tooltipPosition: 'left',
        trigger: { type: 'confirm-action' },
      },
    ],
  },

  // ── Section 7: Shortcuts & Navigation ─────────────────────────────────────
  {
    id: 'shortcuts',
    title: 'Shortcuts',
    steps: [
      {
        id: 'double-click',
        phase: 'shortcuts',
        target: '@board',
        title: 'Double-Click Shortcut',
        body: '**Double-click** any tile to instantly assign it as a deterministic action (probability 1.0). Much faster when you\'re sure about a move. Try it now.',
        tooltipPosition: 'left',
        trigger: { type: 'double-click-tile' },
      },
      {
        id: 'undo',
        phase: 'shortcuts',
        target: '@action-panel',
        title: 'Undo',
        body: 'Made a mistake? **Undo** steps back one information state.',
        tooltipPosition: 'top',
        trigger: { type: 'click-next' },
      },
      {
        id: 'restart',
        phase: 'shortcuts',
        target: '@action-panel',
        title: 'Restart',
        body: '**Restart** returns to the very first information state, clearing all progress.',
        tooltipPosition: 'top',
        trigger: { type: 'click-next' },
      },
    ],
  },

  // ── Section 8: Completion & Investigation ─────────────────────────────────
  {
    id: 'completion',
    title: 'Completion & Investigation',
    steps: [
      {
        id: 'fast-forward',
        phase: 'completion',
        target: '@none',
        title: 'Fast Forward',
        body: 'Let\'s fast-forward through the remaining states to see what a complete strategy looks like...',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
        setup: 'auto-complete',
      },
      {
        id: 'strategy-complete',
        phase: 'completion',
        target: '@none',
        title: 'Strategy Complete!',
        body: 'You\'ve assigned an action to **every reachable information state**. This is a **complete behavioral strategy** — a full plan for every situation your player could face.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
      {
        id: 'download',
        phase: 'completion',
        target: '@none',
        title: 'Download',
        body: '**Download** saves your strategy as a JSON file. You can reload it later for investigation or share it.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
      {
        id: 'investigate-btn',
        phase: 'completion',
        target: '@none',
        title: 'Investigate',
        body: 'Now let\'s visualize the strategy as a **decision tree**. We\'ll enter the investigation view automatically.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
        setup: 'enter-investigation',
      },
      {
        id: 'investigation-overview',
        phase: 'investigation',
        target: '@none',
        title: 'The Strategy Tree',
        body: 'This is the **investigation view**. Each node is an information state with a mini hex board. Edges show actions and their probabilities. **Orange dashed edges** indicate collision branches.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
      {
        id: 'click-node',
        phase: 'investigation',
        target: '@none',
        title: 'Click a Node',
        body: 'Click any node to see its details in the sidebar: the full info state string, action probabilities, and a larger board view.',
        tooltipPosition: 'center',
        trigger: { type: 'custom', id: 'node-selected' },
      },
      {
        id: 'depth-slider',
        phase: 'investigation',
        target: '[data-tutorial="depth-slider"]',
        title: 'Depth Control',
        body: 'The **depth slider** collapses or expands the tree by level. **Expand All** / **Collapse All** for quick navigation.',
        tooltipPosition: 'bottom',
        trigger: { type: 'click-next' },
      },
      {
        id: 'keyboard-nav',
        phase: 'investigation',
        target: '@none',
        title: 'Keyboard Navigation',
        body: 'Navigate the tree with **arrow keys**: ↑ parent, ↓ first child, ←/→ siblings. **Enter** or **Space** toggles collapse. This is the fastest way to explore large trees.',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
      },
    ],
  },

  // ── Section 9: Wrap-up ────────────────────────────────────────────────────
  {
    id: 'wrapup',
    title: 'Wrap-up',
    steps: [
      {
        id: 'export-svg',
        phase: 'wrapup',
        target: '[data-tutorial="export-btn"]',
        title: 'Export SVG',
        body: '**Export SVG** saves the tree as a publication-quality vector graphic for papers or presentations.',
        tooltipPosition: 'bottom',
        trigger: { type: 'click-next' },
      },
      {
        id: 'tutorial-complete',
        phase: 'wrapup',
        target: '@none',
        title: "You're Ready!",
        body: 'You\'ve learned everything DSaGe can do! You can also **load MCCFR solver policies** (JSON files) directly into the investigation view to explore algorithmically computed strategies. Press Esc to return to the menu and start exploring!',
        tooltipPosition: 'center',
        trigger: { type: 'click-next' },
        setup: 'play-chime',
      },
    ],
  },
]

/** Flatten all sections into a single step array. */
export function flattenSteps(): { step: import('./types').TutorialStep; sectionIdx: number; stepIdx: number; globalIdx: number }[] {
  const result: { step: import('./types').TutorialStep; sectionIdx: number; stepIdx: number; globalIdx: number }[] = []
  let globalIdx = 0
  for (let s = 0; s < TUTORIAL_SECTIONS.length; s++) {
    for (let i = 0; i < TUTORIAL_SECTIONS[s].steps.length; i++) {
      result.push({ step: TUTORIAL_SECTIONS[s].steps[i], sectionIdx: s, stepIdx: i, globalIdx })
      globalIdx++
    }
  }
  return result
}
