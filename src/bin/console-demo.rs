use std::time::Duration;
use advent_2021::console::{Color, Console, ConsoleImage, ToConsoleImage};

fn main() {
    let _console = Console::init();

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.get(0) {
        Some(name) => {
            match name.as_str() {
                "expand" => expand(&args[1..]),
                "image" => image(),
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
                print.push_str(&".".repeat(i));
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
                print.push_str(&".".repeat(i));
                print.push('\n');
            }
            Console::interactive_display(print, Duration::from_millis(200));
        }
        Console::clear_interactive();
    }

    fn image() {
        static COLORS: [Color; 7] = [Color::RED, Color::ORANGE, Color::YELLOW, Color::GREEN, Color::CYAN, Color::BLUE, Color::MAGENTA];
        struct Rainbow {
            offset: usize,
        }
        impl ToConsoleImage for Rainbow {
            fn render(&self) -> ConsoleImage {
                let mut pixels = Vec::new();
                for i in 0..9 { // Notice the image is only 9 pixels tall, not 10
                    for j in 0..10 {
                        let idx = 20 - i - j + self.offset;
                        pixels.push(COLORS[idx as usize % COLORS.len()]);
                    }
                }
                ConsoleImage{ pixels, width: 10 }
            }
        }

        for offset in 0..100 {
            Console::interactive_render(&Rainbow{offset}, Duration::from_millis(100));
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