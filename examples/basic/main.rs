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
    Aabb, BilinearPatch, FeatureLineShader, NormalShader, Object,
    PinholeCamera, Plane, RegularGridSampler, Renderer, Scene, Sphere, Texture,
    Vec3,
};
use std::io::Result;
use std::path::Path;
use std::sync::Arc;

pub fn main() -> Result<()> {
    let cwd = Path::new(file!()).parent().unwrap();
    let output_dir = cwd.clone().join("output");

    let width = 960;
    let height = 540;

    let scene = Scene {
        background: Vec3::new(1.0, 1.0, 1.0),
        shaders: vec![Arc::new(FeatureLineShader::new(
            NormalShader::new(),
            Vec3::zeros(),
            1,
            0.5,
            80_f64.to_radians(),
            10_000.0,
        ))],
        primitives: vec![
            Arc::new(Plane::new(Vec3::new(0.0, 1.0, 0.0), -5.0)),
            Arc::new(Aabb::new(
                Vec3::new(10.0, 0.0, 5.0),
                Vec3::new(16.0, 6.0, 11.0),
            )),
            Arc::new(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 5.0)),
            Arc::new(Sphere::new(Vec3::new(-10.0, 5.0, 5.0), 1.5)),
            Arc::new(BilinearPatch::new(
                Vec3::new(-5.0 - 15.0, 0.0 + 8.0, -5.0), // nw
                Vec3::new(5.0 - 15.0, 0.0 + 8.0, -5.0),  // ne
                Vec3::new(5.0 - 15.0, 5.0 - 8.0, 5.0),   // se
                Vec3::new(-5.0 - 15.0, 0.0 + 8.0, 5.0),  // sw
            )),
        ],
        objects: vec![
            Object::new(0, 0),
            Object::new(1, 0),
            Object::new(2, 0),
            Object::new(3, 0),
            Object::new(4, 0),
        ],
    };

    let camera = PinholeCamera::new(
        width,
        height,
        Vec3::new(-2.0, 10.0, 18.0),
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
    peaks::io::png::export(&output_dir.clone().join("render.png"), &output)
}
