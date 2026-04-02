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
    let n = b.neighbors(0);
    assert_eq!(n.len(), 2);
    assert!(n.contains(&1));
    assert!(n.contains(&2));
    let n = b.neighbors(3);
    assert_eq!(n.len(), 2);
    assert!(n.contains(&2));
    assert!(n.contains(&1));
}

#[test]
fn board_2x2_black_wins_vertical() {
    let mut b = HexBoard::new(2, 2);
    b.place_stone(0, Player::Black);
    assert_eq!(b.winner(), None);
    b.place_stone(1, Player::White);
    b.place_stone(2, Player::Black);
    assert_eq!(b.winner(), Some(Player::Black));
}

#[test]
fn board_2x2_white_wins_horizontal() {
    let mut b = HexBoard::new(2, 2);
    b.place_stone(0, Player::Black);
    b.place_stone(2, Player::White);
    b.place_stone(1, Player::Black);
    b.place_stone(3, Player::White);
    assert_eq!(b.winner(), Some(Player::White));
}

#[test]
fn collision_returns_false() {
    let mut b = HexBoard::new(2, 2);
    assert!(b.place_stone(0, Player::Black));
    assert!(!b.place_stone(0, Player::White));
    assert_eq!(b.num_stones, [1, 0]);
}
