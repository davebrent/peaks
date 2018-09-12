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

extern crate gdal;
extern crate png;

mod cameras;
pub mod exec;
pub mod io;
mod math;
pub mod ops;
mod primitives;
mod render;
mod samplers;
mod scene;
mod shaders;
mod shapes;
mod textures;

pub use cameras::{OrthographicCamera, PinholeCamera};
pub use exec::{render, render_threaded};
pub use math::{transform_coords, AffineTransform, Color, Ray, Vec3};
pub use primitives::{
    Aabb, BilinearPatch, HeightMap, Intersection, Plane, Primitive, Sphere,
};
pub use render::Renderer;
pub use samplers::{RayStencilSampler, RegularGridSampler, Sampler};
pub use scene::{Object, Scene};
pub use shaders::{
    ConstantShader, DirectionalLight, FeatureLineShader, NormalShader,
    PhongShader, SdfShader, Shader, TextureShader,
};
pub use shapes::{LineString, Point, Polygon, Ring, Shape};
pub use textures::{Bilinear, Texture};
