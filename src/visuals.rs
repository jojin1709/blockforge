use std::collections::HashMap;

use bevy::math::EulerRot;
use bevy::prelude::*;

use crate::inventory::{Inventory, InventoryOpen, ItemId};
use crate::player::{self, Player};
use crate::textures;
use crate::world::World;

/// Counts down while the player is swinging (breaking / placing).
#[derive(Resource, Default)]
pub struct SwingTimer(pub f32);

pub const SWING_DURATION: f32 = 0.25;

#[derive(Component)]
pub struct HeldItemCube;

#[derive(Component)]
pub struct TargetOutline;

/// Cached per-item materials for the held-item cube.
#[derive(Resource, Default)]
pub struct HeldMats(pub HashMap<ItemId, Handle<StandardMaterial>>);

/// Raw atlas pixels, reused to build held-item textures on demand.
#[derive(Resource)]
pub struct AtlasBytes(pub Vec<u8>);

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SwingTimer>()
            .init_resource::<HeldMats>()
            .add_systems(Startup, setup_visuals)
            .add_systems(Update, (update_outline, update_held_item).chain());
    }
}

fn setup_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    camera_q: Query<Entity, With<Player>>,
) {
    commands.insert_resource(AtlasBytes(textures::build_atlas_data()));

    // ---- Target block outline (wireframe cube built from 12 thin edges) ----
    let edge_mat = mats.add(StandardMaterial {
        base_color: Color::srgba(0.02, 0.02, 0.02, 0.75),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        depth_bias: -2.0,
        ..default()
    });
    let s = 1.004f32;
    let t = 0.006f32;
    let h = s * 0.5;
    let edge_mesh = meshes.add(Mesh::from(Cuboid::from_size(Vec3::ONE)));
    let root = commands
        .spawn((TargetOutline, Transform::default(), Visibility::Hidden))
        .id();
    let edge_defs: [(Vec3, Vec3); 12] = [
        (Vec3::new(0., -h, -h), Vec3::new(s, t, t)),
        (Vec3::new(0., h, -h), Vec3::new(s, t, t)),
        (Vec3::new(0., -h, h), Vec3::new(s, t, t)),
        (Vec3::new(0., h, h), Vec3::new(s, t, t)),
        (Vec3::new(-h, 0., -h), Vec3::new(t, s, t)),
        (Vec3::new(h, 0., -h), Vec3::new(t, s, t)),
        (Vec3::new(-h, 0., h), Vec3::new(t, s, t)),
        (Vec3::new(h, 0., h), Vec3::new(t, s, t)),
        (Vec3::new(-h, -h, 0.), Vec3::new(t, t, s)),
        (Vec3::new(h, -h, 0.), Vec3::new(t, t, s)),
        (Vec3::new(-h, h, 0.), Vec3::new(t, t, s)),
        (Vec3::new(h, h, 0.), Vec3::new(t, t, s)),
    ];
    for (pos, scale) in edge_defs {
        let e = commands
            .spawn((
                Mesh3d(edge_mesh.clone()),
                MeshMaterial3d(edge_mat.clone()),
                Transform::from_translation(pos).with_scale(scale),
            ))
            .id();
        commands.entity(e).set_parent_in_place(root);
    }

    // ---- Held item cube, child of the camera so it stays on screen ----
    let Ok(camera) = camera_q.single() else { return };
    let held_mesh = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(0.4))));
    let placeholder = mats.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.12, 0.14),
        perceptual_roughness: 0.8,
        // Large depth bias so the held item always renders in front of world geometry.
        depth_bias: 100.0,
        ..default()
    });
    let held = commands
        .spawn((
            HeldItemCube,
            Mesh3d(held_mesh),
            MeshMaterial3d(placeholder),
            Transform::from_xyz(0.45, -0.32, -0.5)
                .with_rotation(Quat::from_euler(EulerRot::YXZ, -0.4, 0.7, 0.1)),
            Visibility::Visible,
        ))
        .id();
    commands.entity(camera).add_child(held);
}

fn update_outline(
    world: Res<World>,
    open: Res<InventoryOpen>,
    player_q: Query<(&Player, &Transform)>,
    mut outline_q: Query<(&mut Transform, &mut Visibility), (With<TargetOutline>, Without<Player>)>,
) {
    let Ok((player, _tf)) = player_q.single() else { return };
    let Ok((mut otf, mut vis)) = outline_q.single_mut() else { return };
    if open.0 {
        *vis = Visibility::Hidden;
        return;
    }
    let eye_h = if player.sneak { 1.45 } else { 1.62 };
    let eye = player.pos + Vec3::new(0.0, eye_h, 0.0);
    let rot = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
    let dir = rot * Vec3::NEG_Z;
    if let Some((hit, _)) = player::raycast(&world, eye, dir, player::REACH) {
        otf.translation = hit.as_vec3() + Vec3::splat(0.5);
        *vis = Visibility::Visible;
    } else {
        *vis = Visibility::Hidden;
    }
}

#[allow(clippy::too_many_arguments)]
fn update_held_item(
    time: Res<Time>,
    inv: Res<Inventory>,
    open: Res<InventoryOpen>,
    mut swing: ResMut<SwingTimer>,
    atlas: Res<AtlasBytes>,
    mut images: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut held_mats: ResMut<HeldMats>,
    mut q: Query<
        (&mut Transform, &mut MeshMaterial3d<StandardMaterial>, &mut Visibility),
        With<HeldItemCube>,
    >,
) {
    let Ok((mut tf, mut mat, mut vis)) = q.single_mut() else { return };
    let item = if open.0 {
        None
    } else {
        inv.slots[inv.selected].map(|s| s.id)
    };
    match item {
        None => {
            *vis = Visibility::Hidden;
        }
        Some(id) => {
            *vis = Visibility::Visible;
            if !held_mats.0.contains_key(&id) {
                let img = textures::extract_tile_image(&atlas.0, textures::item_tile(id));
                let ih = images.add(img);
                let mh = mats.add(StandardMaterial {
                    base_color: Color::WHITE,
                    base_color_texture: Some(ih),
                    perceptual_roughness: 0.75,
                    reflectance: 0.05,
                    depth_bias: 100.0,
                    ..default()
                });
                held_mats.0.insert(id, mh);
            }
            if let Some(h) = held_mats.0.get(&id) {
                **mat = h.clone();
            }
            // Grip pose relative to the first-person arm (child of the arm at
            // the hand). The arm's swing animation carries the item with it.
            tf.translation = Vec3::new(0.0, -0.72, -0.1);
            tf.rotation = Quat::from_euler(EulerRot::YXZ, -0.2, 0.5, 0.0);
        }
    }

    // Decay the shared swing timer (used by the arm + third-person body).
    if swing.0 > 0.0 {
        swing.0 = (swing.0 - time.delta_secs()).max(0.0);
    }
}
