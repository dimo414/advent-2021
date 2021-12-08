use anyhow::{bail,ensure,Result};
use bitmaps::Bitmap;

use advent_2021::parsing::*;

// TODO interactive! with https://en.wikipedia.org/wiki/Symbols_for_Legacy_Computing
fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"))?;

    let easy_digits = input.iter().map(|d| d.count_easy_digits()).sum::<u32>();
    println!("Easy Digits: {}", easy_digits);

    let outputs: Vec<_> = input.iter().map(|d| d.read_display()).collect::<Result<_>>()?;
    println!("Summed Outputs: {}", outputs.iter().sum::<u32>());

    Ok(())
}

// See also CharSet in https://github.com/dimo414/advent-2019/blob/e0b2414e90/src/aoc18.rs#L260
type Segments = Bitmap<7>;

fn to_segments(desc: &str) -> Result<Segments> {
    let mut segments = Segments::new();
    for c in desc.chars() {
        ensure!(('a'..='g').contains(&c));
        segments.set(c as usize - 'a' as usize, true);
    }
    Ok(segments)
}

struct SegmentDisplay {
    digits: [Segments; 10],
    output: Vec<Segments>,
}

impl SegmentDisplay {
    fn create(descs: &[&str], output: &[&str]) -> Result<SegmentDisplay> {
        use std::mem::swap;
        ensure!(descs.len() == 10);

        // Order candidates by the number of "on" segments
        let mut candidates = descs.iter().map(|d| to_segments(d)).collect::<Result<Vec<_>>>()?;
        candidates.sort_by_key(|s| s.len());

        let mut digits = [Segments::new(); 10];

        //  11
        // 2  3
        //  44
        // 5  6
        //  77
        //
        // "On" segments per digit:
        // 1: 36
        // 7: 136
        // 4: 2346
        // 2: 13457
        // 3: 13467
        // 5: 12467
        // 0: 123567
        // 6: 124567
        // 9: 123467
        // 8: 1234567

        // Solve 1, 4, 7, and 8 which can be determined just by segment count
        ensure!(candidates[0].len() == 2); // 1
        swap(&mut digits[1], &mut candidates[0]);
        ensure!(candidates[2].len() == 4); // 4
        swap(&mut digits[4], &mut candidates[2]);
        ensure!(candidates[1].len() == 3); // 7
        swap(&mut digits[7], &mut candidates[1]);
        ensure!(candidates[9].len() == 7); // 8
        swap(&mut digits[8], &mut candidates[9]);

        fn find_candidate<F: Fn(&Segments) -> bool>(candidates: &[Segments], filter: F) -> Result<usize> {
            let candidate: Vec<_> = candidates.iter().enumerate()
                .filter(|(_, c)| filter(c))
                .map(|(i, _)| i).collect();
            ensure!(candidate.len() == 1, format!("Unexpected result: {:?}", candidate));
            Ok(candidate[0])
        }

        // 3 is a superset of 1
        let found = find_candidate(&candidates, |&c| c.len() == 5 && c & digits[1] == digits[1])?;
        swap(&mut digits[3], &mut candidates[found]);

        // 9 is a superset of 3
        let found = find_candidate(&candidates, |&c| c.len() == 6 && c & digits[3] == digits[3])?;
        swap(&mut digits[9], &mut candidates[found]);

        // 0 is a superset of 1
        let found = find_candidate(&candidates, |&c| c.len() == 6 && c & digits[1] == digits[1])?;
        swap(&mut digits[0], &mut candidates[found]);

        // now 6 is trivial
        let found = find_candidate(&candidates, |&c| c.len() == 6)?;
        swap(&mut digits[6], &mut candidates[found]);

        // 5 is a subset of 9
        let found = find_candidate(&candidates, |&c| c.len() == 5 && c & digits[9] == c)?;
        swap(&mut digits[5], &mut candidates[found]);

        // leaving us with 2 (do find_candidate to validate the candidate array has one left
        let found = find_candidate(&candidates, |&c| !c.is_empty())?;
        swap(&mut digits[2], &mut candidates[found]);

        ensure!(digits.iter().find(|&d| d.is_empty()) == None, "All digits should be set");

        Ok(SegmentDisplay { digits, output: output.iter().map(|o| to_segments(o)).collect::<Result<_>>()? })
    }

    fn count_easy_digits(&self) -> u32 {
        self.output.iter().filter(|d| [2, 3, 4, 7].contains(&d.len())).count() as u32
    }

    fn lookup_digit(&self, digit: &Segments) -> Result<u32> {
        for (i, s) in self.digits.iter().enumerate() {
            if s == digit { return Ok(i as u32) }
        }
        bail!("Unknown digit!")
    }

    fn read_display(&self) -> Result<u32> {
        let mut ret = 0;
        for digit in self.output.iter() {
            ret *= 10;
            ret += self.lookup_digit(digit)?;
        }
        Ok(ret)
    }
}

fn parse_input(input: &str) -> Result<Vec<SegmentDisplay>> {
    fn parse_line(line: &str) -> Result<SegmentDisplay> {
        let regex = static_regex!(r"(.*) \| (.*)");
        let caps = regex_captures(regex, line)?;
        SegmentDisplay::create(
            &capture_group(&caps, 1).split(' ').collect::<Vec<_>>(),
            &capture_group(&caps, 2).split(' ').collect::<Vec<_>>())
    }
    input.lines().map(parse_line).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE1: &str = include_str!("example1.txt");
    const EXAMPLE2: &str = include_str!("example2.txt");

    parameterized_test::create!{ easy_digits, (input, expected), {
        let example = parse_input(input).unwrap();
        assert_eq!(&example.iter().map(SegmentDisplay::count_easy_digits).collect::<Vec<_>>(), &expected);
    } }
    easy_digits! {
        one: (EXAMPLE1, [0]),
        two: (EXAMPLE2, [2, 3, 3, 1, 3, 4, 3, 1, 4, 2]),
    }

    parameterized_test::create!{ display, (input, expected), {
        let example = parse_input(input).unwrap();
        assert_eq!(&example.iter().map(SegmentDisplay::read_display).collect::<Result<Vec<_>>>().unwrap(), &expected);
    } }
    display! {
        one: (EXAMPLE1, [5353]),
        two: (EXAMPLE2, [8394, 9781, 1197, 9361, 4873, 8418, 4548, 1625, 8717, 4315]),
    }
}
