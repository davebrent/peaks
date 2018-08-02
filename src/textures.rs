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

use std::cmp;
use std::ops::{Add, Mul};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    start_x: usize,
    start_y: usize,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TileIterator {
    width: usize,
    height: usize,
    size: usize,
    x: usize,
    y: usize,
}

impl Iterator for TileIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == self.height {
            return None;
        }

        let tw = cmp::min(self.size, self.width - self.x);
        let th = cmp::min(self.size, self.height - self.y);
        let (tx, ty) = (self.x, self.y);

        self.x += tw;
        if self.x == self.width {
            self.x = 0;
            self.y += th;
        }

        Some(Tile::new(tx, ty, tw, th))
    }
}

impl Tile {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Tile {
        Tile {
            x,
            y,
            width,
            height,
            start_x: x,
            start_y: y,
        }
    }
}

impl Iterator for Tile {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == self.start_y + self.height {
            return None;
        }

        let result = (self.x, self.y);
        if self.x == self.start_x + self.width - 1 {
            self.x = self.start_x;
            self.y += 1;
        } else {
            self.x += 1;
        }

        Some(result)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Texture<T>
where
    T: Send + Sync + Mul<f64, Output = T> + Add<Output = T> + Copy + Default,
{
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<T>,
}

impl<T> Texture<T>
where
    T: Send + Sync + Mul<f64, Output = T> + Add<Output = T> + Copy + Default,
{
    pub fn blank(width: usize, height: usize) -> Texture<T> {
        Texture {
            width,
            height,
            buffer: vec![Default::default(); width * height],
        }
    }

    pub fn new(width: usize, height: usize, buffer: Vec<T>) -> Texture<T> {
        Texture {
            width,
            height,
            buffer,
        }
    }

    /// Write a single value to the texture
    pub fn write1x1(&mut self, x: usize, y: usize, value: T) {
        let i = self.width * y + x;
        self.buffer[i] = value;
    }

    /// Read a single value from the texture
    pub fn lookup1x1(&self, x: usize, y: usize) -> T {
        let i = self.width * y + x;
        self.buffer[i]
    }

    /// Read a 2x2 window from the texture
    pub fn lookup2x2(&self, x: usize, y: usize) -> [T; 4] {
        let i1 = self.width * y + x;
        let i2 = self.width * (y + 1) + x;
        [
            self.buffer[i1],
            self.buffer[i1 + 1],
            self.buffer[i2],
            self.buffer[i2 + 1],
        ]
    }

    /// Read a 3x3 window from the texture
    pub fn lookup3x3(&self, x: usize, y: usize) -> [T; 9] {
        let i1 = self.width * y + x;
        let i2 = self.width * (y + 1) + x;
        let i3 = self.width * (y + 2) + x;

        [
            self.buffer[i1],
            self.buffer[i1 + 1],
            self.buffer[i1 + 2],
            self.buffer[i2],
            self.buffer[i2 + 1],
            self.buffer[i2 + 2],
            self.buffer[i3],
            self.buffer[i3 + 1],
            self.buffer[i3 + 2],
        ]
    }

    /// Lookup texture values for an arbitrary sized window
    pub fn lookup(&self, x: usize, y: usize, w: usize, h: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(w * h);
        for yi in y..y + h {
            for xi in x..x + w {
                let i = self.width * yi + xi;
                out.push(self.buffer[i]);
            }
        }
        out
    }

    /// Write a single value to the texture
    pub fn write(&mut self, x: usize, y: usize, w: usize, h: usize, value: T) {
        for yi in y..y + h {
            for xi in x..x + w {
                let i = self.width * yi + xi;
                self.buffer[i] = value;
            }
        }
    }

    /// Blit another texture onto this texture
    pub fn blit(&mut self, x: usize, y: usize, other: &Texture<T>) {
        let row_len = other.width;
        for src_y in 0..other.height {
            let src_offset = other.width * src_y;
            let dest_offset = self.width * (src_y + y) + x;
            let dest = &mut self.buffer[dest_offset..dest_offset + row_len];
            let row = &other.buffer[src_offset..src_offset + row_len];
            dest.copy_from_slice(row);
        }
    }

    /// Return a bilinearly filtered value from the texture
    pub fn bilinear(&self, x: f64, y: f64) -> T {
        if x < 0.0
            || x + 1.0 >= self.width as f64
            || y < 0.0
            || y + 1.0 >= self.height as f64
        {
            return Default::default();
        }

        let xf = x.floor();
        let yf = y.floor();

        let [c00, c10, c01, c11] = self.lookup2x2(xf as usize, yf as usize);

        let tx = x - xf;
        let ty = y - yf;

        let a = c00 * (1.0 - tx) * (1.0 - ty);
        let b = c10 * tx * (1.0 - ty);
        let c = c01 * (1.0 - tx) * ty;
        let d = c11 * tx * ty;

        a + b + c + d
    }

    /// Iterate over the image in fixed size square tiles
    pub fn tiles(&mut self, size: usize) -> TileIterator {
        TileIterator {
            width: self.width,
            height: self.height,
            size,
            x: 0,
            y: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blitting_textures() {
        let mut dest = Texture::new(8, 8, vec![0.0; 8 * 8]);
        let mut src = Texture::new(2, 2, vec![0.0; 2 * 2]);

        src.write1x1(0, 0, 250.0);
        src.write1x1(1, 1, 255.0);
        dest.blit(6, 6, &src);

        assert_eq!(dest.lookup1x1(0, 0), 0.0);
        assert_eq!(dest.lookup1x1(6, 6), 250.0);
        assert_eq!(dest.lookup1x1(7, 7), 255.0);
    }

    #[test]
    fn iterating_image_tiles() {
        let mut img = Texture::new(64, 64, vec![0.0; 64 * 64 * 3]);
        let tiles = img.tiles(8);
        assert_eq!(tiles.count(), 64);
    }
}
