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
use peaks::{render_threaded, Renderer, Scene, Texture};

use std::io::Result;

pub fn main() -> Result<()> {
    let deff = serde_json::from_str(include_str!("basic.json"))?;
    let scene = Scene::new(deff);

    let multi_samples = 4;
    let num_workers = 4;
    let tile_size = 8;

    let (width, height) = scene.camera.view_plane();
    let renderer = Renderer::new(multi_samples, scene);
    let mut surface = Texture::blank(width, height);
    let mut output = Texture::blank(width, height);
    render_threaded(&mut surface, &renderer, num_workers, tile_size);
    linear_to_srgb(&surface, &mut output);
    export("basic.png", &output)
}
