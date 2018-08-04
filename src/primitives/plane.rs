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
use render::{Intersectable, Intersection};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Plane {
    normal: Vec3,
    distance: f64,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f64) -> Plane {
        Plane { normal, distance }
    }
}

impl Intersectable for Plane {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        let denom = Vec3::dot(self.normal, ray.direction);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = (-self.distance - Vec3::dot(self.normal, ray.origin)) / denom;
        Some(Intersection::new(t, self.normal))
    }
}
