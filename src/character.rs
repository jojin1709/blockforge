use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, PrimitiveTopology, TextureDimension, TextureFormat,
};

use crate::inventory::InventoryOpen;
use crate::player::Player;
use crate::visuals::{HeldItemCube, SwingTimer};

const SKIN_PATH: &str = "character/skin.png";
const PX: f32 = 1.0 / 16.0; // 16 px per block

// ---------------------------------------------------------------------------
// Skin layout helpers
// ---------------------------------------------------------------------------

/// UV rectangles for one box-unwrapped body part (pixel coords, top-left origin).
#[derive(Clone, Copy)]
pub struct FaceRects {
    pub top: [f32; 4],
    pub bottom: [f32; 4],
    pub right: [f32; 4],
    pub front: [f32; 4],
    pub left: [f32; 4],
    pub back: [f32; 4],
}

/// Compute face rects for a box of size w×h×d at origin (x, y) in the skin.
fn face_rects(x: f32, y: f32, w: f32, h: f32, d: f32) -> FaceRects {
    FaceRects {
        top: [x + d, y, w, d],
        bottom: [x + d + w, y, w, d],
        right: [x, y + d, d, h],
        front: [x + d, y + d, w, h],
        left: [x + d + w, y + d, d, h],
        back: [x + 2.0 * d + w, y + d, w, h],
    }
}

// Skin layout constants (pixel origins) — standard 64×64 Minecraft skin layout
fn head_rects() -> FaceRects {
    face_rects(0.0, 0.0, 8.0, 8.0, 8.0)
}
fn torso_rects() -> FaceRects {
    face_rects(0.0, 16.0, 8.0, 12.0, 4.0)
}
pub fn right_arm_rects() -> FaceRects {
    face_rects(24.0, 16.0, 4.0, 12.0, 4.0)
}
fn left_arm_rects() -> FaceRects {
    face_rects(40.0, 16.0, 4.0, 12.0, 4.0)
}
fn right_leg_rects() -> FaceRects {
    face_rects(0.0, 32.0, 4.0, 12.0, 4.0)
}
fn left_leg_rects() -> FaceRects {
    face_rects(16.0, 32.0, 4.0, 12.0, 4.0)
}

// ---------------------------------------------------------------------------
// Mesh builder — cuboid with custom per-face UVs from the skin atlas
// ---------------------------------------------------------------------------

