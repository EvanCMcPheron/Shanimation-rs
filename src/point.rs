use fast_inv_sqrt::InvSqrt64;
use num_traits::{float::Float, int::PrimInt, Num};
use std::ops::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Point<T: Num + Copy> {
    pub x: T,
    pub y: T,
}

impl<T: Num + Copy> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
    pub fn map_x<F: FnOnce(T) -> T>(self, func: F) -> Self {
        Point::new(func(self.x), self.y)
    }
    pub fn map_y<F: FnOnce(T) -> T>(self, func: F) -> Self {
        Point::new(self.x, func(self.y))
    }
    pub fn map_both<P: Num + Copy, F: Fn(T) -> P>(self, func: F) -> Point<P> {
        //! Great for unit conversion:
        //! ```
        //! let point = Point::new(1.0, 2.0);
        //! let point: Point<u32> = point.map_both(|v| v as u32);
        //! ```
        Point::new(func(self.x), func(self.y))
    }
}

impl<T: Float + Copy> Point<T> {
    pub fn angle(self) -> T {
        self.y.atan2(self.x)
    }
    pub fn to_cartesian(self) -> Point<T> {
        Point::new(self.x * self.y.cos(), self.x * self.y.sin())
    }
}

impl<T: Num + Copy + InvSqrt64 + From<f64>> Point<T> {
    pub fn length(self) -> T {
        (1.0 / (self.x * self.x + self.y * self.y).inv_sqrt64()).into()
    }
    pub fn length_squared(self) -> T {
        self.x * self.x + self.y * self.y
    }
}

impl<T: Float + Copy + InvSqrt64 + From<f64>> Point<T> {
    pub fn normalize(self) -> Self {
        let inv_square = (self.x * self.x + self.y * self.y).inv_sqrt64();
        Point::new(self.x * inv_square.into(), self.y * inv_square.into())
    }
    pub fn to_polar(self) -> Point<T> {
        //! x: radius, y: angle
        Point::new(self.length(), self.angle())
    }
}

impl<T: Num + Copy> Add<Point<T>> for Point<T> {
    type Output = Self;

    fn add(self, rhs: Point<T>) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: Num + Copy> Sub<Point<T>> for Point<T> {
    type Output = Self;

    fn sub(self, rhs: Point<T>) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Num + Copy + Copy> Mul<T> for Point<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

impl<T: Num + Copy + Copy> Div<T> for Point<T> {
    type Output = Point<T>;

    fn div(self, rhs: T) -> Self::Output {
        Point::new(self.x / rhs, self.y / rhs)
    }
}

impl<T: Num + Copy> AddAssign for Point<T> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl<T: Num + Copy> SubAssign for Point<T> {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}
