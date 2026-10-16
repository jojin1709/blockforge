use bevy::prelude::*;

use crate::blocks::{def, BlockId, AIR, BEDROCK, COBBLE, DIRT, GRASS, LEAVES, STONE, WATER};
use crate::items;

pub type ItemId = u16;

pub const STICK: ItemId = 100;
pub const APPLE: ItemId = 101;
pub const IRON_INGOT: ItemId = 102;

pub const MAX_STACK: u8 = 64;
pub const INV_SLOTS: usize = 36;
pub const CRAFT_SLOTS: usize = 9;
pub const CRAFT_OUT: usize = INV_SLOTS + CRAFT_SLOTS + 1;

#[derive(Clone, Copy, PartialEq)]
pub struct Stack {
    pub id: ItemId,
    pub count: u8,
    /// Remaining durability for tools; 0 = not a tool / full.
    pub dura: u8,
}

impl Stack {
    pub fn new(id: ItemId, count: u8) -> Self {
        Stack { id, count, dura: items::max_dura(id) }
    }
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

/// True while the 3x3 crafting-table UI is open (vs the personal 2x2).
#[derive(Resource, Default)]
pub struct TableCraftOpen(pub bool);

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .init_resource::<CraftGrid>()
            .init_resource::<InventoryOpen>()
            .init_resource::<TableCraftOpen>();
    }
}

impl Inventory {
    pub fn add(&mut self, id: ItemId, count: u8) -> u8 {
        let mut left = count;
        let stackable = items::max_dura(id) == 0;
        if stackable {
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
        }
        for slot in self.slots.iter_mut() {
            if left == 0 {
                break;
            }
            if slot.is_none() {
                let m = if stackable { left.min(MAX_STACK) } else { 1 };
                *slot = Some(Stack::new(id, m));
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

    /// Damage the held tool by 1; removes it when durability runs out.
    pub fn damage_selected_tool(&mut self) {
        if let Some(s) = self.slots[self.selected].as_mut() {
            if s.dura > 0 {
                s.dura -= 1;
                if s.dura == 0 {
                    self.slots[self.selected] = None;
                }
            }
        }
    }
}

pub fn as_block(id: ItemId) -> Option<BlockId> {
    if (1..=16).contains(&id) && id != WATER {
        Some(id)
    } else {
        None
    }
}

pub fn item_name(id: ItemId) -> &'static str {
    if (id as usize) <= 16 {
        def(id).name
    } else if id == STICK {
        "stick"
    } else if id == APPLE {
        "apple"
    } else if id == IRON_INGOT {
        "iron ingot"
    } else {
        items::tool_name(id)
    }
}

pub fn item_color(id: ItemId) -> [f32; 3] {
    if (id as usize) <= 16 {
        def(id).side
    } else {
        match id {
            STICK => [0.55, 0.40, 0.22],
            APPLE => [0.75, 0.12, 0.10],
            IRON_INGOT => [0.82, 0.82, 0.86],
            _ => items::tool_color(id),
        }
    }
}

pub fn break_drop(id: BlockId) -> Option<ItemId> {
    match id {
        AIR | WATER => None,
        BEDROCK => None,
        GRASS => Some(DIRT),
        STONE => Some(COBBLE),
        crate::blocks::IRON_ORE => Some(IRON_INGOT),
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

pub fn click_slot(
    idx: usize,
    inv: &mut Inventory,
    craft: &mut CraftGrid,
    table: bool,
) {
    let slot: &mut Slot = if idx < INV_SLOTS {
        &mut inv.slots[idx]
    } else if idx < CRAFT_OUT {
        &mut craft.slots[idx - INV_SLOTS]
    } else {
        take_craft_output(craft, &mut inv.cursor, table);
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
            if c.id == s.id && c.dura == s.dura {
                let moved = c.count.min(MAX_STACK - s.count);
                slot.as_mut().unwrap().count += moved;
                if c.count > moved {
                    *cursor_slot = Some(Stack {
                        id: c.id,
                        count: c.count - moved,
                        dura: c.dura,
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

fn take_craft_output(craft: &mut CraftGrid, cursor: &mut Slot, table: bool) {
    let Some((id, n, consume)) = items::craft_result(&craft.slots, table) else {
        return;
    };
    let can = match cursor {
        None => true,
        Some(c) => c.id == id && c.count + n <= MAX_STACK,
    };
    if !can {
        return;
    }
    for i in 0..CRAFT_SLOTS {
        if consume[i] {
            if let Some(st) = craft.slots[i].as_mut() {
                st.count -= 1;
                if st.count == 0 {
                    craft.slots[i] = None;
                }
            }
        }
    }
    match cursor {
        None => *cursor = Some(Stack::new(id, n)),
        Some(c) => c.count += n,
    }
}
