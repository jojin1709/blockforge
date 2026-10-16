use crate::inventory::{ItemId, Slot, APPLE, IRON_INGOT, STICK};
use crate::blocks::{COBBLE, CRAFTING_TABLE, LOG, PLANKS};

pub const WOOD_PICK: ItemId = 110;
pub const WOOD_AXE: ItemId = 111;
pub const WOOD_SHOVEL: ItemId = 112;
pub const STONE_PICK: ItemId = 120;
pub const STONE_AXE: ItemId = 121;
pub const STONE_SHOVEL: ItemId = 122;
pub const IRON_PICK: ItemId = 130;
pub const IRON_AXE: ItemId = 131;
pub const IRON_SHOVEL: ItemId = 132;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Pick,
    Axe,
    Shovel,
}

/// (tool kind, mining speed multiplier for matching block class, max durability)
pub fn tool_info(id: ItemId) -> Option<(ToolKind, f32, u8)> {
    match id {
        WOOD_PICK => Some((ToolKind::Pick, 2.0, 59)),
        WOOD_AXE => Some((ToolKind::Axe, 2.0, 59)),
        WOOD_SHOVEL => Some((ToolKind::Shovel, 2.0, 59)),
        STONE_PICK => Some((ToolKind::Pick, 4.0, 131)),
        STONE_AXE => Some((ToolKind::Axe, 4.0, 131)),
        STONE_SHOVEL => Some((ToolKind::Shovel, 4.0, 131)),
        IRON_PICK => Some((ToolKind::Pick, 6.0, 250)),
        IRON_AXE => Some((ToolKind::Axe, 6.0, 250)),
        IRON_SHOVEL => Some((ToolKind::Shovel, 6.0, 250)),
        _ => None,
    }
}

/// Max durability; 0 means the item is stackable / not damageable.
pub fn max_dura(id: ItemId) -> u8 {
    tool_info(id).map(|(_, _, d)| d).unwrap_or(0)
}

pub fn is_tool(id: ItemId) -> bool {
    max_dura(id) > 0
}

/// Preferred tool class for a block (which tool speeds it up).
pub fn block_class(id: crate::blocks::BlockId) -> ToolKind {
    use crate::blocks::*;
    match id {
        STONE | COBBLE | COAL_ORE | IRON_ORE | SANDSTONE => ToolKind::Pick,
        LOG | PLANKS | CRAFTING_TABLE => ToolKind::Axe,
        DIRT | GRASS | SAND | GRAVEL | SNOW => ToolKind::Shovel,
        _ => ToolKind::Pick,
    }
}

/// Mining speed multiplier when breaking `block` while holding `held`.
pub fn mining_speed(held: Option<ItemId>, block: crate::blocks::BlockId) -> f32 {
    let Some(id) = held else { return 1.0 };
    let Some((kind, mult, _)) = tool_info(id) else { return 1.0 };
    if kind == block_class(block) {
        mult
    } else {
        1.0
    }
}

pub fn tool_name(id: ItemId) -> &'static str {
    match id {
        WOOD_PICK => "wooden pickaxe",
        WOOD_AXE => "wooden axe",
        WOOD_SHOVEL => "wooden shovel",
        STONE_PICK => "stone pickaxe",
        STONE_AXE => "stone axe",
        STONE_SHOVEL => "stone shovel",
        IRON_PICK => "iron pickaxe",
        IRON_AXE => "iron axe",
        IRON_SHOVEL => "iron shovel",
        _ => "tool",
    }
}

pub fn tool_color(id: ItemId) -> [f32; 3] {
    match id {
        WOOD_PICK | WOOD_AXE | WOOD_SHOVEL => [0.62, 0.45, 0.26],
        STONE_PICK | STONE_AXE | STONE_SHOVEL => [0.55, 0.55, 0.58],
        IRON_PICK | IRON_AXE | IRON_SHOVEL => [0.82, 0.82, 0.86],
        _ => [1.0, 0.0, 1.0],
    }
}

/// Every non-block item id (for icon generation).
pub const ALL_ITEMS: &[ItemId] = &[
    STICK,
    APPLE,
    IRON_INGOT,
    WOOD_PICK,
    WOOD_AXE,
    WOOD_SHOVEL,
    STONE_PICK,
    STONE_AXE,
    STONE_SHOVEL,
    IRON_PICK,
    IRON_AXE,
    IRON_SHOVEL,
];

/// Pattern is row-major over a size×size grid; 0 = empty cell.
pub struct Recipe {
    pub size: usize,
    pub pattern: &'static [ItemId],
    pub result: ItemId,
    pub count: u8,
}

const PLANKS_G: ItemId = PLANKS;
const COBBLE_G: ItemId = COBBLE;
const INGOT_G: ItemId = IRON_INGOT;

