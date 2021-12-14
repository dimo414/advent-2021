use anyhow::{bail, Result};

fn main() -> Result<()> {
    // https://stackoverflow.com/q/70340147/113632
    let (valid, invalid, incomplete) = partition(parse_input(include_str!("input.txt"))?);
      assert_eq!(valid.len(), 0, "Shouldn't be any valid expressions");

    println!("Invalid Score: {}", score_invalid(&invalid));
    println!("Incomplete Score: {}", score_incomplete(&incomplete));

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Status {
    Valid,
    Incomplete(Vec<char>),
    Invalid(char),
}

fn partition(statuses: Vec<Status>) -> (Vec<Status>, Vec<Status>, Vec<Status>) {
    statuses.into_iter()
        .fold((Vec::new(), Vec::new(), Vec::new()),
              |(mut valid, mut invalid, mut incomplete), status| {
                  match status {
                      Status::Valid => valid.push(status),
                      Status::Invalid(_) => invalid.push(status),
                      Status::Incomplete(_) => incomplete.push(status),
                  }
                  (valid, invalid, incomplete)
              })
}

fn score_invalid(invalid: &[Status]) -> u64 {
    invalid.iter()
        .map(|s| match s { Status::Invalid(c) => match c {
            ')' => 3,
            ']' => 57,
            '}' => 1197,
            '>' => 25137,
            _ => panic!(),
        }, _ => panic!(), })
        .sum()
}

fn score_incomplete(incomplete: &[Status]) -> u64 {
    let mut scores: Vec<u64> = incomplete.iter()
        .map(|s| match s { Status::Incomplete(s) => s, _ => panic!(), })
        .map(|s| s.iter().rev()
            .fold(0, |s, c| s*5 + match c {
                '(' => 1,
                '[' => 2,
                '{' => 3,
                '<' => 4,
                _ => panic!(),
            }))
        .collect();
    scores.sort_unstable();
    scores[scores.len()/2]
}

fn match_head(tail: char) -> Result<char> {
    Ok(match tail {
        ')' => '(',
        ']' => '[',
        '}' => '{',
        '>' => '<',
        _ => { bail!("Unexpected closing character: {}", tail) },
    })
}

fn parse(chunk: &str) -> Result<Status> {
    let mut stack = Vec::new();
    for char in chunk.chars() {
        match char {
            '('|'['|'{'|'<' => stack.push(char),
            ')'|']'|'}'|'>' => {
                match stack.pop() {
                    Some(head) => {
                        if head != match_head(char)? {
                            return Ok(Status::Invalid(char));
                        }
                    },
                    None => { return Ok(Status::Invalid(char)); }
                }
            },
            _ => { bail!("Unexpected character: {}", char) },
        }
    }

    Ok(if stack.is_empty() { Status::Valid } else { Status::Incomplete(stack) })
}

fn parse_input(input: &str) -> Result<Vec<Status>> {
    input.lines().map(parse).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let example = parse_input(include_str!("example.txt")).unwrap();

        let (valid, invalid, incomplete) = partition(example);
        assert_eq!(valid.len(), 0, "Shouldn't be any valid expressions");

        assert_eq!(score_invalid(&invalid), 26397);
        assert_eq!(score_incomplete(&incomplete), 288957);
    }

    parameterized_test::create!{ inline_examples, (example, expected), {
        assert_eq!(parse(example).unwrap(), expected);
    } }
    inline_examples! {
        valid1: ("()", Status::Valid),
        valid2: ("[]", Status::Valid),
        valid3: ("([])", Status::Valid),
        valid4: ("{()()()}", Status::Valid),
        valid5: ("<([{}])>", Status::Valid),
        valid6: ("[<>({}){}[([])<>]]", Status::Valid),
        valid7: ("(((((((((())))))))))", Status::Valid),
        invalid1: ("(]", Status::Invalid(']')),
        invalid2: ("{()()()>", Status::Invalid('>')),
        invalid3: ("(((()))}", Status::Invalid('}')),
        invalid4: ("<([]){()}[{}])", Status::Invalid(')')),
    }
}
