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

use math::{Color, Vec3};
use std::f64::consts::PI;
use std::ops::Mul;
use textures::Texture;

/// Map a function over each pixel in a texture
pub fn operator1x1<F, I, O>(
    input: &Texture<I>,
    output: &mut Texture<O>,
    mut callback: F,
) where
    F: FnMut(I) -> O,
    I: Copy + Default,
    O: Copy + Default,
{
    assert_eq!(input.width, output.width);
    assert_eq!(input.height, output.height);

    let width = input.width;
    let height = input.height;

    for y in 0..height {
        for x in 0..width {
            let value = input.lookup1x1(x, y);
            let result = callback(value);
            output.write1x1(x, y, result);
        }
    }
}

/// Map a function over a texture with a 3x3 window
pub fn operator3x3<F, I, O>(
    input: &Texture<I>,
    output: &mut Texture<O>,
    mut callback: F,
) where
    F: FnMut(&[I; 9]) -> O,
    I: Copy + Default,
    O: Copy + Default,
{
    assert_eq!(input.width, output.width);
    assert_eq!(input.height, output.height);

    let width = input.width;
    let height = input.height;

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let window = input.lookup3x3(x - 1, y - 1);
            let result = callback(&window);
            output.write1x1(x, y, result);
        }
    }
}

/// Blit one texture onto another
pub fn blit<T>(input: &Texture<T>, output: &mut Texture<T>, x: usize, y: usize)
where
    T: Copy + Default,
{
    let row_len = input.width;
    for src_y in 0..input.height {
        let src_offset = input.width * src_y;
        let dest_offset = output.width * (src_y + y) + x;
        let dest = &mut output.buffer[dest_offset..dest_offset + row_len];
        let row = &input.buffer[src_offset..src_offset + row_len];
        dest.copy_from_slice(row);
    }
}

/// Create map of bilinear patches and its first mipmap level from a height map
pub fn height_map_to_bilinear_patch(
    input: &Texture<f64>,
    level0: &mut Texture<[f64; 4]>,
    level1: &mut Texture<f64>,
) {
    assert_eq!(level0.width, level1.width);
    assert_eq!(level0.height, level1.height);
    assert_eq!(input.width - 1, level0.width);
    assert_eq!(input.height - 1, level0.height);

    for y in 0..level0.height {
        for x in 0..level0.width {
            // Read data in `z` order but write out in a clockwise order
            let [nw, ne, sw, se] = input.lookup2x2(x, y);
            level0.write1x1(x, y, [nw, ne, se, sw]);
            level1.write1x1(x, y, nw.max(ne).max(se).max(sw));
        }
    }
}

/// Create the next maximum mipmap level for a floating point texture
pub fn maximum_mipmap_bilinear_patch(
    input: &Texture<f64>,
    output: &mut Texture<f64>,
) {
    assert_eq!(input.width / 2, output.width);
    assert_eq!(input.height / 2, output.height);

    for y in 0..output.height {
        for x in 0..output.width {
            let [p1, p2, p3, p4] = input.lookup2x2(x * 2, y * 2);
            output.write1x1(x, y, p1.max(p2).max(p3).max(p4));
        }
    }
}

/// Scale a surface by `n`
pub fn scale<T>(input: &Texture<T>, output: &mut Texture<T>, n: f64)
where
    T: Mul<f64, Output = T> + Copy + Default,
{
    operator1x1(input, output, |value| value * n)
}

/// Unpack a texture as a floating point height map
pub fn rgb_height_map(
    input: &Texture<Color>,
    output: &mut Texture<f64>,
    scaler: f64,
) {
    operator1x1(input, output, |color| f64::from(color.r) / 255.0 * scaler)
}

