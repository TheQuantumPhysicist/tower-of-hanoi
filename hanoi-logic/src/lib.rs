/// Tower of Hanoi game logic.
///
/// This crate provides a pure game-logic implementation with no rendering
/// dependencies, so it can be reused with any graphics engine.

/// Represents one of the three pegs (columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Peg {
    Left,
    Middle,
    Right,
}

impl Peg {
    pub const ALL: [Peg; 3] = [Peg::Left, Peg::Middle, Peg::Right];

    /// Return the peg corresponding to a 0-based index.
    pub fn from_index(i: usize) -> Option<Peg> {
        match i {
            0 => Some(Peg::Left),
            1 => Some(Peg::Middle),
            2 => Some(Peg::Right),
            _ => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Peg::Left => 0,
            Peg::Middle => 1,
            Peg::Right => 2,
        }
    }
}

/// A single move: take the top disk from `from` and place it on `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Peg,
    pub to: Peg,
}

/// Errors that can occur when attempting a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The source peg has no disks.
    EmptySource,
    /// Cannot place a larger disk on a smaller one.
    InvalidPlacement,
    /// Source and destination are the same peg.
    SamePeg,
}

/// The main game state.
///
/// Disks are represented by their size (1 = smallest). Each peg holds a
/// `Vec<u8>` where index 0 is the bottom disk.
#[derive(Debug, Clone)]
pub struct HanoiGame {
    pegs: [Vec<u8>; 3],
    num_disks: u8,
    move_count: u32,
}

impl HanoiGame {
    /// Create a new game with `n` disks, all on the left peg.
    pub fn new(n: u8) -> Self {
        assert!(n > 0 && n <= 20, "disk count must be 1..=20");
        let mut left = Vec::with_capacity(n as usize);
        for size in (1..=n).rev() {
            left.push(size);
        }
        HanoiGame {
            pegs: [left, Vec::new(), Vec::new()],
            num_disks: n,
            move_count: 0,
        }
    }

    /// Number of disks in this game.
    pub fn num_disks(&self) -> u8 {
        self.num_disks
    }

    /// Number of moves made so far.
    pub fn move_count(&self) -> u32 {
        self.move_count
    }

    /// The minimum number of moves to solve from the initial state.
    pub fn minimum_moves(&self) -> u32 {
        (1u32 << self.num_disks) - 1
    }

    /// Get the disks on a peg (bottom to top).
    pub fn disks_on(&self, peg: Peg) -> &[u8] {
        &self.pegs[peg.index()]
    }

    /// The top disk on a peg, if any.
    pub fn top_disk(&self, peg: Peg) -> Option<u8> {
        self.pegs[peg.index()].last().copied()
    }

    /// Check whether the game is solved (all disks on the right peg).
    pub fn is_solved(&self) -> bool {
        self.pegs[Peg::Right.index()].len() == self.num_disks as usize
    }

    /// Try to move the top disk from `from` to `to`.
    pub fn make_move(&mut self, m: Move) -> Result<u8, MoveError> {
        if m.from == m.to {
            return Err(MoveError::SamePeg);
        }
        let disk = *self.pegs[m.from.index()]
            .last()
            .ok_or(MoveError::EmptySource)?;
        if let Some(&top) = self.pegs[m.to.index()].last() {
            if disk > top {
                return Err(MoveError::InvalidPlacement);
            }
        }
        self.pegs[m.from.index()].pop();
        self.pegs[m.to.index()].push(disk);
        self.move_count += 1;
        Ok(disk)
    }

    /// Check if a move is valid without performing it.
    pub fn is_valid_move(&self, m: Move) -> bool {
        if m.from == m.to {
            return false;
        }
        let Some(&disk) = self.pegs[m.from.index()].last() else {
            return false;
        };
        if let Some(&top) = self.pegs[m.to.index()].last() {
            if disk > top {
                return false;
            }
        }
        true
    }

    /// Reset the game to the starting position with the same number of disks.
    pub fn reset(&mut self) {
        *self = Self::new(self.num_disks);
    }

    /// Reset the game with a new number of disks.
    pub fn reset_with(&mut self, n: u8) {
        *self = Self::new(n);
    }
}

