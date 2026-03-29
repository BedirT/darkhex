use super::*;

#[test]
fn union_find_basics() {
    let mut uf = UnionFind::new(5);
    assert!(!uf.connected(0, 4));
    uf.union(0, 1);
    uf.union(1, 2);
    uf.union(3, 4);
    assert!(uf.connected(0, 2));
    assert!(!uf.connected(0, 3));
    uf.union(2, 3);
    assert!(uf.connected(0, 4));
}

#[test]
fn board_2x2_neighbors() {
    let b = HexBoard::new(2, 2);
    // Cell 0 (0,0): neighbors are 1 (right), 2 (below)
    let n = b.neighbors(0);
    assert_eq!(n.len(), 2);
    assert!(n.contains(&1));
    assert!(n.contains(&2));
    // Cell 3 (1,1): neighbors are 2 (left), 0 (above), 1 (above-right)
    // Wait: (1,1) -> row=1,col=1
    // right: col+1=2 >= cols=2, no
    // left: col-1=0 >= 0, yes -> pos(1,0)=2
    // below: row+1=2 >= rows=2, no
    // above: row-1=0, yes -> pos(0,1)=1
    //   above-right: col+1=2 >= cols=2, no
    let n = b.neighbors(3);
    assert_eq!(n.len(), 2);
    assert!(n.contains(&2));
    assert!(n.contains(&1));
}

#[test]
fn board_2x2_black_wins_vertical() {
    // 2x2 board:
    //   0 1
    //   2 3
    // Black connects North-South: needs 0→2 or 1→3
    let mut b = HexBoard::new(2, 2);
    b.place_stone(0, Player::Black);
    assert_eq!(b.winner(), None);
    b.place_stone(1, Player::White);
    b.place_stone(2, Player::Black);
    assert_eq!(b.winner(), Some(Player::Black));
}

#[test]
fn board_2x2_white_wins_horizontal() {
    // White connects West-East: needs 0→1 or 2→3
    let mut b = HexBoard::new(2, 2);
    b.place_stone(0, Player::Black);
    b.place_stone(2, Player::White); // col 0
    b.place_stone(1, Player::Black);
    b.place_stone(3, Player::White); // col 1
    assert_eq!(b.winner(), Some(Player::White));
}

#[test]
fn collision_returns_false() {
    let mut b = HexBoard::new(2, 2);
    assert!(b.place_stone(0, Player::Black));
    assert!(!b.place_stone(0, Player::White));
    assert_eq!(b.num_stones, [1, 0]);
}
