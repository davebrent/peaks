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

use materials::Material;
use math::Vec3;
use primitives::Primitive;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Object {
    pub geometry: usize,
    pub material: usize,
}

impl Object {
    pub fn new(geometry: usize, material: usize) -> Object {
        Object { geometry, material }
    }
}

#[derive(Clone)]
pub struct Scene {
    pub background: Vec3,
    pub materials: Vec<Arc<Material>>,
    pub primitives: Vec<Arc<Primitive>>,
    pub objects: Vec<Object>,
}
