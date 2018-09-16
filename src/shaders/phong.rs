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

use super::shader::{Shader, TraceInfo, Tracer};
use math::{Ray, Vec3};
use options::PhongShaderOpts;

#[derive(Clone, Default)]
pub struct PhongShader {
    wraps: usize,
    directional_lights: Vec<usize>,
    bias: f64,
    ambient_color: Vec3,
    specular_color: Vec3,
    specular_exponent: f64,
    ks: f64,
    cel_shading: Option<(usize, f64)>,
}

impl PhongShader {
    pub fn new(
        wraps: usize,
        directional_lights: Vec<usize>,
        bias: f64,
        ambient_color: Vec3,
        specular_color: Vec3,
        specular_exponent: f64,
        ks: f64,
        cel_shading: Option<(usize, f64)>,
    ) -> PhongShader {
        PhongShader {
            wraps,
            directional_lights,
            bias,
            ambient_color,
            specular_color,
            specular_exponent,
            ks,
            cel_shading,
        }
    }
}

impl From<PhongShaderOpts> for PhongShader {
    fn from(options: PhongShaderOpts) -> PhongShader {
        PhongShader::new(
            options.wraps,
            options.lights,
            options.bias,
            From::from(options.ambient),
            From::from(options.specular_color),
            options.specular_exponent,
            options.ks,
            options.cel_shading,
        )
    }
}

impl Shader for PhongShader {
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3 {
        let point = info.ray.origin + info.ray.direction * info.intersection.t;
        let point = point + info.intersection.normal * self.bias;

        let normal = info.intersection.normal;
        let eye = info.ray.direction;

        let mut diffuse = 0.0;
        let mut specular = 0.0;

        for index in &self.directional_lights {
            let light = tracer.light(*index).unwrap();
            let light_dir = light.direction;
            let secondary = Ray::new(point, light_dir);
            if tracer.trace_ray(secondary, info.x, info.y).is_some() {
                continue;
            }

            let reflection = Vec3::reflect(light_dir, normal);
            specular += Vec3::dot(reflection, eye).powf(self.specular_exponent)
                * self.ks;
            diffuse += Vec3::dot(light_dir, normal);
        }

        diffuse = diffuse.max(0.0).min(1.0);
        specular = specular.max(0.0).min(1.0);

        if let Some((bands, specular_threshold)) = self.cel_shading {
            let interval = 1.0 / bands as f64;
            diffuse = (diffuse / interval).round() * interval;
            specular = if specular > specular_threshold {
                1.0
            } else {
                0.0
            };
        }

        let color = match tracer.shader(self.wraps) {
            Some(shader) => shader.shade(tracer, info),
            None => Vec3::zeros(),
        };

        self.ambient_color
            + (color * diffuse)
            + (self.specular_color * specular)
    }
}
