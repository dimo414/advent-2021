use std::collections::HashSet;
use anyhow::{bail, Result};
use advent_2021::euclid::{Point, point};

use advent_2021::parsing::*;

fn main() -> Result<()> {
    let (mut points, folds) = parse_input(include_str!("input.txt"))?;

    points = folds[0].fold_all(&points);
    println!("Points after first fold: {}", points.len());

    for fold in &folds[1..] {
        points = fold.fold_all(&points);
    }
    println!("Folded image:\n{}", display(&points));

    Ok(())
}

enum Fold {
    X(i32),
    Y(i32),
}

impl Fold {
    fn fold(&self, p: Point) -> Point {
        match self {
            Fold::X(v) => if p.x > *v { point(v - (p.x - v), p.y) } else { p },
            Fold::Y(v) => if p.y > *v { point(p.x, v - (p.y - v)) } else { p },
        }
    }

    fn fold_all(&self, points: &HashSet<Point>) -> HashSet<Point> {
        points.iter().map(|&p| self.fold(p)).collect()
    }
}

fn parse_input(input: &str) -> Result<(HashSet<Point>, Vec<Fold>)> {
    let parts: Vec<_> = input.split("\n\n").collect();
    let points = parts[0].lines().map(|l| l.parse()).collect::<Result<HashSet<_>>>()?;

    let regex = static_regex!("fold along (.)=(.*)");
    let folds = parts[1].lines().map(|l| {
        let caps = regex_captures(regex, l)?;
        let value: i32 = capture_group(&caps, 2).parse().unwrap();
        let fold = match capture_group(&caps, 1) {
            "x" => Fold::X(value),
            "y" => Fold::Y(value),
            _ => bail!("invalid"),
        };
        Ok(fold)
    }).collect::<Result<Vec<_>>>()?;
    Ok((points, folds))
}

// TODO move this into euclid
fn display(points: &HashSet<Point>) -> String {
    let mut out = String::new();
    let mut last_y = None;
    for p in Point::display_order_box(points.iter().cloned()).unwrap() {
        if let Some(last_y) = last_y {
            if last_y != p.y { out.push('\n'); }
        }
        if points.contains(&p) {
            out.push('█');
        } else {
            out.push(' ');
        }

        last_y = Some(p.y);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let (mut points, folds) = parse_input(include_str!("example.txt")).unwrap();

        points = folds[0].fold_all(&points);
        assert_eq!(points.len(), 17);

        for fold in &folds[1..] {
            points = fold.fold_all(&points);
        }
        assert_eq!(display(&points), "█████\n█   █\n█   █\n█   █\n█████");
    }
}