/// Build a cuboid mesh. `size` is full extents in world units.
/// `pivot` is the local-space origin relative to the cuboid's corner
/// (0,0,0 = min corner; 1,1,1 = max corner) — used so limbs rotate about joints.
pub fn build_cuboid_mesh(fr: FaceRects, size: Vec3, pivot: Vec3) -> Mesh {
    let s = size * 0.5;
    let p = pivot * size - size * 0.5; // pivot offset from center

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Push a quad: c0=BL, c1=BR, c2=TR, c3=TL (CCW from outside).
    // UV rect [px, py, pw, ph] in pixel coords. flip_u swaps left/right for back faces.
    let mut quad = |c0: Vec3, c1: Vec3, c2: Vec3, c3: Vec3, n: Vec3, uv: [f32; 4], flip_u: bool| {
        let base = positions.len() as u32;
        let u0 = uv[0] / 64.0;
        let v0 = uv[1] / 64.0;
        let u1 = (uv[0] + uv[2]) / 64.0;
        let v1 = (uv[1] + uv[3]) / 64.0;
        // Swap u0/u1 for horizontal flip (back face).
        let (ua, ub) = if flip_u { (u1, u0) } else { (u0, u1) };
        for c in [c0, c1, c2, c3] {
            positions.push((c - p).to_array());
            normals.push(n.to_array());
        }
        // BL=(ua,v1), BR=(ub,v1), TR=(ub,v0), TL=(ua,v0)
        uvs.extend_from_slice(&[[ua, v1], [ub, v1], [ub, v0], [ua, v0]]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    };

    let (hx, hy, hz) = (s.x, s.y, s.z);

    // Front (+Z) — viewed from +Z, +X is right, +Y is up.
    quad(
        Vec3::new(-hx, -hy, hz),
        Vec3::new(hx, -hy, hz),
        Vec3::new(hx, hy, hz),
        Vec3::new(-hx, hy, hz),
        Vec3::Z,
        fr.front,
        false,
    );
    // Back (-Z) — viewed from -Z, +X is left, so flip U.
    quad(
        Vec3::new(hx, -hy, -hz),
        Vec3::new(-hx, -hy, -hz),
        Vec3::new(-hx, hy, -hz),
        Vec3::new(hx, hy, -hz),
        Vec3::NEG_Z,
        fr.back,
        true,
    );
    // Right (+X) — viewed from +X, -Z is right, +Y is up.
    quad(
        Vec3::new(hx, -hy, hz),
        Vec3::new(hx, -hy, -hz),
        Vec3::new(hx, hy, -hz),
        Vec3::new(hx, hy, hz),
        Vec3::X,
        fr.right,
        false,
    );
    // Left (-X) — viewed from -X, +Z is right, +Y is up.
    quad(
        Vec3::new(-hx, -hy, -hz),
        Vec3::new(-hx, -hy, hz),
        Vec3::new(-hx, hy, hz),
        Vec3::new(-hx, hy, -hz),
        Vec3::NEG_X,
        fr.left,
        false,
    );
    // Top (+Y) — viewed from above, +X is right, -Z is up (north).
    quad(
        Vec3::new(-hx, hy, hz),
        Vec3::new(hx, hy, hz),
        Vec3::new(hx, hy, -hz),
        Vec3::new(-hx, hy, -hz),
        Vec3::Y,
        fr.top,
        false,
    );
    // Bottom (-Y) — viewed from below, +X is right, +Z is up.
    quad(
        Vec3::new(-hx, -hy, -hz),
        Vec3::new(hx, -hy, -hz),
        Vec3::new(hx, -hy, hz),
        Vec3::new(-hx, -hy, hz),
        Vec3::NEG_Y,
        fr.bottom,
        false,
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}

// ---------------------------------------------------------------------------
// Procedural fallback skin (simple colored blocks, 64×64)
// ---------------------------------------------------------------------------

fn build_fallback_skin() -> Image {
    let mut data = vec![0u8; 64 * 64 * 4];
    // Fill whole skin with a light blue "skin" colour, then paint distinct parts.
    let paint = |data: &mut Vec<u8>, x0: usize, y0: usize, w: usize, h: usize, r: u8, g: u8, b: u8| {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                let i = (y * 64 + x) * 4;
                data[i] = r;
                data[i + 1] = g;
                data[i + 2] = b;
                data[i + 3] = 255;
            }
        }
    };
    // Head area (0,0)-(16,8): skin tone
    paint(&mut data, 0, 0, 16, 8, 220, 170, 130);
    // Face on front of head (8,8)-(16,16): skin with eyes
    paint(&mut data, 8, 8, 8, 8, 220, 170, 130);
    // Eyes
    paint(&mut data, 10, 11, 1, 1, 40, 40, 180);
    paint(&mut data, 13, 11, 1, 1, 40, 40, 180);
    // Torso (0,16)-(16,32): shirt colour
    paint(&mut data, 0, 16, 16, 16, 60, 120, 200);
    // Right arm (40,16)-(48,28) and shoulder/back (48,16)-(56,28)
    paint(&mut data, 40, 16, 16, 12, 220, 170, 130);
    // Left arm (32,48)-(40,60)
    paint(&mut data, 32, 48, 8, 12, 220, 170, 130);
    // Right leg (0,16)-(8,28) area — pants
    paint(&mut data, 0, 16, 8, 12, 50, 60, 120);
    // Left leg (16,48)-(24,60)
    paint(&mut data, 16, 48, 8, 12, 50, 60, 120);

    let mut img = Image::new(
        Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
    img
}

// ---------------------------------------------------------------------------
// CharacterRig — hierarchical humanoid
// ---------------------------------------------------------------------------

/// Marker for the root entity of a character.
#[derive(Component)]
pub struct CharacterRig {
    /// Entity refs for animation.
    pub head: Entity,
    pub torso: Entity,
    pub right_arm: Entity,
    pub left_arm: Entity,
    pub right_leg: Entity,
    pub left_leg: Entity,
    /// Material handle so we can apply a hurt flash.
    pub materials: Vec<Handle<StandardMaterial>>,
}

/// Marker so we can find/hide the local player body.
#[derive(Component)]
pub struct LocalCharacter;

/// First-person right arm (child of the camera). Holds the held item too.
#[derive(Component)]
pub struct FirstPersonArm;

/// Rest pose of the first-person arm in camera space (shoulder position).
const FP_ARM_POS: Vec3 = Vec3::new(0.45, -0.7, -0.3);
/// (yaw, pitch, roll) — YXZ euler. Points the arm up-left-forward so the
/// hand lands near screen centre-right.
const FP_ARM_EULER: (f32, f32, f32) = (0.35, 2.0, 0.1);

/// Floating name tag above a character.
#[derive(Component)]
pub struct NameTag;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkinOverride>()
            .init_resource::<SkinHandle>()
            .init_resource::<FallbackSkin>()
            .init_resource::<CharAnimState>()
            .init_resource::<CameraMode>()
            .add_systems(Startup, (load_skin, spawn_local_character).chain())
            .add_systems(
                Update,
                (
                    spawn_fp_arm,
                    skin_watchdog,
                    f6_preview,
                    rotate_preview,
                    cycle_camera_mode,
                    update_third_person_camera,
                    update_body_visibility,
                    update_fp_arm_visibility,
                    animate_character,
                    animate_first_person_arm,
                )
                    .chain(),
            );
    }
}

