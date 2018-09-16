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

use lights::DirectionalLight;
use math::{Ray, Vec3};
use primitives::Intersection;

pub struct TraceInfo {
    /// The ray used to populate this object
    pub ray: Ray,
    /// Intersection with a primitive shape
    pub intersection: Intersection,
    /// Intersecting primitives id
    pub primitive: usize,
    /// X coordinate on the view plane
    pub x: f64,
    /// Y coordinate on the view plane
    pub y: f64,
}

pub trait Tracer {
    /// Returns information for tracing a ray specified in screen space
    fn trace_pixel(&self, x: f64, y: f64) -> Option<TraceInfo>;
    /// Returns information for a ray trace
    fn trace_ray(&self, ray: Ray, x: f64, y: f64) -> Option<TraceInfo>;
    /// Return a shader with a given index
    fn shader(&self, index: usize) -> Option<&Shader>;
    /// Return the light for a given index
    fn light(&self, index: usize) -> Option<&DirectionalLight>;
}

pub trait Shader {
    /// Return the resulting color for a ray trace
    fn shade(&self, tracer: &Tracer, info: &TraceInfo) -> Vec3;
}
