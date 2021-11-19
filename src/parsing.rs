use anyhow::{Context, Result};
use regex::{Captures, Regex};

macro_rules! static_regex {
  ($pattern:literal) => {{
    lazy_static! { static ref RE: regex::Regex = regex::Regex::new($pattern).unwrap(); }
    &*RE
  }}
}

pub fn regex_captures<'a>(regex: &Regex, string: &'a str) -> Result<Captures<'a>> {
    regex.captures(string).with_context(|| format!("`{}` did not match `{}`", string, regex.as_str()))
}

pub fn capture_group<'a>(captures: &'a Captures, group: usize) -> &'a str {
    captures.get(group)
        .ok_or_else(|| format!("Invalid capture group {} for {:?}", group, captures)).unwrap().as_str()
}
