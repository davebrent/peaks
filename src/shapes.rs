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

use math::{AffineTransform, Vec3};
use std::f64::INFINITY;
use textures::{Bilinear, Texture};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    point: Vec3,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    x0y0: Vec3,
    x1y0: Vec3,
    x1y1: Vec3,
    x0y1: Vec3,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineString {
    points: Vec<Vec3>,
    bounds: Rect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ring {
    line: LineString,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    exterior: Ring,
    holes: Vec<Ring>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Shape {
    Point(Point),
    LineString(LineString),
    Ring(Ring),
    Polygon(Polygon),
}

impl Shape {
    /// Return the bounding box of the shape
    pub fn bbox(&self) -> Rect {
        match *self {
            Shape::Point(shape) => shape.bbox(),
            Shape::LineString(ref shape) => shape.bbox(),
            Shape::Ring(ref shape) => shape.bbox(),
            Shape::Polygon(ref shape) => shape.bbox(),
        }
    }

    /// Return a distance to the edge of the shape
    pub fn distance(&self, point: Vec3) -> f64 {
        match *self {
            Shape::Point(ref shape) => shape.distance(point),
            Shape::LineString(ref shape) => shape.distance(point),
            Shape::Ring(ref shape) => shape.distance(point),
            Shape::Polygon(ref shape) => shape.distance(point),
        }
    }

    /// Project the shape onto a height map
    pub fn project(
        &mut self,
        transform: AffineTransform,
        surface: &Texture<f64>,
    ) {
        match *self {
            Shape::Point(ref mut shape) => shape.project(transform, surface),
            Shape::LineString(ref mut shape) => {
                shape.project(transform, surface)
            }
            Shape::Ring(ref mut shape) => shape.project(transform, surface),
            Shape::Polygon(ref mut shape) => shape.project(transform, surface),
        }
    }
}

impl Point {
    pub fn new(point: Vec3) -> Point {
        Point { point }
    }

    pub fn bbox(&self) -> Rect {
        Rect::new(self.point, self.point, self.point, self.point)
    }

    pub fn distance(&self, point: Vec3) -> f64 {
        Vec3::distance(self.point, point)
    }

    pub fn project(
        &mut self,
        transform: AffineTransform,
        surface: &Texture<f64>,
    ) {
        let (u, v) = transform.inverse(self.point.x, self.point.z);
        self.point.y = surface.bilinear(u, v);
    }
}

impl Rect {
    pub fn new(x0y0: Vec3, x1y0: Vec3, x1y1: Vec3, x0y1: Vec3) -> Rect {
        Rect {
            x0y0,
            x1y0,
            x1y1,
            x0y1,
        }
    }

    pub fn offset(&self, amount: f64) -> Rect {
        let minx = self.x0y0.x;
        let maxx = self.x1y0.x;

        let miny = self.x0y0.z;
        let maxy = self.x0y1.z;

        Rect::new(
            Vec3::new(minx - amount, self.x0y0.y, miny - amount),
            Vec3::new(maxx + amount, self.x1y0.y, miny - amount),
            Vec3::new(maxx + amount, self.x1y1.y, maxy + amount),
            Vec3::new(minx - amount, self.x0y1.y, maxy + amount),
        )
    }

    pub fn contains(&self, point: Vec3) -> bool {
        let minx = self.x0y0.x;
        let maxx = self.x1y0.x;

        let miny = self.x0y0.z;
        let maxy = self.x0y1.z;

        point.x >= minx && point.x <= maxx && point.z >= miny && point.z <= maxy
    }
}

impl LineString {
    pub fn new(points: Vec<Vec3>) -> LineString {
        let (mut minx, mut miny) = (INFINITY, INFINITY);
        let (mut maxx, mut maxy) = (-INFINITY, -INFINITY);

        for point in &points {
            minx = minx.min(point.x);
            miny = miny.min(point.z);
            maxx = maxx.max(point.x);
            maxy = maxy.max(point.z);
        }

        LineString {
            points,
            bounds: Rect::new(
                Vec3::new(minx, 0.0, miny),
                Vec3::new(maxx, 0.0, miny),
                Vec3::new(maxx, 0.0, maxy),
                Vec3::new(minx, 0.0, maxy),
            ),
        }
    }

    pub fn distance(&self, point: Vec3) -> f64 {
        let mut minimum = INFINITY;

        // Based on http://paulbourke.net/geometry/pointlineplane/
        let p3 = point;
        for i in 0..self.points.len() - 1 {
            let p1 = self.points[i];
            let p2 = self.points[i + 1];
            let u = Vec3::dot(p3 - p1, p2 - p1) / Vec3::dot(p2 - p1, p2 - p1);
            let u = u.min(1.0).max(0.0);
            let other = p1 + (p2 - p1) * u;
            minimum = minimum.min(Vec3::distance(other, p3));
        }

        minimum
    }

    pub fn bbox(&self) -> Rect {
        self.bounds
    }

    pub fn project(
        &mut self,
        transform: AffineTransform,
        surface: &Texture<f64>,
    ) {
        for point in &mut self.points {
            let (u, v) = transform.inverse(point.x, point.z);
            point.y = surface.bilinear(u, v);
        }
    }
}

impl Ring {
    pub fn new(points: Vec<Vec3>) -> Ring {
        Ring {
            line: LineString::new(points),
        }
    }

    pub fn bbox(&self) -> Rect {
        self.line.bbox()
    }

    pub fn distance(&self, point: Vec3) -> f64 {
        let sign = if self.contains(point) { -1.0 } else { 1.0 };
        self.line.distance(point) * sign
    }

    pub fn contains(&self, point: Vec3) -> bool {
        let mut j = self.line.points.len() - 1;
        let mut signed = false;

        // Based on https://wrf.ecse.rpi.edu//Research/Short_Notes/pnpoly.html
        for i in 0..self.line.points.len() {
            let p1 = self.line.points[j];
            let p2 = self.line.points[i];
            let p = (p1.x - p2.x) * (point.z - p2.z) / (p1.z - p2.z) + p2.x;
            if (p2.z > point.z) != (p1.z > point.z) && (point.x < p) {
                signed = !signed;
            }
            j = i;
        }

        signed
    }

    pub fn project(
        &mut self,
        transform: AffineTransform,
        surface: &Texture<f64>,
    ) {
        self.line.project(transform, surface);
    }
}

impl Polygon {
    pub fn new(exterior: Ring, holes: Vec<Ring>) -> Polygon {
        Polygon { exterior, holes }
    }

    pub fn bbox(&self) -> Rect {
        self.exterior.bbox()
    }

    pub fn distance(&self, point: Vec3) -> f64 {
        let mut distance = self.exterior.distance(point);
        for hole in &self.holes {
            distance = distance.min(hole.distance(point));
        }
        distance
    }

    pub fn contains(&self, point: Vec3) -> bool {
        if !self.exterior.contains(point) {
            return false;
        }
        for hole in &self.holes {
            if hole.contains(point) {
                return false;
            }
        }
        true
    }

    pub fn project(
        &mut self,
        transform: AffineTransform,
        surface: &Texture<f64>,
    ) {
        self.exterior.project(transform, surface);
        for hole in &mut self.holes {
            hole.project(transform, surface);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line_segment() {
        let line_string = LineString::new(vec![
            Vec3::new(0.5, 0.0, 0.0),
            Vec3::new(0.5, 0.0, 1.0),
        ]);
        assert_eq!(line_string.distance(Vec3::new(1.0, 0.0, 0.5)), 0.5);
    }

    #[test]
    fn test_multi_line_segment() {
        let line_string = LineString::new(vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.5, 0.0, 0.0),
            Vec3::new(0.5, 0.0, 1.0),
        ]);
        assert_eq!(line_string.distance(Vec3::new(1.0, 0.0, 0.5)), 0.5);
    }

    #[test]
    fn test_simple_polygon() {
        let polygon = Polygon::new(
            Ring::new(vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, 0.0),
            ]),
            vec![],
        );
        assert_eq!(polygon.distance(Vec3::new(0.5, 0.0, 0.5)), -0.5);
        assert_eq!(polygon.distance(Vec3::new(1.5, 0.0, 0.5)), 0.5);
    }
}
