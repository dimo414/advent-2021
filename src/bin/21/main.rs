use std::collections::HashMap;

fn main() {
    let input = [10, 7];
    let mut board = Board::create(input, 1000);
    let (winner, rolls) = play_with_fake_die(&mut board);
    println!("Winner: Player {} - Rolls: {} - Scores: {:?}", winner+1, rolls, board.scores);
    println!("Loser's Score * Rolls: {}", rolls * board.scores[(winner + 1) % board.scores.len()]);

    let mut universes = Universes::create(input);
    while !universes.roll() {}
    let (winner, count) = universes.wins.iter().enumerate().max_by_key(|(_, c)| **c).expect("Non-empty");
    println!("Player {} wins in {} universes", winner, count);
}

fn play_with_fake_die(board: &mut Board) -> (usize, u32) {
    // Actual die is 100, but since the board is 10 long the 10s place is irrelevant
    let fake_die = &mut (1..=10).cycle();
    let mut rolls = 0;
    loop {
        rolls += 3;
        if let Some(winner) = board.turn(fake_die.take(3).sum()) {
            return (winner, rolls);
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct Board {
    positions: [u8;2],
    target_score: u32,
    scores: [u32;2],
    cur_player: usize,
}

impl Board {
    fn create(positions: [u8;2], target_score: u32) -> Board {
        Board { positions, target_score, scores: [0,0], cur_player: 0, }
    }

    fn turn(&mut self, steps: u8) -> Option<usize> {
        self.positions[self.cur_player] = (self.positions[self.cur_player] + steps - 1) % 10 + 1;
        self.scores[self.cur_player] += self.positions[self.cur_player] as u32;
        if self.scores[self.cur_player] >= self.target_score { return Some(self.cur_player); }

        self.cur_player = (self.cur_player + 1) % self.positions.len();

        None
    }
}

#[derive(Debug)]
struct Universes {
    // We could reduce how much we're storing by just storing player positions and scores (and
    // tracking the current player globally) but reusing the Board is convenient.
    boards: HashMap<Board, u64>,
    wins: [u64; 2],
}

impl Universes {
    fn create(players: [u8; 2]) -> Universes {
        Universes{ boards: [(Board::create(players, 21), 1)].iter().cloned().collect(), wins: [0; 2], }
    }

    // 3d3 can roll the values 3-9 with the following frequency: 1,3,6,7,6,3,1 (27 possible rolls)
    fn roll(&mut self) -> bool {
        let mut next_iter = HashMap::new();
        for (board, u_freq) in &self.boards {
            for (roll, r_freq) in [(3,1), (4,3), (5,6), (6,7), (7,6), (8,3), (9,1)] {
                let mut board = *board;
                let freq = u_freq * r_freq;
                match board.turn(roll) {
                    Some(winner) => self.wins[winner] += freq,
                    None => {
                        *next_iter.entry(board).or_insert(0) += freq;
                    },
                }
            }
        }
        self.boards = next_iter;
        self.boards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_game() {
        let mut board = Board::create([4, 8], 1000);
        let (winner, rolls) = play_with_fake_die(&mut board);
        assert_eq!(winner, 0);
        assert_eq!(board.scores, [1000,745]);
        assert_eq!(rolls, 993);
    }

    #[test]
    fn quantum() {
        let mut universes = Universes::create([4, 8]);
        while !universes.roll() {}
        assert_eq!(universes.wins, [444356092776315, 341960390180808]);
    }
}
