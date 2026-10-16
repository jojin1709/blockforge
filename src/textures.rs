use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::blocks::*;
use crate::inventory::{ItemId, APPLE, IRON_INGOT, STICK};
use crate::items;

pub const TILE: usize = 16;
pub const ATLAS_COLS: usize = 8;
pub const ATLAS_ROWS: usize = 4;

// Tile indices in the 8x4 atlas.
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
pub const T_STICK: usize = 14;
pub const T_APPLE: usize = 15;
pub const T_SANDSTONE: usize = 16;
pub const T_GRAVEL: usize = 17;
pub const T_SNOW: usize = 18;
pub const T_CRAFT_TOP: usize = 19;
pub const T_CRAFT_SIDE: usize = 20;
pub const T_IRON_INGOT: usize = 21;
pub const T_WOOD_PICK: usize = 22;
pub const T_WOOD_AXE: usize = 23;
pub const T_WOOD_SHOVEL: usize = 24;
pub const T_STONE_PICK: usize = 25;
pub const T_STONE_AXE: usize = 26;
pub const T_STONE_SHOVEL: usize = 27;
pub const T_IRON_PICK: usize = 28;
pub const T_IRON_AXE: usize = 29;
pub const T_IRON_SHOVEL: usize = 30;

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

pub fn build_atlas_data() -> Vec<u8> {
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

    // Stick: brown vertical stick with transparent background.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_STICK,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let in_stick = x >= 7 && x <= 8 && y >= 3 && y <= 12;
            let in_guard = y == 3 && (6..=9).contains(&x);
            if in_stick || in_guard {
                let n = hash(x, y, 20) * 0.06 - 0.03;
                tw.set(x, y, 0.55 + n, 0.40 + n, 0.22 + n, 1.0);
            } else {
                tw.set(x, y, 0.0, 0.0, 0.0, 0.0);
            }
        }
    }

    // Apple: red circle with stem, transparent background.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_APPLE,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let cx = x as f32 - 7.5;
            let cy = y as f32 - 8.5;
            let r = (cx * cx + cy * cy).sqrt();
            if r < 4.5 {
                let n = hash(x, y, 21) * 0.06;
                let shade = 1.0 - ((cy + 4.5) / 9.0) * 0.3;
                tw.set(x, y, (0.78 + n) * shade, (0.10 + n) * shade, 0.08 * shade, 1.0);
            } else if (2..=4).contains(&y) && (7..=8).contains(&x) {
                tw.set(x, y, 0.20, 0.50, 0.15, 1.0);
            } else {
                tw.set(x, y, 0.0, 0.0, 0.0, 0.0);
            }
        }
    }

    // Sandstone: banded sand.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_SANDSTONE,
    };
    for y in 0..TILE {
        let band = if y % 5 == 0 { -0.06 } else { 0.0 };
        for x in 0..TILE {
            let n = hash(x, y, 30) * 0.05 - 0.025;
            let base = 0.82 + band + n;
            tw.set(x, y, base, base * 0.94, base * 0.70, 1.0);
        }
    }

    // Gravel: mixed grey speckles.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_GRAVEL,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let n = hash(x, y, 31);
            let v = 0.42 + n * 0.22;
            let tint = if n > 0.7 { 0.05 } else { 0.0 };
            tw.set(x, y, v + tint, v, v * 0.95, 1.0);
        }
    }

    // Snow: near-white with faint blue noise.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_SNOW,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let n = hash(x, y, 32) * 0.05;
            tw.set(x, y, 0.93 + n * 0.3, 0.94 + n * 0.3, 0.98 + n * 0.2, 1.0);
        }
    }

    // Crafting table top: planks + grid lines.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_CRAFT_TOP,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let n = hash(x, y, 33) * 0.05 - 0.025;
            let grid = x % 5 == 0 || y % 5 == 0;
            let base = if grid { 0.35 } else { 0.68 + n };
            tw.set(x, y, base, base * 0.76, base * 0.46, 1.0);
        }
    }

    // Crafting table side: planks with a dark tool-band across the middle.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_CRAFT_SIDE,
    };
    for y in 0..TILE {
        let seam = y % 4 == 3;
        let band = (5..=10).contains(&y);
        for x in 0..TILE {
            let n = hash(x, y, 34) * 0.05 - 0.025;
            if band && (x + y) % 3 == 0 {
                tw.set(x, y, 0.30, 0.22, 0.12, 1.0);
            } else {
                let base = if seam { 0.38 } else { 0.62 + n };
                tw.set(x, y, base, base * 0.76, base * 0.46, 1.0);
            }
        }
    }

    // Iron ingot: grey bar shape, transparent bg.
    let mut tw = TileWriter {
        data: &mut data,
        tile: T_IRON_INGOT,
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let in_bar = (4..=11).contains(&y) && (3..=12).contains(&x) && !(y >= 9 && (x < 4 || x > 11));
            let top_hi = y == 4 || y == 5;
            if in_bar {
                let n = hash(x, y, 35) * 0.06;
                let v = if top_hi { 0.90 + n } else { 0.72 + n };
                tw.set(x, y, v, v, v * 1.04, 1.0);
            } else {
                tw.set(x, y, 0.0, 0.0, 0.0, 0.0);
            }
        }
    }

    // Tool icons: diagonal stick + head. Drawn per kind and material color.
    let mut tool = |tile: usize, kind: u8, col: [f32; 3]| {
        let mut tw = TileWriter {
            data: &mut data,
            tile,
        };
        for y in 0..TILE {
            for x in 0..TILE {
                let mut on = false;
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;
                // Handle: diagonal from bottom-left toward center.
                let hx = 4 + (11 - y as i32) / 2;
                if y >= 4 && (x as i32 == hx || x as i32 == hx + 1) && y <= 13 {
                    on = true;
                    let n = hash(x, y, 40) * 0.06 - 0.03;
                    r = 0.55 + n;
                    g = 0.40 + n;
                    b = 0.22 + n;
                }
                // Heads.
                match kind {
                    0 => {
                        // Pickaxe: wide arc along the top.
                        if (2..=4).contains(&y) && (3..=12).contains(&x) {
                            on = true;
                        } else if y == 5 && ((3..=5).contains(&x) || (10..=12).contains(&x)) {
                            on = true;
                        }
                    }
                    1 => {
                        // Axe: blob on the top-right.
                        if (2..=7).contains(&y) && (8..=13).contains(&x) {
                            if !(y == 7 && x == 13) && !(y == 2 && x == 8) {
                                on = true;
                            }
                        }
                    }
                    _ => {
                        // Shovel: square blade at the top.
                        if (2..=6).contains(&y) && (5..=10).contains(&x) {
                            if !(y == 6 && (x == 5 || x == 10)) {
                                on = true;
                            }
                        }
                    }
                }
                if on && !(4..=13).contains(&y) {
                    let n = hash(x, y, 41) * 0.05;
                    r = col[0] + n;
                    g = col[1] + n;
                    b = col[2] + n;
                }
                if on {
                    tw.set(x, y, r, g, b, 1.0);
                } else {
                    tw.set(x, y, 0.0, 0.0, 0.0, 0.0);
                }
            }
        }
    };
    let wood = [0.72, 0.56, 0.32];
    let stone = [0.58, 0.58, 0.60];
    let iron = [0.85, 0.85, 0.90];
    tool(T_WOOD_PICK, 0, wood);
    tool(T_WOOD_AXE, 1, wood);
    tool(T_WOOD_SHOVEL, 2, wood);
    tool(T_STONE_PICK, 0, stone);
    tool(T_STONE_AXE, 1, stone);
    tool(T_STONE_SHOVEL, 2, stone);
    tool(T_IRON_PICK, 0, iron);
    tool(T_IRON_AXE, 1, iron);
    tool(T_IRON_SHOVEL, 2, iron);

    data
}

