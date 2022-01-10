use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;
use anyhow::{Result, Error, bail};

use advent_2021::pathfinding::{Graph, Edge};
use advent_2021::terminal::{elapsed,Terminal};

// Credit to https://github.com/githuib/AdventOfCode/blob/master/year2021/day23/__init__.py for
// some of the equations used below.

fn main() -> Result<()> {
    let _drop = Terminal::init();
    let burrow: Burrow = include_str!("input.txt").parse()?;

    let route = elapsed!(burrow.use_a_star(|b| b.heuristic_distance()).unwrap());
    display_route(&route);
    let cost = route.iter().map(|e| e.weight()).sum::<i32>();
    compare_algorithms(&burrow, cost);
    Terminal::end_interactive();
    println!("Energy required for the initial burrow:  {}", cost);

    let burrow: Burrow = unfold_input(include_str!("input.txt")).parse()?;
    let route = elapsed!(burrow.use_a_star(|b| b.heuristic_distance()).unwrap());
    display_route(&route);
    let cost = route.iter().map(|e| e.weight()).sum::<i32>();
    compare_algorithms(&burrow, cost);
    println!("Energy required for the unfolded burrow: {}", cost);

    Ok(())
}

fn display_route(route: &[Edge<Burrow>]) {
    if Terminal::active() {
        Terminal::interactive_display(route[0].source(), Duration::from_millis(500));
        for edge in route {
            Terminal::interactive_display(edge.dest(), Duration::from_millis(500));
        }
    }
}

#[cfg(feature="timing")]
fn compare_algorithms(burrow: &Burrow, expected_cost: i32) {
    let ast = elapsed!(burrow.use_a_star(|_| 0).unwrap());
    let ast_cost = ast.iter().map(|e| e.weight()).sum::<i32>();
    assert_eq!(ast_cost, expected_cost, "A* (with no heuristic) found a different cost!");

    let ast = elapsed!(burrow.use_a_star(|b| b.simple_heuristic_distance()).unwrap());
    let ast_cost = ast.iter().map(|e| e.weight()).sum::<i32>();
    assert_eq!(ast_cost, expected_cost, "A* (with simple heuristic) found a different cost!");

    let djk = elapsed!(burrow.use_dijkstras().unwrap());
    let djk_cost = djk.iter().map(|e| e.weight()).sum::<i32>();
    assert_eq!(djk_cost, expected_cost, "Dijkstra's found a different cost!");
}
#[cfg(not(feature="timing"))] #[inline]
fn compare_algorithms(_: &Burrow, _: i32) {}

