use std::fmt;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Coord {
    fn from(point: (usize, usize)) -> Self {
        Coord {
            x: point.0,
            y: point.1,
        }
    }
}

impl Default for Coord {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl Add for Coord {
    type Output = Coord;
    fn add(self, other: Coord) -> Coord {
        Coord {
            x: self.x.saturating_add(other.x),
            y: self.y.saturating_add(other.y),
        }
    }
}

impl Sub for Coord {
    type Output = Coord;
    fn sub(self, other: Coord) -> Coord {
        Coord {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
        }
    }
}
