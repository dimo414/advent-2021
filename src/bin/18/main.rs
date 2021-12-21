use std::collections::VecDeque;
use std::fmt::Write;
use std::ops::{Add, AddAssign};
use std::str::FromStr;
use anyhow::{anyhow, bail, Context, ensure, Error, Result};

fn main() -> Result<()> {
    let nums = parse_input(include_str!("input.txt"))?;

    let summed = sum_nums(&nums).ok_or_else(|| anyhow!("No numbers in input"))?;
    println!("Sum: {}\nMagnitude: {}", summed, summed.magnitude());

    let (a, b, mag) = max_magnitude(&nums).ok_or_else(|| anyhow!("No numbers in input"))?;
    println!("Max magnitude: {}\n    {}\n  + {}", mag, a, b);

    Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Part {
    Open,
    Close,
    N(i32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Num {
    parts: VecDeque<Part>,
}

impl Num {
    fn create(parts: &[Part]) -> Result<Num> {
        fn partial(parts: &[Part], i: usize) -> Result<usize> {
            match parts[i] {
                Part::N(_) => Ok(i + 1),
                Part::Open => {
                    let j = partial(parts, i + 1)?;
                    let k = partial(parts, j)?;
                    ensure!(k < parts.len(), "left@{} right@{} but ']'@{} missing in {:?}", i+1, j, k, parts);
                    ensure!(parts[k] == Part::Close);
                    Ok(k + 1)
                },
                _ => bail!("Unexpected: {:?} @{} in {:?}", parts[i], i, parts),
            }
        }

        ensure!(!parts.is_empty());
        ensure!(matches!(parts[0], Part::Open|Part::N(_)), "{:?}", parts);
        let end = partial(parts, 0)?;
        ensure!(end == parts.len());
        Ok(Num { parts: parts.iter().cloned().collect() })
    }

    fn reduce(&mut self) {
        loop {
            if self.do_explode() { continue; }
            if self.do_split() { continue; }
            break;
        }
        self.parts.make_contiguous();
    }

    fn do_explode(&mut self) -> bool {
        let mut depth = 0;
        for i in 0..self.parts.len() {
            match self.parts[i] {
                Part::Open => { depth += 1; },
                Part::Close => { depth -= 1; },
                Part::N(l) => {
                    if depth <= 4 { continue; }
                    // Explode!
                    debug_assert!(i+1 < self.parts.len());
                    if let Part::N(r) = self.parts[i+1] {
                        let removed = self.parts.remove(i+1);
                        debug_assert_eq!(removed, Some(Part::N(r)));
                        for j in (0..i).rev() {
                            if let Part::N(ll) = self.parts[j] {
                                self.parts[j] = Part::N(ll + l);
                                break;
                            }
                        }
                        for j in (i+1)..self.parts.len() {
                            if let Part::N(rr) = self.parts[j] {
                                self.parts[j] = Part::N(r + rr);
                                break;
                            }
                        }
                        self.parts[i] = Part::N(0);
                        let removed = self.parts.remove(i+1);
                        debug_assert_eq!(removed, Some(Part::Close));
                        let removed = self.parts.remove(i-1);
                        debug_assert_eq!(removed, Some(Part::Open));

                        return true;
                    } else { panic!("Deep pair has unexpected additional nesting @{}: {:?}", i+1, self.parts); }
                }
            }
        }
        false
    }

    fn do_split(&mut self) -> bool {
        for i in 0..self.parts.len() {
            if let Part::N(n) = self.parts[i] {
                if n > 9 {
                    self.parts[i] = Part::N(n / 2);
                    self.parts.insert(i + 1, Part::Close);
                    self.parts.insert(i + 1, Part::N((n + 1) / 2));
                    self.parts.insert(i, Part::Open);
                    return true;
                }
            }
        }
        false
    }

    fn magnitude(&self) -> i32 {
        fn partial(parts: &[Part], i: usize) -> (usize, i32) {
            match parts[i] {
                Part::N(n) => (i + 1, n),
                Part::Open => {
                    let (j, left) = partial(parts, i + 1);
                    let (k, right) = partial(parts, j);
                    assert!(k < parts.len(), "left:{} right:{} but ']'@{} missing: {:?}", left, right, k, parts);
                    assert_eq!(parts[k], Part::Close);
                    (k + 1, 3 * left + 2 * right)
                },
                _ => panic!("Unexpected: {:?} @ {} in {:?}", parts[i], i, parts)
            }
        }

        if let (parts, &[]) = self.parts.as_slices() {
            let (end, mag) = partial(parts, 0);
            assert_eq!(end, parts.len());
            return mag;
        }
        panic!()
    }
}

impl<'a, 'b> Add<&'b Num> for &'a Num {
    type Output = Num;

    fn add(self, other: &'b Num) -> Num {
        let mut parts = VecDeque::new();
        parts.push_back(Part::Open);
        for part in self.parts.iter().chain(other.parts.iter()) {
            parts.push_back(*part);
        }
        parts.push_back(Part::Close);
        let mut new = Num{ parts };
        new.reduce();
        new
    }
}

impl Add<Num> for Num {
    type Output = Num;

    fn add(self, other: Num) -> Num { &self + &other }
}

impl Add<&Num> for Num {
    type Output = Num;

    fn add(self, other: &Num) -> Num { &self + other }
}

impl Add<Num> for &Num {
    type Output = Num;

    fn add(self, other: Num) -> Num { self + &other }
}

impl AddAssign<&Num> for Num {
    fn add_assign(&mut self, num: &Num) { *self = (self as &Num) + num; }
}

impl FromStr for Num {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.replace("],", "]");

        let get_num = |i: &mut Option<usize>, j:usize| -> Result<i32> {
            let t = &s[i.expect("start index not set")..j];
            let d = t.parse::<i32>().context(format!("{}", t))?;
            *i = None;
            Ok(d)
        };

        let mut parts = Vec::new();
        let mut i = None;
        for (j, c) in s.chars().enumerate() {
            match c {
                '[' => parts.push(Part::Open),
                ']' => {
                    if i.is_some() {
                        parts.push(Part::N(get_num(&mut i, j)?));
                    }
                    parts.push(Part::Close);
                },
                ',' => {
                    parts.push(Part::N(get_num(&mut i, j)?));
                },
                '0'..='9' => {
                    if i.is_none() { i = Some(j); }
                },
                _ => bail!("Unexpected char {} @{}", c, j),
            }
        }
        if let Some(i) = i {
            // If i is still Some the string must be a single N literal, otherwise it won't parse
            let d = s[i..].parse::<i32>().context(format!("{}", &s[i..]))?;
            parts.push(Part::N(d));
        }
        Num::create(&parts)
    }
}

impl std::fmt::Display for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut out = String::new();
        for p in &self.parts {
            match p {
                Part::Open => {
                    if out.ends_with(']') { out.push(','); }
                    out.push('[');
                },
                Part::Close => out.push(']'),
                Part::N(n) => {
                    if out.ends_with(']') { out.push(','); }
                    let write_comma_after = out.ends_with('[');
                    write!(out, "{}", n).expect("impossible");
                    if write_comma_after { out.push(','); }
                }
            }
        }
        write!(f, "{}", out)
    }
}

