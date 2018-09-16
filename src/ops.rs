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

    let mut window: [I; 9] = Default::default();
    let indices: [isize; 3] = [-1, 0, 1];

    for y in 0..height {
        for x in 0..width {
            for yw in &indices {
                for xw in &indices {
                    let bx = (x as isize + xw).max(0).min(width as isize - 1);
                    let by = (y as isize + yw).max(0).min(height as isize - 1);
                    let iw = (3 * (yw + 1) + (xw + 1)) as usize;
                    window[iw] = input.lookup1x1(bx as usize, by as usize);
                }
            }
            output.write1x1(x, y, callback(&window));
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

/// Blit one texture onto another
pub fn blit_region<T>(
    input: &Texture<T>,
    output: &mut Texture<T>,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) where
    T: Copy + Default,
{
    for src_y in 0..h {
        let src_offset = input.width * src_y;
        let dest_offset = output.width * (src_y + y) + x;
        let dest = &mut output.buffer[dest_offset..dest_offset + w];
        let row = &input.buffer[src_offset..src_offset + w];
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

/// Shift a surface by `n`
pub fn shift<T>(input: &Texture<T>, output: &mut Texture<T>, n: f64)
where
    T: Add<f64, Output = T> + Copy + Default,
{
    operator1x1(input, output, |value| value + n)
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

pub fn byte_stack_to_rgb(
    red: &Texture<u8>,
    green: &Texture<u8>,
    blue: &Texture<u8>,
    output: &mut Texture<Color>,
) {
    for i in 0..output.buffer.len() {
        let color = Color::new(red.buffer[i], green.buffer[i], blue.buffer[i]);
        output.buffer[i] = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_3x3() {
        let mut output = Texture::blank(1, 1);
        let input = Texture::new(1, 1, vec![50]);
        operator3x3(&input, &mut output, |window| window[4]);
        assert_eq!(output.lookup1x1(0, 0), 50);
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
