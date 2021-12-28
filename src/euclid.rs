// Referenced https://docs.rs/rusttype/0.5.2/src/rusttype/geometry.rs.html
// Other resources:
//   https://crates.io/crates/euclid - https://doc.servo.org/src/euclid/point.rs.html
mod point {
    use std::collections::{HashMap, HashSet};
    use super::*;
    use std::fmt;
    use std::fmt::Write;
    use std::ops::{Add,AddAssign,Sub};
    use std::str::FromStr;
    use anyhow::{Error, Result};
    use crate::parsing::{static_regex,capture_group,regex_captures};

    #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[inline]
    pub const fn point(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    impl Point {
        pub const ORIGIN: Point = point(0, 0);

        pub fn bounding_box<'a>(points: impl IntoIterator<Item = &'a Point>) -> Option<(Point, Point)> {
            points.into_iter().fold(None, |r , c|
                match r {
                    Some((min, max)) => {
                        Some((
                            point(std::cmp::min(min.x, c.x), std::cmp::min(min.y, c.y)),
                            point(std::cmp::max(max.x, c.x), std::cmp::max(max.y, c.y))
                        ))
                    },
                    None => Some((*c, *c)),
                }
            )
        }

        pub fn display_order<'a>(points: impl IntoIterator<Item = &'a Point>) -> Option<impl Iterator<Item = impl Iterator<Item = Point>>> {
            if let Some((min, max)) = Point::bounding_box(points) {
                return Some((min.y..=max.y)
                    .map(move |y| (min.x..=max.x).map(move |x| point(x, y))));
            }
            None
        }

