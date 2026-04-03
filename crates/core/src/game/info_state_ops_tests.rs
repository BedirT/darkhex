use super::*;

#[test]
fn parse_imperfect_recall_2x2() {
    let info = "P0\n..\n..";
    let parsed = parse_info_state(info, 2, 2).unwrap();
    assert_eq!(parsed.player, 0);
    assert_eq!(parsed.board, vec!['.', '.', '.', '.']);
    assert_eq!(parsed.rows, 2);
    assert_eq!(parsed.cols, 2);
    assert!(parsed.action_history.is_empty());
}

#[test]
fn parse_with_stones() {
    let info = "P1\nx.\n.o";
    let parsed = parse_info_state(info, 2, 2).unwrap();
    assert_eq!(parsed.player, 1);
    assert_eq!(parsed.board, vec!['x', '.', '.', 'o']);
}

#[test]
fn parse_perfect_recall() {
    let info = "P0\nx.\n.o\n0,0 0,3 ";
    let parsed = parse_info_state(info, 2, 2).unwrap();
    assert_eq!(parsed.player, 0);
    assert_eq!(parsed.action_history, vec![0, 3]);
}

#[test]
fn parse_invalid_header() {
    assert!(parse_info_state("X0\n..\n..", 2, 2).is_err());
    assert!(parse_info_state("P5\n..\n..", 2, 2).is_err());
    assert!(parse_info_state("", 2, 2).is_err());
}

#[test]
fn parse_roundtrip() {
    let original = "P0\n.x\no.";
    let parsed = parse_info_state(original, 2, 2).unwrap();
    let reconstructed = to_info_state_string(&parsed, false);
    assert_eq!(reconstructed, original);
}

#[test]
fn parse_roundtrip_perfect_recall() {
    let original = "P1\nxo\n..\n1,2 1,0 ";
    let parsed = parse_info_state(original, 2, 2).unwrap();
    let reconstructed = to_info_state_string(&parsed, true);
    assert_eq!(reconstructed, original);
}

#[test]
fn legal_actions_empty_board() {
    let parsed = parse_info_state("P0\n..\n..", 2, 2).unwrap();
    assert_eq!(legal_actions(&parsed), vec![0, 1, 2, 3]);
}

#[test]
fn legal_actions_partial_board() {
    let parsed = parse_info_state("P0\nx.\n.o", 2, 2).unwrap();
    assert_eq!(legal_actions(&parsed), vec![1, 2]);
}

#[test]
fn legal_actions_full_board() {
    let parsed = parse_info_state("P0\nxo\nox", 2, 2).unwrap();
    assert!(legal_actions(&parsed).is_empty());
}

#[test]
fn collision_possible_black_turn() {
    // Black has 1 stone, White has 0 visible → collision IS possible
    // (After Black placed, White placed a hidden stone, so collision can happen)
    let parsed = parse_info_state("P0\nx.\n..", 2, 2).unwrap();
    assert!(is_collision_possible(&parsed));

    // Black has 2 stones, White has 1 visible → collision possible
    // (White could have placed a second stone that Black hasn't seen)
    let parsed = parse_info_state("P0\nxx\no.", 2, 2).unwrap();
    assert!(is_collision_possible(&parsed));

    // Black has 1 stone, White has 1 visible → no collision possible
    // (White has placed 1 stone and we see it, so no hidden white stones)
    let parsed = parse_info_state("P0\nx.\no.", 2, 2).unwrap();
    assert!(!is_collision_possible(&parsed));
}

#[test]
fn collision_possible_white_turn() {
    // White has 0 stones, Black has 0 visible → collision possible
    // (Black moves first, could have hidden stone)
    let parsed = parse_info_state("P1\n..\n..", 2, 2).unwrap();
    assert!(is_collision_possible(&parsed));

    // White has 1 stone, Black has 1 visible → collision possible
    let parsed = parse_info_state("P1\nx.\n.o", 2, 2).unwrap();
    assert!(is_collision_possible(&parsed));

    // White has 0 stones, Black has 1 visible → no collision possible
    // (We see Black's stone, so no hidden Black stones beyond that)
    let parsed = parse_info_state("P1\nx.\n..", 2, 2).unwrap();
    assert!(!is_collision_possible(&parsed));
}

