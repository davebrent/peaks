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

use std::f64::INFINITY;

use cameras::Camera;
use math::{AffineTransform, Ray, Vec3};
use samplers::RegularGridSampler;
use textures::{Bilinear, Texture};

pub trait Intersectable {
    /// Object ray intersection test
    fn intersects(&self, ray: Ray) -> Option<Intersection>;
}

pub trait Material {
    /// Return a color for an intersection
    fn shade(&self, ray: Ray, intersection: Intersection) -> Vec3;
}

/// Ray intersection point
#[derive(Copy, Clone, Debug, Default)]
pub struct Intersection {
    /// Distance along the ray the intersection occurs
    pub t: f64,
    /// Normal vector at the intersection point
    pub normal: Vec3,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct DirectionalLight {
    pub direction: Vec3,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct BasicMaterial {
    color: Vec3,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct NormalMaterial;

#[derive(Clone, Debug, Default)]
pub struct TextureMaterial {
    transform: AffineTransform,
    texture: Texture<Vec3>,
}

pub struct Object {
    geometry: Box<Intersectable>,
    material: usize,
}

pub struct Scene {
    pub background: Vec3,
    pub materials: Vec<Box<Material>>,
    pub objects: Vec<Object>,
    pub directional_lights: Vec<DirectionalLight>,
}

pub struct Renderer<C> {
    camera: C,
    scene: Scene,
    sampler: RegularGridSampler,
}

impl Intersection {
    pub fn new(t: f64, normal: Vec3) -> Intersection {
        Intersection { t, normal }
    }

    pub fn none() -> Intersection {
        Intersection {
            t: INFINITY,
            normal: Vec3::zeros(),
        }
    }

    pub fn is_none(&self) -> bool {
        self.t == INFINITY
    }

    pub fn to_option(&self) -> Option<Intersection> {
        if self.is_none() {
            None
        } else {
            Some(*self)
        }
    }
}

impl Object {
    pub fn new(geometry: Box<Intersectable>, material: usize) -> Object {
        Object { geometry, material }
    }
}

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
                if let Some(other) = obj.geometry.intersects(ray) {
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
                let material = &self.scene.materials[object.material];
                material.shade(ray, intersection)
            };

            color += sub_color * weight;
        }

        color
    }
}