/// Convert linear colors to sRGB
pub fn linear_to_srgb(input: &Texture<Vec3>, output: &mut Texture<Color>) {
    let encode = |component: f64| {
        if component <= 0.003_130_8 {
            component * 12.92
        } else {
            1.055 * component.powf(1.0 / 2.4) - 0.055
        }
    };

    operator1x1(input, output, |val| {
        Color::new(
            (encode(val.x) * 255.0).round().min(255.0).max(0.0) as u8,
            (encode(val.y) * 255.0).round().min(255.0).max(0.0) as u8,
            (encode(val.z) * 255.0).round().min(255.0).max(0.0) as u8,
        )
    })
}

/// Convert sRGB colors to linear
pub fn srgb_to_linear(input: &Texture<Color>, output: &mut Texture<Vec3>) {
    let decode = |component: f64| {
        if component <= 0.04045 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    };

    operator1x1(input, output, |val| {
        Vec3::new(
            decode(f64::from(val.r) / 255.0),
            decode(f64::from(val.g) / 255.0),
            decode(f64::from(val.b) / 255.0),
        )
    })
}

/// Apply a lowpass filter to a surface
pub fn lowpass(input: &Texture<f64>, output: &mut Texture<f64>) {
    let weight = 1.0 / 9.0;
    operator3x3(input, output, |window| {
        window.iter().map(|val| val * weight).sum()
    })
}

/// Calculate a surfaces normals
pub fn normals(
    input: &Texture<f64>,
    output: &mut Texture<Vec3>,
    pixel_width: f64,
    pixel_height: f64,
) {
    for y in 0..output.height - 1 {
        for x in 0..output.width - 1 {
            let [nw, ne, sw, _] = input.lookup2x2(x, y);
            let nw = Vec3::new(-0.5 * pixel_width, nw, -0.5 * pixel_height);
            let ne = Vec3::new(0.5 * pixel_width, ne, -0.5 * pixel_height);
            let sw = Vec3::new(-0.5 * pixel_width, sw, 0.5 * pixel_height);
            let normal = Vec3::normalize(Vec3::cross(sw - ne, ne - nw));
            output.write1x1(x, y, normal);
        }
    }
}

/// Calculate a surfaces 'slope' in radians
///
/// See explanation on the [ArcGIS tools documentation][1].
///
/// [1]: https://desktop.arcgis.com/en/arcmap/10.3/tools/\
/// spatial-analyst-toolbox/how-slope-works.htm
pub fn slope(input: &Texture<f64>, output: &mut Texture<f64>, pixel_size: f64) {
    let cell_size = 8.0 * pixel_size;
    operator3x3(input, output, |window| {
        let [a, b, c, d, _, f, g, h, i] = window;
        let dzdx = ((c + (2.0 * f) + i) - (a + (2.0 * d) + g)) / cell_size;
        let dzdy = ((g + (2.0 * h) + i) - (a + (2.0 * b) + c)) / cell_size;
        let rise_run = (dzdx.powi(2) + dzdy.powi(2)).sqrt();
        rise_run.atan()
    })
}

/// Calculate a surfaces curvature
///
/// See explanation on the ArcGIS documentation [here][1] and [here][2].
///
/// [1]: https://desktop.arcgis.com/en/arcmap/10.3/tools/\
/// spatial-analyst-toolbox/how-curvature-works.htm
/// [2]: https://desktop.arcgis.com/en/arcmap/10.3/manage-data/\
/// raster-and-images/curvature-function.htm
pub fn curvature(
    input: &Texture<f64>,
    output: &mut Texture<f64>,
    pixel_size: f64,
) {
    let l2 = pixel_size.powi(2);
    operator3x3(input, output, |window| {
        let [_, z2, _, z4, z5, z6, _, z8, _] = window;
        let d = ((z4 + z6) / 2.0 - z5) / l2;
        let e = ((z2 + z8) / 2.0 - z5) / l2;
        -2.0 * (d + e) * 100.0
    })
}

