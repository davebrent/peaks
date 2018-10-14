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
use super::primitive::{Intersection, Primitive};

use io::gdal;
use math::{AffineTransform, Ray, Vec3};
use ops::{blit, height_map_to_bilinear_patch, maximum_mipmap_bilinear_patch};
use options::{HeightMapOpts, Loader};
use textures::Texture;
use shapes::Rect;

use std::cmp;

fn ceil_pow2(num: usize) -> usize {
    let num = num as f64;
    let exp = (num.log2() / 2.0_f64.log2()).ceil();
    2.0_f64.powf(exp) as usize
}

pub struct HeightMap {
    pub rect: Rect,
    /// A transform from world space coordinates to raster space
    pub transform: AffineTransform,
    /// Map containing bilinear patches
    pub bilinear_patches: Texture<[f64; 4]>,
    /// Maximum mipmaps for the bilinear patches
    pub maximum_mipmaps: Vec<Texture<f64>>,
}

impl HeightMap {
    pub fn new(
        transform: AffineTransform,
        height_map: &Texture<f64>,
    ) -> HeightMap {
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

        let rect = {
            let width = height_map.width;
            let depth = height_map.height;

            let (x0, z0) = transform.forward(0.0, 0.0);
            let (x1, _) = transform.forward(width as f64, 0.0);
            let (_, z1) = transform.forward(width as f64, depth as f64);
            let (_, _) = transform.forward(0.0, depth as f64);

            Rect::new(
                Vec3::new(x0, 0.0, z0),
                Vec3::new(x1, 0.0, z0),
                Vec3::new(x1, 0.0, z1),
                Vec3::new(x0, 0.0, z1),
            )
        };

        HeightMap {
            rect,
            transform,
            bilinear_patches,
            maximum_mipmaps,
        }
    }
}

impl From<HeightMapOpts> for HeightMap {
    fn from(options: HeightMapOpts) -> HeightMap {
        let (transform, texture) = match options.data {
            Loader::Gdal(opts) => {
                let (_, transform, rasters) =
                    gdal::import(opts.filepath, &[opts.band]).unwrap();
                (transform, rasters[0].clone())
            }
            _ => panic!("Unsupported format"),
        };

        HeightMap::new(transform, &texture)
    }
}

impl Primitive for HeightMap {
    fn intersects(&self, ray: Ray) -> Option<Intersection> {
        if self.maximum_mipmaps.is_empty() {
            return None;
        }

        let origin = Vec3::new(ray.origin.x, 0.0, ray.origin.z);
        let flat_dist_comp =
            |(al, ax, ay): &(usize, usize, usize),
             (bl, bx, by): &(usize, usize, usize)| {
                let (ax, az) = self.transform.quadtree(
                    *al,
                    *ax as f64 + 0.5,
                    *ay as f64 + 0.5,
                );
                let (bx, bz) = self.transform.quadtree(
                    *bl,
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

            let (fx, fx1) = (x as f64, x as f64 + 1.0);
            let (fy, fy1) = (y as f64, y as f64 + 1.0);

            let (min_x, min_z) = self.transform.quadtree(level, fx, fy);
            let (max_x, max_z) = self.transform.quadtree(level, fx1, fy1);
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
                        let p = ray.origin + ray.direction * intersection.t;
                        if self.rect.contains(Vec3::new(p.x, 0.0, p.z)) {
                            return Some(intersection)
                        }
                    },
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
