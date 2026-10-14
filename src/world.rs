use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::tasks::{futures::check_ready, AsyncComputeTaskPool, Task};

use crate::blocks::*;
use crate::mesh::{build_meshes, PAD_BLOCKS};
use crate::player::Player;
use crate::terrain::{self, ColumnBlocks, CHUNK_BLOCKS, CHUNK_SIZE, WORLD_HEIGHT, Y_CHUNKS};
use crate::textures::build_atlas_image;

pub const SEED: u64 = 20_261_013;
pub const RENDER_DIST: i32 = 6;
const START_DIST: i32 = 2;
const GEN_PER_FRAME: usize = 3;
const MESH_PER_FRAME: usize = 3;

pub struct Chunk {
    pub blocks: [BlockId; CHUNK_BLOCKS],
}

#[derive(Default)]
pub struct ChunkRender {
    pub solid_entity: Option<Entity>,
    pub solid_mesh: Option<Handle<Mesh>>,
    pub water_entity: Option<Entity>,
    pub water_mesh: Option<Handle<Mesh>>,
}

#[derive(Resource)]
pub struct World {
    pub seed: u64,
    pub chunks: HashMap<IVec3, Chunk>,
    pub dirty: HashSet<IVec3>,
    pub renders: HashMap<IVec3, ChunkRender>,
    pub edits: HashMap<IVec3, BlockId>,
    pending_gen: HashSet<(i32, i32)>,
    pending_mesh: HashSet<IVec3>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        World {
            seed,
            chunks: HashMap::new(),
            dirty: HashSet::new(),
            renders: HashMap::new(),
            edits: HashMap::new(),
            pending_gen: HashSet::new(),
            pending_mesh: HashSet::new(),
        }
    }

    pub fn block(&self, x: i32, y: i32, z: i32) -> BlockId {
        if y < 0 {
            return STONE;
        }
        if y >= WORLD_HEIGHT {
            return AIR;
        }
        let cp = IVec3::new(x.div_euclid(16), y.div_euclid(16), z.div_euclid(16));
        match self.chunks.get(&cp) {
            Some(c) => c.blocks
                [((y.rem_euclid(16) * 16 + z.rem_euclid(16)) * 16 + x.rem_euclid(16)) as usize],
            None => AIR,
        }
    }

    pub fn column_loaded(&self, cx: i32, cz: i32) -> bool {
        self.chunks.contains_key(&IVec3::new(cx, 0, cz))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, id: BlockId) {
        if y < 0 || y >= WORLD_HEIGHT {
            return;
        }
        let cp = IVec3::new(x.div_euclid(16), y.div_euclid(16), z.div_euclid(16));
        let Some(chunk) = self.chunks.get_mut(&cp) else { return };
        chunk.blocks[((y.rem_euclid(16) * 16 + z.rem_euclid(16)) * 16 + x.rem_euclid(16)) as usize] = id;
        self.edits.insert(IVec3::new(x, y, z), id);
        self.dirty.insert(cp);
        let (lx, ly, lz) = (
            x.rem_euclid(16),
            y.rem_euclid(16),
            z.rem_euclid(16),
        );
        if lx == 0 { self.dirty.insert(cp + IVec3::new(-1, 0, 0)); }
        if lx == 15 { self.dirty.insert(cp + IVec3::new(1, 0, 0)); }
        if ly == 0 { self.dirty.insert(cp + IVec3::new(0, -1, 0)); }
        if ly == 15 { self.dirty.insert(cp + IVec3::new(0, 1, 0)); }
        if lz == 0 { self.dirty.insert(cp + IVec3::new(0, 0, -1)); }
        if lz == 15 { self.dirty.insert(cp + IVec3::new(0, 0, 1)); }
    }
}

#[derive(Component)]
pub struct GenTask {
    col: (i32, i32),
    task: Task<ColumnBlocks>,
}

#[derive(Component)]
pub struct MeshTask {
    cp: IVec3,
    task: Task<(Option<Mesh>, Option<Mesh>)>,
}

#[derive(Resource)]
pub struct GameMaterials {
    pub opaque: Handle<StandardMaterial>,
    pub water: Handle<StandardMaterial>,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(World::new(SEED))
            .add_systems(Startup, (setup_materials, init_world).chain())
            .add_systems(Update, tick_world);
    }
}

fn setup_materials(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let atlas = images.add(build_atlas_image());
    let opaque = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(atlas.clone()),
        perceptual_roughness: 0.9,
        reflectance: 0.05,
        ..default()
    });
    let water = mats.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.55),
        base_color_texture: Some(atlas),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.2,
        reflectance: 0.4,
        cull_mode: None,
        ..default()
    });
    commands.insert_resource(GameMaterials { opaque, water });
}