        pub fn display_points<'a>(points: impl IntoIterator<Item = &'a Point>, present: char, absent: char) -> String {
            let points: HashSet<_> = points.into_iter().cloned().collect();
            Self::display_point_set(&points, present, absent)
        }

        pub fn display_point_set(points: &HashSet<Point>, present: char, absent: char) -> String {
            let mut out = String::new();
            if points.is_empty() { return out; }
            for row in Point::display_order(points.iter()).expect("not empty") {
                for p in row {
                    if points.contains(&p) {
                        out.push(present);
                    } else {
                        out.push(absent);
                    }
                }
                out.push('\n');
            }
            out
        }

        pub fn display_point_map<V, F: Fn(Option<&V>) -> String>(map: &HashMap<Point, V>, render: F) -> String {
            let mut out = String::new();
            if map.is_empty() { return out; }
            for row in Point::display_order(map.keys()).expect("not empty") {
                for p in row {
                    write!(out, "{}", render(map.get(&p))).expect("impossible");
                }
                out.push('\n');
            }
            out
        }

        // https://en.wikipedia.org/wiki/Braille_Patterns#Identifying,_naming_and_ordering
        pub fn points_to_braille<F: Fn(&Point)->bool>(left_corner: Point, contains: F) -> char {
            let braille_blank = 0x2800;
            // in bit order from left-to-right, dot 1 is bit 8, dot 8 is bit 1
            let dots = [
                vector(1, 3),
                vector(0, 3),
                vector(1, 2),
                vector(1, 1),
                vector(1, 0),
                vector(0, 2),
                vector(0,1),
                vector(0, 0),
            ];
            let mut braille_parts = 0;
            for dot in dots {
                braille_parts <<= 1;
                if contains(&(left_corner + dot)) {
                    braille_parts |= 1;
                }
            }
            char::from_u32(braille_parts + braille_blank).expect("Valid Braille char")
        }

        pub fn display_point_set_braille(points: &HashSet<Point>) -> String {
            let mut out = String::new();
            out.push_str("\u{001B}[1m");
            if points.is_empty() { return out; }
            let (min, max) = Point::bounding_box(points).expect("Non-empty");
            for y in (min.y..=max.y).step_by(4) {
                for x in (min.x..=max.x).step_by(2) {
                    out.push(Point::points_to_braille(point(x, y), |p| points.contains(p)));
                }
                out.push('\n');
            }
            out.push_str("\u{001B}[0m");
            out
        }

        pub fn display_point_map_braille(points: &HashMap<Point, bool>) -> String {
            let mut out = String::new();
            out.push_str("\u{001B}[1m");
            if points.is_empty() { return out; }
            let (min, max) = Point::bounding_box(points.keys()).expect("Non-empty");
            for y in (min.y..=max.y).step_by(4) {
                for x in (min.x..=max.x).step_by(2) {
                    out.push(Point::points_to_braille(point(x, y), |p| *points.get(p).unwrap_or(&false)));
                }
                out.push('\n');
            }
            out.push_str("\u{001B}[0m");
            out
        }

        pub fn in_bounds(&self, min: Point, max: Point) -> bool {
            assert!(min.x <= max.x);
            assert!(min.y <= max.y);
            min.x <= self.x && min.y <= self.y && max.x >= self.x && max.y >= self.y
        }
    }

    impl Add<&Vector> for Point {
        type Output = Point;

        fn add(self, vec: &Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y)
        }
    }

    impl Add<&Vector> for &Point {
        type Output = Point;

        fn add(self, vec: &Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y)
        }
    }

    impl Add<Vector> for &Point {
        type Output = Point;

        fn add(self, vec: Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y)
        }
    }

    impl Add<Vector> for Point {
        type Output = Point;

        fn add(self, vec: Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y)
        }
    }

    impl AddAssign<Vector> for Point {
        fn add_assign(&mut self, vec: Vector) {
            *self = point(self.x + vec.x, self.y + vec.y);
        }
    }

    impl AddAssign<&Vector> for Point {
        fn add_assign(&mut self, vec: &Vector) {
            *self = point(self.x + vec.x, self.y + vec.y);
        }
    }

    impl Sub for Point {
        type Output = Vector;

        fn sub(self, point: Point) -> Vector { vector(self.x - point.x, self.y - point.y) }
    }

    impl Sub for &Point {
        type Output = Vector;

        fn sub(self, point: &Point) -> Vector { vector(self.x - point.x, self.y - point.y) }
    }

    impl FromStr for Point {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self> {
            // r"^([^,]+),([^,]+)$" would be more strict - worth it?
            let regex = static_regex!(r"^\(?([^(,]+),([^),]+)\)?$");
            let caps = regex_captures(regex, s)?;
            let x: i32 = capture_group(&caps, 1).trim().parse()?;
            let y: i32 = capture_group(&caps, 2).trim().parse()?;
            Ok(point(x, y))
        }
    }

    impl fmt::Debug for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }

    impl fmt::Display for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse() {
            assert_eq!("3, 4".parse::<Point>().unwrap(), point(3, 4));
            assert_eq!("-3,-4".parse::<Point>().unwrap(), point(-3, -4));
            assert_eq!("(40,30)".parse::<Point>().unwrap(), point(40, 30));
            assert_eq!("(-3, -5)".parse::<Point>().unwrap(), point(-3, -5));

            assert!("abc".parse::<Point>().is_err());
        }

        #[test]
        fn bounding() {
            let points = vec!(point(1, 2), point(2, 3), point(0, 5));
            assert_eq!(Point::bounding_box(&points), Some((point(0, 2), point(2, 5))));
        }

        #[test]
        fn display_bounds() {
            let points = vec!(point(1, 2), point(3, 1), point(0, -1));
            let display_points: Vec<Vec<_>> = Point::display_order(&points).unwrap()
                .map(|row| row.collect())
                .collect();
            assert_eq!(display_points, vec!(
                vec!(point(0, -1), point(1, -1), point(2, -1), point(3, -1)),
                vec!(point(0, 0), point(1, 0), point(2, 0), point(3, 0)),
                vec!(point(0, 1), point(1, 1), point(2, 1), point(3, 1)),
                vec!(point(0, 2), point(1, 2), point(2, 2), point(3, 2)),
            ));
        }

        #[test]
        fn display_points() {
            let points = &[point(1,1), point(2,2), point(3,3), point(4,0)];
            assert_eq!(Point::display_points(points, '#', ' '), "   #\n#   \n #  \n  # \n");

            // Bounding box is _not_ fixed at the origin
            assert_eq!(Point::display_points(&[point(1,1), point(2,2), point(3,3)], '#', ' '),
                       Point::display_points(&[point(10,10), point(11,11), point(12,12)], '#', ' '));
        }

        #[test]
        fn display_map() {
            let map: HashMap<Point, char> = [(point(1,1), 'O'), (point(2,2), 'T'), (point(4,4), 'F')].iter().cloned().collect();
            assert_eq!(
                Point::display_point_map(&map, |v| v.map(|c| c.to_string()).unwrap_or_else(|| " ".to_string())),
                "O   \n T  \n    \n   F\n");
        }

        #[test]
        fn braille_chars() {
            let dots125: HashSet<_> = [point(0, 0), point(0, 1), point(1, 1)].into_iter().collect();
            assert_eq!(Point::points_to_braille(point(0,0), |p| dots125.contains(p)), '⠓');
            let dots12378: HashSet<_> = [point(0, 0), point(0, 1), point(0, 2), point(0, 3), point(1, 3)].into_iter().collect();
            assert_eq!(Point::points_to_braille(point(0,0), |p| dots12378.contains(p)), '⣇');

            let left_corner = point(10, 5);
            // dots ordered 5, 3, 8, 7, 6, 2, 4, 1
            let dots =  [
                (vector(1, 1), '⠐'),
                (vector(0, 2), '⠔'),
                (vector(1, 3), '⢔'),
                (vector(0, 3), '⣔'),
                (vector(1, 2), '⣴'),
                (vector(0, 1), '⣶'),
                (vector(1, 0), '⣾'),
                (vector(0, 0), '⣿'),
            ];
            let mut points = HashSet::new();
            assert_eq!(Point::points_to_braille(left_corner, |p| points.contains(p)), '\u{2800}');
            for (dot, c) in dots {
                points.insert(left_corner + dot);
                assert_eq!(Point::points_to_braille(left_corner, |p| points.contains(p)), c);
            }
        }

        #[test]
        fn braille_display() {
            let points: HashSet<_> = [point(1,1), point(2,2), point(3,3), point(4,4), point(5,5), point(0,4)]
                .into_iter().collect();
            // ⡈⠢⡀
            // ⠀⠀⠈
            assert_eq!(&Point::display_point_set_braille(&points), "\u{1b}[1m⡈⠢⡀\n⠀⠀⠈\n\u{1b}[0m");
        }

        #[test]
        fn in_bounds() {
            let zero_zero = point(0, 0);
            let two_two = point(2, 2);
            let five_six = point(5, 6);
            assert!(two_two.in_bounds(zero_zero, two_two));
            assert!(!five_six.in_bounds(zero_zero, two_two));
        }

        #[test]
        fn add() {
            assert_eq!(point(1, 0) + super::super::vector(2, 3), point(3, 3));
        }
        #[test]
        fn sub() {
            assert_eq!(point(3, 3) - point(1, 0), super::super::vector(2, 3));
        }
    }
}
pub use self::point::{Point,point};

