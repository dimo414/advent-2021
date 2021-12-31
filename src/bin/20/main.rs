use anyhow::{anyhow, ensure, Error, Result};

use advent_2021::euclid::{point,Point,vector};
use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::Display;
use advent_2021::console::Console;
use std::time::Duration;

fn main() -> Result<()> {
    let _console = Console::init();
    if std::env::args().len() > 1 {
        // https://old.reddit.com/r/adventofcode/comments/rkgmx9/2021_day_20_images_come_to_life/
        // https://old.reddit.com/r/adventofcode/comments/rkvfov/2021_day_20_an_image_enhancement_algorithm_that/
        let (algorithm, mut image) = parse_input(include_str!("conway.txt"))?;

        Console::min_interactive_lines(10);
        for _ in 0..300 {
            image = image.enhance(&algorithm);
            Console::interactive_display(&image, Duration::from_millis(75));
        }
        Console::clear_interactive();

        return Ok(());
    }

    let (algorithm, mut image) = parse_input(include_str!("input.txt"))?;
    image = image.enhance(&algorithm);
    Console::interactive_display(&image, Duration::from_millis(200));
    image = image.enhance(&algorithm);
    let after_two = image.lit_pixels()?;
    Console::interactive_display(&image, Duration::from_millis(200));


    for _ in 2..50 {
        image = image.enhance(&algorithm);
        Console::interactive_display(&image, Duration::from_millis(200));
    }
    Console::clear_interactive();
    println!("Lit pixels after two iterations: {}", after_two);
    println!("Lit pixels after 50 iterations: {}", image.lit_pixels()?);

    Ok(())
}

#[derive(Clone)]
struct Image {
    pixels: HashMap<Point, bool>,
    background: bool,
}

impl Image {
    fn create(pixels: HashMap<Point, bool>) -> Image {
        Image{ pixels, background: false, }
    }

    fn lit_pixels(&self) -> Result<usize> {
        ensure!(!self.background, "Image has infinitely many lit pixels");
        Ok(self.pixels.values().filter(|v| **v).count())
    }

    fn enhance(&self, algorithm: &[bool]) -> Image {
        let background = algorithm[if self.background { 511 } else { 0 }];
        // Remove empty pixels from the bounds, otherwise we can end up flipping large areas of
        // nothing indefinitely. This is more of an issue for Conway than the actual problem.
        let bounds = Point::bounding_box(
            self.pixels.iter().filter(|(_, v)| **v != self.background).map(|(p, _)| p))
            .expect("Empty image");
        let mut pixels = HashMap::new();

        for y in bounds.0.y-1..=bounds.1.y+1 {
            for x in bounds.0.x-1..=bounds.1.x+1 {
                let consider = point(x, y);
                let index = self.pixel_state(consider);
                pixels.insert(consider, algorithm[index]);
            }
        }

        Image{ pixels, background }
    }

    fn pixel_state(&self, pixel: Point) -> usize {
        let mut ret = 0;
        for y in -1..=1 {
            for x in -1..=1 {
                let p = pixel + vector(x, y);
                ret <<= 1;
                if *self.pixels.get(&p).unwrap_or(&self.background) {
                    ret |= 1;
                }
            }
        }
        ret
    }
}

impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Point::display_point_map_braille(&self.pixels))
    }
}

impl FromStr for Image {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pixels = s.lines().enumerate()
            .flat_map(|(y, row)| row.chars().enumerate().map(move |(x, c)| match c {
                '#' => Ok((point(x as i32, y as i32), true)),
                '.' => Ok((point(x as i32, y as i32), false)),
                _ => Err(anyhow!("Unexpected char: {:?}", c)),
            }))
            .collect::<Result<HashMap<Point, bool>>>()?;
        Ok(Image::create(pixels))
    }
}

fn parse_input(input: &str) -> Result<(Vec<bool>, Image)> {
    let parts: Vec<_> = input.split("\n\n").collect();
    assert_eq!(parts.len(), 2);

    let algorithm = parts[0].chars().map(|c| match c {
        '#' => Ok(true),
        '.' => Ok(false),
        _ => Err(anyhow!("Unexpected char {:?}", c)),
    }).collect::<Result<Vec<_>>>()?;
    ensure!(algorithm.len() == 512);

    let image: Image = parts[1].parse()?;
    Ok((algorithm, image))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let (algorithm, mut image) = parse_input(include_str!("example.txt")).unwrap();
        image = image.enhance(&algorithm);
        image = image.enhance(&algorithm);
        assert_eq!(image.lit_pixels().unwrap(), 35);

        for _ in 2..50 {
            image = image.enhance(&algorithm);
        }
        assert_eq!(image.lit_pixels().unwrap(), 3351);
    }
}
