use std::collections::HashMap;
use anyhow::Result;

use advent_2021::parsing::*;
use advent_2021::euclid::{Point, vector};

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    println!("Oriented Overlaps: {}", count_overlaps(&filter_diagonals(&input)));
    println!("All Overlaps:      {}", count_overlaps(&input));
    Ok(())
}

fn count_overlaps(paths: &[(Point, Point)]) -> usize {
    let points: Vec<Point> = paths.iter().flat_map(|(a, b)| points_between(*a, *b).into_iter()).collect();
    let mut grid: HashMap<Point, u32> = HashMap::new();
    for point in &points {
        *grid.entry(*point).or_insert(0) += 1;
    }
    grid.values().filter(|&&n| n > 1).count()
}

fn filter_diagonals(paths: &[(Point, Point)]) -> Vec<(Point, Point)> {
    paths.iter().cloned().filter(|(a, b)| a.x == b.x || a.y == b.y).collect()
}

fn points_between(a: Point, b: Point) -> Vec<Point> {
    let dist = b - a;
    let dir = vector(dist.x.signum(), dist.y.signum());
    let mut points = vec![a];
    while points[points.len()-1] != b {
        points.push(points[points.len()-1] + dir);
    }
    points
}

fn parse_input(input: &str) -> Result<Vec<(Point, Point)>> {
    fn to_pair(line: &str) -> Result<(Point, Point)> {
        let regex = static_regex!(r"(.*) -> (.*)");
        let caps = regex_captures(regex, line)?;
        let a = capture_group(&caps, 1);
        let b = capture_group(&caps, 2);
        Ok((a.parse()?, b.parse()?))
    }

    input.lines().map(to_pair).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use advent_2021::euclid::point;

    parameterized_test::create!{ between, (a, b, expected), { assert_eq!(&points_between(a, b), expected); } }
    between! {
        vertical: (point(2,2), point(2,4), &[point(2,2), point(2,3), point(2,4)]),
        horizontal: (point(2,2), point(4,2), &[point(2,2), point(3,2), point(4,2)]),
        backwards: (point(4,2), point(2,2), &[point(4,2), point(3,2), point(2,2)]),
        diagonal: (point(2,2), point(4,4), &[point(2,2), point(3,3), point(4,4)]),
        neg_diagonal: (point(2,4), point(4,2), &[point(2,4), point(3,3), point(4,2)]),
    }

    #[test]
    fn exclude_diagonals() {
        let example = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(&filter_diagonals(&example), &[
            (point(0, 9), point(5, 9)),
            (point(9, 4), point(3, 4)),
            (point(2, 2), point(2, 1)),
            (point(7, 0), point(7, 4)),
            (point(0, 9), point(2, 9)),
            (point(3, 4), point(1, 4)),
        ]);
    }

    #[test]
    fn overlaps() {
        let example = parse_input(include_str!("example.txt")).unwrap();

        assert_eq!(count_overlaps(&filter_diagonals(&example)), 5);
        assert_eq!(count_overlaps(&example), 12);
    }
}
