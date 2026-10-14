use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::blocks::*;

pub const TILE: usize = 16;
pub const ATLAS_COLS: usize = 4;
pub const ATLAS_ROWS: usize = 4;

// Tile indices in the 4x4 atlas.
pub const T_GRASS_TOP: usize = 0;
pub const T_GRASS_SIDE: usize = 1;
pub const T_DIRT: usize = 2;
pub const T_STONE: usize = 3;
pub const T_SAND: usize = 4;
pub const T_LOG_SIDE: usize = 5;
pub const T_LOG_TOP: usize = 6;
pub const T_LEAVES: usize = 7;
pub const T_BEDROCK: usize = 8;
pub const T_COAL: usize = 9;
pub const T_IRON: usize = 10;
pub const T_PLANKS: usize = 11;
pub const T_COBBLE: usize = 12;
pub const T_WATER: usize = 13;

fn hash(px: usize, py: usize, salt: usize) -> f32 {
    let mut h = (px as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (py as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (salt as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h = h.wrapping_mul(h.wrapping_mul(h >> 30) | 1);
    ((h >> 40) & 0xFF) as f32 / 255.0
}

struct TileWriter<'a> {
    data: &'a mut [u8],
    tile: usize,
}

impl TileWriter<'_> {
    fn set(&mut self, x: usize, y: usize, r: f32, g: f32, b: f32, a: f32) {
        let tx = (self.tile % ATLAS_COLS) * TILE + x;
        let ty = (self.tile / ATLAS_COLS) * TILE + y;
        let i = (ty * ATLAS_COLS * TILE + tx) * 4;
        self.data[i] = (r.clamp(0.0, 1.0) * 255.0) as u8;
        self.data[i + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
        self.data[i + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        self.data[i + 3] = (a.clamp(0.0, 1.0) * 255.0) as u8;
    }
}

fn fill_noise(
    tw: &mut TileWriter,
    base: [f32; 3],
    variance: f32,
    salt: usize,
    alpha: f32,
) {
    for y in 0..TILE {
        for x in 0..TILE {
            let n = hash(x, y, salt) * variance - variance * 0.5;
            tw.set(
                x,
                y,
                base[0] + n,
                base[1] + n * 0.8,
                base[2] + n * 0.6,
                alpha,
            );
        }
    }
}

pub fn build_atlas_image() -> Image {
    let w = ATLAS_COLS * TILE;
    let h = ATLAS_ROWS * TILE;
    let mut data = vec![0u8; w * h * 4];

    // Grass top: green noise.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_GRASS_TOP,
    };
    fill_noise(&mut tw, [0.30, 0.55, 0.20], 0.10, 1, 1.0);

    // Dirt base for grass side lower part, drawn first.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_GRASS_SIDE,
    };
    fill_noise(&mut tw, [0.42, 0.33, 0.22], 0.08, 2, 1.0);
    // Green fringe on top ~2-5px with ragged per-column edge.
    for x in 0..TILE {
        let depth = 2 + (hash(x, 0, 88) * 3.5) as usize;
        for y in 0..depth.min(TILE) {
            let n = hash(x, y, 3) * 0.08 - 0.04;
            tw.set(x, y, 0.30 + n, 0.55 + n, 0.20 + n * 0.5, 1.0);
        }
    }

    let mut tw = TileWriter {
        data: &mut data,
        tile: T_DIRT,
    };
    fill_noise(&mut tw, [0.42, 0.33, 0.22], 0.09, 4, 1.0);

    let mut tw = TileWriter {
        data: &mut data,
        tile: T_STONE,
    };
    fill_noise(&mut tw, [0.50, 0.50, 0.50], 0.08, 5, 1.0);
    // Subtle darker blotches.
    for y in 0..TILE {
        for x in 0..TILE {
            if hash(x, y, 50) > 0.85 {
                let n = hash(x, y, 51) * 0.06;
                tw.set(x, y, 0.42 - n, 0.42 - n, 0.43 - n, 1.0);
            }
        }
    }

    let mut tw = TileWriter {
        data: &mut data,
        tile: T_SAND,
    };
    fill_noise(&mut tw, [0.86, 0.80, 0.58], 0.06, 6, 1.0);

    // Log side: vertical bark stripes.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_LOG_SIDE,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let stripe = if x % 4 == 0 || x % 7 == 0 { -0.07 } else { 0.0 };
            let n = hash(x, y, 7) * 0.07 - 0.035;
            let base = 0.38 + stripe + n;
            tw.set(x, y, base, base * 0.78, base * 0.50, 1.0);
        }
    }

    // Log top: rings.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_LOG_TOP,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let cx = x as f32 - 7.5;
            let cy = y as f32 - 7.5;
            let r = (cx * cx + cy * cy).sqrt();
            let ring = ((r * 1.4).sin() * 0.5 + 0.5) * 0.10;
            let n = hash(x, y, 8) * 0.05 - 0.025;
            let base = 0.52 - ring + n;
            tw.set(x, y, base, base * 0.76, base * 0.48, 1.0);
        }
    }

    // Leaves: green with darker holes.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_LEAVES,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let n = hash(x, y, 9);
            if n > 0.82 {
                tw.set(x, y, 0.08, 0.16, 0.05, 1.0);
            } else {
                let v = 0.18 + n * 0.12;
                tw.set(x, y, v * 0.9, v * 1.8, v * 0.55, 1.0);
            }
        }
    }

    let mut tw = TileWriter {
        data: &mut data,
        tile: T_BEDROCK,
    };
    fill_noise(&mut tw, [0.16, 0.16, 0.17], 0.10, 10, 1.0);

    // Coal ore: stone + black blobs.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_COAL,
    };
    fill_noise(&mut tw, [0.48, 0.48, 0.49], 0.07, 11, 1.0);
    for y in 0..TILE {
        for x in 0..TILE {
            if hash(x, y, 60) > 0.80 {
                tw.set(x, y, 0.10, 0.10, 0.11, 1.0);
            }
        }
    }

    // Iron ore: stone + tan blobs.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_IRON,
    };
    fill_noise(&mut tw, [0.48, 0.48, 0.49], 0.07, 12, 1.0);
    for y in 0..TILE {
        for x in 0..TILE {
            if hash(x, y, 61) > 0.78 {
                tw.set(x, y, 0.72, 0.58, 0.48, 1.0);
            }
        }
    }

    // Planks: horizontal boards with seams.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_PLANKS,
    };
    for y in 0..TILE {
        let seam = y % 4 == 3;
        for x in 0..TILE {
            let n = hash(x, y, 13) * 0.06 - 0.03;
            let base = if seam { 0.38 } else { 0.66 + n };
            let stagger = if !seam && (y / 4) % 2 == 1 && x == 8 { 0.38 } else { base };
            tw.set(x, y, stagger, stagger * 0.78, stagger * 0.48, 1.0);
        }
    }

    // Cobble: irregular stones with mortar.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_COBBLE,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let cell_x = x / 5;
            let cell_y = y / 5;
            let mortar = x % 5 == 4 || y % 5 == 4;
            let n = hash(x, y, 14) * 0.10 - 0.05;
            let base = if mortar {
                0.30
            } else {
                0.48 + hash(cell_x, cell_y, 15) * 0.12 + n
            };
            tw.set(x, y, base, base, base * 1.02, 1.0);
        }
    }

    // Water: blue with light ripple lines.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_WATER,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let wave = ((x as f32 * 0.8 + y as f32 * 0.3).sin() * 0.5 + 0.5) * 0.08;
            let n = hash(x, y, 16) * 0.04;
            tw.set(x, y, 0.18 + wave + n, 0.38 + wave, 0.78 + wave * 0.5, 1.0);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
    image
}

