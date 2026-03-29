use crate::types::{Cell, Player};

/// Disjoint-set / union-find with path halving and union by rank.
#[derive(Clone)]
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return;
        }
        if self.rank[rx] < self.rank[ry] {
            self.parent[rx] = ry;
        } else if self.rank[rx] > self.rank[ry] {
            self.parent[ry] = rx;
        } else {
            self.parent[ry] = rx;
            self.rank[rx] += 1;
        }
    }

    fn connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }
}

/// Hex board with union-find based win detection.
///
/// Virtual nodes after the cell indices:
///   `n+0` = Black North edge
///   `n+1` = Black South edge
///   `n+2` = White West edge
///   `n+3` = White East edge
///
/// Black wins by connecting North-South, White by connecting West-East.
#[derive(Clone)]
pub struct HexBoard {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Cell>,
    uf: UnionFind,
    pub num_stones: [usize; 2],
}

impl HexBoard {
    pub fn new(rows: usize, cols: usize) -> Self {
        let n = rows * cols;
        Self {
            rows,
            cols,
            cells: vec![Cell::Empty; n],
            uf: UnionFind::new(n + 4),
            num_stones: [0, 0],
        }
    }

    pub fn size(&self) -> usize {
        self.rows * self.cols
    }

    fn black_north(&self) -> usize {
        self.size()
    }
    fn black_south(&self) -> usize {
        self.size() + 1
    }
    fn white_west(&self) -> usize {
        self.size() + 2
    }
    fn white_east(&self) -> usize {
        self.size() + 3
    }

    pub fn row_col(&self, pos: usize) -> (usize, usize) {
        (pos / self.cols, pos % self.cols)
    }

    /// Return neighbour indices for a hex cell (up to 6).
    pub fn neighbors(&self, pos: usize) -> Vec<usize> {
        let (row, col) = self.row_col(pos);
        let mut out = Vec::with_capacity(6);
        if col + 1 < self.cols {
            out.push(row * self.cols + col + 1);
        }
        if col > 0 {
            out.push(row * self.cols + col - 1);
        }
        if row + 1 < self.rows {
            out.push((row + 1) * self.cols + col);
            if col > 0 {
                out.push((row + 1) * self.cols + col - 1);
            }
        }
        if row > 0 {
            out.push((row - 1) * self.cols + col);
            if col + 1 < self.cols {
                out.push((row - 1) * self.cols + col + 1);
            }
        }
        out
    }

    /// Place a stone on the board. Returns `true` on success, `false` on collision.
    pub fn place_stone(&mut self, pos: usize, player: Player) -> bool {
        if self.cells[pos] != Cell::Empty {
            return false;
        }
        let cell = Cell::from_player(player);
        self.cells[pos] = cell;
        self.num_stones[player.index()] += 1;

        // Union with same-colour neighbours
        for nbr in self.neighbors(pos) {
            if self.cells[nbr] == cell {
                self.uf.union(pos, nbr);
            }
        }

        // Union with edge virtual nodes
        let (row, col) = self.row_col(pos);
        match player {
            Player::Black => {
                if row == 0 {
                    self.uf.union(pos, self.black_north());
                }
                if row == self.rows - 1 {
                    self.uf.union(pos, self.black_south());
                }
            }
            Player::White => {
                if col == 0 {
                    self.uf.union(pos, self.white_west());
                }
                if col == self.cols - 1 {
                    self.uf.union(pos, self.white_east());
                }
            }
        }
        true
    }

    /// Check if a player has won (connected their edges).
    pub fn winner(&mut self) -> Option<Player> {
        if self.uf.connected(self.black_north(), self.black_south()) {
            Some(Player::Black)
        } else if self.uf.connected(self.white_west(), self.white_east()) {
            Some(Player::White)
        } else {
            None
        }
    }

    pub fn is_full(&self) -> bool {
        self.num_stones[0] + self.num_stones[1] == self.size()
    }
}

#[cfg(test)]
mod tests {
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
}
