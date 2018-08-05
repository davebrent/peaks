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

use super::aabb::Aabb;
use super::bilinear_patch::BilinearPatch;
use math::{ceil_pow2, Ray, Vec3};
use ops::{
    blit, height_map_to_bilinear_patch, maximum_mipmap_bilinear_patch, normals,
};
use render::{Intersectable, Intersection};
use std::cmp;
use textures::{Bilinear, Texture};

pub struct HeightMap {
    /// A transform from world space coordinates to raster space
    pub transform: [f64; 4],
    /// Normal values for the terrain
    pub normal_map: Texture<Vec3>,
    /// Map containing bilinear patches
    pub bilinear_patches: Texture<[f64; 4]>,
    /// Maximum mipmaps for the bilinear patches
    pub maximum_mipmaps: Vec<Texture<f64>>,
}

impl HeightMap {
    pub fn new(transform: [f64; 4], height_map: &Texture<f64>) -> HeightMap {
        // Generate a normal map from the height map
        let mut normal_map =
            Texture::blank(height_map.width, height_map.height);
        normals(height_map, &mut normal_map, transform[2], transform[3]);

        // Round the height map size to the nearest power of two
        let height_map_size = height_map.width.max(height_map.height);
        let mut size = ceil_pow2(height_map_size);

        // Create a new height map with the size of n^2+1 and blit the original
        let mut height_map2 = Texture::blank(size + 1, size + 1);
        blit(&height_map, &mut height_map2, 0, 0);

        // Then create the bilinear patch texture and its initial mipmap
        let mut bilinear_patches = Texture::blank(size, size);
        let mut bilinear_patches_mipmap0 = Texture::blank(size, size);
        height_map_to_bilinear_patch(
            &height_map2,
            &mut bilinear_patches,
            &mut bilinear_patches_mipmap0,
        );

        // Create the subsequent mipmap levels from level0
        let mut maximum_mipmaps = vec![bilinear_patches_mipmap0];
        while size > 1 {
            size /= 2;
            let mut next = Texture::blank(size, size);
            {
                let previous = &maximum_mipmaps[maximum_mipmaps.len() - 1];
                maximum_mipmap_bilinear_patch(previous, &mut next);
            }
            maximum_mipmaps.push(next);
        }

        HeightMap {
            transform,
            normal_map,
            bilinear_patches,
            maximum_mipmaps,
        }
    }

    /// Transform world coordinates to UVs
    fn world_to_raster(&self, x: f64, y: f64) -> (f64, f64) {
        let x = (x - self.transform[0]) / self.transform[2];
        let y = (y - self.transform[1]) / self.transform[3];
        (x, y)
    }

    fn raster_to_world(&self, level: u32, x: f64, y: f64) -> (f64, f64) {
        let width = self.transform[2] * (2_usize.pow(level as u32) as f64);
        let depth = self.transform[3] * (2_usize.pow(level as u32) as f64);
        let x = self.transform[0] + (x * width);
        let y = self.transform[1] + (y * depth);
        (x, y)
    }
}

impl Intersectable for HeightMap {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        if self.maximum_mipmaps.is_empty() {
            return None;
        }

        let origin = Vec3::new(ray.origin.x, 0.0, ray.origin.z);
        let flat_dist_comp =
            |(al, ax, ay): &(usize, usize, usize),
             (bl, bx, by): &(usize, usize, usize)| {
                let (ax, az) = self.raster_to_world(
                    *al as u32,
                    *ax as f64 + 0.5,
                    *ay as f64 + 0.5,
                );
                let (bx, bz) = self.raster_to_world(
                    *bl as u32,
                    *bx as f64 + 0.5,
                    *by as f64 + 0.5,
                );
                let a = Vec3::distance(Vec3::new(ax, 0.0, az), origin);
                let b = Vec3::distance(Vec3::new(bx, 0.0, bz), origin);
                if a > b {
                    cmp::Ordering::Less
                } else {
                    cmp::Ordering::Greater
                }
            };

        let mut stack = vec![(self.maximum_mipmaps.len() - 1, 0, 0)];
        while let Some((level, x, y)) = stack.pop() {
            let mipmap = &self.maximum_mipmaps[level];

            let lu = level as u32;
            let (fx, fx1) = (x as f64, x as f64 + 1.0);
            let (fy, fy1) = (y as f64, y as f64 + 1.0);

            let (min_x, min_z) = self.raster_to_world(lu, fx, fy);
            let (max_x, max_z) = self.raster_to_world(lu, fx1, fy1);
            let (min_y, max_y) = (0.0, mipmap.lookup1x1(x, y));

            let aabb = Aabb::new(
                Vec3::new(min_x, min_y, min_z),
                Vec3::new(max_x, max_y, max_z),
            );

            let intersection = match aabb.intersects(ray) {
                None => continue,
                Some(intersection) => intersection,
            };

            if intersection.t < 0.0 {
                continue;
            }

            if level == 0 {
                let [nw, ne, se, sw] = self.bilinear_patches.lookup1x1(x, y);
                let nw = Vec3::new(min_x, nw, min_z);
                let ne = Vec3::new(max_x, ne, min_z);
                let se = Vec3::new(max_x, se, max_z);
                let sw = Vec3::new(min_x, sw, max_z);
                let patch = BilinearPatch::new(nw, ne, se, sw);
                match patch.intersects(ray) {
                    Some(intersection) => {
                        let point = ray.origin + ray.direction * intersection.t;
                        let (x, y) = self.world_to_raster(point.x, point.z);
                        let normal = self.normal_map.bilinear(x, y);
                        return Some(Intersection::new(intersection.t, normal));
                    }
                    _ => continue,
                };
            } else {
                let (cx, cy) = (x * 2, y * 2);
                let mut children = vec![
                    (level - 1, cx, cy),
                    (level - 1, cx + 1, cy),
                    (level - 1, cx, cy + 1),
                    (level - 1, cx + 1, cy + 1),
                ];
                children.sort_by(flat_dist_comp);
                stack.append(&mut children);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_map_transform_raster_to_world() {
        let height_map =
            HeightMap::new([-100.0, -100.0, 2.0, 2.0], &Texture::blank(1, 1));
        assert_eq!(height_map.world_to_raster(-100.0, -100.0), (0.0, 0.0));
        assert_eq!(height_map.world_to_raster(100.0, -100.0), (100.0, 0.0));
        assert_eq!(height_map.world_to_raster(100.0, 100.0), (100.0, 100.0));
        assert_eq!(height_map.world_to_raster(-100.0, 100.0), (0.0, 100.0));
    }
}
