use anyhow::{Context, Result};
use regex::{Captures, Regex};

#[macro_export]
macro_rules! static_regex {
  ($pattern:literal) => {{
    lazy_static::lazy_static! { static ref RE: regex::Regex = regex::Regex::new($pattern).unwrap(); }
    &*RE
  }}
}
pub use static_regex;

pub fn regex_captures<'a>(regex: &Regex, string: &'a str) -> Result<Captures<'a>> {
    regex.captures(string).with_context(|| format!("`{}` did not match `{}`", string, regex.as_str()))
}

pub fn capture_group<'a>(captures: &'a Captures, group: usize) -> &'a str {
    captures.get(group)
        .ok_or_else(|| format!("Invalid capture group {} for {:?}", group, captures)).unwrap().as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches() {
        let re = static_regex!("Foo.*(B.*r)");
        let caps = regex_captures(re, "FooFarBaaar").unwrap();
        assert_eq!(capture_group(&caps, 0), "FooFarBaaar");
        assert_eq!(capture_group(&caps, 1), "Baaar");
    }

    #[test]
    fn no_match() {
        let re = static_regex!("Foo.*(B.*r)");
        let caps = regex_captures(re, "Foo");
        let err = caps.expect_err("Should not match").to_string();
        assert!(err.contains("did not match"), "Was: {}", err);
    }
}
