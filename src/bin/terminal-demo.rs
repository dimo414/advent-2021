use std::time::Duration;
use advent_2021::terminal::{Color, Terminal, TerminalImage, TerminalRender};

fn main() {
    let _drop = Terminal::init();

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.get(0) {
        Some(name) => {
            match name.as_str() {
                "expand" => expand(&args[1..]),
                "image" => image(),
                "one_line" => one_line(),
                _ => panic!("Unknown: {}", name),
            }
        }
        None => demo(),
    }

    fn demo() {
        expand(&[]);
        image();
        one_line();
    }

    fn expand(args: &[String]) {
        let n = args.get(0).map(|a| a.parse().expect("Invalid num")).unwrap_or(50);
        for i in (0..=n).chain((0..n).rev()) {
            let mut print = i.to_string();
            print.push('\n');
            for _ in 0..i {
                let c = format!("{}", if i%2 == 0 {'*'} else {'#'});
                print.push_str(&c.repeat(i));
                print.push('\n');
            }
            Terminal::interactive_display(print, Duration::from_millis(200));
        }
    }

    fn image() {
        static COLORS: [Color; 7] = [Color::RED, Color::ORANGE, Color::YELLOW, Color::GREEN, Color::CYAN, Color::BLUE, Color::MAGENTA];
        struct Rainbow {
            offset: usize,
        }
        impl TerminalRender for Rainbow {
            fn render(&self) -> TerminalImage {
                let width = 20;
                let mut pixels = Vec::new();
                for i in 0..(width-1) { // Notice the image an odd number of pixels tall
                    for j in 0..width {
                        let idx = 100 - i - j + self.offset;
                        pixels.push(COLORS[idx as usize % COLORS.len()]);
                    }
                }
                TerminalImage{ pixels, width, }
            }
        }

        for offset in 0..100 {
            Terminal::interactive_render(&Rainbow{offset}, Duration::from_millis(100));
        }
    }

    fn one_line() {
        for i in (0..=200).rev() {
            Terminal::interactive_display(i, Duration::from_millis(20));
        }
        std::thread::sleep(Duration::from_secs(1));
        println!("Final Message");
    }
}