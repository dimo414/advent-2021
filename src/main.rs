// https://github.com/rust-lang/cargo/issues/3591#issuecomment-475701083
//#![ allow( dead_code, unused_imports, unused_macros, unused_variables ) ]
#[macro_use] extern crate lazy_static;
extern crate parameterized_test;
extern crate regex;
extern crate anyhow;

use std::env;

#[macro_use] mod console;
#[macro_use] mod parsing;
mod euclid;

mod aoc01;

fn main() {
    let _console = console::Console::init();
    println!(); // split build output from runtime output
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} DAY_OF_ADVENT", args[0]);
        return;
    }
    let day: u32 = args[1].parse().expect("Should be a natural number");
    match day {
        1 => aoc01::advent(),
        x => {
            eprintln!("Day {} hasn't happened yet.", x);
            ::std::process::exit(1);
        },
    }
}
