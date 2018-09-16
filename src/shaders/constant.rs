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
use options::ConstantShaderOpts;

#[derive(Copy, Clone, Debug, Default)]
pub struct ConstantShader {
    color: Vec3,
}

impl ConstantShader {
    pub fn new(color: Vec3) -> ConstantShader {
        ConstantShader { color }
    }
}

impl From<ConstantShaderOpts> for ConstantShader {
    fn from(options: ConstantShaderOpts) -> ConstantShader {
        ConstantShader {
            color: From::from(options.color),
        }
    }
}

impl Shader for ConstantShader {
    fn shade(&self, _: &Tracer, _: &TraceInfo) -> Vec3 {
        self.color
    }
}
