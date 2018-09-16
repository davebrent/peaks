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

use super::shader::{Shader, TraceInfo, Tracer};
use math::{AffineTransform, Vec3};
use options::TextureShaderOpts;
use textures::{Bilinear, Texture};

#[derive(Clone, Debug, Default)]
pub struct TextureShader {
    transform: AffineTransform,
    texture: Texture<Vec3>,
}

impl TextureShader {
    pub fn new(
        transform: AffineTransform,
        texture: Texture<Vec3>,
    ) -> TextureShader {
        TextureShader { transform, texture }
    }
}

impl From<TextureShaderOpts> for TextureShader {
    fn from(options: TextureShaderOpts) -> TextureShader {
        match options.components {
            1 => {
                let data = options
                    .data
                    .into_iter()
                    .map(|d| Vec3::new(d, d, d))
                    .collect();
                let texture = Texture::new(options.width, options.height, data);
                TextureShader::new(From::from(options.transform), texture)
            }
            3 => {
                assert_eq!(options.data.len() % 3, 0);
                let mut data = Vec::with_capacity(options.data.len());
                for i in 0..options.width * options.height {
                    let pixel = &options.data[i * 3..i * 3 + 3];
                    data.push(Vec3::new(pixel[0], pixel[1], pixel[2]));
                }
                TextureShader::new(
                    From::from(options.transform),
                    Texture::new(options.width, options.height, data),
                )
            }
            _ => {
                // FIXME: Return an error instead
                TextureShader::new(Default::default(), Texture::blank(1, 1))
            }
        }
    }
}

impl Shader for TextureShader {
    fn shade(&self, _: &Tracer, info: &TraceInfo) -> Vec3 {
        let point = info.ray.origin + info.ray.direction * info.intersection.t;
        let (u, v) = self.transform.inverse(point.x, point.z);
        self.texture.bilinear(u, v)
    }
}
