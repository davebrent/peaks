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

extern crate bmp;
extern crate gdal;
extern crate png;

mod cameras;
pub mod exec;
pub mod export;
pub mod import;
mod materials;
mod math;
pub mod ops;
mod primitives;
mod render;
mod samplers;
mod scene;
mod textures;

pub use cameras::{OrthographicCamera, PinholeCamera};
pub use materials::{BasicMaterial, Material, NormalMaterial, TextureMaterial};
pub use math::{AffineTransform, Color, Ray, Vec3};
pub use primitives::{
    Aabb, BilinearPatch, HeightMap, Intersection, Plane, Primitive, Sphere,
};
pub use render::Renderer;
pub use samplers::RegularGridSampler;
pub use scene::{Object, Scene};
pub use textures::{Bilinear, Texture};
