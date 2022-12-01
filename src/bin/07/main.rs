use anyhow::{Context, Result};

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    // Observed the example answer looked like the median, not very principled
    println!("Simple Fuel Cost: {}", simple_fuel_cost(&input, median(&input)));
    // Guessed the mean might be helpful given the median worked before, also not right but it
    // turns out to find _almost_ the right value. Would be nice to do a proper hill-climb solution.
    let mean = mean(&input);
    let floor = complex_fuel_cost(&input, mean.floor() as i32);
    let ceil = complex_fuel_cost(&input, mean.ceil() as i32);
    println!("Complex Fuel Cost: {}", std::cmp::min(floor, ceil));

    Ok(())
}

fn median<T: Copy+std::cmp::Ord>(sequence: &[T]) -> T {
    let mut vec = sequence.to_vec();
    vec.sort();
    vec[vec.len() / 2]
}

fn mean(sequence: &[i32]) -> f64 {
    sequence.iter().sum::<i32>() as f64 / sequence.len() as f64
}

fn simple_fuel_cost(positions: &[i32], target: i32) -> i32 {
    positions.iter().map(|n| (n - target).abs()).sum()
}

fn complex_fuel_cost(positions: &[i32], target: i32) -> i32 {
    // naively: |d| (1..(d+1)).sum::<i32>()
    positions.iter().map(|&n| (n - target).abs()).map(|d| (d*(d+1))/2).sum()
}

fn parse_input(input: &str) -> Result<Vec<i32>> {
    input.trim().split(',').map(|n| n.parse().context(n.to_string())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    static CRABS: &[i32] = &[16,1,2,0,4,2,7,1,2,14];

    #[test]
    fn check_median() {
        let seq = vec![1,3,6,9,10];
        assert_eq!(median(&seq), 6);
        assert_eq!(median(CRABS), 2);
    }

    #[test]
    fn check_mean() {
        let seq = vec![1,3,6,9,10];
        assert!((mean(&seq) - 5.8).abs() < f64::EPSILON);
        assert!((mean(CRABS) - 4.9).abs() < f64::EPSILON);
    }

    parameterized_test::create!{ simple_fuel, (target, cost),
        { assert_eq!(simple_fuel_cost(CRABS, target), cost); } }
    simple_fuel! {
        p1: (1, 41),
        p2: (2, 37),
        p3: (3, 39),
        p10: (10, 71),
    }

    parameterized_test::create!{ complex_fuel, (target, cost),
        { assert_eq!(complex_fuel_cost(CRABS, target), cost); } }
    complex_fuel! {
        p2: (2, 206),
        p5: (5, 168),
    }
}
