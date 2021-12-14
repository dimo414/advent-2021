use anyhow::Result;

use advent_2021::euclid::{vector, Vector};
use advent_2021::parsing::{capture_group, regex_captures, static_regex};

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;

    let distance = sum_directions(&input);
    println!("Naive Dist: {} = {}", distance, distance.x*distance.y);

    let distance = aim_directions(&input);
    println!("Aimed Dist: {} = {}", distance, distance.x*distance.y);

    Ok(())
}

fn sum_directions(directions: &[Vector]) -> Vector {
    directions.iter().fold(Vector::ZERO, |a, b| a + b)
}

fn aim_directions(directions: &[Vector]) -> Vector {
    let mut aim = 0;
    let mut distance = vector(0, 0);
    for v in directions {
        aim += v.y;
        distance += vector(v.x, v.x*aim);
    }
    distance
}

fn to_vector(inst: &str) -> Result<Vector> {
    let re = static_regex!("(.*) (.*)");
    let caps = regex_captures(re, inst)?;
    let dir = match capture_group(&caps, 1) {
        "forward" => vector(1,0),
        "down" => vector(0, 1),
        "up" => vector(0, -1),
        _ => panic!(),
    };
    let dist: i32 = capture_group(&caps, 2).parse()?;
    Ok(dir * dist)
}

fn parse_input(input: &str) -> Result<Vec<Vector>> {
    input.lines().map(to_vector).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Vec<Vector> {
        parse_input(include_str!("example.txt")).unwrap()
    }

    #[test]
    fn travel() {
        assert_eq!(sum_directions(&example()), vector(15, 10));
    }

    #[test]
    fn aim() {
        assert_eq!(aim_directions(&example()), vector(15, 60));
    }
}
