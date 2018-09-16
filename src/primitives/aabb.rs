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

use super::primitive::{Intersection, Primitive};
use math::{Ray, Vec3};
use options::AabbOpts;

use std::f64::INFINITY;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Aabb {
        Aabb { min, max }
    }

    fn center(&self) -> Vec3 {
        Vec3::new(
            self.min.x + (self.max.x - self.min.x) / 2.0,
            self.min.y + (self.max.y - self.min.y) / 2.0,
            self.min.z + (self.max.z - self.min.z) / 2.0,
        )
    }
}

impl From<AabbOpts> for Aabb {
    fn from(options: AabbOpts) -> Aabb {
        Aabb::new(From::from(options.min), From::from(options.max))
    }
}

impl Primitive for Aabb {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        let bounds = [self.min, self.max];

        let inverse_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );

        let sign_x = if inverse_dir.x < 0.0 { 1 } else { 0 };
        let sign_y = if inverse_dir.y < 0.0 { 1 } else { 0 };
        let sign_z = if inverse_dir.z < 0.0 { 1 } else { 0 };

        let txmin = (bounds[sign_x].x - ray.origin.x) * inverse_dir.x;
        let txmax = (bounds[1 - sign_x].x - ray.origin.x) * inverse_dir.x;

        let tymin = (bounds[sign_y].y - ray.origin.y) * inverse_dir.y;
        let tymax = (bounds[1 - sign_y].y - ray.origin.y) * inverse_dir.y;

        let tzmin = (bounds[sign_z].z - ray.origin.z) * inverse_dir.z;
        let tzmax = (bounds[1 - sign_z].z - ray.origin.z) * inverse_dir.z;

        let tmin = txmin.max(-INFINITY).max(tymin).max(tzmin);
        let tmax = txmax.min(INFINITY).min(tymax).min(tzmax);
        if tmin > tmax {
            return None;
        }

        let t = if tmin < 0.0 { tmax } else { tmin };
        let bias = 1.000_001;

        let p = (ray.origin + ray.direction * t) - self.center();
        let d = (self.min - self.max).abs() * 0.5;
        let n = Vec3::normalize((p / d * bias).integral());

        Some(Intersection::new(t, n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_intersection_inside() {
        let aabb =
            Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(5.0, 5.0, 5.0));
        let ray = Ray::new(Vec3::new(2.5, 2.5, 2.5), Vec3::new(0.0, 1.0, 0.0));
        let hit = aabb.intersects(ray).unwrap();
        assert_eq!(hit.t, 2.5);
        assert_eq!(
            ray.origin + ray.direction * hit.t,
            Vec3::new(2.5, 5.0, 2.5)
        );
    }

    #[test]
    fn aabb_center() {
        let aabb =
            Aabb::new(Vec3::new(2.5, 2.5, 2.5), Vec3::new(7.5, 7.5, 7.5));
        assert_eq!(aabb.center(), Vec3::new(5.0, 5.0, 5.0));
    }
}
