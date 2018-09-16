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

use math::Vec3;
use options::DirectionalLightOpts;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f64,
}

impl DirectionalLight {
    pub fn new(
        direction: Vec3,
        color: Vec3,
        intensity: f64,
    ) -> DirectionalLight {
        DirectionalLight {
            direction,
            color,
            intensity,
        }
    }
}

impl From<DirectionalLightOpts> for DirectionalLight {
    fn from(options: DirectionalLightOpts) -> DirectionalLight {
        DirectionalLight::new(
            From::from(options.direction),
            Vec3::zeros(),
            options.intensity,
        )
    }
}