pub fn build_atlas_image() -> Image {
    let w = ATLAS_COLS * TILE;
    let h = ATLAS_ROWS * TILE;
    let data = build_atlas_data();
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

/// Extract a single 16x16 tile from atlas pixel data into its own Image.
pub fn extract_tile_image(atlas_data: &[u8], tile: usize) -> Image {
    let mut data = vec![0u8; TILE * TILE * 4];
    let tx = (tile % ATLAS_COLS) * TILE;
    let ty = (tile / ATLAS_COLS) * TILE;
    for y in 0..TILE {
        for x in 0..TILE {
            let src = ((ty + y) * ATLAS_COLS * TILE + tx + x) * 4;
            let dst = (y * TILE + x) * 4;
            data[dst..dst + 4].copy_from_slice(&atlas_data[src..src + 4]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: TILE as u32,
            height: TILE as u32,
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

/// Atlas tile used for the icon / held-item face of an item.
pub fn item_tile(id: ItemId) -> usize {
    match id {
        STICK => T_STICK,
        APPLE => T_APPLE,
        IRON_INGOT => T_IRON_INGOT,
        items::WOOD_PICK => T_WOOD_PICK,
        items::WOOD_AXE => T_WOOD_AXE,
        items::WOOD_SHOVEL => T_WOOD_SHOVEL,
        items::STONE_PICK => T_STONE_PICK,
        items::STONE_AXE => T_STONE_AXE,
        items::STONE_SHOVEL => T_STONE_SHOVEL,
        items::IRON_PICK => T_IRON_PICK,
        items::IRON_AXE => T_IRON_AXE,
        items::IRON_SHOVEL => T_IRON_SHOVEL,
        _ => block_tiles(id).1,
    }
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
        SANDSTONE => (T_SANDSTONE, T_SANDSTONE, T_SANDSTONE),
        GRAVEL => (T_GRAVEL, T_GRAVEL, T_GRAVEL),
        SNOW => (T_SNOW, T_SNOW, T_SNOW),
        CRAFTING_TABLE => (T_CRAFT_TOP, T_CRAFT_SIDE, T_PLANKS),
        _ => (T_STONE, T_STONE, T_STONE),
    }
}

/// Generates a 16x16 heart-shaped RGBA image. `fill` 0.0=empty, 0.5=half, 1.0=full.
/// fill fills left-to-right so half hearts clip naturally.
pub fn build_heart_image(fill: f32) -> Image {
    let s = TILE as f32;
    let mut data = vec![0u8; TILE * TILE * 4];
    for y in 0..TILE {
        for x in 0..TILE {
            // Normalise to -1..1, flip Y so +y is up.
            let nx = (x as f32 + 0.5) / s * 2.0 - 1.0;
            let ny = 1.0 - (y as f32 + 0.5) / s * 2.0;
            // Implicit heart curve: (x² + y² − 1)³ − x² y³ ≤ 0
            let x2 = nx * nx;
            let y2 = ny * ny;
            let val = (x2 + y2 - 1.0).powi(3) - x2 * ny * ny2_val(ny);
            let inside = val <= 0.0;
            let i = (y * TILE + x) * 4;
            if inside {
                // Fill left-to-right by `fill` fraction.
                let px = x as f32 / s;
                if px <= fill {
                    // Full red heart interior with slight shading.
                    let shade = 1.0 - (ny * 0.15);
                    data[i] = (200.0 * shade) as u8;
                    data[i + 1] = (30.0 * shade) as u8;
                    data[i + 2] = (30.0 * shade) as u8;
                    data[i + 3] = 255;
                } else {
                    // Dark "empty" portion still shows the outline shape.
                    data[i] = 50;
                    data[i + 1] = 20;
                    data[i + 2] = 20;
                    data[i + 3] = 200;
                }
            } else {
                data[i + 3] = 0;
            }
        }
    }
    let mut image = Image::new(
        Extent3d { width: TILE as u32, height: TILE as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
    image
}

// Helper for the heart equation term y³.
#[inline]
fn ny2_val(ny: f32) -> f32 {
    ny * ny
}

/// Generates a 16x16 drumstick-shaped RGBA image for the food bar. `fill` 0..1 left-to-right.
pub fn build_food_image(fill: f32) -> Image {
    let s = TILE as f32;
    let mut data = vec![0u8; TILE * TILE * 4];
    for y in 0..TILE {
        for x in 0..TILE {
            let nx = (x as f32 + 0.5) / s * 2.0 - 1.0;
            let ny = 1.0 - (y as f32 + 0.5) / s * 2.0;
            // Drumstick: meat blob on top-right, bone stick on bottom-left.
            let meat = ((nx - 0.2).powi(2) + (ny - 0.2).powi(2)) < 0.35;
            let bone = (nx.abs() + (ny + 0.4).abs()) < 0.55 && ny < 0.1;
            let inside = meat || bone;
            let i = (y * TILE + x) * 4;
            if inside {
                let px = x as f32 / s;
                if px <= fill {
                    if meat {
                        data[i] = 180;
                        data[i + 1] = 100;
                        data[i + 2] = 40;
                    } else {
                        data[i] = 220;
                        data[i + 1] = 210;
                        data[i + 2] = 180;
                    }
                    data[i + 3] = 255;
                } else {
                    data[i] = 50;
                    data[i + 1] = 40;
                    data[i + 2] = 25;
                    data[i + 3] = 200;
                }
            } else {
                data[i + 3] = 0;
            }
        }
    }
    let mut image = Image::new(
        Extent3d { width: TILE as u32, height: TILE as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
    image
}

pub const CRACK_STAGES: usize = 10;

/// Generates 10 crack-overlay textures (one per break stage).
/// Each is a 16x16 RGBA image: transparent where intact, dark cracks where broken.
pub fn build_crack_images() -> Vec<Image> {
    let mut images = Vec::with_capacity(CRACK_STAGES);
    for stage in 0..CRACK_STAGES {
        let progress = (stage as f32 + 1.0) / CRACK_STAGES as f32;
        let mut data = vec![0u8; TILE * TILE * 4];
        for y in 0..TILE {
            for x in 0..TILE {
                // Crack intensity: combination of radial lines from center + noise.
                let cx = x as f32 - 7.5;
                let cy = y as f32 - 7.5;
                let angle = cy.atan2(cx);
                let dist = (cx * cx + cy * cy).sqrt() / 11.0; // 0..~1

                // Radial crack lines that grow with progress.
                let line_freq = 5.0;
                let crack_line = ((angle * line_freq).sin().abs()).powf(8.0);
                // Only show cracks beyond a certain radius threshold that shrinks as progress grows.
                let threshold = 1.0 - progress;
                let radial = if dist > threshold { crack_line * progress } else { 0.0 };

                // Random noise cracks.
                let n = hash(x, y, 90 + stage);
                let noise_crack = if n > (1.0 - progress * 0.6) { progress * 0.7 } else { 0.0 };

                let crack = (radial + noise_crack).min(1.0);
                let i = (y * TILE + x) * 4;
                // Dark crack color, alpha = crack intensity.
                let v = (crack * 200.0) as u8;
                data[i] = v / 3;
                data[i + 1] = v / 3;
                data[i + 2] = v / 3;
                data[i + 3] = v;
            }
        }
        let mut image = Image::new(
            Extent3d { width: TILE as u32, height: TILE as u32, depth_or_array_layers: 1 },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            Default::default(),
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
        images.push(image);
    }
    images
}
