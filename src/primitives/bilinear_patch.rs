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

pub struct BilinearPatch {
    p00: Vec3,
    p01: Vec3,
    p10: Vec3,
    p11: Vec3,
}

#[derive(Clone, Copy)]
struct Variables {
    a1: f64,
    a2: f64,
    b1: f64,
    b2: f64,
    c1: f64,
    c2: f64,
    d1: f64,
    d2: f64,
}

impl BilinearPatch {
    /// Construct a bilinear patch with coordinates in (nw, ne, se, sw) order
    pub fn new(p01: Vec3, p11: Vec3, p10: Vec3, p00: Vec3) -> BilinearPatch {
        BilinearPatch { p00, p01, p10, p11 }
    }

    /// Return a corresponding `u` value for `v` picking the best denominator
    fn compute_u(&self, v: f64, vars: Variables) -> f64 {
        let denom1 = v * (vars.a2 - vars.a1) + vars.b2 - vars.b1;
        let denom2 = v * vars.a2 + vars.b2;
        if denom1.abs() > denom2.abs() {
            return (v * (vars.c1 - vars.c2) + vars.d1 - vars.d2) / denom1;
        }
        -(v * vars.c2 + vars.d2) / denom2
    }

    /// Return a value for `t` along the ray for a position on the surface
    fn compute_t(&self, ray: Ray, position: Vec3) -> f64 {
        let ts = (position - ray.origin) / ray.direction;
        ts.x.min(ts.y).min(ts.z)
    }

    /// Return the 3d position for a 2d point on the surface
    fn position(&self, u: f64, v: f64) -> Vec3 {
        let x = (1.0 - u) * (1.0 - v) * self.p00.x
            + (1.0 - u) * v * self.p01.x
            + u * (1.0 - v) * self.p10.x
            + u * v * self.p11.x;
        let y = (1.0 - u) * (1.0 - v) * self.p00.y
            + (1.0 - u) * v * self.p01.y
            + u * (1.0 - v) * self.p10.y
            + u * v * self.p11.y;
        let z = (1.0 - u) * (1.0 - v) * self.p00.z
            + (1.0 - u) * v * self.p01.z
            + u * (1.0 - v) * self.p10.z
            + u * v * self.p11.z;
        Vec3::new(x, y, z)
    }

    /// Find the tangent (du)
    fn tan_u(&self, v: f64) -> Vec3 {
        let x = (1.0 - v) * (self.p10.x - self.p00.x)
            + v * (self.p11.x - self.p01.x);
        let y = (1.0 - v) * (self.p10.y - self.p00.y)
            + v * (self.p11.y - self.p01.y);
        let z = (1.0 - v) * (self.p10.z - self.p00.z)
            + v * (self.p11.z - self.p01.z);
        Vec3::new(x, y, z)
    }

    /// Find the tanget (dv)
    fn tan_v(&self, u: f64) -> Vec3 {
        let x = (1.0 - u) * (self.p01.x - self.p00.x)
            + u * (self.p11.x - self.p10.x);
        let y = (1.0 - u) * (self.p01.y - self.p00.y)
            + u * (self.p11.y - self.p10.y);
        let z = (1.0 - u) * (self.p01.z - self.p00.z)
            + u * (self.p11.z - self.p10.z);
        Vec3::new(x, y, z)
    }

    /// Find the normal of the 2d position on the patch
    fn normal(&self, u: f64, v: f64) -> Vec3 {
        let tan_u = self.tan_u(v);
        let tan_v = self.tan_v(u);
        Vec3::normalize(Vec3::cross(tan_u, tan_v))
    }

    /// Return possible solutions for `v`
    fn solutions(&self, a: f64, b: f64, c: f64) -> Vec<f64> {
        if a == 0.0 && b != 0.0 {
            return vec![-c / b];
        } else if a == 0.0 {
            return vec![];
        }

        let d = b * b - 4.0 * a * c;
        if d == 0.0 {
            return vec![-b / a];
        } else if d < 0.0 {
            return vec![];
        }

        let b_sign = if b < 0.0 { -1.0 } else { 1.0 };
        let q = -0.5 * (b + d.sqrt() * b_sign);
        return vec![c / q, q / a];
    }

    /// Solve the intersection with `v`
    fn solve(&self, ray: Ray, v: f64, vars: Variables) -> Option<Intersection> {
        let u = self.compute_u(v, vars);
        let p = self.position(u, v);
        let t = self.compute_t(ray, p);

        if t > 0.0 && u <= 1.0 && u >= 0.0 {
            let normal = self.normal(u, v);
            return Some(Intersection::new(t, normal));
        }

        None
    }
}

impl Primitive for BilinearPatch {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        let vars = {
            let a = self.p11 - self.p10 - self.p01 + self.p00;
            let b = self.p10 - self.p00;
            let c = self.p01 - self.p00;
            let d = self.p00 - ray.origin;

            Variables {
                a1: a.x * ray.direction.z - a.z * ray.direction.x,
                a2: a.y * ray.direction.z - a.z * ray.direction.y,
                b1: b.x * ray.direction.z - b.z * ray.direction.x,
                b2: b.y * ray.direction.z - b.z * ray.direction.y,
                c1: c.x * ray.direction.z - c.z * ray.direction.x,
                c2: c.y * ray.direction.z - c.z * ray.direction.y,
                d1: d.x * ray.direction.z - d.z * ray.direction.x,
                d2: d.y * ray.direction.z - d.z * ray.direction.y,
            }
        };

        let a = vars.a2 * vars.c1 - vars.a1 * vars.c2;
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let b = vars.a2 * vars.d1 - vars.a1 * vars.d2 +
                vars.b2 * vars.c1 - vars.b1 * vars.c2;
        let c = vars.b2 * vars.d1 - vars.b1 * vars.d2;

        // Find the closest intersection for the possible solutions of `v`
        self.solutions(a, b, c)
            .into_iter()
            .filter(|v| *v >= 0.0 && *v <= 1.0)
            .filter_map(|v| self.solve(ray, v, vars))
            .fold(Intersection::none(), |closest, current| {
                if current.t < closest.t {
                    current
                } else {
                    closest
                }
            })
            .to_option()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bilinear_patch_intersection() {
        // Values taken from the example code provided by the "Ray Bilinear
        // Patch Intersections" (2004) paper.
        let patch = BilinearPatch::new(
            Vec3::new(3.0, 1.0, 3.0),
            Vec3::new(1.0, -2.0, 4.0),
            Vec3::new(1.0, 3.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
        );
        let direction = Vec3::normalize(Vec3::new(0.100499, 0.0, -0.994937));
        let ray = Ray::new(Vec3::new(1.0, 0.3, 10.0), direction);
        let hit = patch.intersects(ray).unwrap();
        assert_eq!(hit.t, 7.583153100172977);
        assert_eq!(
            hit.normal,
            Vec3::new(
                -0.39795424262671825,
                0.7622455889334387,
                0.5105037540771958
            )
        );
    }
}
