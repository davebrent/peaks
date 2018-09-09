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
use math::Vec3;
use primitives::Intersection;
use samplers::RegularGridSampler;
use scene::Scene;

pub struct Renderer<C> {
    camera: C,
    scene: Scene,
    sampler: RegularGridSampler,
}

impl<C> Renderer<C>
where
    C: Camera,
{
    pub fn new(
        scene: Scene,
        camera: C,
        sampler: RegularGridSampler,
    ) -> Renderer<C> {
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
            let mut index = 0;
            let mut intersection = Intersection::none();

            let ray = self.camera.cast_ray(x as f64 + sub_x, y as f64 + sub_y);
            for (i, obj) in self.scene.objects.iter().enumerate() {
                let primitive = &self.scene.primitives[obj.geometry];
                if let Some(other) = primitive.intersects(ray) {
                    if other.t < intersection.t && other.t > 0.0 {
                        intersection = other;
                        index = i;
                    }
                }
            }

            let sub_color = if intersection.is_none() {
                self.scene.background
            } else {
                let object = &self.scene.objects[index];
                let shader = &self.scene.shaders[object.shader];
                shader.shade(ray, intersection)
            };

            color += sub_color * weight;
        }

        color
    }
}
