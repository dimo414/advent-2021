use anyhow::{Result, Error, bail};

use advent_2021::parsing::*;
use advent_2021::euclid3d::{Point,point};
use std::str::FromStr;
use std::collections::VecDeque;
use std::cmp;
use advent_2021::terminal::Terminal;

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    let constrained = constrain_to_initialization_area(&input);

    if std::env::args().len() > 1 {
        println!("{}", openscad(&input));
        return Ok(())
    }

    if Terminal::active() {
        println!("Simulated:           {}", simulate(&constrained));
    }

    println!("Initialization Area: {}", emulate(&constrained));
    println!("Full Area:           {}", emulate(&input));

    Ok(())
}

fn openscad(steps: &[Step]) -> String {
    let mut lines = VecDeque::new();
    for step in steps {
        let min = step.region.min;
        let max = step.region.max;
        let mut annotation = "";
        if matches!(step.state, State::Off) {
            lines.push_front("difference() { union() {".into());
            lines.push_back("}".into());
            annotation = ""; // Set to # to see removed regions
        }
        lines.push_back(format!("// {} x={}..{},y={}..{},z={}..{}", step.state, min.x, max.x, min.y, max.y, min.z, max.z));
        lines.push_back(format!("{}ocube({},{}, {},{}, {},{});", annotation, min.x, max.x, min.y, max.y, min.z, max.z));
        if matches!(step.state, State::Off) {
            lines.push_back("}".into());
        }
    }
    lines.push_front("".into());
    lines.push_front("module ocube(x1, x2, y1, y2, z1, z2) { translate([x1, y1, z1]) cube([x2-x1+1, y2-y1+1, z2-z1+1]); }".into());
    lines.make_contiguous().join("\n")
}

fn constrain_to_initialization_area(steps: &[Step]) -> Vec<Step> {
    steps.iter().filter(|s| s.region.min.x.abs() <= 50).cloned().collect()
}

// Much too inefficient to handle the whole region
fn simulate(steps: &[Step]) -> u64 {
    let mut area = Area::empty();
    for step in steps {
        match step.state {
            State::On => {
                area = area.union(Area::create(step.region));
            },
            State::Off => {
                area = area.difference(Area::create(step.region));
            }
        }
    }
    area.count_points()
}

fn emulate(steps: &[Step]) -> u64 {
    let mut points: Vec<CuboidParts> = Vec::new();
    for step in steps {
        points.iter_mut().for_each(|p| p.subtract(&step.region));
        if let State::On = step.state {
            points.push(CuboidParts::create(step.region));
        }
    }
    points.iter().map(|p| p.size()).sum()
}

#[derive(Debug, Copy, Clone)]
enum State { On, Off, }

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            State::On => "on",
            State::Off => "off",
        })
    }
}

#[derive(Debug, Copy, Clone)]
struct Cuboid {
    min: Point,
    max: Point,
}

impl Cuboid {
    fn size(&self) -> u64 {
        let d = self.max - self.min;
        debug_assert_eq!(d, d.abs());
        (d.x+1) as u64 * (d.y+1) as u64 * (d.z+1) as u64
    }
}

#[derive(Debug, Copy, Clone)]
struct Step {
    state: State,
    region: Cuboid,
}

impl FromStr for Step {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = static_regex!(r"(.*) x=(.*)\.\.(.*),y=(.*)\.\.(.*),z=(.*)\.\.(.*)");
        let caps = regex_captures(regex, s)?;
        let state = match capture_group(&caps, 1) { // Could use parse()
            "on" => State::On,
            "off" => State::Off,
            _ => bail!("Invalid {}", s),
        };
        let min = point(
            capture_group(&caps, 2).parse()?,
            capture_group(&caps, 4).parse()?,
            capture_group(&caps, 6).parse()?,
        );
        let max = point(
            capture_group(&caps, 3).parse()?,
            capture_group(&caps, 5).parse()?,
            capture_group(&caps, 7).parse()?,
        );
        Ok(Step{ state, region: Cuboid {min, max, }, })
    }
}

fn parse_input(input: &str) -> Result<Vec<Step>> {
    input.lines().map(|l| l.parse()).collect()
}

#[derive(Debug, Clone)]
struct CuboidParts {
    cuboid: Cuboid,
    remove: Vec<CuboidParts>,
}

impl CuboidParts {
    fn create(cuboid: Cuboid) -> CuboidParts {
        CuboidParts{ cuboid, remove: Vec::new(), }
    }

