use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use anyhow::{anyhow, bail, Error, Result};
use advent_2021::euclid::{point, Point, vector};
use advent_2021::terminal::{Color, Terminal, TerminalImage, TerminalRender};

fn main() -> Result<()> {
    let _drop = Terminal::init();
    let mut input: SeaFloor = include_str!("input.txt").parse()?;
    let mut count = 1;
    Terminal::interactive_render(&input, Duration::from_millis(100));
    while input.advance() {
        Terminal::interactive_render(&input, Duration::from_millis(10));
        count += 1;
    }
    println!("Stopped after {} iterations", count);

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum Cucumber { South, East, }

struct SeaFloor {
    cucumbers: HashMap<Point, Cucumber>,
    bound: Point, // min bound is the origin
}

impl SeaFloor {
    fn east_of(&self, pos: Point) -> Point {
        if pos.x == self.bound.x {
            return point(0, pos.y);
        }
        pos + vector(1, 0)
    }

    fn south_of(&self, pos: Point) -> Point {
        if pos.y == self.bound.y {
            return point(pos.x, 0);
        }
        pos + vector(0, 1)
    }

    fn advance(&mut self) -> bool {
        let mut something_moved = false;
        let mut east_moves = HashMap::new();
        for (pos, cuke) in &self.cucumbers {
            if let Cucumber::East = cuke {
                let next = self.east_of(*pos);
                if !self.cucumbers.contains_key(&next) {
                    east_moves.insert(next, *cuke);
                    something_moved = true;
                    continue;
                }
            }
            east_moves.insert(*pos, *cuke);
        }

        let mut south_moves = HashMap::new();
        for (pos, cuke) in &east_moves {
            if let Cucumber::South = cuke {
                let next = self.south_of(*pos);
                if !east_moves.contains_key(&next) {
                    south_moves.insert(next, *cuke);
                    something_moved = true;
                    continue;
                }
            }
            south_moves.insert(*pos, *cuke);
        }
        self.cucumbers = south_moves;

        something_moved
    }
}

impl std::fmt::Display for SeaFloor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Point::display_point_map(&self.cucumbers, |v|
            match v {
                Some(Cucumber::South) => 'v',
                Some(Cucumber::East) => '>',
                None => '.',
            }.into()
        ))
    }
}

impl TerminalRender for SeaFloor {
    fn render(&self, _w: usize, _h: usize) -> TerminalImage {
        if let Some((min, max)) = Point::bounding_box(self.cucumbers.keys()) {
            let width = (max.x-min.x+1) as usize;
            let mut pixels = Vec::with_capacity(width*(max.y-min.y+1) as usize);
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let color = match self.cucumbers.get(&point(x, y)) {
                        Some(Cucumber::South) => Color::GREEN,
                        Some(Cucumber::East) => Color::YELLOW,
                        None => Color::BLACK,
                    };
                    pixels.push(color);
                }
            }
            return TerminalImage{ pixels, width, }
        }
        TerminalImage{ pixels: vec![Color::RED,Color::WHITE,Color::RED,Color::WHITE,], width:2, }
    }
}

impl FromStr for SeaFloor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cucumbers = HashMap::new();
        let mut last_point = None;
        for (y, l) in s.lines().enumerate() {
            for (x, c) in l.chars().enumerate() {
                let p = point(x as i32, y as i32);
                last_point = Some(p);
                match c {
                    'v' => cucumbers.insert(p, Cucumber::South),
                    '>' => cucumbers.insert(p, Cucumber::East),
                    '.' => { continue; },
                    _ => bail!("Invalid char: {:?}", c),
                };
            }
        }
        Ok(SeaFloor{ cucumbers, bound: last_point.ok_or_else(||anyhow!("No points found"))?, })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let mut example: SeaFloor = include_str!("example.txt").parse().unwrap();
        let mut count = 1;
        while example.advance() {
            count += 1;
        }
        assert_eq!(count, 58);
        assert_eq!(example.to_string().trim(), include_str!("example-done.txt"));
    }

    parameterized_test::create!{ simple, (start, end), {
        let mut simple: SeaFloor = start.parse().unwrap();
        assert!(simple.advance());
        assert_eq!(simple.to_string().trim(), end);
    } }
    simple! {
        // Examples aren't exact copies since the Display impl doesn't print empty areas
        example1: ("...>>>>>...", ">>>>.>"),  // ...>>>>.>..
        example2: ("...>>>>.>..", ">>>.>.>"), // ...>>>.>.>.
        example3: ("..........\n.>v....v..\n.......>..\n..........", ">.......\n.v....v>"),
        example4_1: (
            "...>...\n.......\n......>\nv.....>\n......>\n.......\n..vvv..",
            "..vv>..\n.......\n>......\nv.....>\n>......\n.......\n....v.."),
        example4_2: (
            "..vv>..\n.......\n>......\nv.....>\n>......\n.......\n....v..",
            "....v>.\n..vv...\n.>.....\n......>\nv>....."),
        example4_3: (
            "....v>.\n..vv...\n.>.....\n......>\nv>.....\n.......\n.......",
            "......>\n..v.v..\n..>v...\n>......\n..>....\nv......"),
        example4_4: (
            "......>\n..v.v..\n..>v...\n>......\n..>....\nv......\n.......",
            ">....\n..v..\n..>.v\n.>.v.\n...>.\n.....\nv...."),
    }
}
