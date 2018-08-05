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

use peaks::{
    BasicMaterial, HeightMap, NormalMaterial, Object, PinholeCamera,
    RegularGridSampler, Renderer, Scene, Sphere, Texture, Vec3,
};
use std::io::Result;
use std::path::Path;

pub fn main() -> Result<()> {
    let cwd = Path::new(file!()).parent().unwrap();
    let data_dir = cwd.clone().join("data");
    let output_dir = cwd.clone().join("output");

    let mono_map = peaks::import::BmpImporter::import(&Path::new(&data_dir
        .clone()
        .join("height_map.bmp")));

    let mut height_map = Texture::blank(mono_map.width, mono_map.height);
    peaks::ops::rgb_height_map(&mono_map, &mut height_map, 20.0);

    let height_map = HeightMap::new(
        [
            -(mono_map.width as f64 / 2.0),
            -(mono_map.height as f64 / 2.0),
            1.0,
            1.0,
        ],
        &height_map,
    );

    let width = 960;
    let height = 540;
    let num_samples = 1;

    let camera = PinholeCamera::new(
        width,
        height,
        Vec3::new(10.0, 80.0, 60.0),
        Vec3::zeros(),
        75.0_f64.to_radians(),
        1.0,
        Vec3::new(0.0, 1.0, 0.0),
    );

    let scene = Scene {
        background: Vec3::new(1.0, 1.0, 1.0),
        directional_lights: vec![],
        materials: vec![
            Box::new(NormalMaterial::new()),
            Box::new(BasicMaterial::new(Vec3::new(0.1, 0.8, 0.8))),
            Box::new(BasicMaterial::new(Vec3::new(1.0, 0.0, 0.0))),
            Box::new(BasicMaterial::new(Vec3::new(0.0, 1.0, 0.0))),
        ],
        objects: vec![
            Object::new(
                Box::new(Sphere::new(
                    Vec3::new(
                        -(mono_map.width as f64 / 2.0),
                        0.0,
                        -(mono_map.height as f64 / 2.0),
                    ),
                    2.0,
                )),
                2,
            ), // nw
            Object::new(
                Box::new(Sphere::new(
                    Vec3::new(
                        mono_map.width as f64 / 2.0,
                        0.0,
                        -(mono_map.height as f64 / 2.0),
                    ),
                    2.0,
                )),
                0,
            ), // ne
            Object::new(
                Box::new(Sphere::new(
                    Vec3::new(
                        mono_map.width as f64 / 2.0,
                        0.0,
                        mono_map.height as f64 / 2.0,
                    ),
                    2.0,
                )),
                3,
            ), // se
            Object::new(
                Box::new(Sphere::new(
                    Vec3::new(
                        -(mono_map.width as f64 / 2.0),
                        0.0,
                        mono_map.height as f64 / 2.0,
                    ),
                    2.0,
                )),
                0,
            ), // sw
            Object::new(
                Box::new(Sphere::new(Vec3::new(0.0, 20.0, 0.0), 10.0)),
                1,
            ),
            Object::new(Box::new(height_map), 0),
        ],
    };

    let sampler = RegularGridSampler::new(num_samples);
    let renderer = Renderer::new(scene, camera, sampler);

    let mut render_surface = Texture::blank(width, height);
    let mut output = Texture::blank(width, height);

    assert!(peaks::exec::render(
        width,
        height,
        &renderer,
        &mut render_surface
    ));
    peaks::ops::linear_to_srgb(&render_surface, &mut output);
    peaks::export::PpmExporter::export(
        &output_dir.clone().join("render.ppm"),
        &output,
    )
}
