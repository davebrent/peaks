// This file is part of Peaks.
//
// Peaks is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Peaks is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Peaks. If not, see <https://www.gnu.org/licenses/>.

use std::default::Default;
use std::fmt;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// Round to the nearest power of two
pub fn ceil_pow2(num: usize) -> usize {
    let num = num as f64;
    let exp = (num.log2() / 2.0_f64.log2()).ceil();
    2.0_f64.powf(exp) as usize
}

/// Represents a simple linear transformation
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AffineTransform {
    transform: [f64; 4],
}

impl AffineTransform {
    pub fn new(e: f64, f: f64, a: f64, d: f64) -> AffineTransform {
        AffineTransform {
            transform: [e, f, a, d],
        }
    }

    /// Return the result of the transform
    #[inline(always)]
    pub fn forward(&self, x: f64, y: f64) -> (f64, f64) {
        let [e, f, a, d] = self.transform;
        let x = e + x * a;
        let y = f + y * d;
        (x, y)
    }

    /// Return the transformation across quadtree levels
    #[inline(always)]
    pub fn quadtree(&self, level: usize, x: f64, y: f64) -> (f64, f64) {
        let [e, f, a, d] = self.transform;
        let a = a * (2_usize.pow(level as u32) as f64);
        let d = d * (2_usize.pow(level as u32) as f64);
        (e + x * a, f + y * d)
    }

    /// Return the inverse of the transform
    #[inline(always)]
    pub fn inverse(&self, x: f64, y: f64) -> (f64, f64) {
        let [e, f, a, d] = self.transform;
        let x = (x - e) / a;
        let y = (y - f) / d;
        (x, y)
    }

    /// Return the x/y units size of the transform
    #[inline(always)]
    pub fn unit_size(&self) -> (f64, f64) {
        (self.transform[2], self.transform[3])
    }
}

impl Default for AffineTransform {
    fn default() -> AffineTransform {
        AffineTransform::new(0.0, 0.0, 1.0, 1.0)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.r, self.g, self.b)
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.4}, {:.4}, {:.4})", self.x, self.y, self.z)
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(origin: {}, direction: {})",
            self.origin, self.direction
        )
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

impl Add for Color {
    type Output = Color;

    #[inline(always)]
    fn add(self, rhs: Color) -> Color {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    #[inline(always)]
    fn mul(self, rhs: f64) -> Color {
        Color {
            r: (f64::from(self.r) * rhs).round().min(255.0) as u8,
            g: (f64::from(self.g) * rhs).round().min(255.0) as u8,
            b: (f64::from(self.b) * rhs).round().min(255.0) as u8,
        }
    }
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn zeros() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[inline(always)]
    pub fn dot(a: Vec3, b: Vec3) -> f64 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    #[inline(always)]
    pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
        Vec3::new(
            a.y * b.z - a.z * b.y,
            a.z * b.x - a.x * b.z,
            a.x * b.y - a.y * b.x,
        )
    }

    #[inline(always)]
    pub fn distance(a: Vec3, b: Vec3) -> f64 {
        let diff = b - a;
        (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z).sqrt()
    }

    #[inline(always)]
    pub fn sqrt(a: Vec3) -> Vec3 {
        Vec3::new(a.x.sqrt(), a.y.sqrt(), a.z.sqrt())
    }

    #[inline(always)]
    pub fn normalize(a: Vec3) -> Vec3 {
        let len = Vec3::dot(a, a);
        if len > 0.0 {
            return a * 1.0 / len.sqrt();
        }
        Default::default()
    }

    #[inline(always)]
    pub fn round(&self) -> Vec3 {
        Vec3::new(self.x.round(), self.y.round(), self.z.round())
    }

    #[inline(always)]
    pub fn abs(&self) -> Vec3 {
        Vec3::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    #[inline(always)]
    pub fn integral(&self) -> Vec3 {
        Vec3::new(
            if self.x > 0.0 {
                self.x.floor()
            } else {
                self.x.ceil()
            },
            if self.y > 0.0 {
                self.y.floor()
            } else {
                self.y.ceil()
            },
            if self.z > 0.0 {
                self.z.floor()
            } else {
                self.z.ceil()
            },
        )
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    #[inline(always)]
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<f64> for Vec3 {
    type Output = Vec3;

    #[inline(always)]
    fn add(self, rhs: f64) -> Vec3 {
        Vec3::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl AddAssign for Vec3 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Vec3) {
        *self = Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z);
    }
}

impl AddAssign<f64> for Vec3 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: f64) {
        *self = Vec3::new(self.x + rhs, self.y + rhs, self.z + rhs);
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    #[inline(always)]
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Vec3 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Vec3) {
        *self = Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z);
    }
}

impl SubAssign<f64> for Vec3 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: f64) {
        *self = Vec3::new(self.x - rhs, self.y - rhs, self.z - rhs);
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, rhs: f64) -> Vec3 {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign for Vec3 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Vec3) {
        *self = Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z);
    }
}

impl MulAssign<f64> for Vec3 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f64) {
        *self = Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs);
    }
}

impl Div<Vec3> for Vec3 {
    type Output = Vec3;

    #[inline(always)]
    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    #[inline(always)]
    fn div(self, rhs: f64) -> Vec3 {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign for Vec3 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Vec3) {
        *self = Vec3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z);
    }
}

impl DivAssign<f64> for Vec3 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: f64) {
        *self = Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs);
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    #[inline(always)]
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_between_points() {
        let a = Vec3::new(0.0, 0.0, 10.0);
        let b = Vec3::new(0.0, 0.0, 20.0);
        assert_eq!(Vec3::distance(a, b), 10.0);
        assert_eq!(Vec3::distance(b, a), 10.0);
    }

    #[test]
    fn inverse_affine_transforms() {
        let t = AffineTransform::new(-100.0, -100.0, 2.0, 2.0);
        assert_eq!(t.inverse(-100.0, -100.0), (0.0, 0.0));
        assert_eq!(t.inverse(100.0, -100.0), (100.0, 0.0));
        assert_eq!(t.inverse(100.0, 100.0), (100.0, 100.0));
        assert_eq!(t.inverse(-100.0, 100.0), (0.0, 100.0));
    }
}