pub fn tile_uv(tile: usize) -> ([f32; 2], [f32; 2]) {
    let cols = ATLAS_COLS as f32;
    let rows = ATLAS_ROWS as f32;
    let tx = (tile % ATLAS_COLS) as f32;
    let ty = (tile / ATLAS_COLS) as f32;
    // Small inset to avoid bleeding between tiles.
    let inset = 0.02 / cols;
    let u0 = (tx + inset) / cols;
    let u1 = (tx + 1.0 - inset) / cols;
    // wgpu UV v=0 is image top. Face top must sample tile image-top (small v).
    let v_top = (ty + inset) / rows;
    let v_bot = (ty + 1.0 - inset) / rows;
    ([u0, v_bot], [u1, v_top])
}

pub fn block_tiles(id: BlockId) -> (usize, usize, usize) {
    // (top, side, bottom)
    match id {
        GRASS => (T_GRASS_TOP, T_GRASS_SIDE, T_DIRT),
        DIRT => (T_DIRT, T_DIRT, T_DIRT),
        STONE => (T_STONE, T_STONE, T_STONE),
        SAND => (T_SAND, T_SAND, T_SAND),
        WATER => (T_WATER, T_WATER, T_WATER),
        LOG => (T_LOG_TOP, T_LOG_SIDE, T_LOG_TOP),
        LEAVES => (T_LEAVES, T_LEAVES, T_LEAVES),
        BEDROCK => (T_BEDROCK, T_BEDROCK, T_BEDROCK),
        COAL_ORE => (T_COAL, T_COAL, T_COAL),
        IRON_ORE => (T_IRON, T_IRON, T_IRON),
        PLANKS => (T_PLANKS, T_PLANKS, T_PLANKS),
        COBBLE => (T_COBBLE, T_COBBLE, T_COBBLE),
        _ => (T_STONE, T_STONE, T_STONE),
    }
}
