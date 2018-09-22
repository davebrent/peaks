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
mod exec;
mod io;
mod lights;
mod math;
mod ops;
mod options;
mod primitives;
mod render;
mod samplers;
mod scene;
mod shaders;
mod shapes;
mod textures;

pub use exec::{render, render_threaded};
pub use io::png::export;
pub use math::{Color, Ray, Vec3};
pub use ops::{linear_to_srgb, srgb_to_linear};
pub use options::*;
pub use render::Renderer;
pub use scene::Scene;
pub use textures::Texture;