#[test]
fn collision_not_possible_initial_black() {
    // Initial board, Black's turn: no opponent has placed yet
    let parsed = parse_info_state("P0\n..\n..", 2, 2).unwrap();
    assert!(!is_collision_possible(&parsed));
}

#[test]
fn successor_placed() {
    let parsed = parse_info_state("P0\n..\n..", 2, 2).unwrap();
    let next = info_state_after_action(&parsed, 0, 0, false).unwrap();
    assert_eq!(next, "P0\nx.\n..");
}

#[test]
fn successor_collision() {
    // Player 0 discovers opponent stone at cell 1
    let parsed = parse_info_state("P0\nx.\n..", 2, 2).unwrap();
    let next = info_state_after_action(&parsed, 1, 1, false).unwrap();
    assert_eq!(next, "P0\nxo\n..");
}

#[test]
fn successor_perfect_recall() {
    let parsed = parse_info_state("P0\n..\n..\n", 2, 2).unwrap();
    let next = info_state_after_action(&parsed, 2, 0, true).unwrap();
    assert_eq!(next, "P0\n..\nx.\n0,2 ");
}

#[test]
fn successor_invalid_action() {
    let parsed = parse_info_state("P0\nx.\n..", 2, 2).unwrap();
    // Cell 0 already occupied
    assert!(info_state_after_action(&parsed, 0, 0, false).is_err());
    // Out of bounds
    assert!(info_state_after_action(&parsed, 99, 0, false).is_err());
}

#[test]
fn terminal_by_connection_black_wins() {
    // 2x2: Black has column 0 (cells 0, 2) → connects North-South
    let parsed = parse_info_state("P1\nx.\nxo", 2, 2).unwrap();
    assert!(is_info_state_terminal(&parsed));
}

#[test]
fn terminal_by_connection_white_wins() {
    // 2x2: White has row 1 (cells 2, 3) → connects West-East
    let parsed = parse_info_state("P0\nx.\noo", 2, 2).unwrap();
    assert!(is_info_state_terminal(&parsed));
}

#[test]
fn not_terminal_partial() {
    let parsed = parse_info_state("P0\nx.\n.o", 2, 2).unwrap();
    assert!(!is_info_state_terminal(&parsed));
}

#[test]
fn terminal_by_piece_count_black() {
    // Player 0 (Black): terminal when white_count + empty_count == black_count
    // 2x3 board: Black has 3 stones, White has 1, empty has 2 → 1 + 2 == 3 ✓
    let parsed = parse_info_state("P0\nxx.\nx.o", 2, 3).unwrap();
    // Check: not a win by connection (Black doesn't connect N-S on 2x3 with cells 0,1,3)
    // But piece count: black=3, white=1, empty=2 → 1+2=3 → terminal
    assert!(is_info_state_terminal(&parsed));
}

#[test]
fn terminal_by_piece_count_white() {
    // Player 1 (White): terminal when black_count + empty_count == white_count + 1
    // 2x2 board: Black=1, White=1, empty=2 → 1+2=1+1=2? No: 3 ≠ 2
    let parsed = parse_info_state("P1\nx.\n.o", 2, 2).unwrap();
    assert!(!is_info_state_terminal(&parsed));

    // 2x2: Black=2, White=1, empty=1 → 2+1=1+1=2? No: 3 ≠ 2
    // Actually: black_count + empty_count = 2+1 = 3, white_count + 1 = 2. Not terminal.
    let parsed = parse_info_state("P1\nxx\n.o", 2, 2).unwrap();
    assert!(!is_info_state_terminal(&parsed));

    // 2x2: Black=1, White=2, empty=1 → 1+1=2+1=3? No: 2 ≠ 3
    // Actually let's construct a proper terminal case for White on 2x3:
    // White=3, Black=2, empty=1 → black+empty = 2+1 = 3 = white+1? 3=3+1=4? No
    // White=2, Black=2, empty=2 → 2+2=4, 2+1=3. No.
    // White=2, Black=1, empty=3 → 1+3=4, 2+1=3. No.
    // For 2x2: White=2, Black=1, empty=1 → 1+1=2+1=3? 2≠3. No.
    // Actually the formula: terminal if black_count + empty_count == white_count + 1
    // 2x2: W=1, B=2, E=1 → B+E=3, W+1=2. No.
    // 2x2: W=2, B=1, E=1 → B+E=2, W+1=3. No.
    // 2x3: W=3, B=3, E=0 → B+E=3, W+1=4. No (but board is full, should be terminal by connection)
    // 2x3: W=2, B=3, E=1 → B+E=4, W+1=3. No.
    // Player 1 (White): terminal when black_count + empty_count == white_count + 1
    // On 1x3: W=1, B=1, E=1 → B+E=2, W+1=2. Terminal!
    let parsed = parse_info_state("P1\nxo.", 1, 3).unwrap();
    assert!(is_info_state_terminal(&parsed));
}

