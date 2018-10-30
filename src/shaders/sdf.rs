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
use io::ogr;
use math::Vec3;
use options::{Loader, SdfShaderOpts};
use shapes::Shape;

#[derive(Clone, Default)]
pub struct SdfShader {
    wraps: usize,
    shapes: Vec<Shape>,
    tolerance: f64,
    color: Vec3,
    alpha: f64,
    stroke_width: f64,
    stroke_color: Vec3,
    stroke_alpha: f64,
    offset: f64,
}

impl SdfShader {
    pub fn new(
        wraps: usize,
        shapes: Vec<Shape>,
        tolerance: f64,
        color: Vec3,
        alpha: f64,
        stroke_width: f64,
        stroke_color: Vec3,
        stroke_alpha: f64,
        offset: f64,
    ) -> SdfShader {
        SdfShader {
            wraps,
            shapes,
            tolerance,
            color,
            alpha,
            stroke_width,
            stroke_color,
            stroke_alpha,
            offset,
        }
    }
}

impl From<SdfShaderOpts> for SdfShader {
    fn from(options: SdfShaderOpts) -> SdfShader {
        let shapes = match options.data {
            Loader::Shp(opts) => {
                let layers = ogr::import(opts.filepath, &[opts.layer]).unwrap();
                layers[0].clone()
            }
            _ => panic!("Unsupported format"),
        };

        SdfShader::new(
            options.wraps,
            shapes,
            options.tolerance,
            From::from(options.color),
            options.alpha,
            options.stroke_width,
            From::from(options.stroke_color),
            From::from(options.stroke_alpha),
            options.offset,
        )
    }
}

impl Shader for SdfShader {
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3 {
        let point = info.ray.origin + info.ray.direction * info.intersection.t;
        let base = match tracer.shader(self.wraps) {
            Some(shader) => shader.shade(tracer, info),
            None => Vec3::zeros(),
        };

        for shape in &self.shapes {
            if !shape.bbox().offset(self.offset).contains(point) {
                continue;
            }

            let distance = shape.distance(point);
            if distance < self.tolerance {
                let (color, alpha) = if distance > self.tolerance - self.stroke_width {
                    (self.stroke_color, self.stroke_alpha)
                } else {
                    (self.color, self.alpha)
                };
                return color * alpha + base * (1.0 - alpha);
            }
        }

        base
    }
}
