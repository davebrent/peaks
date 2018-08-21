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
use std::io::{Result, Write};
use std::path::Path;

use math::Color;
use textures::Texture;

pub fn export(file_path: &Path, texture: &Texture<Color>) -> Result<()> {
    let width = texture.width;
    let height = texture.height;

    let mut bytes = Vec::with_capacity(width * height * 3);
    for color in &texture.buffer {
        bytes.push(color.r);
        bytes.push(color.g);
        bytes.push(color.b);
    }

    let mut f = try!(File::create(file_path));
    try!(f.write_all(format!("P6 {} {} 255\n", width, height).as_bytes()));
    f.write_all(&bytes)
}