#[test]
fn not_terminal_empty_board() {
    let parsed = parse_info_state("P0\n..\n..", 2, 2).unwrap();
    assert!(!is_info_state_terminal(&parsed));
}

#[test]
fn board_view_flat_conversion() {
    let parsed = parse_info_state("P0\nx.\n.o", 2, 2).unwrap();
    assert_eq!(board_view_flat(&parsed), vec![1, 0, 0, 2]);
}

#[test]
fn board_view_flat_empty() {
    let parsed = parse_info_state("P0\n..\n..", 2, 2).unwrap();
    assert_eq!(board_view_flat(&parsed), vec![0, 0, 0, 0]);
}

#[test]
fn initial_info_state_black() {
    let s = initial_info_state(2, 2, 0);
    assert_eq!(s, "P0\n..\n..");
}

#[test]
fn initial_info_state_white() {
    let s = initial_info_state(2, 3, 1);
    assert_eq!(s, "P1\n...\n...");
}

#[test]
fn strategy_generator_workflow_2x2() {
    // Simulate a mini strategy generator workflow on 2x2 board for Black (player 0).
    // Initial state: all empty, Black's turn, no collision possible.
    let info = initial_info_state(2, 2, 0);
    let parsed = parse_info_state(&info, 2, 2).unwrap();
    assert_eq!(legal_actions(&parsed), vec![0, 1, 2, 3]);
    assert!(!is_collision_possible(&parsed));

    // Black chooses action 0 (deterministic). Only one successor (no collision).
    let next = info_state_after_action(&parsed, 0, 0, false).unwrap();
    assert_eq!(next, "P0\nx.\n..");
    assert!(!is_info_state_terminal(
        &parse_info_state(&next, 2, 2).unwrap()
    ));

    // From "P0\nx.\n..", Black chooses action 2. Still no collision possible
    // (Black=1, White=0, 0 < 1 is false... wait)
    // Actually let me check: Black=1, White=0 → collision check: white < black → 0 < 1 → true!
    // Wait no — at "P0\nx.\n..", Black already has 1 stone. But White hasn't played yet
    // from Black's perspective. However, in the game tree, after Black places at 0,
    // White would have placed next. So from Black's next info state, White HAS placed
    // one stone (hidden). So collision IS possible.
    let parsed2 = parse_info_state(&next, 2, 2).unwrap();
    // Black=1, White=0 → white_count(0) < black_count(1) → true
    assert!(is_collision_possible(&parsed2));

    // Two branches for action 2:
    // Branch 1: placed (stone_player=0)
    let placed = info_state_after_action(&parsed2, 2, 0, false).unwrap();
    assert_eq!(placed, "P0\nx.\nx.");
    // Branch 2: collision (stone_player=1, discovers White at cell 2)
    let collision = info_state_after_action(&parsed2, 2, 1, false).unwrap();
    assert_eq!(collision, "P0\nx.\no.");
}

#[test]
fn parse_3x3_board() {
    let info = "P0\n...\n...\n...";
    let parsed = parse_info_state(info, 3, 3).unwrap();
    assert_eq!(parsed.board.len(), 9);
    assert_eq!(legal_actions(&parsed).len(), 9);
}