fn parse_input(input: &str) -> Result<Vec<Num>> {
    input.lines().map(|l| l.parse()).collect()
}

// Like std::iter::Sum but doesn't return a value if there are no inputs (snailfish numbers don't
// have documented zero/one values to use as the initial value).
fn sum_nums(nums: &[Num]) -> Option<Num> {
    let mut iter = nums.iter();
    let mut ret = iter.next()?.clone();
    for num in iter {
        ret += num;
    }
    Some(ret)
}

fn max_magnitude(nums: &[Num]) -> Option<(&Num, &Num, i32)> {
    (0..nums.len()).flat_map(|i| (0..nums.len()).map(move |j| (i, j)))
        .filter(|(i, j)| i != j)
        .map(|(i, j)| (&nums[i], &nums[j]))
        .map(|(a, b)| (a, b, (a+b).magnitude()))
        .max_by_key(|(_, _, v)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_numbers() {
        let nums = parse_input(include_str!("example1.txt")).unwrap();
        for n in nums {
            let s = n.to_string();
            assert_eq!(n, s.parse().context(format!("Couldn't parse {}", s)).unwrap());
        }
    }

    #[test]
    fn example_addition() {
        let a: Num = "[1,2]".parse().unwrap();
        let b: Num = "[[3,4],5]".parse().unwrap();
        let summed = a+b;
        assert_eq!(summed, "[[1,2],[[3,4],5]]".parse().unwrap());
        assert_eq!(summed.magnitude(), 143);
    }

    parameterized_test::create!{ explodes, (input, normalized), {
        let mut input: Num = input.parse().unwrap();
        assert!(input.do_explode());
        let normalized: Num = normalized.parse().unwrap();
        assert_eq!(input, normalized);
    } }
    explodes! {
        e1: ("[[[[[9,8],1],2],3],4]", "[[[[0,9],2],3],4]"),
        e2: ("[7,[6,[5,[4,[3,2]]]]]", "[7,[6,[5,[7,0]]]]"),
        e3: ("[[6,[5,[4,[3,2]]]],1]", "[[6,[5,[7,0]]],3]"),
         // Would explode [3,2] on the next pass
        e4: ("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]", "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]"),
        e5: ("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]", "[[3,[2,[8,0]]],[9,[5,[7,0]]]]"),
    }

    parameterized_test::create!{ splits, (input, normalized), {
        let mut input: Num = input.parse().unwrap();
        assert!(input.do_split());
        let normalized: Num = normalized.parse().unwrap();
        assert_eq!(input, normalized);
    } }
    splits! {
        e1: ("10", "[5,5]"),
        e2: ("11", "[5,6]"),
        e3: ("12", "[6,6]"),
    }

    #[test]
    fn reduce() {
        let mut num: Num = "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]".parse().unwrap();
        assert!(num.do_explode());
        assert_eq!(num, "[[[[0,7],4],[7,[[8,4],9]]],[1,1]]".parse().unwrap());
        assert!(num.do_explode());
        assert_eq!(num, "[[[[0,7],4],[15,[0,13]]],[1,1]]".parse().unwrap());
        assert!(num.do_split());
        assert_eq!(num, "[[[[0,7],4],[[7,8],[0,13]]],[1,1]]".parse().unwrap());
        assert!(num.do_split());
        assert_eq!(num, "[[[[0,7],4],[[7,8],[0,[6,7]]]],[1,1]]".parse().unwrap());
        assert!(num.do_explode());
        assert_eq!(num, "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]".parse().unwrap());
        let num_copy = num.clone();
        num.reduce(); // ensure no more work to do
        assert_eq!(num, num_copy);
        assert_eq!(num.magnitude(), 1384);
    }

    parameterized_test::create!{ sums, (input, expected, magnitude), {
        let input = parse_input(input).unwrap();
        let summed = sum_nums(&input).unwrap();
        let expected: Num = expected.parse().unwrap();
        assert_eq!(summed, expected);
        assert_eq!(summed.magnitude(), magnitude);
    } }
    sums! {
        e1: ("[1,1]\n[2,2]\n[3,3]\n[4,4]", "[[[[1,1],[2,2]],[3,3]],[4,4]]", 445),
        e2: ("[1,1]\n[2,2]\n[3,3]\n[4,4]\n[5,5]", "[[[[3,0],[5,3]],[4,4]],[5,5]]", 791),
        e3: ("[1,1]\n[2,2]\n[3,3]\n[4,4]\n[5,5]\n[6,6]", "[[[[5,0],[7,4]],[5,5]],[6,6]]", 1137),
        e4: (include_str!("example2.txt"), "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]", 3488),
        e5: (include_str!("example3.txt"), "[[[[6,6],[7,6]],[[7,7],[7,0]]],[[[7,7],[7,7]],[[7,8],[9,9]]]]", 4140),
    }

    #[test]
    fn max_mag() {
        let input = parse_input(include_str!("example3.txt")).unwrap();
        let (a, b, mag) = max_magnitude(&input).unwrap();
        assert_eq!(a, &"[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]".parse().unwrap());
        assert_eq!(b, &"[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]".parse().unwrap());
        assert_eq!(mag, 3993);
    }
}
