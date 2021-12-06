// FIXME delete this
// https://github.com/rust-lang/cargo/issues/3591#issuecomment-475701083
#![ allow( dead_code, unused_imports, unused_macros, unused_variables ) ]

use anyhow::Result;

use advent_2021::parsing::*;

fn main() -> Result<()> {
    let input = parse_input(include_str!("input.txt"));
    println!("HELLO {}!", some_regex(input)?);

    Ok(())
}

fn parse_input(input: &str) -> &str {
    input.trim()
}

fn some_regex(s: &str) -> Result<String> {
    let regex = static_regex!(r"Hello (.*)!");
    let caps = regex_captures(regex, s)?;
    Ok(capture_group(&caps, 1).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_me() { parse_input(include_str!("example.txt")); }

    parameterized_test::create!{ delete, n, { assert_eq!(n % 2, 0); } }
    delete! {
        me: 2,
    }
}