fn spawn_gen(commands: &mut Commands, world: &mut World, cx: i32, cz: i32) {
    if world.column_loaded(cx, cz) || world.pending_gen.contains(&(cx, cz)) {
        return;
    }
    world.pending_gen.insert((cx, cz));
    let seed = world.seed;
    let task = AsyncComputeTaskPool::get()
        .spawn(async move { terrain::gen_column(cx, cz, seed) });
    commands.spawn(GenTask { col: (cx, cz), task });
}

fn init_world(mut commands: Commands, mut world: ResMut<World>) {
    for dz in -START_DIST..=START_DIST {
        for dx in -START_DIST..=START_DIST {
            spawn_gen(&mut commands, &mut world, dx, dz);
        }
    }
}

fn neighbors_ready(world: &World, cp: IVec3) -> bool {
    world.chunks.contains_key(&(cp + IVec3::new(1, 0, 0)))
        && world.chunks.contains_key(&(cp + IVec3::new(-1, 0, 0)))
        && world.chunks.contains_key(&(cp + IVec3::new(0, 0, 1)))
        && world.chunks.contains_key(&(cp + IVec3::new(0, 0, -1)))
}

fn gather_padded(world: &World, cp: IVec3) -> Box<[BlockId; PAD_BLOCKS]> {
    let base = cp * CHUNK_SIZE;
    let mut out = Box::new([0u16; PAD_BLOCKS]);
    for y in -1..=CHUNK_SIZE {
        for z in -1..=CHUNK_SIZE {
            for x in -1..=CHUNK_SIZE {
                let i = (((y + 1) * 18 + (z + 1)) * 18 + (x + 1)) as usize;
                out[i] = world.block(base.x + x, base.y + y, base.z + z);
            }
        }
    }
    out
}

