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

pub trait Material {
    /// Return a color for an intersection
    fn shade(&self, ray: Ray, intersection: Intersection) -> Vec3;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct NormalMaterial;

impl NormalMaterial {
    pub fn new() -> NormalMaterial {
        NormalMaterial {}
    }
}

impl Material for NormalMaterial {
    fn shade(&self, _: Ray, intersection: Intersection) -> Vec3 {
        (intersection.normal + 1.0) * 0.5
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct BasicMaterial {
    color: Vec3,
}

impl BasicMaterial {
    pub fn new(color: Vec3) -> BasicMaterial {
        BasicMaterial { color }
    }
}

impl Material for BasicMaterial {
    fn shade(&self, _: Ray, _: Intersection) -> Vec3 {
        self.color
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextureMaterial {
    transform: AffineTransform,
    texture: Texture<Vec3>,
}

impl TextureMaterial {
    pub fn new(
        transform: AffineTransform,
        texture: Texture<Vec3>,
    ) -> TextureMaterial {
        TextureMaterial { transform, texture }
    }
}

impl Material for TextureMaterial {
    fn shade(&self, ray: Ray, intersection: Intersection) -> Vec3 {
        let point = ray.origin + ray.direction * intersection.t;
        let (u, v) = self.transform.inverse(point.x, point.z);
        self.texture.bilinear(u, v)
    }
}

#[derive(Clone, Default)]
pub struct SdfMaterial<M>
where
    M: Material + Clone + Default,
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

impl<M> SdfMaterial<M>
where
    M: Material + Clone + Default,
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
    ) -> SdfMaterial<M> {
        SdfMaterial {
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

impl<M> Material for SdfMaterial<M>
where
    M: Material + Clone + Default,
{
    fn shade(&self, ray: Ray, intersection: Intersection) -> Vec3 {
        let point = ray.origin + ray.direction * intersection.t;
        let base = self.inner.shade(ray, intersection);

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
