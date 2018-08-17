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

/*
gdalwarp \
    -overwrite \
    -r cubic \
    -t_srs "+proj=utm +zone=30 +datum=WGS84" \
    -te_srs "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs" \
    -te -5.146751403808594 56.72353621546779 -4.905052185058594 56.850098435118284 \
    ../maps/data/eu_dem_v11_e30n30_utm_30_wgs84.tif \
    examples/terrain_processing/data/height_map.tif
*/

extern crate peaks;

use peaks::ops::{
    blit, curvature, hillshade, linear_to_srgb, operator1x1, scale, shift,
    slope, smooth, terrain_generalise_weights, terrain_weighted_exaggeration,
};
use peaks::{Texture, Vec3};
use std::io::Result;
use std::path::Path;

pub fn main() -> Result<()> {
    let cwd = Path::new(file!()).parent().unwrap();
    let data_dir = cwd.clone().join("data");
    let output_dir = cwd.clone().join("output");

    let (_, transform, height_map) = peaks::import::GdalRasterImporter::import(
        &data_dir.clone().join("height_map.tif"),
        1,
    ).unwrap();

    let width = height_map.width;
    let height = height_map.height;
    let pixel_size = transform.unit_size().0;
    let mut composited = Texture::blank(width * 3, height * 2);

    let pre_smoothing = 48;
    let post_smoothing = 10;
    let slope_amp = 10.0;
    let curve_amp = 1.0;
    let positive_weights_amp = 5.0;
    let negative_weights_amp = 7.0;
    let exagerate = 0.09;

    {
        let mut hillshaded = Texture::blank(width, height);
        let mut colorised = Texture::blank(width, height);
        let mut output = Texture::blank(width, height);

        hillshade(&height_map, &mut hillshaded, 315.0, 45.0, pixel_size);
        operator1x1(&hillshaded, &mut colorised, |v| Vec3::new(v, v, v));
        linear_to_srgb(&colorised, &mut output);
        blit(&output, &mut composited, 0, 0);
    }

    {
        let mut processed = Texture::blank(width, height);
        let mut hillshaded = Texture::blank(width, height);
        let mut colorised = Texture::blank(width, height);
        let mut output = Texture::blank(width, height);

        smooth(&height_map, &mut processed, pre_smoothing);
        hillshade(&processed, &mut hillshaded, 315.0, 45.0, pixel_size);
        operator1x1(&hillshaded, &mut colorised, |v| Vec3::new(v, v, v));
        linear_to_srgb(&colorised, &mut output);
        blit(&output, &mut composited, width, 0);
    }

    {
        let mut processed = Texture::blank(width, height);
        let mut curved = Texture::blank(width, height);
        let mut colorised = Texture::blank(width, height);
        let mut output = Texture::blank(width, height);

        smooth(&height_map, &mut processed, pre_smoothing);
        curvature(&processed, &mut curved, pixel_size);
        operator1x1(&curved, &mut colorised, |v| {
            let v = (v * curve_amp).max(-1.0).min(1.0) * 0.5 + 0.5;
            Vec3::new(v, v, v)
        });
        linear_to_srgb(&colorised, &mut output);
        blit(&output, &mut composited, 0, height);
    }

    {
        let mut processed = Texture::blank(width, height);
        let mut sloped = Texture::blank(width, height);
        let mut colorised = Texture::blank(width, height);
        let mut output = Texture::blank(width, height);

        smooth(&height_map, &mut processed, pre_smoothing);
        slope(&processed, &mut sloped, pixel_size);
        operator1x1(&sloped, &mut colorised, |v| {
            let v = v.to_degrees() / 360.0 * slope_amp;
            Vec3::new(v, v, v)
        });
        linear_to_srgb(&colorised, &mut output);
        blit(&output, &mut composited, width, height);
    }

    {
        let mut processed = Texture::blank(width, height);
        let mut scaled = Texture::blank(width, height);
        let mut smoothed = Texture::blank(width, height);
        let mut shifted = Texture::blank(width, height);
        let mut colorised = Texture::blank(width, height);
        let mut output = Texture::blank(width, height);

        smooth(&height_map, &mut smoothed, pre_smoothing);
        terrain_generalise_weights(
            &smoothed,
            &mut processed,
            pixel_size,
            curve_amp,
            slope_amp,
            positive_weights_amp,
            negative_weights_amp,
            post_smoothing,
        );
        scale(&processed, &mut scaled, 0.5);
        shift(&scaled, &mut shifted, 0.5);
        operator1x1(&shifted, &mut colorised, |v| Vec3::new(v, v, v));
        linear_to_srgb(&colorised, &mut output);
        blit(&output, &mut composited, width * 2, height);
    }

    {
        let mut processed = Texture::blank(width, height);
        let mut smoothed = Texture::blank(width, height);
        let mut hillshaded = Texture::blank(width, height);
        let mut colorised = Texture::blank(width, height);
        let mut weights = Texture::blank(width, height);
        let mut output = Texture::blank(width, height);

        smooth(&height_map, &mut smoothed, pre_smoothing);
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
            &mut processed,
            &weights,
            exagerate,
        );
        hillshade(&processed, &mut hillshaded, 315.0, 45.0, pixel_size);
        operator1x1(&hillshaded, &mut colorised, |v| Vec3::new(v, v, v));
        linear_to_srgb(&colorised, &mut output);
        blit(&output, &mut composited, width * 2, 0);
    }

    peaks::export::PngExporter::export(
        &output_dir.clone().join("render.png"),
        &composited,
    )
}
