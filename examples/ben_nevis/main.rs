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
    DirectionalLight, HeightMap, NormalMaterial, Object, PinholeCamera,
    RegularGridSampler, Renderer, Scene, Texture, Vec3,
};
use std::io::Result;
use std::path::Path;

pub fn main() -> Result<()> {
    let width = 1260 * 1;
    let height = 540 * 1;
    let num_samples = 4;
    let vertical_exageration = 1.0;

    let cwd = Path::new(file!()).parent().unwrap();
    let output_dir = cwd.clone().join("output");

    let camera_proj4 = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
    // Original DEM take from EU DEM v1.1 was reprojected using this command:
    // $ gdalwarp -r bilinear -t_srs '+proj=utm +zone=30 +datum=WGS84' \
    //  <source> <dest>
    let f =
        "/home/webadmin/Shared/maps/data/eu_dem_v11_e30n30_utm_30_wgs84.tif";
    let (proj4, transform, raw_height_data) =
        peaks::import::GdalRasterImporter::import_bbox(
            &Path::new(f),
            1,
            (-5.75958251953125, 57.260479840933094),
            (-4.2867279052734375, 56.702620872371355),
            &camera_proj4,
        ).unwrap();

    let mut camera_position = peaks::import::GdalRasterImporter::convert(
        Vec3::new(
            -5.000324249267579,
            3_000.0 * vertical_exageration,
            56.77201127687461,
        ),
        &camera_proj4,
        &proj4,
    );
    let mut camera_look_at = peaks::import::GdalRasterImporter::convert(
        Vec3::new(
            -4.993972778320313,
            1_300.0 * vertical_exageration,
            56.81281355548693,
        ),
        &camera_proj4,
        &proj4,
    );

    camera_position.z *= -1.0;
    camera_look_at.z *= -1.0;

    let camera = PinholeCamera::new(
        width,
        height,
        camera_position,
        camera_look_at,
        40.0_f64.to_radians(),
        1.0,
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut height_map =
        Texture::blank(raw_height_data.width, raw_height_data.height);
    peaks::ops::scale(&raw_height_data, &mut height_map, vertical_exageration);
    let height_map = HeightMap::new(transform, &height_map);

    let scene = Scene {
        background: Vec3::new(254.0, 254.0, 200.0) / 255.0,
        objects: vec![Object::new(Box::new(height_map), 0)],
        materials: vec![Box::new(NormalMaterial::new())],
        directional_lights: vec![DirectionalLight {
            direction: Vec3::normalize(Vec3::new(-10.0, 10.0, -10.0)),
        }],
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