/// Compute the full sequence of moves to solve Tower of Hanoi from `source`
/// to `target` using `aux` as the auxiliary peg, for `n` disks.
pub fn solve(n: u8, source: Peg, target: Peg, aux: Peg) -> Vec<Move> {
    let mut moves = Vec::new();
    solve_recursive(n, source, target, aux, &mut moves);
    moves
}

fn solve_recursive(n: u8, source: Peg, target: Peg, aux: Peg, moves: &mut Vec<Move>) {
    if n == 0 {
        return;
    }
    solve_recursive(n - 1, source, aux, target, moves);
    moves.push(Move {
        from: source,
        to: target,
    });
    solve_recursive(n - 1, aux, target, source, moves);
}

/// Compute remaining moves to solve the game from its current state.
pub fn solve_from_current(game: &HanoiGame) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut state = game.clone();
    let n = state.num_disks();
    move_disks_to_target(&mut state, n, Peg::Right, &mut moves);
    moves
}

/// Recursively move disks 1..=n to `target` peg, recording the moves.
fn move_disks_to_target(state: &mut HanoiGame, n: u8, target: Peg, moves: &mut Vec<Move>) {
    if n == 0 {
        return;
    }

    let disk_peg = find_disk(state, n);

    if disk_peg == target {
        move_disks_to_target(state, n - 1, target, moves);
        return;
    }

    let aux = other_peg(disk_peg, target);
    move_disks_to_target(state, n - 1, aux, moves);

    let m = Move {
        from: disk_peg,
        to: target,
    };
    state.make_move(m).expect("solver produced invalid move");
    moves.push(m);

    move_disks_to_target(state, n - 1, target, moves);
}

fn find_disk(state: &HanoiGame, disk: u8) -> Peg {
    for peg in Peg::ALL {
        if state.disks_on(peg).contains(&disk) {
            return peg;
        }
    }
    panic!("disk {} not found", disk);
}