/// Create a hillshade of a height map
///
/// See explanation [here][1].
///
/// [1]: https://pro.arcgis.com/en/pro-app/tool-reference/3d-analyst/
/// how-hillshade-works.htm
pub fn hillshade(
    input: &Texture<f64>,
    output: &mut Texture<f64>,
    azimuth: f64,
    altitude: f64,
    pixel_size: f64,
) {
    let zenith_deg = 90.0 - altitude;
    let zenith_rad = zenith_deg.to_radians();

    let mut azimuth_math = 360.0 - azimuth + 90.0;
    if azimuth_math >= 360.0 {
        azimuth_math -= 360.0;
    }

    let azimuth_rad = azimuth_math.to_radians();

    let cell_size = 8.0 * pixel_size;
    operator3x3(input, output, |window| {
        let [a, b, c, d, _, f, g, h, i] = window;
        let dzdx = ((c + (2.0 * f) + i) - (a + (2.0 * d) + g)) / cell_size;
        let dzdy = ((g + (2.0 * h) + i) - (a + (2.0 * b) + c)) / cell_size;
        let rise_run = (dzdx.powi(2) + dzdy.powi(2)).sqrt();
        let slope_rad = rise_run.atan();

        let mut aspect_rad = 0.0;
        if dzdx != 0.0 {
            aspect_rad = dzdy.atan2(-dzdx);
            if aspect_rad < 0.0 {
                aspect_rad += 2.0 * PI;
            }
        } else if dzdy > 0.0 {
            aspect_rad = PI / 2.0;
        } else if dzdy < 0.0 {
            aspect_rad = 2.0 * PI + -(PI / 2.0);
        }

        zenith_rad.cos() * slope_rad.cos()
            + zenith_rad.sin()
                * slope_rad.sin()
                * (azimuth_rad - aspect_rad).cos()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slope_calculation() {
        // Values taken from the arcgis example
        let mut output = Texture::blank(3, 3);
        let input = Texture::new(
            3,
            3,
            vec![50.0, 45.0, 50.0, 30.0, 30.0, 30.0, 8.0, 10.0, 10.0],
        );
        slope(&input, &mut output, 5.0);
        let result = output.lookup1x1(1, 1).to_degrees().round() as usize;
        assert_eq!(result, 75);
    }

    #[test]
    fn blitting_textures() {
        let mut dest = Texture::new(8, 8, vec![0.0; 8 * 8]);
        let mut src = Texture::new(2, 2, vec![0.0; 2 * 2]);

        src.write1x1(0, 0, 250.0);
        src.write1x1(1, 1, 255.0);
        blit(&src, &mut dest, 6, 6);

        assert_eq!(dest.lookup1x1(0, 0), 0.0);
        assert_eq!(dest.lookup1x1(6, 6), 250.0);
        assert_eq!(dest.lookup1x1(7, 7), 255.0);
    }

    #[test]
    fn normals_flat_surface() {
        let mut output = Texture::blank(3, 3);
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let input = Texture::new(3, 3, vec![
            0.0, 0.0, 0.0,
            0.0, 0.0, 0.0,
            0.0, 0.0, 0.0
        ]);
        normals(&input, &mut output, 1.0, 1.0);
        assert_eq!(output.lookup1x1(1, 1), Vec3::new(0.0, 1.0, 0.0))
    }

    #[test]
    fn normals_sloped_surface() {
        let mut output = Texture::blank(3, 3);
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let input = Texture::new(3, 3, vec![
            1.0, 2.0 / 3.0, 1.0 / 3.0,
            1.0, 2.0 / 3.0, 1.0 / 3.0,
            1.0, 2.0 / 3.0, 1.0 / 3.0,
        ]);
        normals(&input, &mut output, 1.0 / 3.0, 1.0 / 3.0);
        assert_eq!(
            output.lookup1x1(1, 1),
            Vec3::new(0.7071067811865476, 0.7071067811865476, 0.0)
        )
    }

    #[test]
    fn test_maximum_mipmaps_bilinear_patches() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mut bilinear_patches = Texture::blank(4, 4);
        let mut bilinear_patches_mipmap0 = Texture::blank(4, 4);
        let mut bilinear_patches_mipmap1 = Texture::blank(2, 2);
        let mut bilinear_patches_mipmap2 = Texture::blank(1, 1);
        let height_map = Texture::new(
            5,
            5,
            vec![
                1.0, 3.5, 6.0, 8.5, 11.0, 1.5, 4.0, 6.5, 9.0, 11.5, 2.0, 4.5,
                7.0, 9.5, 12.0, 2.5, 5.0, 7.5, 10.0, 12.5, 3.0, 5.5, 8.0, 10.5,
                13.0,
            ],
        );

        height_map_to_bilinear_patch(
            &height_map,
            &mut bilinear_patches,
            &mut bilinear_patches_mipmap0,
        );
        maximum_mipmap_bilinear_patch(
            &bilinear_patches_mipmap0,
            &mut bilinear_patches_mipmap1,
        );
        maximum_mipmap_bilinear_patch(
            &bilinear_patches_mipmap1,
            &mut bilinear_patches_mipmap2,
        );

        #[cfg_attr(rustfmt, rustfmt_skip)]
        assert_eq!(bilinear_patches.buffer, [
            // Row 1
            [1.0, 3.5, 4.0, 1.5],
            [3.5, 6.0, 6.5, 4.0],
            [6.0, 8.5, 9.0, 6.5],
            [8.5, 11.0, 11.5, 9.0],
            // Row 2
            [1.5, 4.0, 4.5, 2.0],
            [4.0, 6.5, 7.0, 4.5],
            [6.5, 9.0, 9.5, 7.0],
            [9.0, 11.5, 12.0, 9.5],
            // Row 3
            [2.0, 4.5, 5.0, 2.5],
            [4.5, 7.0, 7.5, 5.0],
            [7.0, 9.5, 10.0, 7.5],
            [9.5, 12.0, 12.5, 10.0],
            // Row 4
            [2.5, 5.0, 5.5, 3.0],
            [5.0, 7.5, 8.0, 5.5],
            [7.5, 10.0, 10.5, 8.0],
            [10.0, 12.5, 13.0, 10.5],
        ]);

        #[cfg_attr(rustfmt, rustfmt_skip)]
        assert_eq!(bilinear_patches_mipmap0.buffer, [
            4.0, 6.5,  9.0, 11.5,
            4.5, 7.0,  9.5, 12.0,
            5.0, 7.5, 10.0, 12.5,
            5.5, 8.0, 10.5, 13.0,
        ]);
        assert_eq!(bilinear_patches_mipmap1.buffer, [7.0, 12.0, 8.0, 13.0]);
        assert_eq!(bilinear_patches_mipmap2.buffer, [13.0]);
    }

    #[test]
    fn test_hillshade_calculation() {
        let mut output = Texture::blank(3, 3);
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let input = Texture::new(3, 3, vec![
            2450.0, 2461.0, 2483.0,
            2452.0, 2461.0, 2483.0,
            2447.0, 2455.0, 2477.0,
        ]);
        hillshade(&input, &mut output, 315.0, 45.0, 5.0);
        let result = output.lookup1x1(1, 1);
        assert_eq!((result * 255.0).round() as usize, 154);
    }

    #[test]
    fn test_srgb_transforms() {
        let input = Texture::new(1, 1, vec![Vec3::new(0.5, 0.5, 0.5)]);
        let mut color = Texture::blank(1, 1);
        let mut linear = Texture::blank(1, 1);
        linear_to_srgb(&input, &mut color);
        srgb_to_linear(&color, &mut linear);
        assert_eq!(
            (linear.lookup1x1(0, 0) * 100.0).round(),
            (input.lookup1x1(0, 0) * 100.0).round(),
        );
    }
}
