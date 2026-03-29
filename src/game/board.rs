use crate::game::types::{Cell, Player};

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
#[path = "board_tests.rs"]
mod tests;
