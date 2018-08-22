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

/// An example of rendering a DEM textured with land sat data using an
/// orthographic camera. The DEM is from EU-DEM v1.1 from [EU Copernicus][1].
/// Land sat data taken from [Sentinel Hub Explorer][2]. Both geotifs were
/// warped with a bilinear filter using the following command before hand:
///
///     $ gdalwarp -r bilinear -t_srs '+proj=utm +zone=30 +datum=WGS84' \
///         <source> <dest>
///
/// In this example the DEM also undergoes some terrain generalisation before
/// rendering.
///
///   [1]: https://land.copernicus.eu/
///   [2]: https://apps.sentinel-hub.com/eo-browser/
extern crate peaks;

use std::io::Result;
use std::path::Path;
use std::sync::Arc;

use peaks::ops::{
    byte_stack_to_rgb, scale, smooth, srgb_to_linear,
    terrain_generalise_weights, terrain_weighted_exaggeration,
};
use peaks::{
    transform_coords, HeightMap, Object, OrthographicCamera,
    RegularGridSampler, Renderer, Scene, Texture, TextureMaterial, Vec3,
};

const LAND_SAT_DATASET: &'static str =
    "/home/webadmin/Shared/maps/data/nevis-2018-06-30.tif";
const DEM_DATASET: &'static str =
    "/home/webadmin/Shared/maps/data/eu_dem_v11_e30n30_utm_30_wgs84.tif";

pub fn main() -> Result<()> {
    let width = 960 * 1;
    let height = 540 * 1;
    let num_samples = 4;
    let vertical_exageration = 1.25;

    let cwd = Path::new(file!()).parent().unwrap();
    let output_dir = cwd.clone().join("output");

    let (_, satelite_transform, rasters) =
        peaks::io::gdal::import(LAND_SAT_DATASET, &[1, 2, 3]).unwrap();

    let sat_width = rasters[0].width;
    let sat_height = rasters[0].height;

    let mut satelite_texture_color = Texture::blank(sat_width, sat_height);
    let mut satelite_texture_linear = Texture::blank(sat_width, sat_height);

    byte_stack_to_rgb(
        &rasters[0],
        &rasters[1],
        &rasters[2],
        &mut satelite_texture_color,
    );
    srgb_to_linear(&satelite_texture_color, &mut satelite_texture_linear);

    let camera_proj4 = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
    let (proj4, transform, rasters) = peaks::io::gdal::import_spatial(
        DEM_DATASET,
        &[1],
        (-5.75958251953125, 57.260479840933094),
        (-4.2867279052734375, 56.702620872371355),
        &camera_proj4,
    ).unwrap();
    let raw_height_data = &rasters[0];

    let (eye_lat, eye_lon) = transform_coords(
        -5.11962890625,
        56.834700617098356,
        &camera_proj4,
        &proj4,
    );

    let (look_lat, look_lon) = transform_coords(
        -5.003414154052734,
        56.79627235994062,
        &camera_proj4,
        &proj4,
    );

    let camera = OrthographicCamera::new(
        width,
        height,
        Vec3::new(eye_lat, 4_000.0, eye_lon * -1.0),
        Vec3::new(look_lat, 500.0, look_lon * -1.0),
        1.0,
        Vec3::new(0.0, 1.0, 0.0),
        3_500.0,
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

    let scene = Scene {
        background: Vec3::new(254.0, 254.0, 200.0) / 255.0,
        objects: vec![Object::new(0, 0)],
        materials: vec![Arc::new(TextureMaterial::new(
            satelite_transform,
            satelite_texture_linear,
        ))],
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
