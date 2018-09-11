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

use std::io::Result;
use std::path::Path;
use std::sync::Arc;

use peaks::ops::{
    scale, smooth, terrain_generalise_weights, terrain_weighted_exaggeration,
};
use peaks::{
    transform_coords, ConstantShader, DirectionalLight, FeatureLineShader,
    HeightMap, Object, OrthographicCamera, PhongShader, RegularGridSampler,
    Renderer, Scene, SdfShader, Texture, Vec3,
};

const DEM_DATASET: &'static str =
    "/home/webadmin/Shared/maps/ben_nevis/dem.tif";
const WATER_POLYGONS_DATASET: &'static str =
    "/home/webadmin/Shared/maps/ben_nevis/water-polygons";
const ROUTE_DATASET: &'static str =
    "/home/webadmin/Shared/maps/ben_nevis/route";
const RIVERS_DATASET: &'static str =
    "/home/webadmin/Shared/maps/ben_nevis/rivers.geojson";

pub fn main() -> Result<()> {
    let width = 960 * 1;
    let height = 540 * 1;
    let num_samples = 4;
    let vertical_exageration = 1.25;

    let cwd = Path::new(file!()).parent().unwrap();
    let output_dir = cwd.clone().join("output");

    let camera_proj4 = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
    let (proj4, transform, rasters) =
        peaks::io::gdal::import(DEM_DATASET, &[1]).unwrap();
    let raw_height_data = &rasters[0];

    let (look_lat, look_lon) = transform_coords(
        -5.002555847167968,
        56.7966483772629,
        &camera_proj4,
        &proj4,
    );

    let camera_look_at = Vec3::new(look_lat, 380.0, look_lon * -1.0);
    let axis = Vec3::new(0.0, 0.0, 1.0)
        .rotate_x(Vec3::zeros(), (-25_f64).to_radians())
        .rotate_y(Vec3::zeros(), (-128_f64).to_radians());
    let camera_position = camera_look_at + axis * 7_000.0;

    let camera = OrthographicCamera::new(
        width,
        height,
        camera_position,
        camera_look_at,
        1.0,
        Vec3::new(0.0, 1.0, 0.0),
        3_700.0,
    );

    let ter_width = raw_height_data.width;
    let ter_height = raw_height_data.height;
    let mut height_map = Texture::blank(ter_width, ter_height);
    let mut weights = Texture::blank(ter_width, ter_height);
    let mut smoothed = Texture::blank(ter_width, ter_height);
    let mut exagerated = Texture::blank(ter_width, ter_height);

    let pixel_size = transform.unit_size().0;
    let pre_smoothing = 46;
    let post_smoothing = 5;
    let slope_amp = 9.5;
    let curve_amp = 1.0;
    let positive_weights_amp = 1.0;
    let negative_weights_amp = 3.5;
    let exagerate = 0.25;

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

    let mut route_lines =
        peaks::io::ogr::import(ROUTE_DATASET, &[0]).unwrap()[0].to_vec();
    let mut river_lines =
        peaks::io::ogr::import(RIVERS_DATASET, &[0]).unwrap()[0].to_vec();
    let mut water_polygons =
        peaks::io::ogr::import(WATER_POLYGONS_DATASET, &[0]).unwrap()[0]
            .to_vec();

    for shape in &mut route_lines {
        shape.project(transform, &height_map);
    }
    for shape in &mut river_lines {
        shape.project(transform, &height_map);
    }
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
        2,
        0.75,
        35_f64.to_radians(),
        400.0,
    );

    let water_shader = SdfShader::new(
        lines_shader,
        water_polygons,
        0.0,
        Vec3::new(0.1, 0.4, 0.8),
        1.0,
        0.0,
        Vec3::zeros(),
        3.0,
    );

    let river_shader = SdfShader::new(
        water_shader,
        river_lines,
        12.0,
        Vec3::new(0.1, 0.4, 0.8),
        1.0,
        0.0,
        Vec3::new(0.0, 0.0, 0.0),
        3.0,
    );

    let route_shader = SdfShader::new(
        river_shader,
        route_lines,
        30.0,
        Vec3::new(1.0, 0.0, 0.0),
        1.0,
        0.0,
        Vec3::zeros(),
        100.0,
    );

    let scene = Scene {
        background: Vec3::new(254.0, 254.0, 200.0) / 255.0,
        objects: vec![Object::new(0, 0)],
        shaders: vec![Arc::new(route_shader)],
        primitives: vec![Arc::new(HeightMap::new(transform, &height_map))],
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
    peaks::io::png::export(&output_dir.clone().join("render.png"), &output)
}
