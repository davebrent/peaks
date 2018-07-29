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

use math::{Ray, Vec3};
use primitives::Aabb;
use render::{Intersectable, Intersection};
use spatial::{QuadTree, QuadTreeNode};
use std::cmp;
use textures::Texture;

pub struct HeightMap {
    /// A transform from world space coordinates to raster space
    pub transform: [f64; 4],
    /// Elevation values for the terrain
    pub height_map: Texture<f64>,
    /// Normal values for the terrain
    pub normal_map: Texture<Vec3>,
    /// Min and max extents of regions in the height map
    pub quad_tree: QuadTree<Aabb>,
}

impl HeightMap {
    pub fn new(
        transform: [f64; 4],
        height_map: Texture<f64>,
        normal_map: Texture<Vec3>,
        quad_tree: QuadTree<Aabb>,
    ) -> HeightMap {
        HeightMap {
            transform,
            height_map,
            normal_map,
            quad_tree,
        }
    }

    /// Transform world coordinates to UVs
    pub fn world_to_raster(&self, x: f64, y: f64) -> (f64, f64) {
        let x = (x - self.transform[0]) / self.transform[2];
        let y = (y - self.transform[1]) / self.transform[3];
        (x, y)
    }

    /// Transform UVs back to world coordinates
    pub fn raster_to_world(&self, x: f64, y: f64) -> (f64, f64) {
        let x = self.transform[0] + (x * self.transform[2]);
        let y = self.transform[1] + (y * self.transform[3]);
        (x, y)
    }

    /// Performs a depth first search to find the closest intersecting leaf node
    fn intersect_quad_tree(
        &self,
        ray: Ray,
    ) -> Option<(Intersection, QuadTreeNode<Aabb>)> {
        let root = match self.quad_tree.root() {
            Some(root) => root,
            None => return None,
        };

        let mut stack = vec![root];

        let origin = Vec3::new(ray.origin.x, 0.0, ray.origin.z);
        let flat_dist_comp =
            |a: &QuadTreeNode<Aabb>, b: &QuadTreeNode<Aabb>| {
                let ac = a.data.center();
                let bc = b.data.center();

                let (ax, ay) = self.raster_to_world(ac.x, ac.z);
                let (bx, by) = self.raster_to_world(bc.x, bc.z);

                let a = Vec3::distance(Vec3::new(ax, 0.0, ay), origin);
                let b = Vec3::distance(Vec3::new(bx, 0.0, by), origin);

                if a > b {
                    cmp::Ordering::Less
                } else {
                    cmp::Ordering::Greater
                }
            };

        while let Some(node) = stack.pop() {
            let mins = self.raster_to_world(node.data.min.x, node.data.min.z);
            let maxs = self.raster_to_world(node.data.max.x, node.data.max.z);
            let aabb = Aabb::new(
                Vec3::new(mins.0, root.data.min.y, mins.1),
                Vec3::new(maxs.0, node.data.max.y, maxs.1),
            );

            if let Some(intersection) = aabb.intersects(ray) {
                if intersection.t < 0.0 {
                    continue;
                } else if node.is_leaf() {
                    return Some((intersection, node));
                }

                // If this isnt a leaf, then split the node and test the
                // children, closest to the ray origin first
                let mut children = Vec::with_capacity(4);
                for child in &node.children {
                    if let Some(index) = child {
                        children.push(self.quad_tree.nodes[*index]);
                    }
                }
                children.sort_by(flat_dist_comp);
                stack.append(&mut children);
            }
        }

        None
    }
}

impl Intersectable for HeightMap {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        let (intersection, _) = match self.intersect_quad_tree(ray) {
            Some((intersection, node)) => (intersection, node.data),
            None => return None,
        };

        Some(intersection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_map_transform_raster_to_world() {
        let hmap = HeightMap::new(
            [-100.0, -100.0, 2.0, 2.0],
            Default::default(),
            Default::default(),
            Default::default(),
        );

        assert_eq!(hmap.raster_to_world(0.0, 0.0), (-100.0, -100.0));
        assert_eq!(hmap.raster_to_world(100.0, 0.0), (100.0, -100.0));
        assert_eq!(hmap.raster_to_world(100.0, 100.0), (100.0, 100.0));
        assert_eq!(hmap.raster_to_world(0.0, 100.0), (-100.0, 100.0));

        assert_eq!(hmap.world_to_raster(-100.0, -100.0), (0.0, 0.0));
        assert_eq!(hmap.world_to_raster(100.0, -100.0), (100.0, 0.0));
        assert_eq!(hmap.world_to_raster(100.0, 100.0), (100.0, 100.0));
        assert_eq!(hmap.world_to_raster(-100.0, 100.0), (0.0, 100.0));
    }
}
