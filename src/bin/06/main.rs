use anyhow::{Context, Result};

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;
    println!("Population after 80 days:  {}", simulate(&input, 80));
    println!("Population after 256 days: {}", emulate(&input, 256));
    Ok(())
}

fn parse_input(input: &str) -> Result<Vec<u64>> {
    input.trim().split(',').map(|n| n.parse().context(n.to_string())).collect()
}

fn simulate(fish: &[u64], days: usize) -> usize {
    let mut fish = fish.to_vec();
    for _ in 0..days {
        let ready = fish.iter().filter(|&&n| n == 0).count();
        let mut new_fish = vec![8; ready];
        for f in fish.iter_mut() {
            if *f == 0 {
                *f = 7;
            }
            *f -= 1;
        }
        fish.append(&mut new_fish);
    }
    fish.len()
}

fn emulate(fish: &[u64], days: usize) -> u64 {
    let mut fish_days = fish.iter().fold(vec![0; 7], |mut d, &f| { d[f as usize] += 1; d });
    let mut new_fish_days = vec![0; 7];

    for i in 0..days {
        new_fish_days[(i+2)%7] = fish_days[i%7];  // create new fish
        fish_days[i%7] += new_fish_days[i%7];     // mature recently born fish
        new_fish_days[i%7] = 0;
    }

    fish_days.iter().sum::<u64>() + new_fish_days.iter().sum::<u64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    parameterized_test::create!{ simulate, (days, expected), {
        let fish = parse_input("3,4,3,1,2").unwrap();
        assert_eq!(simulate(&fish, days), expected);
    } }
    simulate! {
        d18: (18, 26),
        d80: (80, 5934),
    }

    parameterized_test::create!{ emulate, (days, expected), {
        let fish = parse_input("3,4,3,1,2").unwrap();
        assert_eq!(emulate(&fish, days), expected);
    } }
    emulate! {
        d18: (18, 26),
        d80: (80, 5934),
        d256: (256, 26984457539),
    }
}
