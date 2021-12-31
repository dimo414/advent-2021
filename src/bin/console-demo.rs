use std::time::Duration;
use advent_2021::console::Console;

fn main() {
    let _ = Console::init();

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.get(0) {
        Some(name) => {
            match name.as_str() {
                "expand" => expand(&args[1..]),
                "one_line" => one_line(),
                "shrink" => shrink(&args[1..]),
                _ => panic!("Unknown"),
            }
        }
        None => demo(),
    }

    fn demo() {
        shrink(&[]);
        Console::clear_interactive();
        expand(&[]);
        Console::clear_interactive();
        one_line();
    }

    fn expand(args: &[String]) {
        for i in 0..=args.get(0).map(|a| a.parse().unwrap()).unwrap_or(50) {
            let mut print = i.to_string();
            print.push('\n');
            for _ in 0..i {
                print.push_str(&(0..i).map(|_| '.').collect::<String>());
                print.push('\n');
            }
            Console::interactive_display(print, Duration::from_millis(200));
        }
        Console::clear_interactive();
    }

    fn shrink(args: &[String]) {
        for i in (0..=args.get(0).map(|a| a.parse().unwrap()).unwrap_or(50)).rev() {
            let mut print = i.to_string();
            print.push('\n');
            for _ in 0..i {
                print.push_str(&(0..i).map(|_| '.').collect::<String>());
                print.push('\n');
            }
            Console::interactive_display(print, Duration::from_millis(200));
        }
        Console::clear_interactive();
    }

    fn one_line() {
        for i in 0..=500 {
            Console::interactive_display(i, Duration::from_millis(10));
        }
        Console::clear_interactive();
    }
}