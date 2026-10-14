use bevy::prelude::*;

use crate::blocks::{def, BlockId, AIR, BEDROCK, COBBLE, DIRT, GRASS, LEAVES, LOG, PLANKS, STONE, WATER};

pub type ItemId = u16;

pub const STICK: ItemId = 100;
pub const APPLE: ItemId = 101;

pub const MAX_STACK: u8 = 64;
pub const INV_SLOTS: usize = 36;
pub const CRAFT_SLOTS: usize = 4;
pub const CRAFT_OUT: usize = 40;

#[derive(Clone, Copy, PartialEq)]
pub struct Stack {
    pub id: ItemId,
    pub count: u8,
}

pub type Slot = Option<Stack>;

#[derive(Resource)]
pub struct Inventory {
    pub slots: [Slot; INV_SLOTS],
    pub selected: usize,
    pub cursor: Slot,
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            slots: [None; INV_SLOTS],
            selected: 0,
            cursor: None,
        }
    }
}

#[derive(Resource)]
pub struct CraftGrid {
    pub slots: [Slot; CRAFT_SLOTS],
}

impl Default for CraftGrid {
    fn default() -> Self {
        CraftGrid {
            slots: [None; CRAFT_SLOTS],
        }
    }
}

#[derive(Resource, Default)]
pub struct InventoryOpen(pub bool);

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .init_resource::<CraftGrid>()
            .init_resource::<InventoryOpen>();
    }
}

impl Inventory {
    pub fn add(&mut self, id: ItemId, count: u8) -> u8 {
        let mut left = count;
        for slot in self.slots.iter_mut() {
            if left == 0 {
                break;
            }
            if let Some(s) = slot {
                if s.id == id && s.count < MAX_STACK {
                    let m = left.min(MAX_STACK - s.count);
                    s.count += m;
                    left -= m;
                }
            }
        }
        for slot in self.slots.iter_mut() {
            if left == 0 {
                break;
            }
            if slot.is_none() {
                let m = left.min(MAX_STACK);
                *slot = Some(Stack { id, count: m });
                left -= m;
            }
        }
        left
    }

    pub fn consume_selected(&mut self) {
        if let Some(s) = self.slots[self.selected].as_mut() {
            s.count -= 1;
            if s.count == 0 {
                self.slots[self.selected] = None;
            }
        }
    }
}

pub fn as_block(id: ItemId) -> Option<BlockId> {
    if (1..=12).contains(&id) && id != WATER {
        Some(id)
    } else {
        None
    }
}

pub fn item_name(id: ItemId) -> &'static str {
    if (id as usize) < 13 {
        def(id).name
    } else {
        match id {
            STICK => "stick",
            APPLE => "apple",
            _ => "unknown",
        }
    }
}

pub fn item_color(id: ItemId) -> [f32; 3] {
    if (id as usize) < 13 {
        def(id).side
    } else {
        match id {
            STICK => [0.55, 0.40, 0.22],
            APPLE => [0.75, 0.12, 0.10],
            _ => [1.0, 0.0, 1.0],
        }
    }
}

pub fn break_drop(id: BlockId) -> Option<ItemId> {
    match id {
        AIR | WATER => None,
        BEDROCK => None,
        GRASS => Some(DIRT),
        STONE => Some(COBBLE),
        LEAVES => {
            if rand::random::<f32>() < 0.12 {
                Some(APPLE)
            } else {
                None
            }
        }
        _ => Some(id),
    }
}

pub fn craft_result(grid: &[Slot; CRAFT_SLOTS]) -> Option<(ItemId, u8)> {
    let mut logs = 0u8;
    let mut planks = 0u8;
    for s in grid.iter().flatten() {
        match s.id {
            LOG => logs += 1,
            PLANKS => planks += 1,
            _ => return None,
        }
    }
    if logs >= 1 && planks == 0 {
        Some((PLANKS, logs * 4))
    } else if planks >= 2 && logs == 0 {
        Some((STICK, 4))
    } else {
        None
    }
}

pub fn click_slot(
    idx: usize,
    inv: &mut Inventory,
    craft: &mut CraftGrid,
) {
    let slot: &mut Slot = if idx < INV_SLOTS {
        &mut inv.slots[idx]
    } else if idx < CRAFT_OUT {
        &mut craft.slots[idx - INV_SLOTS]
    } else {
        take_craft_output(craft, &mut inv.cursor);
        return;
    };

    let cursor_slot = &mut inv.cursor;
    match (cursor_slot.take(), *slot) {
        (None, Some(s)) => {
            *cursor_slot = Some(s);
            *slot = None;
        }
        (Some(c), None) => {
            *slot = Some(c);
        }
        (Some(c), Some(s)) => {
            if c.id == s.id {
                let moved = c.count.min(MAX_STACK - s.count);
                slot.as_mut().unwrap().count += moved;
                if c.count > moved {
                    *cursor_slot = Some(Stack {
                        id: c.id,
                        count: c.count - moved,
                    });
                }
            } else {
                *slot = Some(c);
                *cursor_slot = Some(s);
            }
        }
        (None, None) => {}
    }
}

fn take_craft_output(craft: &mut CraftGrid, cursor: &mut Slot) {
    let Some((id, n)) = craft_result(&craft.slots) else {
        return;
    };
    let can = match cursor {
        None => true,
        Some(c) => c.id == id && c.count + n <= MAX_STACK,
    };
    if !can {
        return;
    }
    if id == PLANKS {
        for s in craft.slots.iter_mut() {
            if s.map(|st| st.id == LOG).unwrap_or(false) {
                *s = None;
            }
        }
    } else {
        let mut removed = 0u8;
        for s in craft.slots.iter_mut() {
            if removed >= 2 {
                break;
            }
            if let Some(st) = s.as_mut() {
                if st.id == PLANKS {
                    st.count -= 1;
                    removed += 1;
                    if st.count == 0 {
                        *s = None;
                    }
                }
            }
        }
    }
    match cursor {
        None => *cursor = Some(Stack { id, count: n }),
        Some(c) => c.count += n,
    }
}
