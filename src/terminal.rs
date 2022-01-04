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

trait FormattingCode {
    fn append_code(&self, out: &mut String);

    fn append_escape(&self, out: &mut String) {
        out.push_str("\x1B[");
        self.append_code(out);
        out.push('m');
    }
}

fn append_escapes(formats: &[&dyn FormattingCode], out: &mut String) {
    assert!(!formats.is_empty());
    out.push_str("\x1B[");
    for format in formats {
        format.append_code(out);
        out.push(';');
    }
    out.pop(); // Remove trailing ;
    out.push('m');
}

fn append_formatting_off(out: &mut String) {
    out.push_str("\x1B[0m");
}

#[derive(Copy, Clone)]
pub enum Color {
    BLACK, RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, GREY,
    WHITE,
    C256(u8),
    GREYSCALE(f32),
}

impl Color {
    pub const ORANGE: Color = Color::C256(214);
    pub const BROWN: Color = Color::C256(94);

    pub fn bg(&self) -> BgColor { BgColor{ color: *self } }
}

impl FormattingCode for Color {
    fn append_code(&self, out: &mut String) {
        match self {
            Color::BLACK => out.push_str("30"),
            Color::RED => out.push_str("31"),
            Color::GREEN => out.push_str("32"),
            Color::YELLOW => out.push_str("33"),
            Color::BLUE => out.push_str("34"),
            Color::MAGENTA => out.push_str("35"),
            Color::CYAN => out.push_str("36"),
            Color::GREY => out.push_str("37"),
            Color::WHITE => out.push_str("97"),
            Color::C256(code) => out.push_str(&format!("38;5;{}", code)),
            Color::GREYSCALE(f) => {
                assert!((0.0..=1.0).contains(f), "Greyscale value must be between 0 and 1");
                out.push_str(&format!("38;5;{}", (f * 24.0).round() as u32 + 232));
                // could instead use 24bit colors:
                // out.push_str(&format!("38;2;{n};{n};{n}", n=(f * 255.0).round() as u32));
            },
        }
    }
}

pub struct BgColor {
    color: Color,
}

impl FormattingCode for BgColor {
    fn append_code(&self, out: &mut String) {
        match &self.color {
            Color::BLACK => out.push_str("40"),
            Color::RED => out.push_str("41"),
            Color::GREEN => out.push_str("42"),
            Color::YELLOW => out.push_str("43"),
            Color::BLUE => out.push_str("44"),
            Color::MAGENTA => out.push_str("45"),
            Color::CYAN => out.push_str("46"),
            Color::GREY => out.push_str("47"),
            Color::WHITE => out.push_str("107"),
            Color::C256(code) => out.push_str(&format!("48;5;{}", code)),
            Color::GREYSCALE(f) => {
                assert!((0.0..=1.0).contains(f), "Greyscale value must be between 0 and 1");
                out.push_str(&format!("48;5;{}", (f * 24.0).round() as u32 + 232));
            },
        }
    }
}

pub trait TerminalRender {
    fn render(&self, width_hint: usize, height_hint: usize) -> TerminalImage;
}

pub struct TerminalImage {
    pub pixels: Vec<Color>,
    pub width: usize,
}

impl TerminalImage {
    fn truncate(mut self, max_width: usize, max_height: usize) -> TerminalImage {
        let max_height = max_height * 2; // Display fits two rows into each line of output
        // If the image is too wide the pixels vec needs to be reflowed.
        if max_width < self.width {
            let mut trunc = TerminalImage{ pixels: Vec::new(), width: max_width, };
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

impl std::fmt::Display for TerminalImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert_eq!(self.pixels.len() % self.width, 0, "Incomplete image");
        let mut out = String::new();
        out.reserve(self.pixels.len()*10);

        for r in (0..(self.pixels.len()/self.width)).step_by(2) {
            for c in 0..self.width {
                let i = self.width*r+c;
                let j = i+self.width;
                let i_color = self.pixels[i];
                if let Some(j_color) = self.pixels.get(j) {
                    append_escapes(&[&i_color, &j_color.bg()], &mut out);
                } else {
                    i_color.append_escape(&mut out);
                }
                out.push('▀');
            }
            append_formatting_off(&mut out);
            out.push('\n');
        }
        out.pop(); // Remove trailing newline
        write!(f, "{}", out)
    }
}

#[cfg(not(any(feature = "interactive",all(not(test), debug_assertions))))]
pub use self::disabled::*;
#[cfg(not(any(feature = "interactive",all(not(test), debug_assertions))))]
mod disabled {
    use super::*;
    pub struct Terminal;

    impl Terminal {
        #[inline] pub fn init() {}
        #[inline] pub fn active() -> bool { false }
        #[inline] pub fn interactive_display(_lazy: impl ToString, _delay: std::time::Duration) {}
        #[inline] pub fn interactive_render(_lazy: &impl TerminalRender, _delay: std::time::Duration) {}
        #[inline] pub fn end_interactive() {}
        #[inline] pub fn clear_interactive() {}

        // Throwaway method that calls TerminalImage.truncate() so that method is not considered unused
        #[allow(dead_code)]
        fn disregard(img: TerminalImage) {
            img.truncate(0,0);
            unreachable!();
        }
    }
}

#[cfg(any(feature = "interactive",all(not(test), debug_assertions)))]
pub use self::real::*;
#[cfg(any(feature = "interactive",all(not(test), debug_assertions)))]
mod real {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::terminal::TerminalRender;