// ---------------------------------------------------------------------------
// Camera modes
// ---------------------------------------------------------------------------

#[derive(Resource, PartialEq, Eq, Clone, Copy, Default)]
pub enum CameraMode {
    #[default]
    FirstPerson,
    ThirdPersonBack,
    ThirdPersonFront,
}

const TP_DISTANCE: f32 = 4.0;

fn cycle_camera_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<CameraMode>,
) {
    if keys.just_pressed(KeyCode::F5) {
        *mode = match *mode {
            CameraMode::FirstPerson => CameraMode::ThirdPersonBack,
            CameraMode::ThirdPersonBack => CameraMode::ThirdPersonFront,
            CameraMode::ThirdPersonFront => CameraMode::FirstPerson,
        };
    }
}

fn update_third_person_camera(
    mode: Res<CameraMode>,
    world: Res<crate::world::World>,
    player_q: Query<&Player>,
    mut cam_q: Query<&mut Transform, (With<Camera3d>, Without<CharacterRig>)>,
) {
    let Ok(player) = player_q.single() else { return };
    let Ok(mut cam_tf) = cam_q.single_mut() else { return };

    let eye_h = if player.sneak { 1.45 } else { 1.62 };
    let eye = player.pos + Vec3::new(0.0, eye_h, 0.0);
    let rot = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);

    match *mode {
        CameraMode::FirstPerson => {
            cam_tf.translation = eye;
            cam_tf.rotation = rot;
        }
        CameraMode::ThirdPersonBack => {
            let back = (rot * Vec3::new(0.0, 0.3, TP_DISTANCE)).normalize();
            let mut dist = TP_DISTANCE;
            // Simple block collision: shorten if we'd clip into a block.
            if let Some((hit, _)) = crate::player::raycast(&world, eye, back, TP_DISTANCE + 0.5) {
                let to_hit = (hit.as_vec3() + Vec3::splat(0.5) - eye).length();
                dist = (to_hit - 0.3).max(0.5);
            }
            cam_tf.translation = eye + back * dist;
            cam_tf.rotation = rot;
        }
        CameraMode::ThirdPersonFront => {
            let front = (rot * Vec3::new(0.0, 0.3, -TP_DISTANCE)).normalize();
            let mut dist = TP_DISTANCE;
            if let Some((hit, _)) = crate::player::raycast(&world, eye, front, TP_DISTANCE + 0.5) {
                let to_hit = (hit.as_vec3() + Vec3::splat(0.5) - eye).length();
                dist = (to_hit - 0.3).max(0.5);
            }
            cam_tf.translation = eye + front * dist;
            // Look back at the player.
            cam_tf.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw + std::f32::consts::PI, -player.pitch * 0.5, 0.0);
        }
    }
}

