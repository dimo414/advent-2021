use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::time::Duration;
use anyhow::{anyhow,Error,Result};
use advent_2021::euclid::{point, Point, Vector};
use advent_2021::console::{Color,Console};

fn main() -> Result<()> {
    let _console = Console::init();
    Console::colorize_char('0', Color::WHITE);
    for i in 1..=9 {
        // divide by 12 instead of 10 to make the white "blink" more distinct
        Console::colorize_char(char::from_digit(i, 10).unwrap(), Color::GREYSCALE(i as f32 / 12.0));
    }

    let mut octopi: Octopi = include_str!("input.txt").parse()?;

    let mut blinks = 0;
    for _ in 0..100 {
        octopi.half_step();
        Console::interactive_display(&octopi, Duration::from_millis(75));
        blinks += octopi.step();
        Console::interactive_display(&octopi, Duration::from_millis(150));
    }
    Console::clear_interactive();
    println!("Blinks after 100 generations: {}", blinks);

    let mut first_all_blink = None;
    for _ in 101..500 {
        let blinks = octopi.step();
        Console::interactive_display(&octopi, Duration::from_millis(75));
        if blinks == 100 && first_all_blink.is_none() {
            first_all_blink = Some(octopi.generation);
        }
    }
    Console::clear_interactive();
    println!("All blinked at generation:    {}", first_all_blink.expect("Insufficient generations"));

    Ok(())
}

#[derive(Clone)]
struct Octopi {
    grid: HashMap<Point, u32>,
    generation: u32,
    incomplete: bool,
}

impl Octopi {
    fn create(grid: HashMap<Point, u32>) -> Octopi {
        assert!(!grid.is_empty(), "Grid cannot be empty.");
        Octopi { grid, generation: 0, incomplete: false }
    }

    fn half_step(&mut self) {
        assert!(!self.incomplete, "Cannot take a half-step until the last step is completed.");
        self.generation += 1;
        self.incomplete = true;
        self.grid = self.grid.iter().map(|(&p, &v)| (p, v+1)).collect();
    }

    fn step(&mut self) -> u32 {
        if !self.incomplete {
            self.half_step();
        }
        self.incomplete = false;

        // TODO keeping a set of 'blinked' cells and updating a cells neighbors when it's inserted
        //   would be more efficient than the repeated passes being done here.
        let mut charging: HashSet<_> = self.grid.keys().cloned().collect();
        let mut finished = false;
        while !finished {
            finished = true;
            for point in charging.iter().cloned().collect::<Vec<_>>() {
                if self.grid[&point] > 9 {
                    finished = false;
                    charging.remove(&point);
                    for neighbor in Vector::ORDINAL.iter().map(|v| point + v) {
                        self.grid.entry(neighbor).and_modify(|v| *v+=1);
                    }
                }
            }
        }

        let mut blinks = 0;
        for point in self.grid.keys().cloned().collect::<Vec<_>>() {
            if self.grid[&point] > 9 {
                self.grid.insert(point, 0);
                blinks += 1;
            }
        }
        blinks
    }
}

impl PartialEq for Octopi {
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid
    }
}
impl Eq for Octopi {}

impl FromStr for Octopi {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = s.lines().enumerate()
            .flat_map(|(y,r)|
                r.chars().enumerate()
                    .map(move |(x, c)|
                        c.to_digit(10)
                            .ok_or(anyhow!("invalid digit"))
                            .map(|e| (point(x as i32, y as i32), e))))
            .collect::<Result<HashMap<_, _>>>()?;
        Ok(Octopi::create(grid))
    }
}

impl Display for Octopi {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // I also tried using braille to visualize the counts, which looked nice but doesn't work
        // well with the existing colorizing behavior of Console (it uses a solid block symbol for
        // char that gets colored). So for posterity here's digits as braille, 1-8:
        // ⡀ ⣀ ⣄ ⣤ ⣦ ⣶ ⣷ ⣿
        fn render_digit(d: Option<&u32>) -> String {
            match d {
                Some(d@1..=9) => char::from_digit(*d, 10).expect("valid digit"),
                Some(0|10..=18) => '0',
                Some(d) => panic!("Unexpected number {}", d),
                None => ' ',
            }.to_string()
        }

        write!(f, "{}", Point::display_point_map(&self.grid, render_digit))
    }
}

impl Debug for Octopi {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Generation: {}\n{}", self.generation,
               Point::display_point_map(&self.grid, |v|
                   v.map(|v| format!("{:2}", v)).unwrap_or_else(|| "  ".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    parameterized_test::create!{ steps, (generation, expected), {
        let mut octopi: Octopi = include_str!("example1.txt").parse().unwrap();
        let expected: Octopi = expected.parse().unwrap();
        let expected_blinks = expected.grid.values().filter(|&&e| e == 0).count() as u32;

        let mut last_blinks = None;
        for _ in 0..generation {
            last_blinks = Some(octopi.step());
        }

        assert_eq!(octopi, expected);
        assert_eq!(last_blinks.expect("should be set"), expected_blinks);
    } }
    steps! {
        g1:   (1,   include_str!("example1-1.txt")),
        g2:   (2,   include_str!("example1-2.txt")),
        g3:   (3,   include_str!("example1-3.txt")),
        g4:   (4,   include_str!("example1-4.txt")),
        g5:   (5,   include_str!("example1-5.txt")),
        g6:   (6,   include_str!("example1-6.txt")),
        g7:   (7,   include_str!("example1-7.txt")),
        g8:   (8,   include_str!("example1-8.txt")),
        g9:   (9,   include_str!("example1-9.txt")),
        g10:  (10,  include_str!("example1-10.txt")),
        g20:  (20,  include_str!("example1-20.txt")),
        g30:  (30,  include_str!("example1-30.txt")),
        g40:  (40,  include_str!("example1-40.txt")),
        g50:  (50,  include_str!("example1-50.txt")),
        g60:  (60,  include_str!("example1-60.txt")),
        g70:  (70,  include_str!("example1-70.txt")),
        g80:  (80,  include_str!("example1-80.txt")),
        g90:  (90,  include_str!("example1-90.txt")),
        g100: (100, include_str!("example1-100.txt")),
    }

    parameterized_test::create!{ total_blinks, (generation, expected), {
        let mut octopi: Octopi = include_str!("example1.txt").parse().unwrap();

        let mut total_blinks = 0;
        for _ in 0..generation {
            total_blinks += octopi.step();
        }
        assert_eq!(total_blinks, expected);
    } }
    total_blinks! {
        g10: (10, 204),
        g100: (100, 1656),
    }

    #[test]
    fn energized() {
        let mut octopi: Octopi = include_str!("example2.txt").parse().unwrap();

        let blinks1 = octopi.step();
        assert_eq!(octopi, include_str!("example2-1.txt").parse::<Octopi>().unwrap());
        assert_eq!(blinks1, 9);

        let blinks2 = octopi.step();
        assert_eq!(octopi, include_str!("example2-2.txt").parse::<Octopi>().unwrap());
        assert_eq!(blinks2, 0);
    }
}
