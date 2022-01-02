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
        println!("\x1B[36m[{}...\x1B[0m", $desc);
        let start = std::time::Instant::now();
        let ret = $expression;
        println!("\x1B[36mElapsed: {:?}]\x1B[0m", start.elapsed());
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

#[derive(Copy, Clone)]
pub enum Color {
    BLACK, RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, GREY,
    WHITE,
    ORANGE,
    GREYSCALE(f32),
}

impl Color {
    fn foreground(&self) -> String {
        match self {
            Color::BLACK => "30".into(),
            Color::RED => "31".into(),
            Color::GREEN => "32".into(),
            Color::YELLOW => "33".into(),
            Color::BLUE => "34".into(),
            Color::MAGENTA => "35".into(),
            Color::CYAN => "36".into(),
            Color::GREY => "37".into(),
            Color::WHITE => "97".into(),
            Color::ORANGE => "38;5;214".into(),
            Color::GREYSCALE(f) => {
                assert!((0.0..=1.0).contains(f), "Greyscale value must be between 0 and 1");
                format!("38;5;{}", (f * 24.0).round() as u32 + 232)
                // could instead use 24bit colors:
                //format!("38;2;{n};{n};{n}", n=(f * 255.0).round() as u32)
            }
        }
    }

    fn background(&self) -> String {
        match self {
            Color::BLACK => "40".into(),
            Color::RED => "41".into(),
            Color::GREEN => "42".into(),
            Color::YELLOW => "43".into(),
            Color::BLUE => "44".into(),
            Color::MAGENTA => "45".into(),
            Color::CYAN => "46".into(),
            Color::GREY => "47".into(),
            Color::WHITE => "107".into(),
            Color::ORANGE => "48;5;214".into(),
            Color::GREYSCALE(f) => {
                assert!((0.0..=1.0).contains(f), "Greyscale value must be between 0 and 1");
                format!("48;5;{}", (f * 24.0).round() as u32 + 232)
                // could instead use 24bit colors:
                //format!("48;2;{n};{n};{n}", n=(f * 255.0).round() as u32)
            }
        }
    }
}

pub struct ConsoleImage {
    pub pixels: Vec<Color>,
    pub width: usize,
}

impl ConsoleImage {
    fn truncate(mut self, max_width: usize, max_height: usize) -> ConsoleImage {
        let max_height = max_height * 2; // Display fits two rows into each line of output
        // If the image is too wide the pixels vec needs to be reflowed.
        if max_width < self.width {
            let mut trunc = ConsoleImage{ pixels: Vec::new(), width: max_width, };
            let max_len = max_height * max_width;
            let mut row = 0;
            while trunc.pixels.len() < max_len {
                trunc.pixels.extend(self.pixels.iter().skip(row*self.width).take(trunc.width));
                row += 1;
            }
            debug_assert_eq!(trunc.pixels.len(), row * trunc.width);
            return trunc;
        }
        // Otherwise we can just truncate the excess rows. This is a no-op if image isn't too tall.
        self.pixels.truncate(max_height * self.width);
        self
    }
}

impl std::fmt::Display for ConsoleImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.pixels.len() % self.width == 0, "Incomplete image");
        let mut out = String::new();
        out.reserve(self.pixels.len()*10);

        for r in (0..(self.pixels.len()/self.width)).step_by(2) {
            for c in 0..self.width {
                let i = self.width*r+c;
                let j = i+self.width;
                out.push_str("\x1B[");
                out.push_str(&self.pixels[i].foreground());
                if let Some(jj) = self.pixels.get(j) {
                    out.push(';');
                    out.push_str(&jj.background());
                }
                out.push_str("m▀");
            }
            out.push_str("\x1B[0m\n");
        }
        write!(f, "{}", out)
    }
}

pub trait ToConsoleImage {
    fn render(&self) -> ConsoleImage;
}

pub struct Console;

impl Console {
    /// Hides the cursor until the Console instance goes out of scope. Use with a _-prefixed
    /// variable to ensure it lives long enough. Using `let _ = Console::init();` does _not_ work.
    ///
    /// ```rust
    /// fn main() {
    ///   let _console = Console::init();
    ///   // ...
    /// }
    /// ```
    #[inline]
    pub fn init() -> Console {
        if interactive!() {
            print!("\x1B[?25l"); // hide cursor
        }
        Console
    }

    pub fn colorize_char(c: char, color: Color) {
        if interactive!() {
            TRANSFORMS.lock().unwrap().insert(c,  format!("\x1B[{}m█\x1B[0m", color.foreground()));
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
                if line.chars().count() >= max_width && !line.contains('\x1B') {
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
                if naive_width > width && !str.contains('\x1B') {
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
            let bump = "\x1B[K\n".repeat(reset_lines-lines);
            print!("{}{}\x1B[{}A\x1B[G", str, bump, reset_lines);
            std::thread::sleep(delay);
        }
    }

    fn truncate_image(image: ConsoleImage) -> ConsoleImage {
        match term_size::dimensions() {
            Some((width, height)) => image.truncate(width, height-1),
            None => image,
        }
    }

    pub fn interactive_render(image: &impl ToConsoleImage, delay: std::time::Duration) {
        if interactive!() {
            let str = Console::truncate_image(image.render()).to_string();
            let lines = str.lines().count();
            // https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_max
            let reset_lines = RESET_LINES.fetch_max(lines, Ordering::SeqCst).max(lines);
            // in case output is shorter than RESET_LINES
            let bump = "\x1B[K\n".repeat(reset_lines - lines);
            print!("{}{}\x1B[{}A\x1B[G", str, bump, reset_lines);
            std::thread::sleep(delay);
            std::thread::sleep(delay);
        }
    }

    pub fn clear_interactive() {
        let lines = RESET_LINES.swap(0, Ordering::SeqCst);
        print!("\x1B[{}B", lines);
    }
}

// Take advantage of Drop to (attempt to) unconditionally restore the cursor. See
// https://stackoverflow.com/a/57860708/113632 for more, or
// https://doc.rust-lang.org/std/panic/fn.catch_unwind.html for another potential approach.
#[cfg(any(feature="interactive", all(debug_assertions, not(test))))]
impl Drop for Console {
    fn drop(&mut self) {
        print!("\x1B[?25h"); // restore cursor
    }
}
