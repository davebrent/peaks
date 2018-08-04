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
pub struct Sphere {
    position: Vec3,
    radius: f64,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f64) -> Sphere {
        Sphere { position, radius }
    }
}

impl Intersectable for Sphere {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        let o = ray.origin - self.position;
        let a = Vec3::dot(ray.direction, ray.direction);
        let b = 2.0 * Vec3::dot(ray.direction, o);
        let c = Vec3::dot(o, o) - (self.radius * self.radius);
        let delta = b * b - 4.0 * a * c;

        if delta < 0.0 {
            return None;
        }

        let b_sign = if b > 0.0 { 1.0 } else { -1.0 };
        let t1 = (-b - b_sign * delta.sqrt()) / 2.0 * a;
        let t2 = c / (a * t1);
        let t = t1.min(t2);

        let p = ray.origin + ray.direction * t;
        let normal = Vec3::normalize(p - self.position);
        Some(Intersection::new(t, normal))
    }
}
