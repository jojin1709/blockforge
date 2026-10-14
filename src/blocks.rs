pub type BlockId = u16;

pub const AIR: BlockId = 0;
pub const GRASS: BlockId = 1;
pub const DIRT: BlockId = 2;
pub const STONE: BlockId = 3;
pub const SAND: BlockId = 4;
pub const WATER: BlockId = 5;
pub const LOG: BlockId = 6;
pub const LEAVES: BlockId = 7;
pub const BEDROCK: BlockId = 8;
pub const COAL_ORE: BlockId = 9;
pub const IRON_ORE: BlockId = 10;
pub const PLANKS: BlockId = 11;
pub const COBBLE: BlockId = 12;

pub struct Def {
    pub name: &'static str,
    pub solid: bool,
    pub top: [f32; 3],
    pub side: [f32; 3],
    pub bottom: [f32; 3],
}

const DEFS: [Def; 13] = [
    Def { name: "air", solid: false, top: [0.0, 0.0, 0.0], side: [0.0, 0.0, 0.0], bottom: [0.0, 0.0, 0.0] },
    Def { name: "grass", solid: true, top: [0.33, 0.58, 0.23], side: [0.45, 0.38, 0.24], bottom: [0.42, 0.33, 0.22] },
    Def { name: "dirt", solid: true, top: [0.42, 0.33, 0.22], side: [0.42, 0.33, 0.22], bottom: [0.42, 0.33, 0.22] },
    Def { name: "stone", solid: true, top: [0.5, 0.5, 0.5], side: [0.5, 0.5, 0.5], bottom: [0.5, 0.5, 0.5] },
    Def { name: "sand", solid: true, top: [0.86, 0.8, 0.6], side: [0.82, 0.76, 0.56], bottom: [0.78, 0.72, 0.52] },
    Def { name: "water", solid: false, top: [0.2, 0.4, 0.8], side: [0.2, 0.4, 0.8], bottom: [0.2, 0.4, 0.8] },
    Def { name: "log", solid: true, top: [0.55, 0.44, 0.28], side: [0.42, 0.33, 0.19], bottom: [0.55, 0.44, 0.28] },
    Def { name: "leaves", solid: true, top: [0.22, 0.42, 0.13], side: [0.2, 0.4, 0.12], bottom: [0.18, 0.36, 0.1] },
    Def { name: "bedrock", solid: true, top: [0.18, 0.18, 0.18], side: [0.18, 0.18, 0.18], bottom: [0.18, 0.18, 0.18] },
    Def { name: "coal_ore", solid: true, top: [0.3, 0.3, 0.32], side: [0.3, 0.3, 0.32], bottom: [0.3, 0.3, 0.32] },
    Def { name: "iron_ore", solid: true, top: [0.62, 0.53, 0.46], side: [0.62, 0.53, 0.46], bottom: [0.62, 0.53, 0.46] },
    Def { name: "planks", solid: true, top: [0.72, 0.58, 0.35], side: [0.68, 0.55, 0.33], bottom: [0.68, 0.55, 0.33] },
    Def { name: "cobblestone", solid: true, top: [0.44, 0.44, 0.45], side: [0.44, 0.44, 0.45], bottom: [0.44, 0.44, 0.45] },
];

pub fn def(id: BlockId) -> &'static Def {
    &DEFS[(id as usize).min(DEFS.len() - 1)]
}
