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

use math::{AffineTransform, Ray, Vec3};
use primitives::Intersection;
use shapes::Shape;
use textures::{Bilinear, Texture};

pub struct TraceInfo {
    /// The ray used to populate this object
    pub ray: Ray,
    /// Intersection with a primitive shape
    pub intersection: Intersection,
    /// Intersecting primitives id
    pub primitive: usize,
    /// X coordinate on the view plane
    pub x: f64,
    /// Y coordinate on the view plane
    pub y: f64,
}

pub trait Tracer {
    /// Returns information for tracing a ray specified in screen space
    fn trace(&self, x: f64, y: f64) -> Option<TraceInfo>;
}

pub trait Shader {
    /// Return the resulting color for a ray trace
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct NormalShader;

impl NormalShader {
    pub fn new() -> NormalShader {
        NormalShader {}
    }
}

impl Shader for NormalShader {
    fn shade(&self, _: &Tracer, info: &TraceInfo) -> Vec3 {
        (info.intersection.normal + 1.0) * 0.5
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ConstantShader {
    color: Vec3,
}

impl ConstantShader {
    pub fn new(color: Vec3) -> ConstantShader {
        ConstantShader { color }
    }
}

impl Shader for ConstantShader {
    fn shade(&self, _: &Tracer, _: &TraceInfo) -> Vec3 {
        self.color
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextureShader {
    transform: AffineTransform,
    texture: Texture<Vec3>,
}

impl TextureShader {
    pub fn new(
        transform: AffineTransform,
        texture: Texture<Vec3>,
    ) -> TextureShader {
        TextureShader { transform, texture }
    }
}

impl Shader for TextureShader {
    fn shade(&self, _: &Tracer, info: &TraceInfo) -> Vec3 {
        let point = info.ray.origin + info.ray.direction * info.intersection.t;
        let (u, v) = self.transform.inverse(point.x, point.z);
        self.texture.bilinear(u, v)
    }
}

#[derive(Clone, Default)]
pub struct SdfShader<M>
where
    M: Shader + Clone + Default,
{
    inner: M,
    shapes: Vec<Shape>,
    tolerance: f64,
    color: Vec3,
    alpha: f64,
    stroke_width: f64,
    stroke_color: Vec3,
    offset: f64,
}

impl<M> SdfShader<M>
where
    M: Shader + Clone + Default,
{
    pub fn new(
        inner: M,
        shapes: Vec<Shape>,
        tolerance: f64,
        color: Vec3,
        alpha: f64,
        stroke_width: f64,
        stroke_color: Vec3,
        offset: f64,
    ) -> SdfShader<M> {
        SdfShader {
            inner,
            shapes,
            tolerance,
            color,
            alpha,
            stroke_width,
            stroke_color,
            offset,
        }
    }
}

impl<M> Shader for SdfShader<M>
where
    M: Shader + Clone + Default,
{
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3 {
        let point = info.ray.origin + info.ray.direction * info.intersection.t;
        let base = self.inner.shade(tracer, info);

        for shape in &self.shapes {
            if !shape.bbox().offset(self.offset).contains(point) {
                continue;
            }

            let distance = shape.distance(point);
            if distance < self.tolerance {
                let color = if distance > self.tolerance - self.stroke_width {
                    self.stroke_color
                } else {
                    self.color
                };
                return color * self.alpha + base * (1.0 - self.alpha);
            }
        }

        base
    }
}