fn unfold_input(input: &str) -> String {
    let mut lines: Vec<_> = input.lines().collect();
    let extra_lines = "  #D#C#B#A#\n  #D#B#A#C#";
    lines.insert(3, extra_lines);
    lines.join("\n")
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum Type {
    A, B, C, D,
}

impl Type {
    fn from_char(c: char) -> Result<Type> {
        Ok(match c {
            'A' => Type::A,
            'B' => Type::B,
            'C' => Type::C,
            'D' => Type::D,
            _ => bail!("Unexpected char: {:?}", c),
        })
    }

    fn as_char(&self) -> char {
        match self {
            Type::A => 'A',
            Type::B => 'B',
            Type::C => 'C',
            Type::D => 'D',
        }
    }

    fn home_room_index(&self) -> usize {
        match self {
            Type::A => 0,
            Type::B => 1,
            Type::C => 2,
            Type::D => 3,
        }
    }

    fn energy(&self) -> i32 {
        match self {
            Type::A => 1,
            Type::B => 10,
            Type::C => 100,
            Type::D => 1000,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct Room([Option<Type>; 4]);

impl Room {
    fn is_all(&self, t: Type) -> bool {
        self.0.iter().all(|c| c.is_none() || c.expect("Not-none") == t)
    }

    fn pop(&mut self) -> Option<(i32, Type)> {
        for i in 0..4 {
            if self.0[i].is_some() {
                let mut ret = None;
                std::mem::swap(&mut ret, &mut self.0[i]);
                return ret.map(|t| (i as i32, t));
            }
        }
        None
    }

    // Note this essentially jumps over existing members, but since it only allows jumping over
    // members of the same Type it's equivalent to the existing member moving down and the new
    // member taking its place.
    fn push(&mut self, t: Type) -> i32 {
        for i in 0..4 {
            if self.0[i].is_none() {
                self.0[i] = Some(t);
                return i as i32;
            }
            debug_assert_eq!(self.0[i].expect("Presence already checked"), t, "Cannot push different types");
        }
        panic!("Overflowed room")
    }
}

#[derive(Copy, Clone, Debug, Eq)]
struct Burrow {
    hallway: [Option<Type>; 7],
    rooms: [Room; 4],
    room_size: u8,
}

impl Burrow {
    fn create(rooms: [Room; 4], room_size: u8) -> Burrow {
        Burrow { hallway: [None; 7], rooms, room_size, }
    }

    fn is_arranged(&self) -> bool {
        self.hallway.iter().all(|c| c.is_none()) &&
            self.rooms.iter().enumerate().all(|(i, r)|
                r.0.iter().flatten().all(|c| c.home_room_index() == i))
    }

    #[cfg(feature="timing")]
    fn simple_heuristic_distance(&self) -> i32 {
        // 2x energy for every element in the hallway
        let hallway_cost = self.hallway.iter().flatten().map(|t| t.energy()*2).sum::<i32>();
        // 4x energy for every element in the wrong room
        let rooms_cost = self.rooms.iter().enumerate().flat_map(|(i, r)|
            r.0.iter().flatten()
                .filter(move |t| t.home_room_index() != i)
                .map(|t| t.energy()*4))
            .sum::<i32>();
        hallway_cost + rooms_cost
    }

    fn heuristic_distance(&self) -> i32 {
        // Distance to home room * energy
        let hallway_cost = self.hallway.iter().enumerate()
            .flat_map(|(i, t)| t.map(|v| (i, v)))
            .map(|(i, t)| self.hallway_distance(t.home_room_index(), i) * t.energy()).sum::<i32>();
        // Minimum four steps from room to room
        let rooms_cost = self.rooms.iter().enumerate().flat_map(|(i, r)|
            r.0.iter().flatten()
                .filter(move |t| t.home_room_index() != i)
                .map(|t| 4 * t.energy()))
            .sum::<i32>();
        hallway_cost + rooms_cost
    }

    // Checks if all hall spaces _between_ the room and the given hall space are clear. Does _not_
    // check if either end is clear (since callers can be moving in either direction). Callers must
    // check their destination separately.
    fn can_move(&self, room: usize, hall: usize) -> bool {
        // 01 2 3 4 56
        //   0 1 2 3
        if hall < room+1 {
            self.hallway[hall+1..room+2].iter().all(|c| c.is_none())
        } else if hall > room+2 {
            self.hallway[room+2..hall].iter().all(|c| c.is_none())
        } else {
            true
        }
    }

    fn hallway_distance(&self, room: usize, hall: usize) -> i32 {
        static HALLWAY_REAL: [i32; 7] = [0,1,3,5,7,9,10];
        let hall = HALLWAY_REAL[hall];
        let room = ((room+1)*2) as i32;
        (hall-room).abs()
    }

    #[cfg(any(test,feature="timing"))]
    fn use_dijkstras(&self) -> Option<Vec<Edge<Burrow>>> {
        self.dijkstras(
            &Rc::new(*self),
            |n| n.is_arranged())
    }

    fn use_a_star(&self, heuristic: impl Fn(&Burrow) -> i32) -> Option<Vec<Edge<Burrow>>> {
        self.a_star(
            &Rc::new(*self),
            |n| n.is_arranged(),
            heuristic)
    }
}

// Using a custom Hash implementation is significantly (2-3x) faster than the derived impl.
impl Hash for Burrow {
    fn hash<H>(&self, state: &mut H)
        where H: Hasher {
        assert_eq!(std::mem::size_of::<Self>(), 24);
        // wrong if there's any padding at all, which I don't think there should be
        unsafe {
            (*(self as *const Self as *const [u8; 24])).hash(state)
        }
    }
}

// This is incorrect, given the custom Hash impl, but at least gets us the right answer. Trying to
// derive PartialEq hits https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq
impl PartialEq for Burrow {
    fn eq(&self, other: &Burrow) -> bool {
        self.hallway == other.hallway && self.rooms == other.rooms && self.room_size == other.room_size
    }
}

impl Graph for Burrow {
    type Node = Burrow;

    /// * Amphipods can move up, down, left, or right so long as they are moving into an unoccupied
    ///   open space.
    /// * Each type of amphipod requires a different amount of energy to move one step.
    /// * Amphipods will never stop on the space immediately outside any room.
    /// * Amphipods will never move from the hallway into a room [other than their destination, and
    ///   only if there are no other amphipod types in the room].
    /// * Once an amphipod stops moving in the hallway, it will stay in that spot until it can move
    ///   into a room. Once any amphipod starts moving, any other amphipods currently in the hallway
    ///   are locked in place.
    fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
        let mut ret = Vec::new();
        // First member in each room can leave
        for r in 0..4 {
            let mut next = *source;
            if let Some((room_steps, t)) = next.rooms[r].pop() {
                let room_steps = room_steps + 1;
                for h in 0..next.hallway.len() {
                    if next.hallway[h].is_none() && next.can_move(r, h) {
                        let mut next = next; // copy
                        next.hallway[h] = Some(t);
                        let weight = t.energy() * (source.hallway_distance(r, h) + room_steps);
                        ret.push(Edge::new(weight, *source, next));
                    }
                }
            }
        }

        // Each member in the hallway can go to their home
        for (h, t) in source.hallway.iter().enumerate().flat_map(|(h,t)| t.map(|t| (h, t))) {
            let r = t.home_room_index();
            if source.rooms[r].is_all(t) && source.can_move(r, h) {
                let mut next = *source;
                next.hallway[h] = None;
                let room_steps = next.rooms[r].push(t) + 1;
                let weight = t.energy() * (source.hallway_distance(r, h) + room_steps);
                ret.push(Edge::new(weight, *source, next));
            }
        }

        ret
    }
}

impl FromStr for Burrow {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rooms = [Room([None; 4]); 4];
        let lines: Vec<_> = s.lines().collect();
        for (row, chars) in lines[2..lines.len()-1].iter().enumerate() {
            for (col, c) in chars.chars().enumerate() {
                match col {
                    3|5|7|9 => rooms[col/2-1].0[row] = Some(Type::from_char(c)?),
                    _ => {},
                };
            }
        }
        Ok(Burrow::create(rooms, lines.len() as u8 - 3))
    }
}

impl std::fmt::Display for Burrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn to_letter(t: Option<Type>) -> char {
            t.map(|t| t.as_char()).unwrap_or('.')
        }
        writeln!(f, "{}", "#".repeat(13))?;
        writeln!(f, "#{}{}{}#",
                 to_letter(self.hallway[0]),
                 self.hallway[1..6].iter().map(|a|to_letter(*a).to_string()).collect::<Vec<_>>().join("."),
                 to_letter(self.hallway[6]))?;
        for row in 0..self.room_size as usize {
            let row_text = (0..4)
                .map(|r| to_letter(*self.rooms[r].0.get(row).unwrap_or(&None)).to_string())
                .collect::<Vec<_>>().join("#");
            writeln!(f, "{}#{}#{}", if row==0 {"##"}else{"  "}, row_text, if row==0 {"##"}else{""})?;
        }

        write!(f, "  {}", "#".repeat(9))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_round_trips() {
        let a: Burrow = include_str!("example.txt").parse().unwrap();
        assert_eq!(a.to_string(), include_str!("example.txt"));
        let b: Burrow = a.to_string().parse().unwrap();
        assert_eq!(a, b);

        let a: Burrow = unfold_input(include_str!("example.txt")).parse().unwrap();
        assert_eq!(a.to_string(), unfold_input(include_str!("example.txt")));
        let b: Burrow = a.to_string().parse().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn neighbors() {
        let a: Burrow = include_str!("example.txt").parse().unwrap();
        let next = a.neighbors(&a);
        assert_eq!(next.len(), 28, "{:?}", next);
    }

    #[test]
    fn can_move() {
        let a: Burrow = include_str!("example.txt").parse().unwrap();
        for r in 0..4 {
            for h in 0..7 {
                assert!(a.can_move(r, h), "Can't move from {} to {}", r, h);
            }
        }
    }

    #[test]
    fn path_between() {
        let burrow: Burrow = include_str!("example.txt").parse().unwrap();
        assert!(burrow.can_move(3, 2));

        let mut burrow = burrow;
        let (_, d) = burrow.rooms[3].pop().unwrap();
        let (_, a) = burrow.rooms[3].pop().unwrap();
        let (_, b) = burrow.rooms[0].pop().unwrap();
        assert_eq!(d, Type::D);
        assert_eq!(a, Type::A);
        assert_eq!(b, Type::B);
        burrow.hallway = [None, None, Some(b), None, Some(a), Some(d), None];
        assert!(!burrow.can_move(3, 2));
    }

    #[test]
    fn example1_dijkstras() {
        let burrow: Burrow = include_str!("example.txt").parse().unwrap();
        let djk = burrow.use_dijkstras().unwrap();
        assert_eq!(djk.len(), 12);
        let djk_cost = djk.iter().map(|e| e.weight()).sum::<i32>();
        assert_eq!(djk_cost, 12521);
    }

    #[test]
    fn example1_a_star() {
        let burrow: Burrow = include_str!("example.txt").parse().unwrap();
        let djk = burrow.use_a_star(|b| b.heuristic_distance()).unwrap();
        assert_eq!(djk.len(), 12);
        let djk_cost = djk.iter().map(|e| e.weight()).sum::<i32>();
        assert_eq!(djk_cost, 12521);
    }

    #[cfg(not(debug_assertions))] // Pretty slow without --release, and example1 gives reasonable coverage
    #[test]
    fn example2_dijkstras() {
        let burrow: Burrow = unfold_input(include_str!("example.txt")).parse().unwrap();
        let djk = burrow.use_dijkstras().unwrap();
        assert_eq!(djk.len(), 28); // I count 24 steps in the example, but the cost is what really matters anyways
        let djk_cost = djk.iter().map(|e| e.weight()).sum::<i32>();
        assert_eq!(djk_cost, 44169);
    }

    #[cfg(not(debug_assertions))] // Pretty slow without --release, and example1 gives reasonable coverage
    #[test]
    fn example2_a_star() {
        let burrow: Burrow = unfold_input(include_str!("example.txt")).parse().unwrap();
        let djk = burrow.use_a_star(|b| b.heuristic_distance()).unwrap();
        assert_eq!(djk.len(), 28); // I count 24 steps in the example, but the cost is what really matters anyways
        let djk_cost = djk.iter().map(|e| e.weight()).sum::<i32>();
        assert_eq!(djk_cost, 44169);
    }
}
