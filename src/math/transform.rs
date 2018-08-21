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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_affine_transforms() {
        let t = AffineTransform::new(-100.0, -100.0, 2.0, 2.0);
        assert_eq!(t.inverse(-100.0, -100.0), (0.0, 0.0));
        assert_eq!(t.inverse(100.0, -100.0), (100.0, 0.0));
        assert_eq!(t.inverse(100.0, 100.0), (100.0, 100.0));
        assert_eq!(t.inverse(-100.0, 100.0), (0.0, 100.0));
    }
}
