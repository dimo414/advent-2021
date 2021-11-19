#![allow(dead_code, unused_macros)] // TODO remove

macro_rules! interactive {
    () => {
        cfg!(feature = "interactive") || cfg!(debug_assertions) && !cfg!(test)
    };
}

#[cfg(feature="timing")]
macro_rules! elapsed {
    ($expression:expr) => { elapsed!(stringify!($expression), $expression) };
    ($desc:expr, $expression:expr) => { {
        println!("\u{001B}[36m[{}...\u{001B}[0m", $desc);
        let start = std::time::Instant::now();
        let ret = $expression;
        println!("\u{001B}[36mElapsed: {:?}]\u{001B}[0m", start.elapsed());
        ret
    } };
}
#[cfg(not(feature="timing"))]
macro_rules! elapsed {
    ($expression:expr) => { $expression };
    ($desc:expr, $expression:expr) => { $expression };
}

use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

static RESET_LINES: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref TRANSFORMS: Mutex<HashMap<char, String>> = Mutex::new(HashMap::new());
}

pub enum Color {
    /*BLACK, */RED, GREEN, YELLOW, BLUE, /*MAGENTA, CYAN,*/ GREY,
}

impl Color {
    fn ansi(&self) -> String {
        let code = match self {
            // Color::BLACK=> 30,
            Color::RED=> 31,
            Color::GREEN=> 32,
            Color::YELLOW=> 33,
            Color::BLUE=> 34,
            // Color::MAGENTA=> 35,
            // Color::CYAN=> 36,
            Color::GREY=> 37,
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
            print!("{}\n\u{001B}[{}A", str, reset_lines);
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
