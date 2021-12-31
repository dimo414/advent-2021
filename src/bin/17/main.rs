use std::collections::HashMap;
use anyhow::{anyhow, Result};
use advent_2021::euclid::{Point, point, Vector, vector};

fn main() -> Result<()> {
    // INPUT: x=70..96, y=-179..-124
    let target = (point(70, -179), point(96, -124));

    let trajectories = all_trajectories(target);
    let best = trajectories.values()
        .flat_map(|v| v.iter().map(|p| p.y))
        .max().ok_or_else(|| anyhow!("No trajectories found"))?;
    println!("Best trajectory reaches Y={}", best);
    println!("Total trajectories: {}", trajectories.len());

    Ok(())
}

fn all_trajectories(target: (Point, Point)) -> HashMap<Vector, Vec<Point>> {
    let mut ret = HashMap::new();

    // Since x velocity decreases by 1 each step it will take the form n,n-1,n-2,...,3,2,1; in other
    // words a Triangle Number. Working backwards we approximate n as sqrt(2*t) since
    // T(n) = (n+1)*n/2.
    let min_x = ((target.0.x * 2) as f64).sqrt() as i32;
    // For y we just take the lower value of target.y or 0; values below this will never reach the
    // bottom of the target.
    let min_y = std::cmp::min(target.0.y, 0);
    // Anything larger would overshoot
    let max_x = target.1.x;
    // A positive y velocity will eventually return the probe to y=0, meaning the next step can't
    // overshoot the bottom of the target.
    let max_y = target.0.y.abs();

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let v = vector(x, y);
            if let Attempt::Success(points) = attempt(v, target) {
                ret.insert(v, points);
            }
        }
    }

    ret
}

#[derive(Debug, Eq, PartialEq)]
enum Attempt { Success(Vec<Point>), Miss(Vec<Point>), }

fn attempt(velocity: Vector, target: (Point, Point)) -> Attempt {
    let mut probe = Probe::launch(velocity, target);
    let mut points = Vec::new();
    loop {
        let state = probe.step();
        points.push(probe.position);
        match state {
            State::Success => { return Attempt::Success(points); },
            State::Miss => { return Attempt::Miss(points); },
            _ => {},
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum State { Traveling, Success, Miss, }

struct Probe {
    position: Point,
    velocity: Vector,
    target: (Point, Point),
}

impl Probe {
    fn launch(velocity: Vector, target: (Point, Point)) -> Probe {
        Probe { position: point(0, 0), velocity, target }
    }

    fn step(&mut self) -> State {
        self.position += self.velocity;
        self.velocity += vector(- self.velocity.x.signum(), -1);
        if self.position.in_bounds(self.target.0, self.target.1) {
            return State::Success;
        }
        if self.velocity.y <= 0 && self.position.y < self.target.0.y {
            return State::Miss;
        }
        State::Traveling
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        // EXAMPLE: x=20..30, y=-10..-5
        let target = (point(20,-10), point(30,-5));

        let trajectories = all_trajectories(target);
        let best = trajectories.values()
            .flat_map(|v| v.iter().map(|p| p.y))
            .max().unwrap();
        assert_eq!(best, 45);
        assert_eq!(trajectories.len(), 112);
    }
}
