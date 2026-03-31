---
name: Thesis Strategy Builder Tool (DSaGe)
description: Detailed analysis of the strategy builder/generator tool from the thesis, its UI, data model, and visualization patterns
type: reference
---

## DSaGe (Dark Hex Strategy Generator)

The thesis (Chapter 3, "Analysis of Dark Hex Using Game Theory") describes DSaGe, an interactive tool for manually building complete strategies. It exists in the codebase as legacy Python code using Tkinter/customtkinter.

### What DSaGe Does
- Allows a user to construct a complete strategy by specifying action-probability distributions for every reachable information state
- Ensures completeness: every reachable info state has a defined response
- Two input modes: (1) notational syntax, (2) interactive step-by-step game walk
- Outputs strategy as a dictionary: `info_state -> [(action, probability), ...]`

### Legacy Code Locations
- `darkhex/gui/strategy_generator.py` — StrategyGenerator class (logic)
- `darkhex/gui/policygen_gui.py` — PolicyGenGUI class (Tkinter/customtkinter UI)
- `darkhex/gui/history_buffer.py` — gameBuffer class (rewind/restart state management)
- `darkhex/examples/strategy_generator.py` — Example launcher

### Visual Elements in Thesis

**1. DSaGe UI (Figure in Section 3.2)**
Five panels labeled A-E:
- A: Main window — hex board (honeycomb cells with labels like a1, b1), Rewind/Restart buttons, Input field, Enter/Random buttons, Log area
- B: New Game dialog — board text input, row/column size, first player checkbox, isomorphic checkbox
- C: Search history — lists past info states (e.g. `....`, `.y..`, `y...`)
- D: End of game save dialog — Save/Save to default/Cancel
- E: Terminal output showing dictionary: `info_state: [(action, probability)]`

**2. Strategy notation figures (Figure in Section 3.4, notation_game/a-e.pdf)**
- Hex boards with circles overlaid on cells showing probabilities (e.g., `.3` and `.7`)
- Row/column labels (1,2 / a,b) on axes
- Dark/light shading for player edges (Black=top/bottom, White=left/right)
- Step-by-step walkthrough of strategy construction

**3. Strategy tree figures (strategy_black.pdf, strategy_x.pdf, etc.)**
Hand-drawn style decision trees showing:
- Paired boards (Black view left, White view right) in colored boxes
  - Gray box = Black's move updated last
  - White box = White's move updated last
  - No box = terminal state
- Circles with action labels (filled black = Black move, outlined = White move)
- Arrows between states with probability labels (fractions like 0.14, 0.36, 0.5)
- "Strategy X/Y/Z" labels for sub-tree continuations
- "White Wins" / "Black Wins" terminal labels

**4. Programmatic strategy trees (p1_strategy_tree.pdf, p2_strategy_tree.pdf)**
Generated (likely programmatic) decision trees:
- Hexagonal nodes colored by player (black=P0, red=P1, gray=initial)
- Text inside nodes: player indicator + board state in text form (`P 0\n. . .\n. x .\n...`)
- Edge labels: `action: probability` (e.g., `b2: 1.00`, `a4: 0.50`)
- Directed graph layout (top-to-bottom)
- Terminal nodes shown with circle outlines
- Convergent edges (same info state reached from different paths)

**5. SIP strategy figures (mccfr_game_p0.pdf, subgame_a-d.pdf)**
Publication-quality figures:
- Initial state shows both player boards side-by-side in rounded box
- Branching with probabilities (e.g., b2: 0.49, b3: 0.51)
- "Sub-game A" / "Sub-game B" continuation labels
- Same paired-board-in-box convention as handcrafted strategy trees
- pONE terminals labeled "White Wins (pONE)"
- "White gives up" / "Black wins" outcome labels

### Data Model (from strategy_generator.py)
- `info_states: Dict[str, List[Tuple[int, float]]]` — maps info state string to (action, probability) pairs
- `current_info_state: str` — the info state currently being specified
- `action_stack: List[str]` — pending info states that still need specification
- Input format: `"a4 0.5 b4 0.5"` or `"= a4 b4"` (equiprobable) or `"r"` (random)
- History buffer stores snapshots for rewind/restart

### Board Visualization Details
- Hexagonal cells drawn with Canvas polygon (6-point coordinates)
- Cell coloring: blue=empty, dark=Black stone, beige=White stone
- Cell labels: alphanumeric (a1, b2, c3, etc.)
- Board respects hex topology with row offset
- Edge lengths calculated dynamically based on frame size
