use anyhow::{anyhow, ensure, Error, Result};

use std::collections::{HashMap, HashSet};
use advent_2021::euclid::{point, Point, Vector};
use advent_2021::pathfinding::{Graph, Edge};
use std::str::FromStr;
use std::fmt::Display;
use std::time::Duration;
use advent_2021::terminal::{Color, Terminal, TerminalImage, TerminalRender};

fn main() -> Result<()> {
    let _drop = Terminal::init();
    let input: Cave = include_str!("input.txt").parse()?;
    let path = input.traverse_path().ok_or_else(|| anyhow!("No path"))?;
    if Terminal::active() {
        input.render(&path);
    }
    println!("Initial risk:  {}", path.iter().map(|e| e.weight()).sum::<i32>());

    let big = input.scale(5);
    println!("Expanded risk: {}", big.traverse().ok_or_else(|| anyhow!("No path"))?);

    Ok(())
}

struct Cave {
    scan: HashMap<Point, i32>,
    scan_size: i32,
    scale: i32,
    dest: Point,
}

impl Cave {
    fn new(scan: HashMap<Point, i32>) -> Result<Cave> {
        let (start, dest) = Point::bounding_box(scan.keys()).ok_or_else(|| anyhow!("no points"))?;
        ensure!(start == point(0, 0));
        ensure!(scan.len() as i32 == (dest.x+1) * (dest.y+1), "{} points don't end in {}", scan.len(), dest);
        Ok(Cave { scan, scan_size: dest.x+1, scale: 1, dest })
    }

    fn scale(&self, scale: i32) -> Cave {
        Cave { scan: self.scan.clone(), scan_size: self.scan_size, scale, dest: point((self.dest.x+1) * scale-1, (self.dest.y+1) * scale-1) }
    }

    fn traverse_path(&self) -> Option<Vec<Edge<Point>>> {
        self.dijkstras(&point(0, 0), |&p| p == self.dest)
    }

    fn traverse(&self) -> Option<i32> {
        let path = self.traverse_path()?;
        Some(path.iter().map(|e| e.weight()).sum())
    }

    fn risk(&self, p: Point) -> Option<i32> {
        let s = self.scan_size;
        let (grid_x, grid_y) = (p.x / s, p.y / s);
        if grid_x >= self.scale || grid_y >= self.scale { return None; }
        let dist = (point(grid_x, grid_y) - Point::ORIGIN).grid_len() as i32;
        self.scan.get(&point(p.x % s, p.y % s)).map(|&r| (r + dist - 1) % 9 + 1)
    }

    fn render(&self, path: &[Edge<Point>]) {
        for i in 0..=path.len() {
            let r = CaveRoute{ cave: self, route: &path[..i] };
            Terminal::interactive_render(&r, Duration::from_millis(10));
        }
    }
}

impl Graph for Cave {
    type Node = Point;

    fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
        Vector::CARDINAL.iter()
            .map(|v| source + v)
            .filter_map(|p| self.risk(p).map(|r| (p, r)))
            .map(|(d, r)| Edge::new(r, *source, d))
            .collect()
    }
}

impl FromStr for Cave {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let points = s.lines().enumerate()
            .flat_map(|(y, l)|
                l.chars().enumerate().map(move |(x, r)|
                    r.to_digit(10).map(|r|
                        (point(x as i32, y as i32), r as i32)).ok_or_else(|| anyhow!("Invalid"))))
            .collect::<Result<HashMap<_, _>>>()?;
        Cave::new(points)
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut out = String::new();
        for row in Point::display_order(&[Point::ORIGIN, self.dest]).expect("non-empty") {
            for col in row {
                out.push(self.risk(col).map(|v| char::from_digit(v as u32, 10).unwrap()).unwrap_or('*'));
            }
            out.push('\n');
        }
        write!(f, "{}", out)
    }
}

struct CaveRoute<'a> {
    cave: &'a Cave,
    route: &'a [Edge<Point>],
}

impl<'a> TerminalRender for CaveRoute<'a> {
    fn render(&self, _w: usize, _h: usize) -> TerminalImage {
        let visited: HashSet<_> = self.route.iter().flat_map(|e| [*e.source(), *e.dest()].into_iter()).collect();
        let width = (self.cave.dest.x+1) as usize;
        let mut pixels = Vec::new();
        for y in 0..=self.cave.dest.y {
            for x in 0..=self.cave.dest.x {
                let pos = point(x, y);
                if visited.contains(&pos) {
                    pixels.push(Color::YELLOW);
                } else {
                    pixels.push(match self.cave.risk(pos) {
                        Some(r) => Color::GREYSCALE(1.0 - (r as f32 / 10.0)),
                        None => Color::RED,
                    });
                }
            }
        }
        TerminalImage{ pixels, width, }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_up() {
        let little: Cave = "8".parse().unwrap();
        assert_eq!("8\n", little.to_string());
        let big = little.scale(5);
        assert_eq!(big.dest, point(4, 4));
        assert_eq!("89123\n91234\n12345\n23456\n34567\n", big.to_string());
    }

    #[test]
    fn example() {
        let input: Cave = include_str!("example.txt").parse().unwrap();
        assert_eq!(input.traverse(), Some(40));
    }

    #[test]
    fn example_scaled() {
        let input = include_str!("example.txt").parse::<Cave>().unwrap().scale(5);
        assert_eq!(input.traverse(), Some(315));
    }
}
