use anyhow::Result;

fn main() -> Result<()> {
    let input: Vec<_> = include_str!("input.txt").lines().collect();
    let gamma = gamma(&input);
    let epsilon = epsilon(&gamma);
    let gamma_n = u32::from_str_radix(&gamma, 2)?;
    let epsilon_n = u32::from_str_radix(&epsilon, 2)?;

    println!("Gamma: {} - Epsilon: {} - Power Consumption: {}", gamma_n, epsilon_n, gamma_n * epsilon_n);

    let o2 = o2_gen(&input);
    let co2 = co2_scrub(&input);

    let o2_n = u32::from_str_radix(&o2, 2)?;
    let co2_n = u32::from_str_radix(&co2, 2)?;
    println!("O2: {} - CO2: {} - Life Support: {}", o2_n, co2_n, o2_n * co2_n);

    Ok(())
}

fn gamma(report: &[&str]) -> String {
    let mut gamma = String::new();
    let mut iters: Vec<_> = report.iter().map(|s| s.chars()).collect();
    loop {
        let column: Vec<_> = iters.iter_mut().flat_map(|i| i.next()).collect();
        if column.is_empty() { break; }
        let (zeros, ones) = count_bits(&column);
        if zeros > ones { gamma.push('0'); } else { gamma.push('1'); }
    }
    gamma
}

fn epsilon(gamma: &str) -> String {
    gamma.chars().map(|c| match c { '0' => '1', '1' => '0', _ => panic!(), }).collect()
}

// TODO use try_fold() to return a Result
fn count_bits(chars: &[char]) -> (usize, usize) {
    chars.iter().fold((0,0),
                      |(zeros, ones),c|
                          match c {
                              '1' => (zeros, ones+1),
                              '0' => (zeros+1, ones),
                              _ => panic!(),
                          }
    )
}

fn filter_by_column<F: Fn(usize, usize) -> bool>(report: &[&str], f: F) -> String {
    let mut index = 0;
    let mut candidates: Vec<_> = report.iter().collect();
    while candidates.len() > 1 {
        let mut zeros = vec!();
        let mut ones = vec!();
        for c in candidates {
            match c.chars().skip(index).next().unwrap() {
                '0' => zeros.push(c),
                '1' => ones.push(c),
                _ => panic!(),
            }
        }
        if f(zeros.len(), ones.len()) {
            candidates = ones;
        } else {
            candidates = zeros;
        }
        index += 1;
    }
    candidates[0].to_string()
}

fn o2_gen(report: &[&str]) -> String {
    filter_by_column(&report, |z, o| z <= o)
}

fn co2_scrub(report: &[&str]) -> String {
    filter_by_column(&report, |z, o| z > o)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_and_epsilon() {
        let input: Vec<_> = include_str!("example.txt").lines().collect();
        let gamma = gamma(&input);
        assert_eq!(gamma, "10110");
        assert_eq!(epsilon(&gamma), "01001");
    }

    #[test]
    fn o2_and_co2() {
        let input: Vec<_> = include_str!("example.txt").lines().collect();
        assert_eq!(o2_gen(&input), "10111");
        assert_eq!(co2_scrub(&input), "01010");
    }
}
