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

use lights::DirectionalLight;
use math::{Ray, Vec3};
use primitives::Intersection;
use samplers::{RegularGridSampler, Sampler};
use scene::Scene;
use shaders::{Shader, TraceInfo, Tracer};

#[derive(Clone)]
pub struct Renderer {
    scene: Scene,
    sampler: RegularGridSampler,
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new(multi_samples: usize, scene: Scene) -> Renderer {
        Renderer {
            sampler: RegularGridSampler::new(multi_samples),
            scene,
        }
    }

    /// Return a color for a pixel
    pub fn pixel(&self, x: usize, y: usize) -> Vec3 {
        let mut color = Vec3::zeros();
        let weight = 1.0 / self.sampler.amount() as f64;

        for (sub_x, sub_y) in self.sampler.samples() {
            let px = x as f64 + sub_x;
            let py = y as f64 + sub_y;

            let sub_color = if let Some(info) = self.trace_pixel(px, py) {
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

impl Tracer for Renderer {
    fn trace_ray(&self, ray: Ray, x: f64, y: f64) -> Option<TraceInfo> {
        let mut index = 0;
        let mut intersection = Intersection::none();

        for (i, obj) in self.scene.objects.iter().enumerate() {
            let primitive = &self.scene.primitives[obj.primitive];
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

    fn trace_pixel(&self, x: f64, y: f64) -> Option<TraceInfo> {
        let ray = self.scene.camera.cast_ray(x, y);
        self.trace_ray(ray, x, y)
    }

    fn shader(&self, index: usize) -> Option<&Shader> {
        self.scene.shaders.get(index).map(|shader| &**shader)
    }

    fn light(&self, index: usize) -> Option<&DirectionalLight> {
        self.scene.lights.get(index).map(|light| &**light)
    }
}