    fn subtract(&mut self, other: &Cuboid) {
        let vec = other.max - other.min;
        debug_assert_eq!(vec, vec.abs(), "{:?} does not appear to be a valid cuboid", other);
        let bounded = Cuboid{
            min: point(
                cmp::max(self.cuboid.min.x, other.min.x),
                cmp::max(self.cuboid.min.y, other.min.y),
                cmp::max(self.cuboid.min.z, other.min.z)),
            max: point(
                cmp::min(self.cuboid.max.x, other.max.x),
                cmp::min(self.cuboid.max.y, other.max.y),
                cmp::min(self.cuboid.max.z, other.max.z)),
        };
        let vec = bounded.max - bounded.min;
        if vec != vec.abs() { return; } // invalid cuboid - no overlap
        for r in &mut self.remove {
            r.subtract(&bounded);
        }
        self.remove.push(CuboidParts::create(bounded));
    }

    fn size(&self) -> u64 {
        let removed_points: u64 = self.remove.iter().map(|r| r.size()).sum();
        self.cuboid.size() - removed_points
    }
}

#[derive(Debug)]
enum Geometry {
    None,
    Cuboid(Cuboid),
    Union(Vec<Area>),
    Difference(Box<Area>, Vec<Area>),
}

#[derive(Debug)]
struct Area {
    min: Point,
    max: Point,
    shape: Geometry,
}

impl Area {
    fn empty() -> Area {
        Area{ min: Point::ORIGIN, max: Point::ORIGIN, shape: Geometry::None, }
    }

    fn create(cuboid: Cuboid) -> Area {
        Area{ min: cuboid.min, max: cuboid.max, shape: Geometry::Cuboid(cuboid), }
    }

    // For now just returns a new Area, but we could reorder nested elements that don't overlap
    // and return a union of disjoint Areas at the top level
    fn union(mut self, area: Area) -> Area {
        if matches!(area.shape, Geometry::None) { return self; }
        if matches!(self.shape, Geometry::None) { return area; }
        let (min, max) = Point::bounding_box(&[self.min, self.max, area.min, area.max]).expect("non-empty");
        if let Geometry::Union(m) = &mut self.shape {
            m.push(area);
            self.min = min;
            self.max = max;
            self
        } else {
            Area{ min, max, shape: Geometry::Union(vec![self, area]), }
        }
    }

    fn difference(mut self, area: Area) -> Area {
        if matches!(area.shape, Geometry::None) { return self; }
        if matches!(self.shape, Geometry::None) { return area; }
        let (min, max) = Point::bounding_box(&[self.min, self.max, area.min, area.max]).expect("non-empty");
        if let Geometry::Difference(_, m) = &mut self.shape {
            m.push(area);
            self.min = min;
            self.max = max;
            self
        } else {
            Area{ min, max, shape: Geometry::Difference(Box::new(self), vec![area]), }
        }
    }

    fn contains(&self, point: Point) -> bool {
        if !point.in_bounds(self.min, self.max) { return false; }
        match &self.shape {
            Geometry::None => false,
            Geometry::Cuboid(cuboid) => {
                // Area min/max should equal Cuboid min/max, so no need to check twice
                debug_assert!(point.in_bounds(cuboid.min, cuboid.max));
                true
            },
            Geometry::Union(union) => {
                union.iter().any(|a| a.contains(point))
            },
            Geometry::Difference(base, difference) => {
                base.contains(point) && !difference.iter().any(|a| a.contains(point))
            }
        }
    }

    fn count_points(&self) -> u64 {
        let mut ret = 0;
        for z in self.min.z..=self.max.z {
            for y in self.min.y..=self.max.y {
                for x in self.min.x..=self.max.x {
                    let p = point(x, y, z);
                    if self.contains(p) { ret += 1; }
                }
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example1() {
        let example = parse_input(include_str!("example1.txt")).unwrap();
        assert_eq!(simulate(&example), 39);
        assert_eq!(emulate(&example), 39);
    }

    #[test]
    fn example2() {
        let example = parse_input(include_str!("example2.txt")).unwrap();
        let example_init = constrain_to_initialization_area(&example);
        assert_eq!(simulate(&example_init), 590784);
        assert_eq!(emulate(&example_init), 590784);
        assert_eq!(emulate(&example), 39769202357779); // not specified
    }

    #[test]
    fn example3() {
        let example = parse_input(include_str!("example3.txt")).unwrap();
        let example_init = constrain_to_initialization_area(&example);
        assert_eq!(simulate(&example_init), 474140);
        assert_eq!(emulate(&example_init), 474140);
        assert_eq!(emulate(&example), 2758514936282235);
    }
}
