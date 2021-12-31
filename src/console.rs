use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

#[macro_export]
macro_rules! interactive {
    () => {
        cfg!(feature = "interactive") || cfg!(debug_assertions) && !cfg!(test)
    }
}
pub use interactive;

#[cfg(feature="timing")]
#[macro_export]
macro_rules! elapsed {
    ($expression:expr) => { elapsed!(stringify!($expression), $expression) };
    ($desc:expr, $expression:expr) => { {
        println!("\u{001B}[36m[{}...\u{001B}[0m", $desc);
        let start = std::time::Instant::now();
        let ret = $expression;
        println!("\u{001B}[36mElapsed: {:?}]\u{001B}[0m", start.elapsed());
        ret
    } }
}
#[cfg(not(feature="timing"))]
#[macro_export]
macro_rules! elapsed {
    ($expression:expr) => { $expression };
    ($desc:expr, $expression:expr) => { $expression };
}
pub use elapsed;

static RESET_LINES: AtomicUsize = AtomicUsize::new(0);

lazy_static::lazy_static! {
    static ref TRANSFORMS: Mutex<HashMap<char, String>> = Mutex::new(HashMap::new());
}

pub enum Color {
    BLACK, RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, GREY,
    WHITE,
    GREYSCALE(f32),
}

impl Color {
    fn ansi(&self) -> String {
        let code = match self {
            Color::BLACK => "30".into(),
            Color::RED => "31".into(),
            Color::GREEN => "32".into(),
            Color::YELLOW => "33".into(),
            Color::BLUE => "34".into(),
            Color::MAGENTA => "35".into(),
            Color::CYAN => "36".into(),
            Color::GREY => "37".into(),
            Color::WHITE => "97".into(),
            Color::GREYSCALE(f) => {
                assert!((0.0..=1.0).contains(f), "Greyscale value must be between 0 and 1");
                format!("38;5;{}", (f * 24.0).round() as u32 + 232)
                // could instead use 24bit colors:
                //format!("38;2;{n};{n};{n}", n=(f * 255.0).round() as u32)
            }
        };
        format!("\u{001B}[{}m█\u{001B}[0m", code)
    }
}

pub struct Console;

impl Console {
    #[inline]
    pub fn init() -> Console {
        if interactive!() {
            print!("\u{001B}[?25l"); // hide cursor
        }
        Console
    }

    pub fn colorize_char(c: char, color: Color) {
        if interactive!() {
            TRANSFORMS.lock().unwrap().insert(c,  color.ansi());
        }
    }

    fn random_dots() -> String {
        let mut chars = [' ', '~', '~', '~', '~'];
        fastrand::shuffle(&mut chars);
        chars.iter().collect()
    }

    fn truncate_for_display(str: &mut String) {
        fn truncate_columns(lines: &[&str], max_width: usize) -> String {
            let mut dots = Console::random_dots();
            let mut ret = String::new();
            for line in lines {
                if line.chars().count() >= max_width {
                    let mut line: String = line.chars().take(max_width - 1).collect();
                    if let Some(dot) = dots.pop() {
                        line.push(dot);
                    }
                    ret.push_str(&line);
                } else {
                    ret.push_str(line);
                }
                ret.push('\n');
            }
            ret
        }

        if let Some((width, height)) = term_size::dimensions() {
            // don't bother with oddly-small windows, output's probably already a mess
            if width < 10 || height < 10 { return; }
            let height = height-1;
            let num_lines = str.lines().count();
            if num_lines > height {
                let dots = Console::random_dots();
                let lines: Vec<_> = str.lines().take(height-1).chain(std::iter::once(dots.as_str())).collect();
                *str = truncate_columns(&lines, width);
            } else {
                let naive_width = str.lines().map(|l| l.len()).max().unwrap_or(0);
                debug_assert!(naive_width >= str.lines().map(|l| l.chars().count()).max().unwrap_or(0));
                if naive_width > width {
                    *str = truncate_columns(&str.lines().collect::<Vec<_>>(), width);
                }
            }
        }
    }

    pub fn min_interactive_lines(lines: usize) {
        RESET_LINES.fetch_max(lines, Ordering::SeqCst);
    }

    pub fn interactive_display(lazy: impl ToString, delay: std::time::Duration) {
        if interactive!() {
            let transforms = TRANSFORMS.lock().unwrap();
            let mut str = lazy.to_string();
            if !str.ends_with('\n') {
                str.push('\n');
            }
            Console::truncate_for_display(&mut str); // Do this before color transformations
            if !transforms.is_empty() {
                str = str.chars().map(|c| transforms.get(&c).unwrap_or(&c.to_string()).clone()).collect::<Vec<_>>().concat();
            }
            let lines = str.lines().count();
            // https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_max
            let reset_lines = RESET_LINES.fetch_max(lines, Ordering::SeqCst).max(lines);
            // in case output is shorter than RESET_LINES
            let bump = (lines..reset_lines).map(|_| '\n').collect::<String>();
            print!("\u{001B}[J{}{}\u{001B}[{}A\u{001B}[G", str, bump, reset_lines);
            std::thread::sleep(delay);
        }
    }

    pub fn clear_interactive() {
        let lines = RESET_LINES.swap(0, Ordering::SeqCst);
        print!("\u{001B}[{}B", lines);
    }
}

// Take advantage of Drop to (attempt to) unconditionally restore the cursor. See
// https://stackoverflow.com/a/57860708/113632 for more, or
// https://doc.rust-lang.org/std/panic/fn.catch_unwind.html for another potential approach.
#[cfg(any(feature="interactive", all(debug_assertions, not(test))))]
impl Drop for Console {
    fn drop(&mut self) {
        print!("\u{001B}[?25h"); // restore cursor
    }
}
