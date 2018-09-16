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
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod cameras;
pub mod exec;
pub mod io;
mod lights;
mod math;
pub mod ops;
mod options;
mod primitives;
mod render;
mod samplers;
mod scene;
mod shaders;
mod shapes;
mod textures;

pub use cameras::{Camera, OrthographicCamera, PinholeCamera};
pub use exec::{render, render_threaded};
pub use lights::DirectionalLight;
pub use math::{transform_coords, AffineTransform, Color, Ray, Vec3};
pub use options::SceneOpts;
pub use primitives::{
    Aabb, BilinearPatch, HeightMap, Intersection, Plane, Primitive, Sphere,
};
pub use render::Renderer;
pub use samplers::{RayStencilSampler, RegularGridSampler, Sampler};
pub use scene::{Object, Scene};
pub use shaders::{
    ConstantShader, FeatureLineShader, NormalShader, PhongShader, SdfShader,
    Shader, TextureShader,
};
pub use shapes::{LineString, Point, Polygon, Ring, Shape};
pub use textures::{Bilinear, Texture};
