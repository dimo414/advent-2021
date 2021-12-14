// Referenced https://docs.rs/rusttype/0.5.2/src/rusttype/geometry.rs.html
// Other resources:
//   https://crates.io/crates/euclid - https://doc.servo.org/src/euclid/point.rs.html

use std::fmt;
use std::ops::{Add,AddAssign,Sub};
use std::str::FromStr;
use std::cmp;
use anyhow::{Error,Result};

use crate::parsing::{capture_group,regex_captures,static_regex};

mod point {
    use super::*;

    #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }

    #[inline]
    pub const fn point(x: i32, y: i32, z: i32) -> Point {
        Point { x, y, z }
    }

    impl Point {
        pub const ORIGIN: Point = point(0, 0, 0);

        pub fn bounding_box<'a>(points: impl IntoIterator<Item = &'a Point>) -> Option<(Point, Point)> {
            points.into_iter().fold(None, |r , c|
                match r {
                    Some((min, max)) => {
                        Some((
                            point(cmp::min(min.x, c.x), cmp::min(min.y, c.y), cmp::min(min.z, c.z)),
                            point(cmp::max(max.x, c.x), cmp::max(max.y, c.y), cmp::max(max.z, c.z))
                        ))
                    },
                    None => Some((*c, *c)),
                }
            )
        }
    }

    impl Add<&Vector> for Point {
        type Output = Point;

        fn add(self, vec: &Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y, self.z + vec.z)
        }
    }

    impl Add<&Vector> for &Point {
        type Output = Point;

        fn add(self, vec: &Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y, self.z + vec.z)
        }
    }

    impl Add<Vector> for &Point {
        type Output = Point;

        fn add(self, vec: Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y, self.z + vec.z)
        }
    }

    impl Add<Vector> for Point {
        type Output = Point;

        fn add(self, vec: Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y, self.z + vec.z)
        }
    }

    impl AddAssign<Vector> for Point {
        fn add_assign(&mut self, vec: Vector) {
            *self = point(self.x + vec.x, self.y + vec.y, self.z + vec.z);
        }
    }

    impl Sub for Point {
        type Output = Vector;

        fn sub(self, point: Point) -> Vector { vector(self.x - point.x, self.y - point.y, self.z - point.z) }
    }

    impl Sub<&Point> for Point {
        type Output = Vector;

        fn sub(self, point: &Point) -> Vector { vector(self.x - point.x, self.y - point.y, self.z - point.z) }
    }

    impl Sub for &Point {
        type Output = Vector;

        fn sub(self, point: &Point) -> Vector { vector(self.x - point.x, self.y - point.y, self.z - point.z) }
    }

    impl FromStr for Point {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self> {
            // r"^([^,]+),([^,]+)$" would be more strict - worth it?
            let re = static_regex!(r"^\(?([^(,]+),([^),]+),([^),]+)\)?$");
            let caps = regex_captures(re, s)?;
            let x: i32 = capture_group(&caps, 1).trim().parse()?;
            let y: i32 = capture_group(&caps, 2).trim().parse()?;
            let z: i32 = capture_group(&caps, 3).trim().parse()?;
            Ok(point(x, y, z))
        }
    }

    impl fmt::Debug for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {}, {})", self.x, self.y, self.z)
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
            assert_eq!("3, 4, 5".parse::<Point>().unwrap(), point(3, 4, 5));
            assert_eq!("-3,-4,-5".parse::<Point>().unwrap(), point(-3, -4, -5));
            assert_eq!("(40,30,50)".parse::<Point>().unwrap(), point(40, 30, 50));
            assert_eq!("(-3, -5, -4)".parse::<Point>().unwrap(), point(-3, -5, -4));

            assert!("abc".parse::<Point>().is_err());
            assert!("(1, 2)".parse::<Point>().is_err());
        }

        #[test]
        fn bounding() {
            let points = vec!(point(1, 2, 3), point(2, 3, 4), point(0, 5, 3));
            assert_eq!(Point::bounding_box(&points), Some((point(0, 2, 3), point(2, 5, 4))));
        }

        #[test]
        fn add() {
            assert_eq!(point(1, 0, 2) + vector(2, 3, 1), point(3, 3, 3));
        }
        #[test]
        fn sub() {
            assert_eq!(point(3, 3, 3) - point(1, 0, 2), vector(2, 3, 1));
        }
    }
}
pub use self::point::{Point,point};

mod vector {
    use super::*;

    #[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
    pub struct Vector {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }

    #[inline]
    pub const fn vector(x: i32, y: i32, z: i32) -> Vector {
        Vector { x, y, z }
    }

    impl Vector {
        pub fn abs(&self) -> Vector {
            vector(self.x.abs(), self.y.abs(), self.z.abs())
        }

        pub fn len(&self) -> f64 {
            unimplemented!()
        }

        pub fn grid_len(&self) -> u32 {
            (self.x.abs() + self.y.abs() + self.z.abs()) as u32
        }
    }

    impl FromStr for Vector {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self> {
            // Just reuse point's parser
            let p: super::Point = s.parse()?;
            Ok(vector(p.x, p.y, p.z))
        }
    }

    impl fmt::Debug for Vector {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {}, {})", self.x, self.y, self.z)
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

        #[test]
        fn parse() {
            assert_eq!("3, 4, 5".parse::<Vector>().unwrap(), vector(3, 4, 5));
            assert_eq!("-3,-4,-5".parse::<Vector>().unwrap(), vector(-3, -4, -5));
        }

        parameterized_test::create!{ grid_lens, (p1, p2, d), {
            assert_eq!((p1 - p2).grid_len(), d);
            assert_eq!((p2 - p1).grid_len(), d);
        }}
        grid_lens! {
            a: (point(1,1,1), point(1,1,1), 0),
            b: (point(1,1,1), point(1,2,1), 1),
            c: (point(1,1,1), point(2,2,2), 3),
            d: (point(1,1,1), point(8,3,5), 13),
            e: (point(1,1,1), point(-1,-1,-1), 6),
        }
    }
}
pub use self::vector::{Vector,vector};