fn tick_world(
    mut world: ResMut<World>,
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mats: Res<GameMaterials>,
    mut gen_q: Query<(Entity, &mut GenTask)>,
    mut mesh_q: Query<(Entity, &mut MeshTask)>,
    player_q: Query<&Player>,
) {
    let Ok(player) = player_q.single() else { return };
    let pc = IVec3::new(
        (player.pos.x.floor() as i32).div_euclid(16),
        (player.pos.y.floor() as i32).div_euclid(16),
        (player.pos.z.floor() as i32).div_euclid(16),
    );

    // Poll finished column generations.
    for (entity, mut gt) in gen_q.iter_mut() {
        let Some(col_blocks) = check_ready(&mut gt.task) else {
            continue;
        };
        commands.entity(entity).despawn();
        if !world.pending_gen.remove(&gt.col) {
            continue;
        }
        let (cx, cz) = gt.col;
        let base_x = cx * CHUNK_SIZE;
        let base_z = cz * CHUNK_SIZE;
        let edits: Vec<(IVec3, BlockId)> = world
            .edits
            .iter()
            .filter(|(p, _)| {
                (base_x..base_x + CHUNK_SIZE).contains(&p.x)
                    && (base_z..base_z + CHUNK_SIZE).contains(&p.z)
            })
            .map(|(p, id)| (*p, *id))
            .collect();
        for ych in 0..Y_CHUNKS as i32 {
            let mut chunk = Chunk { blocks: col_blocks[ych as usize] };
            for (p, id) in &edits {
                if p.y.div_euclid(16) == ych {
                    let i = ((p.y.rem_euclid(16) * 16 + p.z.rem_euclid(16)) * 16
                        + p.x.rem_euclid(16)) as usize;
                    chunk.blocks[i] = *id;
                }
            }
            let cp = IVec3::new(cx, ych, cz);
            world.chunks.insert(cp, chunk);
            world.dirty.insert(cp);
        }
    }

    // Poll finished meshes and attach them.
    for (entity, mut mt) in mesh_q.iter_mut() {
        let Some((solid, water)) = check_ready(&mut mt.task) else {
            continue;
        };
        commands.entity(entity).despawn();
        if !world.pending_mesh.remove(&mt.cp) || !world.chunks.contains_key(&mt.cp) {
            continue;
        }
        apply_mesh(
            &mut world,
            &mut commands,
            &mut mesh_assets,
            &mats,
            mt.cp,
            solid,
            water,
        );
    }

    // Stream in columns around the player.
    let mut missing: Vec<(i64, (i32, i32))> = Vec::new();
    for dz in -RENDER_DIST..=RENDER_DIST {
        for dx in -RENDER_DIST..=RENDER_DIST {
            if dx * dx + dz * dz > RENDER_DIST * RENDER_DIST {
                continue;
            }
            let col = (pc.x + dx, pc.z + dz);
            if world.column_loaded(col.0, col.1) || world.pending_gen.contains(&col) {
                continue;
            }
            missing.push(((dx * dx + dz * dz) as i64, col));
        }
    }
    missing.sort_unstable();
    for (_, col) in missing.into_iter().take(GEN_PER_FRAME) {
        spawn_gen(&mut commands, &mut world, col.0, col.1);
    }

    // Queue mesh builds for ready dirty chunks.
    let mut dirty: Vec<(i64, IVec3)> = world
        .dirty
        .iter()
        .filter(|cp| neighbors_ready(&world, **cp) && !world.pending_mesh.contains(*cp))
        .map(|cp| {
            let d = *cp - pc;
            ((d.x as i64).pow(2) + (d.y as i64).pow(2) + (d.z as i64).pow(2), *cp)
        })
        .collect();
    dirty.sort_by_key(|(d, _)| *d);
    let pool = AsyncComputeTaskPool::get();
    let mut spawned = 0;
    for (_, cp) in dirty {
        if spawned >= MESH_PER_FRAME {
            break;
        }
        world.dirty.remove(&cp);
        world.pending_mesh.insert(cp);
        let padded = gather_padded(&world, cp);
        let task = pool.spawn(async move { build_meshes(&padded) });
        commands.spawn(MeshTask { cp, task });
        spawned += 1;
    }

    // Unload far columns.
    let far: Vec<(i32, i32)> = {
        let mut cols = HashSet::new();
        let r2 = (RENDER_DIST + 1) * (RENDER_DIST + 1);
        for cp in world.chunks.keys() {
            let dx = cp.x - pc.x;
            let dz = cp.z - pc.z;
            if dx * dx + dz * dz > r2 {
                cols.insert((cp.x, cp.z));
            }
        }
        cols.into_iter().collect()
    };
    for (cx, cz) in far {
        world.pending_gen.remove(&(cx, cz));
        for ych in 0..Y_CHUNKS as i32 {
            let cp = IVec3::new(cx, ych, cz);
            world.dirty.remove(&cp);
            world.pending_mesh.remove(&cp);
            if let Some(render) = world.renders.remove(&cp) {
                if let Some(e) = render.solid_entity {
                    commands.entity(e).despawn();
                }
                if let Some(e) = render.water_entity {
                    commands.entity(e).despawn();
                }
                if let Some(h) = render.solid_mesh {
                    mesh_assets.remove(&h);
                }
                if let Some(h) = render.water_mesh {
                    mesh_assets.remove(&h);
                }
            }
            world.chunks.remove(&cp);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_mesh(
    world: &mut World,
    commands: &mut Commands,
    mesh_assets: &mut Assets<Mesh>,
    mats: &GameMaterials,
    cp: IVec3,
    solid: Option<Mesh>,
    water: Option<Mesh>,
) {
    let render = world.renders.entry(cp).or_default();
    let origin = Vec3::new(
        (cp.x * CHUNK_SIZE) as f32,
        (cp.y * CHUNK_SIZE) as f32,
        (cp.z * CHUNK_SIZE) as f32,
    );

    if let Some(mesh) = solid {
        let new_handle = mesh_assets.add(mesh);
        match render.solid_entity {
            Some(entity) => {
                if let Some(old) = render.solid_mesh.replace(new_handle.clone()) {
                    mesh_assets.remove(&old);
                }
                commands
                    .entity(entity)
                    .insert((Mesh3d(new_handle), MeshMaterial3d(mats.opaque.clone())));
            }
            None => {
                let entity = commands
                    .spawn((
                        Mesh3d(new_handle.clone()),
                        MeshMaterial3d(mats.opaque.clone()),
                        Transform::from_translation(origin),
                    ))
                    .id();
                render.solid_entity = Some(entity);
                render.solid_mesh = Some(new_handle);
            }
        }
    } else if let Some(entity) = render.solid_entity.take() {
        commands.entity(entity).despawn();
        if let Some(old) = render.solid_mesh.take() {
            mesh_assets.remove(&old);
        }
    }

    if let Some(mesh) = water {
        let new_handle = mesh_assets.add(mesh);
        match render.water_entity {
            Some(entity) => {
                if let Some(old) = render.water_mesh.replace(new_handle.clone()) {
                    mesh_assets.remove(&old);
                }
                commands
                    .entity(entity)
                    .insert((Mesh3d(new_handle), MeshMaterial3d(mats.water.clone())));
            }
            None => {
                let entity = commands
                    .spawn((
                        Mesh3d(new_handle.clone()),
                        MeshMaterial3d(mats.water.clone()),
                        Transform::from_translation(origin),
                    ))
                    .id();
                render.water_entity = Some(entity);
                render.water_mesh = Some(new_handle);
            }
        }
    } else if let Some(entity) = render.water_entity.take() {
        commands.entity(entity).despawn();
        if let Some(old) = render.water_mesh.take() {
            mesh_assets.remove(&old);
        }
    }
}
