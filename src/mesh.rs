use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;

use crate::blocks::*;
use crate::textures::{block_tiles, tile_uv};

pub const PAD: usize = 18;
pub const PAD_BLOCKS: usize = PAD * PAD * PAD;

#[derive(Clone, Copy, PartialEq)]
enum Part {
    Top,
    Bottom,
    SideX,
    SideZ,
}

const FACES: [([i32; 3], [[f32; 3]; 4], Part); 6] = [
    (
        [1, 0, 0],
        [[1., 0., 1.], [1., 0., 0.], [1., 1., 0.], [1., 1., 1.]],
        Part::SideX,
    ),
    (
        [-1, 0, 0],
        [[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]],
        Part::SideX,
    ),
    (
        [0, 1, 0],
        [[0., 1., 1.], [1., 1., 1.], [1., 1., 0.], [0., 1., 0.]],
        Part::Top,
    ),
    (
        [0, -1, 0],
        [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]],
        Part::Bottom,
    ),
    (
        [0, 0, 1],
        [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
        Part::SideZ,
    ),
    (
        [0, 0, -1],
        [[1., 0., 0.], [0., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        Part::SideZ,
    ),
];

fn shade(part: Part) -> f32 {
    match part {
        Part::Top => 1.0,
        Part::Bottom => 0.55,
        Part::SideX => 0.75,
        Part::SideZ => 0.85,
    }
}

fn tile_for(part: Part, tiles: (usize, usize, usize)) -> usize {
    match part {
        Part::Top => tiles.0,
        Part::Bottom => tiles.2,
        _ => tiles.1,
    }
}

#[derive(Default)]
struct Buf {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Buf {
    fn push_quad(
        &mut self,
        base: [f32; 3],
        corners: [[f32; 3]; 4],
        normal: [f32; 3],
        s: f32,
        tint: [f32; 3],
        tile: usize,
    ) {
        let start = self.positions.len() as u32;
        let (uv0, uv1) = tile_uv(tile);
        let quad_uv = [
            [uv0[0], uv0[1]],
            [uv1[0], uv0[1]],
            [uv1[0], uv1[1]],
            [uv0[0], uv1[1]],
        ];
        for (i, c) in corners.iter().enumerate() {
            self.positions
                .push([base[0] + c[0], base[1] + c[1], base[2] + c[2]]);
            self.normals.push(normal);
            self.uvs.push(quad_uv[i]);
            // Tint multiplies the texture; keep near-white so texture shows through.
            self.colors
                .push([tint[0] * s, tint[1] * s, tint[2] * s, 1.0]);
        }
        self.indices
            .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
    }

    fn into_mesh(self) -> Option<Mesh> {
        if self.indices.is_empty() {
            return None;
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        Some(mesh)
    }
}

fn pidx(x: usize, y: usize, z: usize) -> usize {
    (y * PAD + z) * PAD + x
}

pub fn build_meshes(padded: &[BlockId; PAD_BLOCKS]) -> (Option<Mesh>, Option<Mesh>) {
    let mut solid = Buf::default();
    let mut water = Buf::default();

    for y in 1..=16 {
        for z in 1..=16 {
            for x in 1..=16 {
                let id = padded[pidx(x, y, z)];
                if id == AIR {
                    continue;
                }
                let is_water = id == WATER;
                let tiles = block_tiles(id);
                // Slight warm tint so textures aren't flat; texture provides detail.
                let tint = if is_water {
                    [1.0, 1.0, 1.0]
                } else {
                    [1.0, 1.0, 1.0]
                };
                for (dir, corners, part) in FACES.iter() {
                    let nx = x as i32 + dir[0];
                    let ny = y as i32 + dir[1];
                    let nz = z as i32 + dir[2];
                    let nb = padded[pidx(nx as usize, ny as usize, nz as usize)];
                    let draw = if is_water {
                        nb == AIR
                    } else {
                        nb == AIR || nb == WATER
                    };
                    if !draw {
                        continue;
                    }
                    let normal = [dir[0] as f32, dir[1] as f32, dir[2] as f32];
                    let base = [(x - 1) as f32, (y - 1) as f32, (z - 1) as f32];
                    let tile = tile_for(*part, tiles);
                    if is_water {
                        let mut wcorners = *corners;
                        if part == &Part::Top {
                            for c in wcorners.iter_mut() {
                                c[1] -= 0.02;
                            }
                        }
                        water.push_quad(base, wcorners, normal, shade(*part), tint, tile);
                    } else {
                        solid.push_quad(base, *corners, normal, shade(*part), tint, tile);
                    }
                }
            }
        }
    }

    (solid.into_mesh(), water.into_mesh())
}
