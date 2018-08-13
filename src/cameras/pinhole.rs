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

use super::camera::Camera;
use math::{Ray, Vec3};

#[derive(Copy, Clone, Debug)]
pub struct PinholeCamera {
    position: Vec3,
    look_at: Vec3,
    up_axis: Vec3,
    fov: f64,
    view_distance: f64,
    width: usize,
    height: usize,
    aspect: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl PinholeCamera {
    pub fn new(
        width: usize,
        height: usize,
        position: Vec3,
        look_at: Vec3,
        fov: f64,
        view_distance: f64,
        up_axis: Vec3,
    ) -> PinholeCamera {
        // Calculate the correct up vector for the up axis
        let direction = Vec3::normalize(look_at - position);
        let right = Vec3::cross(direction, up_axis);
        let up = Vec3::cross(right, direction);

        // Create the cameras coordinate system (ONB)
        let w = Vec3::normalize(position - look_at);
        let u = Vec3::cross(up, w);
        let v = Vec3::cross(w, u);

        // Account for non-square aspect ratios
        let mut aspect = Vec3::new(1.0, 1.0, 1.0);
        if width > height {
            aspect.x = width as f64 / height as f64;
            aspect.y = 1.0;
        } else if height > width {
            aspect.x = 1.0;
            aspect.y = height as f64 / width as f64;
        }

        PinholeCamera {
            width,
            height,
            position,
            look_at,
            up_axis,
            fov,
            view_distance,
            aspect,
            u,
            v,
            w,
        }
    }
}

impl Camera for PinholeCamera {
    fn cast_ray(&self, x: f64, y: f64) -> Ray {
        // Raster to NDC space
        let mut px = x / self.width as f64 * 2.0 - 1.0;
        let mut py = 1.0 - y / self.height as f64 * 2.0;

        // Account for aspect ratios and fov
        px = px * self.aspect.x * self.fov;
        py = py * self.aspect.y * self.fov;

        let dir = self.u * px + self.v * py - self.w * self.view_distance;
        Ray::new(self.position, Vec3::normalize(dir))
    }
}