/// Hide the local body in first person (except we keep it for shadow in third person).
fn update_body_visibility(
    mode: Res<CameraMode>,
    mut rig_q: Query<(&CharacterRig, &mut Visibility), With<LocalCharacter>>,
) {
    if !mode.is_changed() {
        return;
    }
    for (_rig, mut vis) in rig_q.iter_mut() {
        *vis = match *mode {
            CameraMode::FirstPerson => Visibility::Hidden,
            _ => Visibility::Visible,
        };
    }
}

// ---------------------------------------------------------------------------
// First-person arm
// ---------------------------------------------------------------------------

/// Spawn the first-person right arm as a child of the camera. Runs once.
fn spawn_fp_arm(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    skin: Res<SkinHandle>,
    camera_q: Query<Entity, With<Camera3d>>,
    existing_q: Query<(), With<FirstPersonArm>>,
    held_q: Query<Entity, With<HeldItemCube>>,
) {
    if !existing_q.is_empty() {
        return;
    }
    let Ok(camera) = camera_q.single() else { return };

    let mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(skin.0.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        perceptual_roughness: 0.9,
        reflectance: 0.0,
        // Large depth bias so the arm always renders in front of world geometry.
        depth_bias: 100.0,
        ..default()
    });

    let limb_size = Vec3::new(4.0 * PX, 12.0 * PX, 4.0 * PX);
    let limb_pivot = Vec3::new(0.5, 1.0, 0.5); // pivot at shoulder (top of arm)
    let arm_mesh = meshes.add(build_cuboid_mesh(right_arm_rects(), limb_size, limb_pivot));

    let arm = commands
        .spawn((
            FirstPersonArm,
            Mesh3d(arm_mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(FP_ARM_POS).with_rotation(Quat::from_euler(
                EulerRot::YXZ,
                FP_ARM_EULER.0,
                FP_ARM_EULER.1,
                FP_ARM_EULER.2,
            )),
            Visibility::default(),
            NotShadowCaster,
        ))
        .id();
    commands.entity(camera).add_child(arm);

    // The held item cube rides the arm's hand so it swings with it.
    if let Ok(held) = held_q.single() {
        commands.entity(held).set_parent_in_place(arm);
    }
}

