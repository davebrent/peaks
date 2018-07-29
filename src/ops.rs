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
use std::ops::{Add, Mul};
use textures::Texture;

/// Map a function over each pixel in a texture
fn operator1x1<F, I, O>(
    input: &Texture<I>,
    output: &mut Texture<O>,
    mut callback: F,
) where
    F: FnMut(I) -> O,
    I: Send + Sync + Mul<f64, Output = I> + Add<Output = I> + Copy + Default,
    O: Send + Sync + Mul<f64, Output = O> + Add<Output = O> + Copy + Default,
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
fn operator3x3<F, I, O>(
    input: &Texture<I>,
    output: &mut Texture<O>,
    mut callback: F,
) where
    F: FnMut(&[I; 9]) -> O,
    I: Send + Sync + Mul<f64, Output = I> + Add<Output = I> + Copy + Default,
    O: Send + Sync + Mul<f64, Output = O> + Add<Output = O> + Copy + Default,
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

/// Scale a surface by `n`
pub fn scale<T>(input: &Texture<T>, output: &mut Texture<T>, n: f64)
where
    T: Send + Sync + Mul<f64, Output = T> + Add<Output = T> + Copy + Default,
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

/// Apply a lowpass filter to a surface
pub fn lowpass<T>(input: &Texture<f64>, output: &mut Texture<f64>) {
    let weight = 1.0 / 9.0;
    operator3x3(input, output, |window| {
        window.iter().map(|val| val * weight).sum()
    })
}

/// Calculate a surfaces normals
pub fn normals(input: &Texture<f64>, output: &mut Texture<Vec3>) {
    operator3x3(input, output, |window| {
        let e4 = window[4];
        let e6 = window[6];
        let e2 = window[1];
        let e8 = window[8];
        Vec3::normalize(Vec3::new(e4 - e6, 2.0, e2 - e8))
    })
}

/// Calculate a surfaces 'slope' in radians
///
/// See explanation on the [ArcGIS tools documentation][1].
///
///   [1]: https://desktop.arcgis.com/en/arcmap/10.3/tools/
///   spatial-analyst-toolbox/how-slope-works.htm
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
///   [1]: https://desktop.arcgis.com/en/arcmap/10.3/tools/
///   spatial-analyst-toolbox/how-curvature-works.htm
///   [2]: https://desktop.arcgis.com/en/arcmap/10.3/manage-data/
///   raster-and-images/curvature-function.htm
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
}
