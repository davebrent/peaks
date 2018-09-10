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

use cameras::Camera;
use math::{Ray, Vec3};
use primitives::Intersection;
use samplers::Sampler;
use scene::Scene;
use shaders::{TraceInfo, Tracer};

pub struct Renderer<C, S> {
    camera: C,
    scene: Scene,
    sampler: S,
}

impl<C, S> Renderer<C, S>
where
    C: Camera,
    S: Sampler,
{
    pub fn new(scene: Scene, camera: C, sampler: S) -> Renderer<C, S> {
        Renderer {
            sampler,
            scene,
            camera,
        }
    }

    /// Return a color for a pixel
    pub fn pixel(&self, x: usize, y: usize) -> Vec3 {
        let mut color = Vec3::zeros();
        let weight = 1.0 / self.sampler.amount() as f64;

        for (sub_x, sub_y) in self.sampler.samples() {
            let px = x as f64 + sub_x;
            let py = y as f64 + sub_y;

            let sub_color = if let Some(info) = self.trace(px, py) {
                let object = &self.scene.objects[info.primitive];
                let shader = &self.scene.shaders[object.shader];
                shader.shade(self, &info)
            } else {
                self.scene.background
            };

            color += sub_color * weight;
        }

        color
    }
}

impl<C, S> Tracer for Renderer<C, S>
where
    C: Camera,
    S: Sampler,
{
    fn trace_ray(&self, ray: Ray, x: f64, y: f64) -> Option<TraceInfo> {
        let mut index = 0;
        let mut intersection = Intersection::none();

        for (i, obj) in self.scene.objects.iter().enumerate() {
            let primitive = &self.scene.primitives[obj.geometry];
            if let Some(other) = primitive.intersects(ray) {
                if other.t < intersection.t && other.t > 0.0 {
                    intersection = other;
                    index = i;
                }
            }
        }

        if intersection.is_none() {
            None
        } else {
            Some(TraceInfo {
                ray,
                intersection,
                primitive: index,
                x,
                y,
            })
        }
    }

    fn trace(&self, x: f64, y: f64) -> Option<TraceInfo> {
        let ray = self.camera.cast_ray(x, y);
        self.trace_ray(ray, x, y)
    }
}
