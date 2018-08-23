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

use std::convert::AsRef;
use std::path::Path;

use gdal::errors::Result;
use gdal::vector::{Dataset, Geometry, OGRwkbGeometryType};

use math::Vec3;
use shapes::{LineString, Point, Polygon, Ring, Shape};

/// Import geometry in multiple layers from an OGR supported file
pub fn import<P>(path: P, layers: &[usize]) -> Result<Vec<Vec<Shape>>>
where
    P: AsRef<Path>,
{
    let mut dataset = try!(Dataset::open(path.as_ref()));
    let mut output = Vec::with_capacity(layers.len());

    for index in layers {
        let input_layer = try!(dataset.layer(*index as isize));
        let mut shapes = vec![];
        for feature in input_layer.features() {
            let geometry = feature.geometry();
            for shape in from(geometry) {
                shapes.push(shape);
            }
        }
        output.push(shapes);
    }

    assert_eq!(output.len(), 1);
    Ok(output)
}

fn from_polygon(geometry: &Geometry) -> Shape {
    let rings: Vec<_> = (0..geometry.geometry_count())
        .map(|n: usize| {
            let points = unsafe { geometry._get_geometry(n) }
                .get_point_vec()
                .iter()
                .map(|(x, z, y)| Vec3::new(*x, *y, *z * -1.0))
                .collect();
            Ring::new(points)
        })
        .collect();

    if let Some((exterior, holes)) = rings.split_first() {
        Shape::Polygon(Polygon::new(exterior.clone(), holes.to_vec()))
    } else {
        Shape::Polygon(Polygon::new(Ring::new(vec![]), vec![]))
    }
}

fn from_line(geometry: &Geometry) -> Shape {
    let points = geometry
        .get_point_vec()
        .iter()
        .map(|(x, z, y)| Vec3::new(*x, *y, *z * -1.0))
        .collect();
    Shape::LineString(LineString::new(points))
}

fn from_point(geometry: &Geometry) -> Shape {
    let (x, z, y) = geometry.get_point(0);
    Shape::Point(Point::new(Vec3::new(x, y, z * -1.0)))
}

fn from(geometry: &Geometry) -> Vec<Shape> {
    match geometry.geometry_type() {
        OGRwkbGeometryType::wkbPoint => vec![from_point(&geometry)],
        OGRwkbGeometryType::wkbLineString => vec![from_line(&geometry)],
        OGRwkbGeometryType::wkbPolygon => vec![from_polygon(&geometry)],
        OGRwkbGeometryType::wkbMultiPolygon => (0..geometry.geometry_count())
            .map(|n: usize| {
                let geometry = unsafe { geometry._get_geometry(n) };
                from_polygon(&geometry)
            })
            .collect(),
        OGRwkbGeometryType::wkbMultiLineString => (0..geometry
            .geometry_count())
            .map(|n: usize| {
                let geometry = unsafe { geometry._get_geometry(n) };
                from_line(&geometry)
            })
            .collect(),
        OGRwkbGeometryType::wkbMultiPoint => (0..geometry.geometry_count())
            .map(|n: usize| {
                let geometry = unsafe { geometry._get_geometry(n) };
                from_point(&geometry)
            })
            .collect(),
        _ => vec![],
    }
}