/// The first-person arm is visible only in first person and not while the
/// inventory screen is open (the held item hides too — see visuals.rs).
fn update_fp_arm_visibility(
    mode: Res<CameraMode>,
    open: Res<InventoryOpen>,
    mut q: Query<&mut Visibility, With<FirstPersonArm>>,
) {
    if !mode.is_changed() && !open.is_changed() {
        return;
    }
    for mut vis in q.iter_mut() {
        *vis = if *mode == CameraMode::FirstPerson && !open.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Animate the first-person arm: idle sway + break/place swing.
fn animate_first_person_arm(
    time: Res<Time>,
    swing: Res<SwingTimer>,
    state: Res<CharAnimState>,
    mut q: Query<&mut Transform, With<FirstPersonArm>>,
) {
    let Ok(mut tf) = q.single_mut() else { return };

    // Idle micro-sway (breathing) — very subtle.
    let t = time.elapsed_secs();
    let idle = Vec3::new(t.sin() * 0.004, t.cos() * 0.003, 0.0);

    // Walk bob: small vertical bounce proportional to smoothed speed.
    let bob_amt = (state.speed_smooth / 4.6).clamp(0.0, 1.0) * 0.02;
    let bob = Vec3::new(0.0, (state.swing_phase * 2.0).sin() * bob_amt, 0.0);

    // Swing (break/place): rotate arm forward around its local X.
    let swing_x = if swing.0 > 0.0 {
        let prog = 1.0 - swing.0 / crate::visuals::SWING_DURATION;
        (prog * std::f32::consts::PI).sin() * 0.9
    } else {
        0.0
    };

    let base_rot = Quat::from_euler(EulerRot::YXZ, FP_ARM_EULER.0, FP_ARM_EULER.1, FP_ARM_EULER.2);
    tf.translation = FP_ARM_POS + idle + bob;
    tf.rotation = base_rot * Quat::from_rotation_x(-swing_x);
}

// ---------------------------------------------------------------------------
// Animation state
// ---------------------------------------------------------------------------

/// Smoothed animation parameters for the local character.
#[derive(Resource, Default)]
pub struct CharAnimState {
    /// Current limb swing phase (radians).
    pub swing_phase: f32,
    /// Smoothed horizontal speed (blocks/s).
    pub speed_smooth: f32,
    /// Smoothed vertical velocity.
    pub vy_smooth: f32,
    /// Smoothed sneak amount 0..1.
    pub sneak_smooth: f32,
    /// Smoothed swim amount 0..1.
    pub swim_smooth: f32,
    /// Hurt flash timer (seconds remaining).
    pub hurt_timer: f32,
    /// Death animation timer (seconds remaining, 0 = alive).
    pub death_timer: f32,
    /// Previous health for hurt detection.
    pub prev_health: f32,
}

const LERP_SPEED: f32 = 12.0;

fn lerp_f32(current: f32, target: f32, dt: f32) -> f32 {
    current + (target - current) * (1.0 - (-LERP_SPEED * dt).exp())
}

fn animate_character(
    time: Res<Time>,
    swing: Res<SwingTimer>,
    sur: Res<crate::survival::Survival>,
    mut state: ResMut<CharAnimState>,
    player_q: Query<&Player>,
    mut rig_q: Query<(&CharacterRig, &mut Transform), Without<PreviewRotation>>,
    mut part_q: Query<&mut Transform, Without<CharacterRig>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok(player) = player_q.single() else { return };
    let Ok((rig, mut root_tf)) = rig_q.single_mut() else { return };

    // Detect hurt.
    if sur.health < state.prev_health {
        state.hurt_timer = 0.4;
    }
    state.prev_health = sur.health;
    state.hurt_timer = (state.hurt_timer - dt).max(0.0);

    // Detect death.
    if sur.health <= 0.0 && state.death_timer <= 0.0 {
        state.death_timer = 1.5;
    }
    state.death_timer = (state.death_timer - dt).max(0.0);

    // Horizontal speed.
    let hspeed = Vec2::new(player.vel.x, player.vel.z).length();
    state.speed_smooth = lerp_f32(state.speed_smooth, hspeed, dt);
    state.vy_smooth = lerp_f32(state.vy_smooth, player.vel.y, dt);
    state.sneak_smooth = lerp_f32(state.sneak_smooth, if player.sneak { 1.0 } else { 0.0 }, dt);

    // Swim: in water if feet near sea level and moving slowly downward.
    let in_water = player.pos.y < crate::terrain::SEA_LEVEL as f32 + 0.5;
    state.swim_smooth = lerp_f32(state.swim_smooth, if in_water { 1.0 } else { 0.0 }, dt);

    // Advance swing phase based on speed.
    let swing_freq = if state.speed_smooth > 5.0 { 10.0 } else { 7.0 };
    state.swing_phase += dt * swing_freq * (state.speed_smooth / 4.6).clamp(0.0, 2.0);
    if state.speed_smooth < 0.1 {
        // Ease swing back to rest.
        state.swing_phase = lerp_f32(state.swing_phase, 0.0, dt * 2.0);
    }

    let swing_amt = state.swing_phase.sin();
    let sneak = state.sneak_smooth;
    let swim = state.swim_smooth;

    // --- Root rotation: face movement direction ---
    // (Root already positioned by player pos update elsewhere; we just rotate.)

    // --- Limb rotations ---
    // Leg swing: opposite phase for left/right.
    let leg_angle = swing_amt * 0.7 * (1.0 - sneak * 0.5) * (1.0 - swim);
    // Arm swing: same phase as same-side leg (opposite to arm in Minecraft? — same side).
    let arm_angle = swing_amt * 0.6 * (1.0 - sneak * 0.3);

    // Sneak: bend torso forward slightly, crouch legs.
    let sneak_torso = sneak * 0.3;
    let sneak_leg = sneak * 0.5;

    // Jump/fall: arms up when falling fast, legs tucked.
    let fall_amt = (-state.vy_smooth / 18.0).clamp(0.0, 1.0);
    let jump_amt = (state.vy_smooth / 8.4).clamp(0.0, 1.0);

    // Swim: arms and legs spread horizontally.
    let swim_arm = swim * 1.2;
    let swim_leg = swim * 0.4;

    // Death: rotate whole body 90° forward (fall over).
    let death = if state.death_timer > 0.0 {
        1.0 - (state.death_timer / 1.5)
    } else {
        0.0
    };
    // Follow the player: position at feet, body faces camera yaw.
    root_tf.translation = player.pos;
    root_tf.rotation = Quat::from_rotation_y(player.yaw + std::f32::consts::PI)
        * Quat::from_rotation_x(death * std::f32::consts::FRAC_PI_2);

    // Head pitch follows camera.
    if let Ok(mut head_tf) = part_q.get_mut(rig.head) {
        head_tf.rotation = Quat::from_rotation_x(player.pitch * 0.5);
    }

    // Torso: slight forward lean when sneaking.
    if let Ok(mut torso_tf) = part_q.get_mut(rig.torso) {
        let target = Quat::from_rotation_x(sneak_torso);
        torso_tf.rotation = torso_tf.rotation.lerp(target, 1.0 - (-8.0 * dt).exp());
    }

    // Right arm.
    if let Ok(mut tf) = part_q.get_mut(rig.right_arm) {
        let mut angle = arm_angle;
        // Swing timer overrides (break/place).
        if swing.0 > 0.0 {
            let t = 1.0 - swing.0 / 0.25;
            angle = -t * std::f32::consts::FRAC_PI_2;
        }
        angle += fall_amt * 2.5 + swim_arm;
        let target = Quat::from_rotation_x(angle);
        tf.rotation = tf.rotation.lerp(target, 1.0 - (-10.0 * dt).exp());
    }

    // Left arm.
    if let Ok(mut tf) = part_q.get_mut(rig.left_arm) {
        let mut angle = -arm_angle;
        angle += fall_amt * 2.5 + swim_arm;
        let target = Quat::from_rotation_x(angle);
        tf.rotation = tf.rotation.lerp(target, 1.0 - (-10.0 * dt).exp());
    }

    // Right leg.
    if let Ok(mut tf) = part_q.get_mut(rig.right_leg) {
        let mut angle = leg_angle + sneak_leg;
        angle -= jump_amt * 0.5;
        angle += swim_leg;
        let target = Quat::from_rotation_x(angle);
        tf.rotation = tf.rotation.lerp(target, 1.0 - (-10.0 * dt).exp());
    }

    // Left leg.
    if let Ok(mut tf) = part_q.get_mut(rig.left_leg) {
        let mut angle = -leg_angle + sneak_leg;
        angle -= jump_amt * 0.5;
        angle += swim_leg;
        let target = Quat::from_rotation_x(angle);
        tf.rotation = tf.rotation.lerp(target, 1.0 - (-10.0 * dt).exp());
    }

    // --- Hurt flash: tint material red ---
    for mat_handle in &rig.materials {
        if let Some(mut mat) = materials.get_mut(mat_handle) {
            let flash = if state.hurt_timer > 0.0 {
                (state.hurt_timer / 0.4).sin().abs() * 0.8
            } else {
                0.0
            };
            mat.base_color = Color::srgb(1.0, 1.0 - flash, 1.0 - flash);
        }
    }
}

/// CLI skin path override, parsed from `--skin <path>`.
#[derive(Resource, Default)]
pub struct SkinOverride(pub Option<String>);

// ---------------------------------------------------------------------------
// Skin loading
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct SkinHandle(pub Handle<Image>);

#[derive(Resource, Default)]
pub struct FallbackSkin(pub Handle<Image>);

fn load_skin(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    skin_override: Res<SkinOverride>,
) {
    let path = skin_override.0.clone().unwrap_or_else(|| SKIN_PATH.to_string());
    let handle: Handle<Image> = asset_server.load(&path);

    let fallback = build_fallback_skin();
    let fallback_handle = images.add(fallback);

    commands.insert_resource(SkinHandle(handle.clone()));
    commands.insert_resource(FallbackSkin(fallback_handle));

    commands.spawn(SkinLoadWatchdog { handle, last_print: -10.0, last_state: 0 });
}

#[derive(Component)]
struct SkinLoadWatchdog {
    handle: Handle<Image>,
    last_print: f32,
    last_state: u8, // 0=NotLoaded,1=Loading,2=Loaded,3=Failed
}

fn skin_watchdog(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    fallback: Res<FallbackSkin>,
    mut q: Query<(Entity, &mut SkinLoadWatchdog)>,
    mut skin: ResMut<SkinHandle>,
) {
    for (entity, mut watchdog) in q.iter_mut() {
        let state = asset_server.load_state(&watchdog.handle);
        let elapsed = time.elapsed_secs();

        let (state_code, state_str) = match state {
            bevy::asset::LoadState::NotLoaded => (0u8, "NotLoaded"),
            bevy::asset::LoadState::Loading => (1u8, "Loading"),
            bevy::asset::LoadState::Loaded => (2u8, "Loaded"),
            bevy::asset::LoadState::Failed(_) => (3u8, "Failed"),
        };

        // Print on state change or once per second while loading.
        let state_changed = state_code != watchdog.last_state;
        let periodic = elapsed - watchdog.last_print >= 1.0;
        if state_changed || periodic {
            watchdog.last_print = elapsed;
            watchdog.last_state = state_code;
            let path = asset_server
                .get_path(&watchdog.handle)
                .map(|p| p.path().display().to_string())
                .unwrap_or_else(|| "<unknown>".into());
            info!("[skin] load state = {state_str}, path = {path}");
        }

        match state {
            bevy::asset::LoadState::Loaded => {
                // Apply nearest sampler.
                if let Some(mut img) = images.get_mut(&watchdog.handle) {
                    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
                }
                commands.entity(entity).despawn();
            }
            bevy::asset::LoadState::Failed(err) => {
                warn!("[skin] load FAILED: {err} — swapping to fallback skin");
                skin.0 = fallback.0.clone();
                commands.entity(entity).despawn();
            }
            _ => {
                // Still loading — keep waiting. No timeout.
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Build the humanoid rig
// ---------------------------------------------------------------------------

/// Public API: spawn a character rig at `translation` using `skin_handle`.
/// Returns the root entity. Used for both local player and remote players.
/// If `name` is provided, a floating name tag is added above the head.
pub fn spawn_character(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    skin_handle: &Handle<Image>,
    translation: Vec3,
    name: Option<&str>,
) -> Entity {
    let mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(skin_handle.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        perceptual_roughness: 0.9,
        reflectance: 0.0,
        ..default()
    });

    // Dimensions in world units (px * PX). 16 px = 1 block.
    let head_size = Vec3::splat(8.0 * PX);           // 0.5^3
    let torso_size = Vec3::new(8.0 * PX, 12.0 * PX, 4.0 * PX); // 0.5 x 0.75 x 0.25
    let limb_size = Vec3::new(4.0 * PX, 12.0 * PX, 4.0 * PX);  // 0.25 x 0.75 x 0.25

    // Pivot: head at bottom-centre (neck), torso at top-centre, limbs at top-centre (shoulder/hip).
    let head_pivot = Vec3::new(0.5, 0.0, 0.5);
    let torso_pivot = Vec3::new(0.5, 1.0, 0.5);
    let limb_pivot = Vec3::new(0.5, 1.0, 0.5);

    let head_mesh = meshes.add(build_cuboid_mesh(head_rects(), head_size, head_pivot));
    let torso_mesh = meshes.add(build_cuboid_mesh(torso_rects(), torso_size, torso_pivot));
    let r_arm_mesh = meshes.add(build_cuboid_mesh(right_arm_rects(), limb_size, limb_pivot));
    let l_arm_mesh = meshes.add(build_cuboid_mesh(left_arm_rects(), limb_size, limb_pivot));
    let r_leg_mesh = meshes.add(build_cuboid_mesh(right_leg_rects(), limb_size, limb_pivot));
    let l_leg_mesh = meshes.add(build_cuboid_mesh(left_leg_rects(), limb_size, limb_pivot));

    // Key heights (feet at y=0):
    //   legs:  0 .. 0.75   (hip pivot at y=0.75)
    //   torso: 0.75 .. 1.5 (centre at 1.125, top at 1.5 = shoulder)
    //   head:  1.5 .. 2.0  (neck pivot at y=1.5, centre at 1.75)
    let hip_y = 0.75;       // top of legs / bottom of torso
    let shoulder_y = 1.5;   // top of torso / bottom of head

    // Root
    let root = commands
        .spawn((
            Transform::from_translation(translation),
            Visibility::default(),
        ))
        .id();

    // Torso: pivot at top (shoulder_y), mesh hangs down to hip_y.
    let torso = commands
        .spawn((
            Mesh3d(torso_mesh),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(0.0, shoulder_y, 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(root).add_child(torso);

    // Head: pivot at bottom (neck = shoulder_y), mesh extends up to shoulder_y + 0.5.
    let head = commands
        .spawn((
            Mesh3d(head_mesh),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(0.0, shoulder_y, 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(root).add_child(head);

    // Arms — pivot at shoulder (top of arm). Shoulder at shoulder_y.
    // Player's right arm is at -X (when facing -Z forward).
    let arm_x = 6.0 * PX; // 0.375 = half torso width + half arm width
    let right_arm = commands
        .spawn((
            Mesh3d(r_arm_mesh),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(-arm_x, shoulder_y, 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(root).add_child(right_arm);

    let left_arm = commands
        .spawn((
            Mesh3d(l_arm_mesh),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(arm_x, shoulder_y, 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(root).add_child(left_arm);

    // Legs — pivot at hip (top of leg). Hip at hip_y = 0.75.
    let leg_x = 2.0 * PX; // 0.125
    let right_leg = commands
        .spawn((
            Mesh3d(r_leg_mesh),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(-leg_x, hip_y, 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(root).add_child(right_leg);

    let left_leg = commands
        .spawn((
            Mesh3d(l_leg_mesh),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(leg_x, hip_y, 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(root).add_child(left_leg);

    commands.entity(root).insert(CharacterRig {
        head,
        torso,
        right_arm,
        left_arm,
        right_leg,
        left_leg,
        materials: vec![mat],
    });

    // Floating name tag above the head (head top = 2.0).
    if let Some(name) = name {
        let tag_y = 2.0 + 0.3;
        commands.spawn((
            NameTag,
            Text::new(name.to_string()),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            // Billboard: we'll just offset above; simple approach.
            Transform::from_xyz(0.0, tag_y, 0.0),
        ))
        .set_parent_in_place(root);
    }

    root
}

// ---------------------------------------------------------------------------
// Local player character
// ---------------------------------------------------------------------------

fn spawn_local_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    skin: Res<SkinHandle>,
    player_q: Query<&Player>,
) {
    let Ok(player) = player_q.single() else { return };
    let root = spawn_character(
        &mut commands,
        &mut meshes,
        &mut materials,
        &skin.0,
        player.pos,
        None,
    );
    commands.entity(root).insert(LocalCharacter);
}

// ---------------------------------------------------------------------------
// F6 — rotating preview
// ---------------------------------------------------------------------------

#[derive(Component)]
struct PreviewRotation;

fn f6_preview(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    skin: Res<SkinHandle>,
    player_q: Query<(&Player, &Transform)>,
    preview_q: Query<Entity, With<PreviewRotation>>,
) {
    if !keys.just_pressed(KeyCode::F6) {
        return;
    }
    // Toggle: despawn existing preview if any.
    if !preview_q.is_empty() {
        info!("[F6] despawning existing preview");
        for e in preview_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    }
    let Ok((player, cam_tf)) = player_q.single() else {
        warn!("[F6] no player found for preview");
        return;
    };
    let eye_h = if player.sneak { 1.45 } else { 1.62 };
    let eye = player.pos + Vec3::new(0.0, eye_h, 0.0);
    let forward = cam_tf.rotation * Vec3::NEG_Z;
    let preview_pos = eye + forward * 3.0;
    info!("[F6] spawning preview at {preview_pos:?}, player.pos={:?}", player.pos);

    let root = spawn_character(
        &mut commands,
        &mut meshes,
        &mut materials,
        &skin.0,
        preview_pos,
        Some("Preview"),
    );
    commands.entity(root).insert(PreviewRotation);
}

fn rotate_preview(
    time: Res<Time>,
    mut q: Query<&mut Transform, With<PreviewRotation>>,
) {
    for mut tf in q.iter_mut() {
        tf.rotation *= Quat::from_rotation_y(time.delta_secs() * 1.5);
    }
}
