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
use math::Vec3;
use options::FeatureLineShaderOpts;
use samplers::{RayStencilSampler, Sampler};

#[derive(Clone, Default)]
pub struct FeatureLineShader {
    wraps: usize,
    color: Vec3,
    stencil: RayStencilSampler,
    crease_threshold: f64,
    self_silhoutte_threshold: f64,
}

impl FeatureLineShader {
    pub fn new(
        wraps: usize,
        color: Vec3,
        quality: usize,
        radius: f64,
        crease_threshold: f64,
        self_silhoutte_threshold: f64,
    ) -> FeatureLineShader {
        FeatureLineShader {
            wraps,
            color,
            stencil: RayStencilSampler::new(quality, radius),
            crease_threshold,
            self_silhoutte_threshold,
        }
    }
}

impl From<FeatureLineShaderOpts> for FeatureLineShader {
    fn from(options: FeatureLineShaderOpts) -> FeatureLineShader {
        FeatureLineShader::new(
            options.wraps,
            From::from(options.color),
            options.quality,
            options.radius,
            options.crease_threshold,
            options.self_silhoutte_threshold,
        )
    }
}

impl Shader for FeatureLineShader {
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3 {
        let i = self.stencil
            .samples()
            .map(|(x, y)| tracer.trace_pixel(info.x + x, info.y + y))
            .filter_map(|stencil| stencil)
            .filter(|stencil| stencil.primitive == info.primitive)
            .filter(|stencil| {
                Vec3::angle(
                    stencil.intersection.normal,
                    info.intersection.normal,
                ) < self.crease_threshold
            })
            .filter(|stencil| {
                (stencil.intersection.t - info.intersection.t).abs()
                    < self.self_silhoutte_threshold
            })
            .count() as f64;

        // Normalise, accounting for the region on the otherside of the edge
        let num_stencils = self.stencil.amount() as f64;
        let edge_metric = 1.0 - (i - num_stencils * 0.5) / (num_stencils * 0.5);

        let color = match tracer.shader(self.wraps) {
            Some(shader) => shader.shade(tracer, info),
            None => Vec3::zeros(),
        };
        (color * (1.0 - edge_metric)) + (self.color * edge_metric)
    }
}
