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

use peaks::ops::{
    scale, smooth, terrain_generalise_weights, terrain_weighted_exaggeration,
};
use peaks::{
    transform_coords, HeightMap, NormalMaterial, Object, PinholeCamera,
    RegularGridSampler, Renderer, Scene, Texture, Vec3,
};
use std::io::Result;
use std::path::Path;
use std::sync::Arc;

pub fn main() -> Result<()> {
    let width = 960;
    let height = 540;
    let num_samples = 4;
    let vertical_exageration = 1.25;

    let cwd = Path::new(file!()).parent().unwrap();
    let data_dir = cwd.clone().join("data");
    let output_dir = cwd.clone().join("output");

    let (proj4, transform, raw_height_data) =
        peaks::import::GdalRasterImporter::import(
            &data_dir.clone().join("heightmap.tif"),
            1,
        ).unwrap();

    let mut height_map =
        Texture::blank(raw_height_data.width, raw_height_data.height);
    let mut weights =
        Texture::blank(raw_height_data.width, raw_height_data.height);
    let mut smoothed =
        Texture::blank(raw_height_data.width, raw_height_data.height);
    let mut exagerated =
        Texture::blank(raw_height_data.width, raw_height_data.height);

    let pixel_size = transform.unit_size().0;

    let pre_smoothing = 12;
    let post_smoothing = 1;
    let slope_amp = 8.0;
    let curve_amp = 1.0;
    let positive_weights_amp = 2.0;
    let negative_weights_amp = 4.0;
    let exagerate = 0.14;

    smooth(&raw_height_data, &mut smoothed, pre_smoothing);
    terrain_generalise_weights(
        &smoothed,
        &mut weights,
        pixel_size,
        curve_amp,
        slope_amp,
        positive_weights_amp,
        negative_weights_amp,
        post_smoothing,
    );
    terrain_weighted_exaggeration(
        &smoothed,
        &mut exagerated,
        &weights,
        exagerate,
    );
    scale(&exagerated, &mut height_map, vertical_exageration);

    let camera_proj4 = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";

    let (eye_lat, eye_lon) = transform_coords(
        17.340803146362305,
        42.77064408352934,
        &camera_proj4,
        &proj4,
    );

    let (look_lat, look_lon) = transform_coords(
        17.351274490356445,
        42.78267713877303,
        &camera_proj4,
        &proj4,
    );

    let camera = PinholeCamera::new(
        width,
        height,
        Vec3::new(eye_lat, 2_000.0, eye_lon * -1.0),
        Vec3::new(look_lat, 0.0, look_lon * -1.0),
        90.0_f64.to_radians(),
        1.0,
        Vec3::new(0.0, 1.0, 0.0),
    );

    let height_map = HeightMap::new(transform, &height_map);

    let scene = Scene {
        background: Vec3::new(254.0, 254.0, 200.0) / 255.0,
        objects: vec![Object::new(0, 0)],
        materials: vec![Arc::new(NormalMaterial::new())],
        primitives: vec![Arc::new(height_map)],
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
    peaks::export::PngExporter::export(
        &output_dir.clone().join("render.png"),
        &output,
    )
}
