use std::cell::Cell;
use anyhow::Result;

fn main() -> Result<()> {
    let (moves, mut boards) = parse_input(include_str!("input.txt"));

    let (best, worst) = play(&moves, &mut boards);
    println!("Best Board:  {}\nWorst Board: {}", best, worst);

    Ok(())
}

#[derive(Debug)]
struct Board {
    grid: Vec<u32>,
    seen: Vec<bool>,
    last_seen: Option<u32>,
    done: Cell<bool>,  // optimization to allow repeated is_done() calls
}

impl Board {
    fn create(grid: Vec<u32>) -> Board {
        assert_eq!(grid.len(), 25);
        Board { grid, seen: vec![false; 25], last_seen: None, done: Cell::new(false) }
    }

    fn visit(&mut self, square: u32) {
        self.last_seen = Some(square);
        if let Some(position) = self.grid.iter().position(|&s| s == square) {
            self.seen[position] = true;
        }
    }

    fn rows(&self) -> Vec<Vec<bool>> {
        // This _could_ return Vec<&[bool]> but we construct the inner Vec to be the same as cols()
        self.seen.chunks(5).map(|s| s.to_vec()).collect()
    }

    fn cols(&self) -> Vec<Vec<bool>> {
        let rows = self.rows();
        (0..rows[0].len())
            .map(|i| rows.iter().map(|v| v[i]).collect::<Vec<_>>())
            .collect()
    }

    fn is_done(&self) -> bool {
        if self.done.get() { return true }
        let done = self.rows().iter().chain(self.cols().iter()).any(|s| s.iter().all(|&v| v));
        self.done.set(done);
        done
    }

    fn score(&self) -> u32 {
        let unseen: u32 = self.seen.iter().enumerate().filter(|(_, &s)| !s).map(|(i, _)| self.grid[i]).sum();
        unseen * self.last_seen.expect("Cannot compute a score on an unvisited board.")
    }
}

fn play(moves: &[u32], boards: &mut [Board]) -> (u32, u32) {
    let mut completed = vec!();
    for square in moves {
        for (i, board) in boards.iter_mut().enumerate() {
            if board.is_done() { continue; }
            board.visit(*square);
            if board.is_done() {
                completed.push(i);
            }
        }
    }
    (boards[completed[0]].score(), boards[completed[completed.len()-1]].score())
}

fn parse_input(input: &str) -> (Vec<u32>, Vec<Board>) {
    let mut lines = input.lines();
    let moves = lines.next().unwrap().split(',').map(|n| n.parse().unwrap()).collect();
    lines.next().unwrap();

    let lines: Vec<_> = lines.collect();

    let mut boards = vec!();
    for i in 0..(lines.len()+1)/6 {
        let board: Board = Board::create(lines[i*6..(i*6)+5].join(" ").split_whitespace().map(|n| n.parse().unwrap()).collect());
        boards.push(board);
    }

    (moves, boards)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_example() {
        let (moves, mut boards) = parse_input(include_str!("example.txt"));
        let (best, worst) = play(&moves, &mut boards);
        assert_eq!(best, 4512);
        assert_eq!(worst, 1924);
    }
}
