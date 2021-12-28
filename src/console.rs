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

    pub fn interactive_display(lazy: impl ToString, delay: std::time::Duration) {
        if interactive!() {
            let transforms = TRANSFORMS.lock().unwrap();
            let mut str = lazy.to_string();
            if !transforms.is_empty() {
                str = str.chars().map(|c| transforms.get(&c).unwrap_or(&c.to_string()).clone()).collect::<Vec<_>>().concat();
            }
            let lines = str.chars().filter(|&c| c == '\n').count()+1;
            // https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_max
            let reset_lines = RESET_LINES.fetch_max(lines, Ordering::SeqCst).max(lines);
            // in case output is shorter than RESET_LINES
            let bump = (0..(reset_lines-lines)).map(|_| '\n').collect::<String>();
            print!("{}{}\n\u{001B}[{}A", str, bump, reset_lines);
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
