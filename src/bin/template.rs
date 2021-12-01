// FIXME delete this
// https://github.com/rust-lang/cargo/issues/3591#issuecomment-475701083
#![ allow( dead_code, unused_imports, unused_macros, unused_variables ) ]

use anyhow::Result;

use advent_2021::*;

fn main() -> Result<()> {
    let _console = console::Console::init();
    let input = parse_input();
    println!("HELLO {}!", some_regex(input)?);

    Ok(())
}

fn parse_input() -> &'static str {
    include_str!("../../input/template.txt").trim()
}

fn some_regex(s: &str) -> Result<String> {
    let regex = parsing::static_regex!(r"Hello (.*)!");
    let caps = parsing::regex_captures(regex, s)?;
    Ok(parsing::capture_group(&caps, 1).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_me() { assert!(true); }

    parameterized_test::create!{ delete, n, { assert_eq!(n % 2, 0); } }
    delete! {
        me: 2,
    }
}
