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

use math::{Ray, Vec3};
use std::f64::INFINITY;

pub trait Primitive {
    /// Object ray intersection test
    fn intersects(&self, ray: Ray) -> Option<Intersection>;
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Intersection {
    pub t: f64,
    pub normal: Vec3,
}

impl Intersection {
    pub fn new(t: f64, normal: Vec3) -> Intersection {
        Intersection { t, normal }
    }

    pub fn none() -> Intersection {
        Intersection {
            t: INFINITY,
            normal: Vec3::zeros(),
        }
    }

    pub fn is_none(&self) -> bool {
        self.t == INFINITY
    }

    pub fn to_option(&self) -> Option<Intersection> {
        if self.is_none() {
            None
        } else {
            Some(*self)
        }
    }
}
