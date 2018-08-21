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

use gdal::spatial_ref::{CoordTransform, SpatialRef};

/// Convert a point from one spatial reference to another
pub fn transform_coords(
    lat: f64,
    lon: f64,
    src: &str,
    dest: &str,
) -> (f64, f64) {
    let mut xs = [lat];
    let mut ys = [lon];
    let mut zs = [0.0];

    let input = SpatialRef::from_proj4(src).unwrap();
    let output = SpatialRef::from_proj4(dest).unwrap();
    let transform = CoordTransform::new(&input, &output).unwrap();
    transform
        .transform_coords(&mut xs, &mut ys, &mut zs)
        .unwrap();

    (xs[0], ys[0])
}
