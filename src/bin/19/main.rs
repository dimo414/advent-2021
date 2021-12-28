use std::collections::{BTreeSet, BTreeMap};
use std::str::FromStr;
use anyhow::{anyhow, Result, Error};
use lazy_static::lazy_static;
use advent_2021::euclid3d::{point, Point, Vector};
use advent_2021::parsing::*;

fn main() -> Result<()> {
    let mut input = parse_input(include_str!("input.txt"))?;
    let mut trench = Trench::create(input.swap_remove(0));

    trench.merge_scans(input);
    println!("Beacons: {}", trench.beacons.len());
    println!("Distance between scanners: {}", trench.scanner_distance());

    Ok(())
}

fn parse_input(input: &str) -> Result<Vec<Scanner>> {
    input.split("\n\n").map(|section| section.parse()).collect()
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Sign { Positive, Negative, }
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Axis { X, Y, Z, }

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Orientation {
    transformation: [(Sign, Axis); 3],
}

impl Orientation {
    #[cfg(test)]
    fn create(sign_a: Sign, axis_a: Axis, sign_b: Sign, axis_b: Axis, sign_c: Sign, axis_c: Axis) -> Orientation {
        Orientation{ transformation: [(sign_a, axis_a), (sign_b, axis_b), (sign_c, axis_c)]}
    }

    // Returns 24 orientations
    // https://old.reddit.com/r/adventofcode/comments/rjrpsv/2021_day_19/hp6a4a2/
    fn all() -> &'static Vec<Orientation> {
        lazy_static! {
            static ref ALL: Vec<Orientation> = {
                use Axis::*;
                let (p, n) = (Sign::Positive, Sign::Negative);

                let even = [[X,Y,Z], [Y,Z,X], [Z,X,Y]].iter()
                    .flat_map(|v| [[(p,v[0]),(p,v[1]),(p,v[2])], [(p,v[0]),(n,v[1]),(n,v[2])], [(n,v[0]),(n,v[1]),(p,v[2])], [(n,v[0]),(p,v[1]),(n,v[2])]]);
                let odd = [[X,Z,Y], [Y,X,Z], [Z,Y,X]].iter()
                    .flat_map(|v| [[(n,v[0]),(p,v[1]),(p,v[2])], [(p,v[0]),(n,v[1]),(p,v[2])], [(p,v[0]),(p,v[1]),(n,v[2])], [(n,v[0]),(n,v[1]),(n,v[2])]]);
                even.chain(odd).map(|transformation| Orientation{ transformation }).collect()
            };
        }
        &ALL
    }

    fn resolve(point: (i32,i32,i32), dir: (Sign, Axis)) -> i32 {
        let mag = match dir.1 {
            Axis::X => point.0,
            Axis::Y => point.1,
            Axis::Z => point.2,
        };
        match dir.0 {
            Sign::Positive => mag,
            Sign::Negative => -mag,
        }
    }

    fn transform_point(&self, p: Point) -> Point {
        let p = (p.x, p.y, p.z);
        point(Self::resolve(p, self.transformation[0]),
              Self::resolve(p, self.transformation[1]),
              Self::resolve(p, self.transformation[2]))
    }
}

#[cfg(test)]
mod orientation_tests {
    use super::*;

    #[test]
    fn resolves() {
        let orientation = Orientation::create(
            Sign::Negative, Axis::Y, Sign::Positive, Axis::Z, Sign::Negative, Axis::X);
        assert_eq!(orientation.transform_point(point(1, 2, 3)), point(-2, 3, -1))
    }

    #[test]
    fn all() {
        let expected: Vec<_> = [
            (1, 2, 3), (1, -2, -3), (-1, -2, 3), (-1, 2, -3),
            (2, 3, 1), (2, -3, -1), (-2, -3, 1), (-2, 3, -1),
            (3, 1, 2), (3, -1, -2), (-3, -1, 2), (-3, 1, -2),
            (-1, 3, 2), (1, -3, 2), (1, 3, -2), (-1, -3, -2),
            (-2, 1, 3), (2, -1, 3), (2, 1, -3), (-2, -1, -3),
            (-3, 2, 1), (3, -2, 1), (3, 2, -1), (-3, -2, -1)
        ].iter().map(|(x, y, z)| point(*x, *y, *z)).collect();

        let input = point(1, 2, 3);
        let orientations: Vec<_> = Orientation::all().iter().map(|o| o.transform_point(input)).collect();
        assert_eq!(orientations, expected);
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Scanner {
    id: u32,
    beacons: Vec<Point>,
    distances: BTreeMap<Vector, (Point, Point)>,
    distances_set: BTreeSet<Vector>,
}

impl Scanner {
    fn create(id: u32, beacons: Vec<Point>) -> Scanner {
        let mut distances = BTreeMap::new();
        for i in 0..beacons.len() {
            for j in (i+1)..beacons.len() {
                distances.insert(beacons[i] - beacons[j], (beacons[i], beacons[j]));
                distances.insert(beacons[j] - beacons[i], (beacons[j], beacons[i]));
            }
        }
        let distances_set = distances.keys().cloned().collect();
        Scanner{ id, beacons, distances, distances_set }
    }

    fn reorient(&self, orientation: Orientation) -> Scanner {
        Scanner::create(self.id, self.beacons.iter().map(|&p| orientation.transform_point(p)).collect())
    }

    fn offset(&self, offset: Vector) -> Scanner {
        Scanner::create(self.id, self.beacons.iter().map(|&b| b+offset).collect())
    }
}

impl FromStr for Scanner {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (header, body) = s.split_once('\n').ok_or_else(|| anyhow!("Invalid Scanner"))?;

        let regex = static_regex!(r"--- scanner (\d+) ---");
        let caps = regex_captures(regex, header)?;
        let id = capture_group(&caps, 1).parse()?;

        let beacons = body.lines().map(|p| p.parse::<Point>()).collect::<Result<Vec<_>>>()?;
        Ok(Scanner::create(id, beacons))
    }
}

#[cfg(test)]
mod scanner_tests {
    use super::*;
    use advent_2021::euclid3d::vector;

