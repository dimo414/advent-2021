fn main() {
    let input = parse_input();

    println!("Increasing Measurements: {}", count_increases(&input));
    println!("Increasing Windows:      {}", count_increases(&sum_windows(&input, 3)));
}

fn parse_input() -> Vec<i32> {
    include_str!("input.txt").lines().map(|l| l.parse().unwrap()).collect()
}

fn count_increases(report: &[i32]) -> usize {
    report.windows(2).filter(|w| w[0] < w[1]).count()
}

fn sum_windows(report: &[i32], window: usize) -> Vec<i32> {
    report.windows(window).map(|w| w.iter().sum()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    static REPORT: &[i32] = &[199, 200, 208, 210, 200, 207, 240, 269, 260, 263];

    #[test]
    fn increases() {
        assert_eq!(count_increases(REPORT), 7);
    }

    #[test]
    fn sums() {
        let sums = sum_windows(REPORT, 3);
        assert_eq!(&sums, &[607, 618, 618, 617, 647, 716, 769, 792]);
        assert_eq!(count_increases(&sums), 5);
    }
}