fn other_peg(a: Peg, b: Peg) -> Peg {
    for p in Peg::ALL {
        if p != a && p != b {
            return p;
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_initial_state() {
        let game = HanoiGame::new(3);
        assert_eq!(game.disks_on(Peg::Left), &[3, 2, 1]);
        assert!(game.disks_on(Peg::Middle).is_empty());
        assert!(game.disks_on(Peg::Right).is_empty());
        assert_eq!(game.move_count(), 0);
        assert!(!game.is_solved());
    }

    #[test]
    fn valid_move() {
        let mut game = HanoiGame::new(3);
        let result = game.make_move(Move {
            from: Peg::Left,
            to: Peg::Right,
        });
        assert_eq!(result, Ok(1));
        assert_eq!(game.disks_on(Peg::Left), &[3, 2]);
        assert_eq!(game.disks_on(Peg::Right), &[1]);
        assert_eq!(game.move_count(), 1);
    }

    #[test]
    fn invalid_placement() {
        let mut game = HanoiGame::new(3);
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Right,
        })
        .unwrap();
        let result = game.make_move(Move {
            from: Peg::Left,
            to: Peg::Right,
        });
        assert_eq!(result, Err(MoveError::InvalidPlacement));
    }

    #[test]
    fn empty_source() {
        let mut game = HanoiGame::new(3);
        let result = game.make_move(Move {
            from: Peg::Right,
            to: Peg::Left,
        });
        assert_eq!(result, Err(MoveError::EmptySource));
    }

    #[test]
    fn same_peg() {
        let mut game = HanoiGame::new(3);
        let result = game.make_move(Move {
            from: Peg::Left,
            to: Peg::Left,
        });
        assert_eq!(result, Err(MoveError::SamePeg));
    }

    #[test]
    fn solve_1_disk() {
        let moves = solve(1, Peg::Left, Peg::Right, Peg::Middle);
        assert_eq!(moves.len(), 1);
        let mut game = HanoiGame::new(1);
        for m in moves {
            game.make_move(m).unwrap();
        }
        assert!(game.is_solved());
    }

    #[test]
    fn solve_3_disks() {
        let moves = solve(3, Peg::Left, Peg::Right, Peg::Middle);
        assert_eq!(moves.len(), 7);
        let mut game = HanoiGame::new(3);
        for m in moves {
            game.make_move(m).unwrap();
        }
        assert!(game.is_solved());
    }

    #[test]
    fn solve_5_disks() {
        let moves = solve(5, Peg::Left, Peg::Right, Peg::Middle);
        assert_eq!(moves.len(), 31);
        let mut game = HanoiGame::new(5);
        for m in moves {
            game.make_move(m).unwrap();
        }
        assert!(game.is_solved());
    }

    #[test]
    fn solve_optimal_move_count() {
        for n in 1..=10 {
            let moves = solve(n, Peg::Left, Peg::Right, Peg::Middle);
            assert_eq!(moves.len(), (1usize << n) - 1, "optimal for {} disks", n);
        }
    }

    #[test]
    fn solve_from_current_fresh_game() {
        let game = HanoiGame::new(4);
        let moves = solve_from_current(&game);
        let mut g = game.clone();
        for m in &moves {
            g.make_move(*m).unwrap();
        }
        assert!(g.is_solved());
    }

    #[test]
    fn solve_from_current_mid_game() {
        let mut game = HanoiGame::new(3);
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Right,
        })
        .unwrap();
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Middle,
        })
        .unwrap();

        let moves = solve_from_current(&game);
        let mut g = game.clone();
        for m in &moves {
            g.make_move(*m).unwrap();
        }
        assert!(g.is_solved());
    }

    #[test]
    fn solve_from_current_already_solved() {
        let mut game = HanoiGame::new(2);
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Middle,
        })
        .unwrap();
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Right,
        })
        .unwrap();
        game.make_move(Move {
            from: Peg::Middle,
            to: Peg::Right,
        })
        .unwrap();
        assert!(game.is_solved());

        let moves = solve_from_current(&game);
        assert!(moves.is_empty());
    }

    #[test]
    fn reset_game() {
        let mut game = HanoiGame::new(3);
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Right,
        })
        .unwrap();
        game.reset();
        assert_eq!(game.disks_on(Peg::Left), &[3, 2, 1]);
        assert_eq!(game.move_count(), 0);
    }

    #[test]
    fn reset_with_different_count() {
        let mut game = HanoiGame::new(3);
        game.reset_with(5);
        assert_eq!(game.num_disks(), 5);
        assert_eq!(game.disks_on(Peg::Left), &[5, 4, 3, 2, 1]);
    }

    #[test]
    fn is_valid_move_check() {
        let game = HanoiGame::new(3);
        assert!(game.is_valid_move(Move {
            from: Peg::Left,
            to: Peg::Right
        }));
        assert!(!game.is_valid_move(Move {
            from: Peg::Right,
            to: Peg::Left
        }));
        assert!(!game.is_valid_move(Move {
            from: Peg::Left,
            to: Peg::Left
        }));
    }

    #[test]
    fn peg_from_index() {
        assert_eq!(Peg::from_index(0), Some(Peg::Left));
        assert_eq!(Peg::from_index(1), Some(Peg::Middle));
        assert_eq!(Peg::from_index(2), Some(Peg::Right));
        assert_eq!(Peg::from_index(3), None);
    }

    #[test]
    fn solve_from_scattered_state() {
        let mut game = HanoiGame::new(4);
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Middle,
        })
        .unwrap();
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Right,
        })
        .unwrap();
        game.make_move(Move {
            from: Peg::Middle,
            to: Peg::Right,
        })
        .unwrap();
        game.make_move(Move {
            from: Peg::Left,
            to: Peg::Middle,
        })
        .unwrap();

        let moves = solve_from_current(&game);
        let mut g = game.clone();
        for m in &moves {
            g.make_move(*m).unwrap();
        }
        assert!(g.is_solved());
    }

    #[test]
    fn minimum_moves() {
        let game = HanoiGame::new(3);
        assert_eq!(game.minimum_moves(), 7);
        let game = HanoiGame::new(5);
        assert_eq!(game.minimum_moves(), 31);
    }
}
