use core::fmt::{self, Display};
use core::ops::{Add, AddAssign, Deref, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn min(&self, other: Vec2) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max(&self, other: Vec2) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    pub fn clamp(&self, min: Vec2, max: Vec2) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }

    pub fn round(&self) -> Self {
        Self {
            x: libm::roundf(self.x),
            y: libm::roundf(self.y),
        }
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(value: [f32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Add<f32> for Vec2 {
    type Output = Self;

    fn add(self, other: f32) -> Self {
        Self::new(self.x + other, self.y + other)
    }
}

impl Add<Vec2> for f32 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Self::Output {
        Vec2::new(self + other.x, self + other.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl AddAssign<f32> for Vec2 {
    fn add_assign(&mut self, other: f32) {
        self.x += other;
        self.y += other;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Sub<f32> for Vec2 {
    type Output = Self;

    fn sub(self, other: f32) -> Self {
        Self::new(self.x - other, self.y - other)
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl SubAssign<f32> for Vec2 {
    fn sub_assign(&mut self, other: f32) {
        self.x -= other;
        self.y -= other;
    }
}

impl Mul for Vec2 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self::new(self.x * other, self.y * other)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, other: Vec2) -> Self::Output {
        Vec2::new(self * other.x, self * other.y)
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
    }
}

impl Div for Vec2 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self::new(self.x / other, self.y / other)
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:>8.2}, {:>8.2}]", self.x, self.y)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(bytemuck::Zeroable, bytemuck::Pod)]
pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rect {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub const ONE: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        assert!(width >= 0.0);
        assert!(height >= 0.0);

        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_points(point_a: Vec2, point_b: Vec2) -> Self {
        let min_point = point_a.min(point_b);
        let max_point = point_a.max(point_b);
        let size = max_point - min_point;

        Self {
            x: min_point.x,
            y: min_point.y,
            width: size.x,
            height: size.y,
        }
    }

    pub fn resize(&self, amount: Vec2) -> Self {
        Self {
            x: self.x,
            y: self.y,
            width: f32::max(self.width + amount.x, 0.0),
            height: f32::max(self.height + amount.y, 0.0),
        }
    }

    pub fn extend_by_point(&self, point: Vec2) -> Self {
        let min_point = self.min_point().min(point);
        let max_point = self.max_point().max(point);
        let size = max_point - min_point;

        Self {
            x: min_point.x,
            y: min_point.y,
            width: size.x,
            height: size.y,
        }
    }

    pub fn extend_by_rect(&self, rect: Self) -> Self {
        let min_point = self.min_point().min(rect.min_point());
        let max_point = self.max_point().max(rect.max_point());
        let size = max_point - min_point;

        Self {
            x: min_point.x,
            y: min_point.y,
            width: size.x,
            height: size.y,
        }
    }

    pub fn inset(&self, amount: f32) -> Self {
        assert!(amount >= 0.0);

        let x = f32::min(self.max_x(), self.x + amount);
        let y = f32::min(self.max_y(), self.y + amount);
        let width = f32::max(0.0, self.width - 2.0 * amount);
        let height = f32::max(0.0, self.height - 2.0 * amount);

        Self::new(x, y, width, height)
    }

    pub fn offset(&self, amount: f32) -> Self {
        assert!(amount >= 0.0);

        Self::new(
            self.x - amount,
            self.y - amount,
            self.width + 2.0 * amount,
            self.height + 2.0 * amount,
        )
    }

    /// Clamps a point to lie inside this rectangle.
    ///
    /// If the point initially does no lie inside this rectangle, it is moved to
    /// lie on its edge.
    pub fn clamp_point(&self, point: Vec2) -> Vec2 {
        point.clamp(self.min_point(), self.max_point())
    }

    /// Clamps another rectangle to fit inside this one.
    ///
    /// If the other rectangle has no intersection with this rectangle, a
    /// zero-area rectangle lying on the edge of this one is produced.
    pub fn clamp_rect(&self, rect: Self) -> Self {
        Rect::from_points(
            self.clamp_point(rect.min_point()),
            self.clamp_point(rect.max_point()),
        )
    }

    pub fn round(&self) -> Self {
        Self {
            x: libm::roundf(self.x),
            y: libm::roundf(self.y),
            width: libm::roundf(self.width),
            height: libm::roundf(self.height),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }

    /// Returns, whether this rectangle contains a point.
    pub fn contains_point(&self, point: Vec2) -> bool {
        let contains_x = self.x <= point.x && self.max_x() >= point.x;
        let contains_y = self.y <= point.y && self.max_y() >= point.y;

        contains_x && contains_y
    }

    /// Returns, whether this rectangle contains another.
    pub fn contains_rect(&self, rect: Self) -> bool {
        let contains_x = self.x <= rect.x && self.max_x() >= rect.max_x();
        let contains_y = self.y <= rect.y && self.max_y() >= rect.max_y();

        contains_x && contains_y
    }

    /// Returns, whether this rectangle intersects another.
    pub fn intersects_rect(&self, rect: Self) -> bool {
        let intersects_x = self.x <= rect.max_x() && self.max_x() >= rect.x;
        let intersects_y = self.y <= rect.max_y() && self.max_y() >= rect.y;

        intersects_x && intersects_y
    }

    pub fn max_x(&self) -> f32 {
        self.x + self.width
    }

    pub fn max_y(&self) -> f32 {
        self.y + self.height
    }

    pub fn min_point(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn max_point(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

impl Add<Vec2> for Rect {
    type Output = Self;

    fn add(self, other: Vec2) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            width: self.width,
            height: self.height,
        }
    }
}

impl Add<f32> for Rect {
    type Output = Self;

    fn add(self, other: f32) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
            width: self.width,
            height: self.height,
        }
    }
}

impl Sub<Vec2> for Rect {
    type Output = Self;

    fn sub(self, other: Vec2) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            width: self.width,
            height: self.height,
        }
    }
}

impl Sub<f32> for Rect {
    type Output = Self;

    fn sub(self, other: f32) -> Self {
        Self {
            x: self.x - other,
            y: self.y - other,
            width: self.width,
            height: self.height,
        }
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Rect {{ x: {}, y: {}, width: {}, height: {} }}",
            self.x, self.y, self.width, self.height,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(bytemuck::Zeroable, bytemuck::Pod)]
pub struct RectDeref {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Deref for Rect {
    type Target = RectDeref;

    fn deref(&self) -> &Self::Target {
        bytemuck::cast_ref(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    struct NiceF32(f32);

    impl quickcheck::Arbitrary for NiceF32 {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            use core::num::FpCategory;
            loop {
                let f = f32::arbitrary(g);

                match f.classify() {
                    FpCategory::Nan => continue,
                    FpCategory::Infinite => continue,
                    FpCategory::Zero => (),
                    FpCategory::Subnormal => continue,
                    FpCategory::Normal => (),
                }

                if f <= f32::from(i16::MIN) {
                    // Too small
                    continue;
                }

                if f >= f32::from(i16::MAX) {
                    // Too large
                    continue;
                }

                return Self(f);
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    struct NonNegativeNiceF32(f32);

    impl quickcheck::Arbitrary for NonNegativeNiceF32 {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            use core::num::FpCategory;
            loop {
                let f = f32::arbitrary(g);

                match f.classify() {
                    FpCategory::Nan => continue,
                    FpCategory::Infinite => continue,
                    FpCategory::Zero => (),
                    FpCategory::Subnormal => continue,
                    FpCategory::Normal => (),
                }

                if f.is_sign_negative() {
                    // Negative and -0.0
                    continue;
                }

                if f >= i16::MAX as f32 {
                    // Too large
                    continue;
                }

                return Self(f);
            }
        }
    }

    #[test]
    fn test_rect_contains_rect_contains_min_extreme() {
        let outer = Rect::new(10.0, 10.0, 100.0, 100.0);
        let inner = Rect::new(10.0, 10.0, 0.0, 0.0);

        assert!(outer.contains_rect(inner));
    }

    #[test]
    fn test_rect_contains_rect_contains_max_extreme() {
        let outer = Rect::new(10.0, 10.0, 100.0, 100.0);
        let inner = Rect::new(110.0, 110.0, 0.0, 0.0);

        assert!(outer.contains_rect(inner));
    }

    #[quickcheck]
    fn test_rect_inset_does_not_escape_original_area(
        NiceF32(x): NiceF32,
        NiceF32(y): NiceF32,
        NonNegativeNiceF32(width): NonNegativeNiceF32,
        NonNegativeNiceF32(height): NonNegativeNiceF32,
        NonNegativeNiceF32(inset_amount): NonNegativeNiceF32,
    ) -> bool {
        const EPSILON: f32 = 0.01;

        let outer = Rect::new(x, y, width, height);
        let inner = outer.inset(inset_amount);

        outer.offset(EPSILON).contains_rect(inner)
    }

    #[test]
    fn test_rect_clamp_rect() {
        let outer = Rect::new(10.0, 10.0, 100.0, 100.0);
        let inner = Rect::new(20.0, 20.0, 50.0, 50.0);

        assert!(outer.clamp_rect(inner) == inner);
    }

    #[test]
    fn test_rect_clamp_rect_clamp_min() {
        let outer = Rect::new(10.0, 10.0, 100.0, 100.0);
        let inner = Rect::new(0.0, 0.0, 100.0, 100.0);

        assert!(outer.clamp_rect(inner) == Rect::new(10.0, 10.0, 90.0, 90.0));
    }

    #[test]
    fn test_rect_clamp_rect_clamp_max() {
        let outer = Rect::new(10.0, 10.0, 100.0, 100.0);
        let inner = Rect::new(20.0, 20.0, 100.0, 100.0);

        assert!(outer.clamp_rect(inner) == Rect::new(20.0, 20.0, 90.0, 90.0));
    }

    #[test]
    fn test_rect_clamp_rect_disjoint() {
        let outer = Rect::new(10.0, 10.0, 100.0, 100.0);
        let inner = Rect::new(-50.0, -50.0, 20.0, 20.0);

        assert!(outer.clamp_rect(inner) == Rect::new(10.0, 10.0, 0.0, 0.0));
    }
}
