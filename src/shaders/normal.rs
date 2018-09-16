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
use math::Vec3;
use options::NormalShaderOpts;

#[derive(Copy, Clone, Debug, Default)]
pub struct NormalShader;

impl NormalShader {
    pub fn new() -> NormalShader {
        NormalShader {}
    }
}

impl From<NormalShaderOpts> for NormalShader {
    fn from(_: NormalShaderOpts) -> NormalShader {
        NormalShader::new()
    }
}

impl Shader for NormalShader {
    fn shade(&self, _: &Tracer, info: &TraceInfo) -> Vec3 {
        (info.intersection.normal + 1.0) * 0.5
    }
}