    #[test]
    fn reorient() {
        let orientation = Orientation::create(
            Sign::Negative, Axis::Y, Sign::Positive, Axis::Z, Sign::Negative, Axis::X);
        let scanner = Scanner::create(1, vec![point(-1,-1,0), point(-5,0,0), point(-2,1,0)]);
        let oriented = scanner.reorient(orientation);
        assert_eq!(oriented.beacons, vec![point(1, 0, 1), point(0, 0, 5), point(-1, 0, 2)]);
    }

    #[test]
    fn simple_example() {
        let scanner1 = Scanner::create(1, vec![point(0,2, 0), point(4,1, 0), point(3,3, 0)]);
        let scanner2 = Scanner::create(2, vec![point(-1,-1,0), point(-5,0,0), point(-2,1,0)]);

        assert_eq!(scanner1.distances_set,
                   [vector(-4, 1, 0), vector(-3, -1, 0), vector(-1, 2, 0),
                       vector(1, -2, 0), vector(3, 1, 0), vector(4, -1, 0)].into_iter().collect());
        assert_eq!(scanner2.distances_set,
                   [vector(-4, 1, 0), vector(-3, -1, 0), vector(-1, 2, 0),
                       vector(1, -2, 0), vector(3, 1, 0), vector(4, -1, 0)].into_iter().collect());
    }

    #[test]
    fn orientations() {
        let scanners = parse_input(include_str!("example1.txt")).unwrap();
        assert_eq!(5, scanners.len());

        assert_eq!(scanners[0], scanners[1].reorient(
            Orientation::create(Sign::Negative, Axis::X, Sign::Negative, Axis::Z, Sign::Negative, Axis::Y)));
        assert_eq!(scanners[0], scanners[2].reorient(
            Orientation::create(Sign::Positive, Axis::Z, Sign::Positive, Axis::Y, Sign::Negative, Axis::X)));
        assert_eq!(scanners[0], scanners[3].reorient(
            Orientation::create(Sign::Positive, Axis::Z, Sign::Negative, Axis::Y, Sign::Positive, Axis::X)));
        assert_eq!(scanners[0], scanners[4].reorient(
            Orientation::create(Sign::Negative, Axis::Z, Sign::Negative, Axis::X, Sign::Positive, Axis::Y)));
    }
}

struct Trench {
    scanners: BTreeSet<Point>,
    beacons: BTreeSet<Point>,
    scanner_data: Vec<Scanner>,
}

impl Trench {
    fn create(root_scanner: Scanner) -> Trench {
        let mut scanners = BTreeSet::new();
        scanners.insert(Point::ORIGIN);
        let beacons = root_scanner.beacons.iter().cloned().collect();
        let scanner_data = vec![root_scanner];
        Trench{ scanners, beacons, scanner_data }
    }

    fn composite_scanner(&self) -> Scanner {
        Scanner::create(u32::MAX, self.beacons.iter().cloned().collect())
    }

    fn merge_scans(&mut self, mut scanners: Vec<Scanner>) {
        while !scanners.is_empty() {
            let composite = self.composite_scanner();

            let (idx, next_scanner) = scanners.iter().enumerate()
                .flat_map(|(i, s)| Orientation::all().iter().map(move |o| (i, s.reorient(*o))))
                .max_by_key(|(_, s)| s.distances_set.intersection(&composite.distances_set).count())
                .expect("scanners can't be empty");
            let removed = scanners.swap_remove(idx);
            debug_assert_eq!(removed.id, next_scanner.id);

            self.merge_with(next_scanner, &composite);
        }
    }

    fn merge_with(&mut self, scanner: Scanner, composite: &Scanner) {
        debug_assert!(composite.distances_set.intersection(&scanner.distances_set).count() >= 132, "Permutation 12P2 == 132");

        let mut offsets = BTreeSet::new();
        for (dist, points_c) in &composite.distances {
            if let Some(points_s) = scanner.distances.get(dist) {
                offsets.insert(points_c.0 - points_s.0);
            }
        }
        debug_assert_eq!(offsets.len(), 1);
        let offset = offsets.into_iter().next().expect("One element");
        let offset_scanner = scanner.offset(offset);

        self.scanners.insert(Point::ORIGIN + offset);
        for beacon in &offset_scanner.beacons {
            self.beacons.insert(*beacon);
        }
        self.scanner_data.push(offset_scanner);
    }

    fn scanner_distance(&self) -> u32 {
        let scanners: Vec<_> = self.scanners.iter().cloned().collect();
        let num_scanners = scanners.len();
        (0..num_scanners).flat_map(|i| (0..num_scanners).map(move |j| (i, j)))
            .map(|(i, j)| (scanners[i] - scanners[j]).grid_len()).max().unwrap()
    }
}

#[cfg(test)]
mod trench_tests {
    use super::*;

    #[test]
    fn example2() {
        let mut input = parse_input(include_str!("example2.txt")).unwrap();
        let mut trench = Trench::create(input.swap_remove(0));

        trench.merge_scans(input);
        assert_eq!(trench.beacons.len(), 79);
        let expected: BTreeSet<_> =
            include_str!("expected2.txt").lines().map(|l| l.parse::<Point>().unwrap()).collect();
        assert_eq!(trench.beacons, expected);
        assert_eq!(trench.scanner_distance(), 3621);
    }
}
