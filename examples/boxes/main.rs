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
    Aabb, NormalMaterial, Object, PinholeCamera, RegularGridSampler, Renderer,
    Scene, Texture, Vec3,
};
use std::io::Result;
use std::path::Path;

pub fn main() -> Result<()> {
    let cwd = Path::new(file!()).parent().unwrap();
    let output_dir = cwd.clone().join("output");

    let width = 960;
    let height = 540;

    let scene = Scene {
        background: Vec3::new(1.0, 1.0, 1.0),
        directional_lights: vec![],
        materials: vec![Box::new(NormalMaterial::new())],
        objects: vec![
            Object::new(
                Box::new(Aabb::new(
                    Vec3::new(-100.0, -100.0, -100.0),
                    Vec3::new(100.0, 100.0, 100.0),
                )),
                0,
            ),
            Object::new(
                Box::new(Aabb::new(
                    Vec3::new(-5.0, -5.0, -5.0),
                    Vec3::new(5.0, 5.0, 5.0),
                )),
                0,
            ),
            Object::new(
                Box::new(Aabb::new(
                    Vec3::new(2.5, 2.5, 2.5),
                    Vec3::new(7.5, 7.5, 7.5),
                )),
                0,
            ),
        ],
    };

    let camera = PinholeCamera::new(
        width,
        height,
        Vec3::new(-10.0, 10.0, 10.0),
        Vec3::new(0.0, 0.0, 0.0),
        75.0_f64.to_radians(),
        1.0,
        Vec3::new(0.0, 1.0, 0.0),
    );

    let sampler = RegularGridSampler::new(16);
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