    static CURSOR_SHIFT: AtomicUsize = AtomicUsize::new(0);
    static CLEAR_END_OF_LINE: bool = true;

    pub struct Terminal;

    // Ensures str is no more than height lines long, and no line is more than width columns wide.
    // The width requirement is difficult to enforce properly, so this is not guaranteed to work for
    // all characters. Normal-width Unicode characters without modifiers should work, but other
    // sequences such as emojis or double-width glyphs may fail to render correctly. Notably, this
    // function must be called before any terminal escape sequences are inserted.
    // str will _not_ end with a newline character after this returns.
    fn truncate_string(str: &mut String, width: usize, height: usize) {
        let (_w, _h) = (width, height);
        str.truncate(str.trim().len());
        if !CLEAR_END_OF_LINE {
            // Short-circuit if we're not adding escape sequences to each line and the string fits
            if str.lines().count() <= height && str.lines().all(|l| l.len() <= width) {
                return;
            }
        }
        let mut trunc = String::new();
        for line in str.lines().take(height) {
            trunc.extend(line.chars().take(width));
            if CLEAR_END_OF_LINE {
                trunc.push_str("\x1B[K");
            }
            trunc.push('\n');
        }
        let trimmed = trunc.pop();
        debug_assert_eq!(trimmed, Some('\n')); // Remove trailing newline
        *str = trunc;
    }

    impl Terminal {
        pub fn init() -> Cleanup {
            print!("\x1B[?25l"); // hide cursor
            Cleanup
        }

        #[inline] pub fn active() -> bool { true }

        fn interactive_print(str: String, print_height: usize) {
            debug_assert!(!str.ends_with('\n'), "String should not have trailing newlines");

            let lines = str.lines().count();
            debug_assert!(lines <= print_height, "String cannot be printed safely");

            // Never shift by more than the available height - this can happen when the window is resized
            CURSOR_SHIFT.fetch_min(print_height, Ordering::SeqCst);

            // If lines is longer than the prior shift we need to shift further up
            let prior_cursor_shift = CURSOR_SHIFT.fetch_max(lines, Ordering::SeqCst);
            let cursor_shift = prior_cursor_shift.max(lines);

            // _Then_ update the stored shift if lines is shorter than the prior shift, since the
            // cursor will not be at the bottom of the screen so we don't need to shift as far.
            CURSOR_SHIFT.fetch_min(lines, Ordering::SeqCst);

            // 1. Print sufficient blank lines to push existing text out of the way
            // 2. \e[_A moves the cursor up _ lines
            // 3. Print the str
            // 4. \e[J clears anything that happens to be below the cursor
            // 5. Newline leaves the cursor at column 1 on an empty line
            println!("{}\x1B[{}A{}\x1B[J",
                   "\n".repeat(cursor_shift-prior_cursor_shift),
                   cursor_shift,
                   str);
            //println!("[DEBUG\nLines:{} Prior Shift:{} Shift:{}\n{}\nDEBUG]", lines, prior_cursor_shift, cursor_shift, str);
        }

        // Prints the given input to the console, ensuring that it fits within the terminal window
        // and recording its height so subsequent calls to Terminal functions will overwrite it.
        // The cursor is left on the last line of the terminal at the first column, which is blank.
        pub fn interactive_display(lazy: impl ToString, delay: std::time::Duration) {
            let (term_width, term_height) = term_size::dimensions().expect("Interactive mode unsupported");
            let print_height = term_height-1; // Leave one line for the cursor
            let mut str = lazy.to_string();
            truncate_string(&mut str, term_width, print_height);
            Terminal::interactive_print(str, print_height);
            std::thread::sleep(delay);
        }

        // Prints the given input to the console as an image, ensuring that it fits within the
        // terminal window, and recording its height so subsequent calls to Terminal functions will
        // overwrite it. The cursor is left on the last line of the terminal at the first column,
        // which is blank.
        pub fn interactive_render(lazy: &impl TerminalRender, delay: std::time::Duration) {
            let (term_width, term_height) = term_size::dimensions().expect("Interactive mode unsupported");
            let print_height = term_height-1; // Leave one line for the cursor
            let image = lazy.render(term_width, print_height).truncate(term_width, print_height);
            Terminal::interactive_print(image.to_string(), print_height);
            std::thread::sleep(delay);
        }

        // Resets the interactive cursor's position, so that subsequent interactive calls will not
        // overwrite earlier output. Use this to separate blocks of interactive output (e.g. part 1
        // followed by part 2).
        pub fn end_interactive() {
            CURSOR_SHIFT.store(0, Ordering::SeqCst);
        }

        // Clears any previously printed interactive content, leaving the cursor in position to
        // overwrite the area.
        pub fn clear_interactive() {
            // Reset the cursor shift to zero
            let cursor_shift = CURSOR_SHIFT.swap(0, Ordering::SeqCst);
            // Position the cursor at the shift point and clear all below
            print!("\x1B[{}A\x1B[J", cursor_shift);
        }
    }

    // Take advantage of Drop to (attempt to) unconditionally restore the cursor. See
    // https://stackoverflow.com/a/57860708/113632 for more, or
    // https://doc.rust-lang.org/std/panic/fn.catch_unwind.html for another potential approach.
    pub struct Cleanup;
    impl Drop for Cleanup {
        fn drop(&mut self) {
            print!("\x1B[?25h"); // restore cursor
        }
    }
}