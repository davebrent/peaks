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

mod constant;
mod feature_lines;
mod normal;
mod phong;
mod sdf;
mod shader;
mod texture;

pub use self::constant::ConstantShader;
pub use self::feature_lines::FeatureLineShader;
pub use self::normal::NormalShader;
pub use self::phong::PhongShader;
pub use self::sdf::SdfShader;
pub use self::shader::{Shader, TraceInfo, Tracer};
pub use self::texture::TextureShader;
