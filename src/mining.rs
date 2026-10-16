use bevy::prelude::*;

use crate::audio::{SoundEvent, SoundQueue};
use crate::blocks::{def, AIR, BEDROCK, WATER};
use crate::inventory::{break_drop, Inventory, InventoryOpen};
use crate::items;
use crate::pause::PauseOpen;
use crate::player::{self, Player, REACH};
use crate::textures;
use crate::world::World;

/// Tracks the block currently being mined and its progress (0.0..=1.0).
#[derive(Resource, Default)]
pub struct BreakProgress {
    pub target: Option<IVec3>,
    pub progress: f32,
}

/// Queue of break events consumed by the particle system.
#[derive(Resource, Default)]
pub struct BreakEvents(pub Vec<(IVec3, BlockId)>);

type BlockId = u16;

const BASE_MINING_SPEED: f32 = 1.0;

pub struct MiningPlugin;

impl Plugin for MiningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BreakProgress>()
            .init_resource::<BreakEvents>()
            .add_systems(Startup, setup_crack_overlay)
            .add_systems(Update, (mine_tick, update_crack_overlay).chain());
    }
}

#[derive(Component)]
pub struct CrackOverlay;

#[derive(Resource)]
pub struct CrackMats(pub Vec<Handle<StandardMaterial>>);

fn setup_crack_overlay(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut crack_mats = Vec::with_capacity(textures::CRACK_STAGES);
    for img in textures::build_crack_images() {
        let handle = images.add(img);
        crack_mats.push(mats.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(handle),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            depth_bias: -1.0,
            ..default()
        }));
    }
    commands.insert_resource(CrackMats(crack_mats));

    let mesh = meshes.add(Cuboid::from_size(Vec3::splat(1.002)));
    commands.spawn((
        CrackOverlay,
        Mesh3d(mesh),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            depth_bias: -1.0,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
    ));
}

fn mine_tick(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    open: Res<InventoryOpen>,
    pause: Res<PauseOpen>,
    mut world: ResMut<World>,
    mut bp: ResMut<BreakProgress>,
    mut events: ResMut<BreakEvents>,
    mut inv: ResMut<Inventory>,
    mut swing: ResMut<crate::visuals::SwingTimer>,
    mut sounds: ResMut<SoundQueue>,
    player_q: Query<&Player>,
) {
    if open.0 || pause.0 {
        bp.target = None;
        bp.progress = 0.0;
        return;
    }
    let Ok(player) = player_q.single() else { return };

    let eye_h = if player.sneak { 1.45 } else { 1.62 };
    let eye = player.pos + Vec3::new(0.0, eye_h, 0.0);
    let rot = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
    let dir = rot * Vec3::NEG_Z;

    let hit = player::raycast(&world, eye, dir, REACH);
    let target = hit.map(|(h, _)| h);

    if !mouse.pressed(MouseButton::Left) || target.is_none() {
        bp.target = None;
        bp.progress = 0.0;
        return;
    }

    let target = target.unwrap();
    let id = world.block(target.x, target.y, target.z);
    if id == AIR || id == WATER || id == BEDROCK || !def(id).solid {
        bp.target = None;
        bp.progress = 0.0;
        return;
    }

    if bp.target != Some(target) {
        bp.target = Some(target);
        bp.progress = 0.0;
    }

    let hardness = def(id).hardness;
    if !hardness.is_finite() || hardness <= 0.0 {
        return;
    }

    let speed = items::mining_speed(inv.slots[inv.selected].map(|s| s.id), id);
    let break_time = (hardness / (BASE_MINING_SPEED * speed)).max(0.05);
    bp.progress += time.delta_secs() / break_time;
    swing.0 = 0.25;

    if bp.progress >= 1.0 {
        world.set_block(target.x, target.y, target.z, AIR);
        if let Some(drop) = break_drop(id) {
            inv.add(drop, 1);
        }
        if items::is_tool(inv.slots[inv.selected].map(|s| s.id).unwrap_or(0)) {
            inv.damage_selected_tool();
        }
        events.0.push((target, id));
        sounds.0.push(SoundEvent::Break(id));
        bp.target = None;
        bp.progress = 0.0;
    }
}

fn update_crack_overlay(
    bp: Res<BreakProgress>,
    crack_mats: Option<Res<CrackMats>>,
    mut overlay_q: Query<
        (&mut Transform, &mut Visibility, &mut MeshMaterial3d<StandardMaterial>),
        With<CrackOverlay>,
    >,
) {
    let Ok((mut tf, mut vis, mut mat)) = overlay_q.single_mut() else { return };
    let Some(mats) = crack_mats else { return };

    match bp.target {
        Some(pos) if bp.progress > 0.0 => {
            tf.translation = pos.as_vec3() + Vec3::splat(0.5);
            *vis = Visibility::Visible;
            let stage = ((bp.progress * textures::CRACK_STAGES as f32) as usize)
                .min(textures::CRACK_STAGES - 1);
            if let Some(m) = mats.0.get(stage) {
                **mat = m.clone();
            }
        }
        _ => {
            *vis = Visibility::Hidden;
        }
    }
}
