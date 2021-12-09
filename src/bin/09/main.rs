use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::{anyhow, Result};
use advent_2021::euclid::{Point, point, Vector};

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;

    let low_points = find_low_points(&input);
    println!("Depth Score: {}", low_points.iter().map(|p| input[p]+1).sum::<u32>());

    let mut basin_sizes: Vec<_> = low_points.iter().map(|&p| basin_size(p, &input)).collect();
    basin_sizes.sort_unstable();
    let product = basin_sizes.iter().rev().take(3).product::<u32>();
    println!("Largest Basins: {}", product);

    Ok(())
}

fn find_low_points(depths: &HashMap<Point, u32>) -> Vec<Point> {
    let no_lower_neighbor = |point, depth|
        !Vector::CARDINAL.iter()
            .map(|v| point+v)
            .flat_map(|n| depths.get(&n).into_iter())
            .any(|&nd| nd <= depth);

    let mut low_points = Vec::new();
    for (&point, &depth) in depths.iter() {
        if no_lower_neighbor(point, depth) {
            low_points.push(point);
        }
    }
    low_points.sort();
    low_points
}

fn basin_size(starting_point: Point, depths: &HashMap<Point, u32>) -> u32 {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut size = 0;
    queue.push_back(starting_point);

    while let Some(point) = queue.pop_front() {
        if !visited.insert(point) { continue; }
        if let Some(&depth) = depths.get(&point) {
            if depth < 9 {
                size += 1;
            } else { continue; }
        } else { continue; }
        for dir in Vector::CARDINAL {
            queue.push_back(point + dir);
        }
    }

    size
}

fn parse_input(input: &str) -> Result<HashMap<Point, u32>> {
    let mut ret = HashMap::new();
    for (y, line) in input.lines().enumerate() {
        for (x, d) in line.chars().enumerate() {
            ret.insert(point(x as i32, y as i32), d.to_digit(10).ok_or(anyhow!("Invalid digit"))?);
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_points() {
        let depths = parse_input(include_str!("example.txt")).unwrap();

        assert_eq!(&find_low_points(&depths),
                   &[point(1, 0), point(2, 2), point(6, 4), point(9, 0)]);
    }

    parameterized_test::create!{ basins, (point, expected_size), {
        let depths = parse_input(include_str!("example.txt")).unwrap();
        assert_eq!(basin_size(point, &depths), expected_size);
    } }
    basins! {
        b1_low: (point(1, 0), 3),
        b1_near: (point(0, 0), 3),
        b2_low: (point(2, 2), 14),
        b2_near: (point(4, 3), 14),
        b3_low: (point(6, 4), 9),
        b3_near: (point(7, 3), 9),
        b4_low: (point(9, 0), 9),
        b4_near: (point(6, 1), 9),
    }
}
