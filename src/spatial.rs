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

use std::cmp;
use std::collections::VecDeque;
use std::f64::{EPSILON, INFINITY};

use math::Vec3;
use primitives::Aabb;
use textures::{Texture, Tile};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct QuadTreeNode<T>
where
    T: Copy + Default,
{
    pub data: T,
    pub children: [Option<usize>; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuadTree<T>
where
    T: Copy + Default,
{
    pub nodes: Vec<QuadTreeNode<T>>,
}

impl<T> QuadTreeNode<T>
where
    T: Copy + Default,
{
    pub fn new(data: T) -> QuadTreeNode<T> {
        QuadTreeNode {
            data,
            children: [None; 4],
        }
    }

    /// Returns true if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        for slot in &self.children {
            if slot.is_some() {
                return false;
            }
        }
        true
    }

    /// Append a child node into this node
    pub fn append(&mut self, child: usize) {
        // FIXME: Return an error if all slots are full
        for slot in &mut self.children {
            if slot.is_none() {
                *slot = Some(child);
                return;
            }
        }
    }
}

impl<T> QuadTree<T>
where
    T: Copy + Default,
{
    pub fn new(nodes: Vec<QuadTreeNode<T>>) -> QuadTree<T> {
        QuadTree { nodes }
    }

    pub fn root(&self) -> Option<QuadTreeNode<T>> {
        self.nodes.first().cloned()
    }
}

/// State of a tile to be inserted into a quad tree
#[derive(Debug)]
struct TileState {
    parent: Option<usize>,
    level: usize,
    tile: Tile,
}

// Round to the nearest power of two
fn ceil_pow2(num: usize) -> usize {
    let num = num as f64;
    let exp = (num.log2() / 2.0_f64.log2()).ceil();
    2.0_f64.powf(exp) as usize
}

// Return the min and max elevation values for a section of the texture
fn extents(texture: &Texture<f64>, tile: Tile) -> (f64, f64) {
    let mut max = -INFINITY;
    let mut min = INFINITY;

    if tile.x > texture.width || tile.y > texture.height {
        return (min, max);
    }

    let width = cmp::min(tile.width, texture.width - tile.x);
    let height = cmp::min(tile.height, texture.height - tile.y);
    let data = texture.lookup(tile.x, tile.y, width, height);

    for value in data {
        if value > max {
            max = value;
        }
        if value < min {
            min = value;
        }
    }

    (
        if min == INFINITY { 0.0 } else { min },
        if max == -INFINITY { 0.0 } else { max },
    )
}

/// Build a quad tree from a height map
///
/// Stores the min and max elevations for the height map
pub fn height_map_quad_tree(texture: &Texture<f64>) -> QuadTree<Aabb> {
    let width = texture.width;
    let height = texture.height;

    let mut nodes = vec![];
    let mut visit = VecDeque::new();

    visit.push_back(TileState {
        parent: None,
        level: 0,
        tile: Tile::new(0, 0, ceil_pow2(width), ceil_pow2(height)),
    });

    let mut max_level = 0;

    while let Some(work) = visit.pop_front() {
        if work.level > max_level {
            max_level = work.level;
        }

        let (mut min_height, max_height) = extents(&texture, work.tile);

        let min = Vec3::new(work.tile.x as f64, min_height, work.tile.y as f64);
        let max = Vec3::new(
            (work.tile.x + work.tile.width) as f64,
            max_height,
            (work.tile.y + work.tile.height) as f64,
        );

        let node_index = nodes.len();
        nodes.push(QuadTreeNode::new(Aabb::new(min, max)));

        if let Some(pidx) = work.parent {
            nodes[pidx].append(node_index);
        }

        if work.tile.width >= 4 && (max_height - min_height).abs() > EPSILON {
            let l = work.level + 1;
            let x = work.tile.x;
            let y = work.tile.y;
            let w = work.tile.width / 2;
            let h = work.tile.height / 2;

            visit.push_back(TileState {
                parent: Some(node_index),
                level: l,
                tile: Tile::new(x, y, w, h),
            });
            visit.push_back(TileState {
                parent: Some(node_index),
                level: l,
                tile: Tile::new(x + w, y, w, h),
            });
            visit.push_back(TileState {
                parent: Some(node_index),
                level: l,
                tile: Tile::new(x + w, y + h, w, h),
            });
            visit.push_back(TileState {
                parent: Some(node_index),
                level: l,
                tile: Tile::new(x, y + h, w, h),
            });
        }
    }

    QuadTree::new(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_tree_append_node() {
        let mut node = QuadTreeNode::new("foo");
        node.append(10);
        node.append(20);
        assert_eq!(node.children, [Some(10), Some(20), None, None]);
    }
}
