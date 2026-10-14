use noise::{Fbm, NoiseFn, Perlin};

use crate::blocks::*;

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_BLOCKS: usize = 16 * 16 * 16;
pub const WORLD_HEIGHT: i32 = 96;
pub const Y_CHUNKS: usize = (WORLD_HEIGHT / CHUNK_SIZE) as usize;
pub const SEA_LEVEL: i32 = 33;

pub type ColumnBlocks = [[BlockId; CHUNK_BLOCKS]; Y_CHUNKS];

pub fn hash2(seed: u64, x: i32, z: i32) -> u64 {
    let mut h = seed
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (z as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h = h.wrapping_add(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^ (h >> 31)
}

pub fn hash3(seed: u64, x: i32, y: i32, z: i32) -> u64 {
    let mut h = hash2(seed, x, z)
        ^ (y as u64)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .rotate_left(17);
    h ^= h >> 31;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^ (h >> 31)
}

struct Noises {
    continent: Fbm<Perlin>,
    detail: Fbm<Perlin>,
    cave_a: Perlin,
    cave_b: Perlin,
}

impl Noises {
    fn new(seed: u64) -> Self {
        let s = seed as u32;
        let mut continent = Fbm::<Perlin>::new(s);
        continent.octaves = 4;
        continent.frequency = 0.0055;
        let mut detail = Fbm::<Perlin>::new(s.wrapping_add(1));
        detail.octaves = 3;
        detail.frequency = 0.03;
        Noises {
            continent,
            detail,
            cave_a: Perlin::new(s.wrapping_add(2)),
            cave_b: Perlin::new(s.wrapping_add(3)),
        }
    }

    fn height_at(&self, x: i32, z: i32) -> i32 {
        let c = self.continent.get([x as f64, z as f64]);
        let d = self.detail.get([x as f64, z as f64]);
        (36.0 + c * 16.0 + d * 4.0)
            .round()
            .clamp(4.0, (WORLD_HEIGHT - 12) as f64) as i32
    }

    fn is_cave(&self, x: i32, y: i32, z: i32) -> bool {
        let p = [x as f64 * 0.07, y as f64 * 0.11, z as f64 * 0.07];
        if self.cave_a.get(p).abs() >= 0.075 {
            return false;
        }
        self.cave_b.get(p).abs() < 0.075
    }
}

pub fn surface_height(seed: u64, x: i32, z: i32) -> i32 {
    Noises::new(seed).height_at(x, z)
}

fn idx(lx: i32, ly: i32, lz: i32) -> usize {
    ((ly * 16 + lz) * 16 + lx) as usize
}

fn get(col: &ColumnBlocks, wy: i32, lx: i32, lz: i32) -> BlockId {
    col[(wy / 16) as usize][idx(lx, wy % 16, lz)]
}

fn set(col: &mut ColumnBlocks, wy: i32, lx: i32, lz: i32, id: BlockId) {
    col[(wy / 16) as usize][idx(lx, wy % 16, lz)] = id;
}

pub fn gen_column(cx: i32, cz: i32, seed: u64) -> ColumnBlocks {
    let n = Noises::new(seed);
    let mut heights = std::collections::HashMap::<(i32, i32), i32>::new();
    let mut col = [[AIR; CHUNK_BLOCKS]; Y_CHUNKS];

    let base_x = cx * CHUNK_SIZE;
    let base_z = cz * CHUNK_SIZE;

    let mut height = |x: i32, z: i32| -> i32 {
        if let Some(&h) = heights.get(&(x, z)) {
            return h;
        }
        let h = n.height_at(x, z);
        heights.insert((x, z), h);
        h
    };

    for lz in 0..CHUNK_SIZE {
        for lx in 0..CHUNK_SIZE {
            let wx = base_x + lx;
            let wz = base_z + lz;
            let h = height(wx, wz);
            let beach = h <= SEA_LEVEL + 1;
            for wy in 0..WORLD_HEIGHT {
                let id = if wy == 0 {
                    BEDROCK
                } else if wy <= h {
                    if wy == h {
                        if beach { SAND } else { GRASS }
                    } else if wy >= h - 3 {
                        if beach { SAND } else { DIRT }
                    } else {
                        STONE
                    }
                } else if wy <= SEA_LEVEL {
                    WATER
                } else {
                    AIR
                };
                set(&mut col, wy, lx, lz, id);
            }
        }
    }

    for lz in 0..CHUNK_SIZE {
        for lx in 0..CHUNK_SIZE {
            let wx = base_x + lx;
            let wz = base_z + lz;
            let h = height(wx, wz);
            for wy in 1..=h {
                let id = get(&col, wy, lx, lz);
                if id != STONE && id != DIRT && id != SAND {
                    continue;
                }
                if wy >= 5 && n.is_cave(wx, wy, wz) {
                    set(&mut col, wy, lx, lz, AIR);
                    continue;
                }
                if id == STONE {
                    let hh = hash3(seed, wx, wy, wz);
                    if wy < 76 && hh % 4096 < 48 {
                        set(&mut col, wy, lx, lz, COAL_ORE);
                    } else if wy < 40 && (hh >> 17) % 4096 < 30 {
                        set(&mut col, wy, lx, lz, IRON_ORE);
                    }
                }
            }
        }
    }

    for tz in (base_z - 2)..=(base_z + CHUNK_SIZE + 1) {
        for tx in (base_x - 2)..=(base_x + CHUNK_SIZE + 1) {
            let th = hash2(seed ^ 0x5EED, tx, tz);
            if th % 24 != 0 {
                continue;
            }
            let h = height(tx, tz);
            if h <= SEA_LEVEL + 1 || h > 78 {
                continue;
            }
            let trunk_h = 4 + ((th >> 13) % 3) as i32;
            let top = h + trunk_h;

            for wy in (top - 2)..=(top + 1) {
                for dz in -2i32..=2 {
                    for dx in -2i32..=2 {
                        let r = dx.abs().max(dz.abs());
                        let corner = r == 2 && dx.abs() == 2 && dz.abs() == 2;
                        let ok = if wy == top + 1 {
                            r <= 1 && !(dx.abs() == 1 && dz.abs() == 1)
                        } else {
                            r <= 2 && !corner
                        };
                        if !ok {
                            continue;
                        }
                        place(&mut col, base_x, base_z, tx + dx, wy, tz + dz, LEAVES, false);
                    }
                }
            }
            for wy in (h + 1)..=top {
                place(&mut col, base_x, base_z, tx, wy, tz, LOG, true);
            }
        }
    }

    col
}

fn place(
    col: &mut ColumnBlocks,
    base_x: i32,
    base_z: i32,
    wx: i32,
    wy: i32,
    wz: i32,
    id: BlockId,
    overwrite_leaves: bool,
) {
    let lx = wx - base_x;
    let lz = wz - base_z;
    if !(0..CHUNK_SIZE).contains(&lx) || !(0..CHUNK_SIZE).contains(&lz) {
        return;
    }
    if wy < 1 || wy >= WORLD_HEIGHT {
        return;
    }
    let cur = get(col, wy, lx, lz);
    let ok = if overwrite_leaves {
        cur == AIR || cur == LEAVES
    } else {
        cur == AIR
    };
    if ok {
        set(col, wy, lx, lz, id);
    }
}
