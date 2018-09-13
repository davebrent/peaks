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

use peaks::io::png::export;
use peaks::ops::{
    linear_to_srgb, scale, smooth, terrain_generalise_weights,
    terrain_weighted_exaggeration,
};
use peaks::{
    render_threaded, transform_coords, ConstantShader, DirectionalLight,
    FeatureLineShader, HeightMap, Object, PhongShader, OrthographicCamera,
    RegularGridSampler, Renderer, Scene, SdfShader, Texture, Vec3,
};

use std::io::Result;
use std::path::Path;
use std::sync::Arc;

const DEM_DATASET: &'static str =
    "/home/webadmin/Shared/maps/mljet/dem.tif";
const WATER_POLYGONS_DATASET: &'static str =
    "/home/webadmin/Shared/maps/mljet/water-polygons";

pub fn main() -> Result<()> {
    let width = 960;
    let height = 540;
    let num_samples = 4;
    let vertical_exageration = 1.25;

    let cwd = Path::new(file!()).parent().unwrap();
    let output_dir = cwd.clone().join("output");

    let (proj4, transform, rasters) =
        peaks::io::gdal::import(DEM_DATASET, &[1]).unwrap();
    let raw_height_data = &rasters[0];

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
    let (look_lat, look_lon) = transform_coords(
        17.35668182373047,
        42.77461336018116,
        &camera_proj4,
        &proj4,
    );

    let camera_look_at = Vec3::new(look_lat, 380.0, look_lon * -1.0);
    let axis = Vec3::new(0.0, 0.0, 1.0)
        .rotate_x(Vec3::zeros(), (-25_f64).to_radians())
        .rotate_y(Vec3::zeros(), (-38_f64).to_radians());
    let camera_position = camera_look_at + axis * 7_000.0;

    let camera = OrthographicCamera::new(
        width,
        height,
        camera_position,
        camera_look_at,
        1.0,
        Vec3::new(0.0, 1.0, 0.0),
        2_200.0,
    );

    let mut water_polygons =
        peaks::io::ogr::import(WATER_POLYGONS_DATASET, &[0]).unwrap()[0]
            .to_vec();

    for shape in &mut water_polygons {
        shape.project(transform, &height_map);
    }

    let light = DirectionalLight::new(
        Vec3::normalize(
            Vec3::new(0.0, 0.0, 1.0)
                .rotate_x(Vec3::zeros(), (-45_f64).to_radians())
                .rotate_y(Vec3::zeros(), (225_f64).to_radians()),
        ),
        Vec3::new(1.0, 1.0, 1.0),
        1.0,
    );

    let texture_shader = ConstantShader::new(Vec3::new(0.9, 0.9, 0.9));

    let phong_shader = PhongShader::new(
        texture_shader,
        vec![light],
        1.0e6,
        Vec3::new(0.2, 0.2, 0.2),
        Vec3::new(1.0, 1.0, 1.0),
        1.0,
        0.09,
        Some((4, 0.6)),
    );

    let lines_shader = FeatureLineShader::new(
        phong_shader,
        Vec3::zeros(),
        1,
        0.75,
        25_f64.to_radians(),
        75.0,
    );

    let water_shader = SdfShader::new(
        lines_shader,
        water_polygons,
        0.0,
        Vec3::new(0.1, 0.4, 0.8),
        1.0,
        10.0,
        Vec3::new(0.1, 0.1, 0.1),
        30.0,
    );

    let height_map = HeightMap::new(transform, &height_map);

    let scene = Scene {
        background: Vec3::new(0.1, 0.4, 0.8),
        objects: vec![Object::new(0, 0)],
        shaders: vec![Arc::new(water_shader)],
        primitives: vec![Arc::new(height_map)],
    };

    let sampler = RegularGridSampler::new(num_samples);
    let renderer = Renderer::new(scene, camera, sampler);

    let mut surface = Texture::blank(width, height);
    let mut output = Texture::blank(width, height);

    render_threaded(&mut surface, &renderer, 4, 8);
    linear_to_srgb(&surface, &mut output);
    export(&output_dir.clone().join("render.png"), &output)
}
