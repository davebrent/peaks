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

extern crate peaks;
extern crate serde_json;

use peaks::io::png::export;
use peaks::ops::linear_to_srgb;
use peaks::{render_threaded, RegularGridSampler, Renderer, Scene, Texture};

use std::io::Result;

pub fn main() -> Result<()> {
    let options = serde_json::from_str(include_str!("ben_nevis.json"))?;
    let scene = Scene::new(options);
    let (width, height) = scene.camera.view_plane();

    let sampler = RegularGridSampler::new(4);
    let renderer = Renderer::new(scene, sampler);

    let mut surface = Texture::blank(width, height);
    let mut output = Texture::blank(width, height);
    render_threaded(&mut surface, &renderer, 4, 8);
    linear_to_srgb(&surface, &mut output);
    export("ben_nevis.png", &output)
}
