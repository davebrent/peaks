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

use bmp;
use math::Color;
use std::path::Path;
use textures::Texture;

pub struct BmpImporter;

impl BmpImporter {
    pub fn import(path: &Path) -> Texture<Color> {
        let image = bmp::open(path).unwrap();
        let width = image.get_width();
        let height = image.get_height();

        let mut output = Texture::blank(width as usize, height as usize);
        for (x, y) in image.coordinates() {
            let color = image.get_pixel(x, y);
            output.write1x1(
                x as usize,
                y as usize,
                Color::new(color.r, color.g, color.b),
            );
        }

        output
    }
}
