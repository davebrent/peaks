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