mod vector {
    use std::fmt;
    use std::str::FromStr;
    use std::ops::{Add,AddAssign,Mul};
    use anyhow::{Error, Result};

    #[derive(Copy, Clone, PartialEq, Eq, Hash)]
    pub struct Vector {
        pub x: i32,
        pub y: i32,
    }

    #[inline]
    pub const fn vector(x: i32, y: i32) -> Vector {
        Vector { x, y }
    }

    impl Vector {
        pub const ZERO: Vector = vector(0, 0);

        // https://en.wikipedia.org/wiki/Points_of_the_compass
        pub const CARDINAL: &'static [Vector] = &[
            vector(-1, 0), vector(0, -1), vector(1, 0), vector(0, 1)];
        pub const ORDINAL: &'static [Vector] = &[
            vector(-1, 0), vector(-1, -1), vector(0, -1), vector(1, -1),
            vector(1, 0), vector(1, 1), vector(0, 1), vector(-1, 1)];

        pub fn abs(&self) -> Vector {
            vector(self.x.abs(), self.y.abs())
        }

        pub fn len(&self) -> f64 {
            (self.x as f64).hypot(self.y as f64)
        }

        pub fn grid_len(&self) -> u32 {
            (self.x.abs() + self.y.abs()) as u32
        }
    }

    impl Add<Vector> for Vector {
        type Output = Vector;

        fn add(self, vec: Vector) -> Vector {
            vector(self.x + vec.x, self.y + vec.y)
        }
    }

    impl Add<&Vector> for Vector {
        type Output = Vector;

        fn add(self, vec: &Vector) -> Vector {
            vector(self.x + vec.x, self.y + vec.y)
        }
    }

    impl AddAssign<Vector> for Vector {
        fn add_assign(&mut self, vec: Vector) {
            *self = vector(self.x + vec.x, self.y + vec.y);
        }
    }

    impl AddAssign<&Vector> for Vector {
        fn add_assign(&mut self, vec: &Vector) {
            *self = vector(self.x + vec.x, self.y + vec.y);
        }
    }

    impl Mul<i32> for Vector {
        type Output = Vector;

        fn mul(self, m: i32) -> Vector {
            vector(self.x * m, self.y * m)
        }
    }

    impl FromStr for Vector {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self> {
            // Just reuse point's parser
            let p: super::Point = s.parse()?;
            Ok(vector(p.x, p.y))
        }
    }

    impl fmt::Debug for Vector {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }

    impl fmt::Display for Vector {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::super::point;
        use super::*;
        use assert_approx_eq::assert_approx_eq;

        #[test]
        fn parse() {
            assert_eq!("3, 4".parse::<Vector>().unwrap(), vector(3, 4));
            assert_eq!("-3,-4".parse::<Vector>().unwrap(), vector(-3, -4));
        }

        #[test]
        fn len() {
            assert_approx_eq!(vector(3, -4).len(), 5_f64, f64::EPSILON);
        }

        parameterized_test::create!{ grid_lens, (p1, p2, d), {
            assert_eq!((p1 - p2).grid_len(), d);
            assert_eq!((p2 - p1).grid_len(), d);
        }}
        grid_lens! {
            a: (point(1,1), point(1,1), 0),
            b: (point(1,1), point(1,2), 1),
            c: (point(1,1), point(2,2), 2),
            d: (point(1,1), point(1,5), 4),
            e: (point(1,1), point(8,3), 9),
            f: (point(1,1), point(-1,-1), 4),
        }
    }
}
pub use self::vector::{Vector,vector};
