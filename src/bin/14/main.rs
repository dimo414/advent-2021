use std::collections::BTreeMap;
use std::fmt::Write;
use anyhow::Result;
use advent_2021::console::Console;
use advent_2021::elapsed;

fn main() -> Result<()> {
    let _console = Console::init();
    let (polymer, transforms) = parse_input(include_str!("example.txt"));

    let char_counts = elapsed!(simulate(&polymer, &transforms, 10));
    let char_counts_emulated = elapsed!(emulate(&polymer, &transforms, 10));
    assert_eq!(char_counts, char_counts_emulated);
    println!("Length after 10 iters: {}", char_counts.values().sum::<u64>());
    println!("Longest-Shortest:      {}", score_polymer(&char_counts));

    let char_counts = elapsed!(emulate(&polymer, &transforms, 40));
    println!("Length after 40 iters: {}", char_counts.values().sum::<u64>());
    println!("Longest-Shortest:      {}", score_polymer(&char_counts));

    Ok(())
}

fn to_char_counts(s: &str) -> BTreeMap<char, u64> {
    s.chars()
        .fold(BTreeMap::new(), |mut m, c| {
            m.entry(c).and_modify(|v| *v += 1).or_insert(1); m
        })
}

fn to_transform_map(transforms: &BTreeMap<String, String>) -> BTreeMap<(char, char), char> {
    transforms.iter()
        .map(|(k, v)| (k.chars().collect::<Vec<_>>(), v.chars().collect::<Vec<_>>()))
        .inspect(|(k, v)| { assert_eq!(k.len(), 2); assert_eq!(v.len(), 1); })
        .map(|(k, v)| ((k[0], k[1]), v[0]))
        .collect()
}

fn to_pairs(polymer: &str) -> BTreeMap<(char, char), u64> {
    let polymer: Vec<_> = polymer.chars().collect();
    polymer.windows(2).map(|w| (w[0], w[1])).fold(BTreeMap::new(), |mut m, p| {
        m.entry(p).and_modify(|v| *v += 1).or_insert(1); m
    })
}

fn score_polymer(char_counts: &BTreeMap<char, u64>) -> u64 {
    let mut counts: Vec<_> = char_counts.values().collect();
    counts.sort();
    counts[counts.len()-1] - counts[0]
}

fn simulate_step(polymer: &str, transforms: &BTreeMap<String, String>) -> String {
    let mut ret = String::new();
    for i in 0..polymer.len()-1 {
        write!(ret, "{}", &polymer[i..i+1]).unwrap();
        if let Some(mid) = transforms.get(&polymer[i..i+2]) {
            write!(ret, "{}", mid).unwrap();
        }
    }
    write!(ret, "{}", &polymer[polymer.len()-1..polymer.len()]).unwrap();
    ret
}

fn simulate(initial_polymer: &str, transforms: &BTreeMap<String, String>, iters: usize) -> BTreeMap<char, u64> {
    let mut polymer = initial_polymer.to_string();
    for _ in 0..iters {
        polymer = simulate_step(&polymer, transforms);
    }
    to_char_counts(&polymer)
}

fn emulate_step(polymer: &BTreeMap<(char, char), u64>, transforms: &BTreeMap<(char, char), char>) -> BTreeMap<(char, char), u64>{
    let mut ret = BTreeMap::new();

    for (pair, &count) in polymer {
        match transforms.get(pair) {
            Some(transform) => {
                let (a, b) = pair;
                let left = (*a, *transform);
                let right = (*transform, *b);
                ret.entry(left).and_modify(|v| *v += count).or_insert(count);
                ret.entry(right).and_modify(|v| *v += count).or_insert(count);
            },
            None => { ret.entry(*pair).and_modify(|v| *v += count).or_insert(count); },
        }
    }

    ret
}

fn emulate(initial_polymer: &str, transforms: &BTreeMap<String, String>, iters: usize) -> BTreeMap<char, u64> {
    let mut polymer = to_pairs(&initial_polymer);
    let transforms = to_transform_map(&transforms);
    for _ in 0..iters {
        polymer = emulate_step(&polymer, &transforms);
    }
    let mut char_counts = polymer.iter()
        .fold(BTreeMap::new(), |mut m, ((a, b), c)|{
            m.entry(*a).and_modify(|v| *v+=*c).or_insert(*c);
            m.entry(*b).and_modify(|v| *v+=*c).or_insert(*c);
            m
        });
    char_counts.entry(initial_polymer.chars().next().unwrap()).and_modify(|v| *v+=1);
    char_counts.entry(initial_polymer.chars().last().unwrap()).and_modify(|v| *v+=1);
    char_counts.iter().map(|(&k, &v)| (k, v/2)).collect()
}

fn parse_input(input: &str) -> (String, BTreeMap<String, String>) {
    let parts: Vec<_> = input.split("\n\n").collect();
    assert_eq!(parts.len(), 2);
    let transforms = parts[1].lines().map(|l| {
        let v: Vec<_> = l.split(" -> ").collect();
        assert_eq!(v.len(), 2);
        (v[0].to_string(), v[1].to_string())
    }).collect();
    (parts[0].to_string(), transforms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated() {
        let (polymer, transforms) = parse_input(include_str!("example.txt"));
        assert_eq!(simulate(&polymer, &transforms, 1), to_char_counts("NCNBCHB"));
        assert_eq!(simulate(&polymer, &transforms, 2), to_char_counts("NBCCNBBBCBHCB"));
        assert_eq!(simulate(&polymer, &transforms, 3), to_char_counts("NBBBCNCCNBBNBNBBCHBHHBCHB"));
        assert_eq!(simulate(&polymer, &transforms, 4),
                   to_char_counts("NBBNBNBBCCNBCNCCNBBNBBNBBBNBBNBBCBHCBHHNHCBBCBHCB"));

        assert_eq!(simulate(&polymer, &transforms, 5).values().sum::<u64>(), 97);

        let step10 = simulate(&polymer, &transforms, 10);
        assert_eq!(step10.values().sum::<u64>(), 3073);
        assert_eq!(step10, [('B', 1749), ('C', 298), ('H', 161), ('N', 865)].into_iter().collect());
    }

    #[test]
    fn emulated() {
        let (polymer, transforms) = parse_input(include_str!("example.txt"));
        assert_eq!(emulate(&polymer, &transforms, 1), to_char_counts("NCNBCHB"));
        assert_eq!(emulate(&polymer, &transforms, 2), to_char_counts("NBCCNBBBCBHCB"));
        assert_eq!(emulate(&polymer, &transforms, 3), to_char_counts("NBBBCNCCNBBNBNBBCHBHHBCHB"));
        assert_eq!(emulate(&polymer, &transforms, 4),
                   to_char_counts("NBBNBNBBCCNBCNCCNBBNBBNBBBNBBNBBCBHCBHHNHCBBCBHCB"));

        assert_eq!(emulate(&polymer, &transforms, 5).values().sum::<u64>(), 97);

        let step10 = emulate(&polymer, &transforms, 10);
        assert_eq!(step10.values().sum::<u64>(), 3073);
        assert_eq!(step10, [('B', 1749), ('C', 298), ('H', 161), ('N', 865)].into_iter().collect());

        let step40 = emulate(&polymer, &transforms, 40);
        assert_eq!(step40[&'B'], 2192039569602);
        assert_eq!(step40[&'H'], 3849876073);
        assert_eq!(score_polymer(&step40), 2188189693529);
    }
}
