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

use std::fs::File;
use std::io::{BufWriter, Result};
use std::path::Path;

use png::{self, HasParameters};

use math::Color;
use textures::Texture;

pub struct PngExporter;

impl PngExporter {
    pub fn export(file_path: &Path, texture: &Texture<Color>) -> Result<()> {
        let width = texture.width as u32;
        let height = texture.height as u32;

        let mut bytes = Vec::with_capacity((width * height * 3) as usize);
        for color in &texture.buffer {
            bytes.push(color.r);
            bytes.push(color.g);
            bytes.push(color.b);
        }

        let file = try!(File::create(file_path));
        let writer = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);

        let mut writer = try!(encoder.write_header());
        try!(writer.write_image_data(&bytes));
        Ok(())
    }
}
