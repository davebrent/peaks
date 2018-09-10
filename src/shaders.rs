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
use samplers::{RayStencilSampler, Sampler};
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
    /// Returns information for a ray trace
    fn trace_ray(&self, ray: Ray, x: f64, y: f64) -> Option<TraceInfo>;
}

pub trait Shader {
    /// Return the resulting color for a ray trace
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3;
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct DirectionalLight {
    direction: Vec3,
    color: Vec3,
    intensity: f64,
}

impl DirectionalLight {
    pub fn new(
        direction: Vec3,
        color: Vec3,
        intensity: f64,
    ) -> DirectionalLight {
        DirectionalLight {
            direction,
            color,
            intensity,
        }
    }
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

#[derive(Clone, Default)]
pub struct FeatureLineShader<M>
where
    M: Shader + Clone + Default,
{
    inner: M,
    color: Vec3,
    stencil: RayStencilSampler,
    crease_threshold: f64,
    self_silhoutte_threshold: f64,
}

impl<M> FeatureLineShader<M>
where
    M: Shader + Clone + Default,
{
    pub fn new(
        inner: M,
        color: Vec3,
        quality: usize,
        radius: f64,
        crease_threshold: f64,
        self_silhoutte_threshold: f64,
    ) -> FeatureLineShader<M> {
        FeatureLineShader {
            inner,
            color,
            stencil: RayStencilSampler::new(quality, radius),
            crease_threshold,
            self_silhoutte_threshold,
        }
    }
}

impl<M> Shader for FeatureLineShader<M>
where
    M: Shader + Clone + Default,
{
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3 {
        let i = self.stencil
            .samples()
            .map(|(x, y)| tracer.trace(info.x + x, info.y + y))
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

        let color = self.inner.shade(tracer, info);
        (color * (1.0 - edge_metric)) + (self.color * edge_metric)
    }
}

#[derive(Clone, Default)]
pub struct PhongShader<M>
where
    M: Shader + Clone + Default,
{
    inner: M,
    directional_lights: Vec<DirectionalLight>,
    offset: f64,
    ambient_color: Vec3,
    specular_color: Vec3,
    specular_exponent: f64,
    ks: f64,
    cel_shading: Option<(usize, f64)>,
}

impl<M> PhongShader<M>
where
    M: Shader + Clone + Default,
{
    pub fn new(
        inner: M,
        directional_lights: Vec<DirectionalLight>,
        offset: f64,
        ambient_color: Vec3,
        specular_color: Vec3,
        specular_exponent: f64,
        ks: f64,
        cel_shading: Option<(usize, f64)>,
    ) -> PhongShader<M> {
        PhongShader {
            inner,
            directional_lights,
            offset,
            ambient_color,
            specular_color,
            specular_exponent,
            ks,
            cel_shading,
        }
    }
}

impl<M> Shader for PhongShader<M>
where
    M: Shader + Clone + Default,
{
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3 {
        let point = info.ray.origin + info.ray.direction * info.intersection.t;
        let point = point + info.intersection.normal * self.offset;

        let normal = info.intersection.normal;
        let eye = info.ray.direction;

        let mut diffuse = 0.0;
        let mut specular = 0.0;

        for light in &self.directional_lights {
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

        let color = self.inner.shade(tracer, info);
        self.ambient_color
            + (color * diffuse)
            + (self.specular_color * specular)
    }
}
