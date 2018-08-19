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

use gpx;

use std::convert::AsRef;
use std::fs::File;
use std::io::{BufReader, Result};
use std::path::Path;

use math::Vec3;

pub fn import<P>(path: P, track: usize, segment: usize) -> Result<Vec<Vec3>>
where
    P: AsRef<Path>,
{
    let file = try!(File::open(path));
    let reader = BufReader::new(file);
    let gpx = gpx::read(reader).unwrap();

    let track = &gpx.tracks[track];
    let segment = &track.segments[segment];
    let mut output = Vec::with_capacity(segment.points.len());

    for waypoint in &segment.points {
        let point = waypoint.point();
        let elevation = waypoint.elevation.unwrap_or(0.0);
        output.push(Vec3::new(point.x(), elevation, point.y()));
    }

    Ok(output)
}