pub fn recipes() -> &'static [Recipe] {
    &[
        Recipe {
            size: 1,
            pattern: &[LOG],
            result: PLANKS,
            count: 4,
        },
        Recipe {
            size: 2,
            pattern: &[PLANKS_G, 0, PLANKS_G, 0],
            result: STICK,
            count: 4,
        },
        Recipe {
            size: 2,
            pattern: &[PLANKS_G, PLANKS_G, PLANKS_G, PLANKS_G],
            result: CRAFTING_TABLE,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[PLANKS_G, PLANKS_G, PLANKS_G, 0, STICK, 0, 0, STICK, 0],
            result: WOOD_PICK,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[PLANKS_G, PLANKS_G, 0, PLANKS_G, STICK, 0, 0, STICK, 0],
            result: WOOD_AXE,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[0, PLANKS_G, 0, 0, STICK, 0, 0, STICK, 0],
            result: WOOD_SHOVEL,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[COBBLE_G, COBBLE_G, COBBLE_G, 0, STICK, 0, 0, STICK, 0],
            result: STONE_PICK,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[COBBLE_G, COBBLE_G, 0, COBBLE_G, STICK, 0, 0, STICK, 0],
            result: STONE_AXE,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[0, COBBLE_G, 0, 0, STICK, 0, 0, STICK, 0],
            result: STONE_SHOVEL,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[INGOT_G, INGOT_G, INGOT_G, 0, STICK, 0, 0, STICK, 0],
            result: IRON_PICK,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[INGOT_G, INGOT_G, 0, INGOT_G, STICK, 0, 0, STICK, 0],
            result: IRON_AXE,
            count: 1,
        },
        Recipe {
            size: 3,
            pattern: &[0, INGOT_G, 0, 0, STICK, 0, 0, STICK, 0],
            result: IRON_SHOVEL,
            count: 1,
        },
    ]
}

/// Bounding box of non-empty cells in the active grid, plus the active cell set.
/// Player mode uses the 2×2 sub-grid at indices [0,1,3,4]; table mode uses all 9.
fn active_cells(table: bool) -> [usize; 9] {
    if table {
        [0, 1, 2, 3, 4, 5, 6, 7, 8]
    } else {
        [0, 1, 3, 4, usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX]
    }
}

fn grid_size(table: bool) -> usize {
    if table { 3 } else { 2 }
}

/// Match the grid against recipes. `table` selects 3×3 vs 2×2.
/// Returns (result, count) plus the cell indices to consume one item from.
pub fn craft_result(
    grid: &[Slot; 9],
    table: bool,
) -> Option<(ItemId, u8, [bool; 9])> {
    let size = grid_size(table);
    let cells = active_cells(table);

    // Find bounding box of placed items within the active layout.
    let (mut min_r, mut max_r, mut min_c, mut max_c) = (usize::MAX, 0usize, usize::MAX, 0usize);
    let mut any = false;
    for r in 0..size {
        for c in 0..size {
            let idx = cells[r * 3 + c];
            if idx == usize::MAX {
                continue;
            }
            if grid[idx].is_some() {
                any = true;
                min_r = min_r.min(r);
                max_r = max_r.max(r);
                min_c = min_c.min(c);
                max_c = max_c.max(c);
            }
        }
    }
    if !any {
        return None;
    }
    let bh = max_r - min_r + 1;
    let bw = max_c - min_c + 1;

    for rec in recipes() {
        // Trim recipe bounding box.
        let (mut rmin_r, mut rmax_r, mut rmin_c, mut rmax_c) =
            (usize::MAX, 0usize, usize::MAX, 0usize);
        for r in 0..rec.size {
            for c in 0..rec.size {
                if rec.pattern[r * rec.size + c] != 0 {
                    rmin_r = rmin_r.min(r);
                    rmax_r = rmax_r.max(r);
                    rmin_c = rmin_c.min(c);
                    rmax_c = rmax_c.max(c);
                }
            }
        }
        let rh = rmax_r - rmin_r + 1;
        let rw = rmax_c - rmin_c + 1;
        if rh != bh || rw != bw {
            continue;
        }

        // Compare cell by cell.
        let mut ok = true;
        for r in 0..bh {
            for c in 0..bw {
                let want = rec.pattern[(rmin_r + r) * rec.size + (rmin_c + c)];
                let idx = cells[(min_r + r) * 3 + (min_c + c)];
                let got = if idx == usize::MAX {
                    0
                } else {
                    grid[idx].map(|s| s.id).unwrap_or(0)
                };
                if want != got {
                    ok = false;
                    break;
                }
            }
            if !ok {
                break;
        }
        }
        if !ok {
            continue;
        }

        let mut consume = [false; 9];
        for r in 0..bh {
            for c in 0..bw {
                let idx = cells[(min_r + r) * 3 + (min_c + c)];
                if idx != usize::MAX && grid[idx].is_some() {
                    consume[idx] = true;
                }
            }
        }
        return Some((rec.result, rec.count, consume));
    }
    None
}
