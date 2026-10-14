use bevy::input::mouse::MouseMotion;
use bevy::math::EulerRot;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::blocks::{def, AIR, BEDROCK, WATER};
use crate::inventory::{as_block, break_drop, APPLE, Inventory, InventoryOpen};
use crate::survival::Survival;
use crate::terrain::{surface_height, SEA_LEVEL, WORLD_HEIGHT};
use crate::world::{World, SEED};

const HALF: f32 = 0.3;
const BODY_H: f32 = 1.8;
const EYE: f32 = 1.62;
const EYE_SNEAK: f32 = 1.45;
const EPS: f32 = 0.001;
const SENS: f32 = 0.0022;
const GRAVITY: f32 = 24.0;
const JUMP: f32 = 8.4;
const TERMINAL: f32 = -18.0;
const WALK: f32 = 4.6;
const SPRINT: f32 = 7.2;
const SNEAK: f32 = 1.5;
const REACH: f32 = 6.0;

#[derive(Component)]
pub struct Player {
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub sneak: bool,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_player, grab_cursor))
            .add_systems(Update, player_control);
    }
}

fn spawn_player(mut commands: Commands) {
    let h = surface_height(SEED, 0, 0).max(SEA_LEVEL) as f32;
    commands.spawn((
        Player {
            pos: Vec3::new(0.5, h + 1.05, 0.5),
            vel: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            sneak: false,
        },
        Camera3d::default(),
        Transform::from_xyz(0.5, h + 1.05 + EYE, 0.5),
    ));
}

fn grab_cursor(mut q: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if let Ok(mut co) = q.single_mut() {
        co.grab_mode = CursorGrabMode::Locked;
        co.visible = false;
    }
}

fn collides(world: &World, feet: Vec3) -> bool {
    let min = feet + Vec3::new(-HALF + EPS, EPS, -HALF + EPS);
    let max = feet + Vec3::new(HALF - EPS, BODY_H - EPS, HALF - EPS);
    for bx in (min.x.floor() as i32)..=(max.x.floor() as i32) {
        for by in (min.y.floor() as i32)..=(max.y.floor() as i32) {
            for bz in (min.z.floor() as i32)..=(max.z.floor() as i32) {
                if def(world.block(bx, by, bz)).solid {
                    return true;
                }
            }
        }
    }
    false
}

fn block_intersects_player(feet: Vec3, b: IVec3) -> bool {
    let min = feet + Vec3::new(-HALF, 0.0, -HALF);
    let max = feet + Vec3::new(HALF, BODY_H, HALF);
    let bmin = b.as_vec3();
    let bmax = bmin + Vec3::ONE;
    min.x < bmax.x && max.x > bmin.x && min.y < bmax.y && max.y > bmin.y && min.z < bmax.z
        && max.z > bmin.z
}

fn raycast(world: &World, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<(IVec3, IVec3)> {
    let mut v = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );
    let step = [
        dir.x.signum() as i32,
        dir.y.signum() as i32,
        dir.z.signum() as i32,
    ];
    let t_delta = [
        if dir.x.abs() < 1e-6 { f32::INFINITY } else { 1.0 / dir.x.abs() },
        if dir.y.abs() < 1e-6 { f32::INFINITY } else { 1.0 / dir.y.abs() },
        if dir.z.abs() < 1e-6 { f32::INFINITY } else { 1.0 / dir.z.abs() },
    ];
    let mut t_max = [f32::INFINITY; 3];
    let o = [origin.x, origin.y, origin.z];
    let vv = [v.x as f32, v.y as f32, v.z as f32];
    for a in 0..3 {
        if step[a] > 0 {
            t_max[a] = (vv[a] + 1.0 - o[a]) / dir[a].abs();
        } else if step[a] < 0 {
            t_max[a] = (o[a] - vv[a]) / dir[a].abs();
        }
    }

    let mut prev = v;
    let mut t = 0.0f32;
    for _ in 0..256 {
        let id = world.block(v.x, v.y, v.z);
        if id != AIR && id != WATER && def(id).solid {
            return Some((v, prev));
        }
        if t > max_dist {
            return None;
        }
        let axis = if t_max[0] <= t_max[1] && t_max[0] <= t_max[2] {
            0
        } else if t_max[1] <= t_max[2] {
            1
        } else {
            2
        };
        t = t_max[axis];
        t_max[axis] += t_delta[axis];
        prev = v;
        match axis {
            0 => v.x += step[0],
            1 => v.y += step[1],
            _ => v.z += step[2],
        }
        if v.y < -1 || v.y > WORLD_HEIGHT + 2 {
            return None;
        }
    }
    None
}

pub fn player_control(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut world: ResMut<World>,
    mut inv: ResMut<Inventory>,
    mut sur: ResMut<Survival>,
    mut open: ResMut<InventoryOpen>,
    mut q: Query<(&mut Player, &mut Transform)>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let grabbed = cursor_q
        .single()
        .map(|co| co.grab_mode != CursorGrabMode::None)
        .unwrap_or(false);

    let mut look = Vec2::ZERO;
    for ev in motion.read() {
        if grabbed {
            look += ev.delta;
        }
    }

    let Ok(mut co) = cursor_q.single_mut() else { return };
    if keys.just_pressed(KeyCode::Escape) {
        if open.0 {
            open.0 = false;
            co.grab_mode = CursorGrabMode::Locked;
            co.visible = false;
        } else if co.grab_mode == CursorGrabMode::None {
            co.grab_mode = CursorGrabMode::Locked;
            co.visible = false;
        } else {
            co.grab_mode = CursorGrabMode::None;
            co.visible = true;
        }
    }
    drop(co);

    let Ok((mut player, mut transform)) = q.single_mut() else { return };

    player.yaw -= look.x * SENS;
    player.pitch = (player.pitch - look.y * SENS).clamp(-1.55, 1.55);
    let rot = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
    let forward = Vec3::new(-player.yaw.sin(), 0.0, -player.yaw.cos());
    let right = Vec3::new(player.yaw.cos(), 0.0, -player.yaw.sin());

    let eye_h = if player.sneak { EYE_SNEAK } else { EYE };
    let eye = player.pos + Vec3::new(0.0, eye_h, 0.0);
    let dir = rot * Vec3::NEG_Z;

    // While inventory is open: no world interaction.
    if !open.0 && grabbed && mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some((hit, _)) = raycast(&world, eye, dir, REACH) {
            let id = world.block(hit.x, hit.y, hit.z);
            if id != AIR && id != WATER && id != BEDROCK && def(id).solid {
                world.set_block(hit.x, hit.y, hit.z, AIR);
                if let Some(drop) = break_drop(id) {
                    inv.add(drop, 1);
                }
            }
        }
    }
    if !open.0 && grabbed && mouse_buttons.just_pressed(MouseButton::Right) {
        let held = inv.slots[inv.selected];
        if let Some(stack) = held {
            if stack.id == APPLE {
                if sur.hunger < 20.0 {
                    sur.hunger = (sur.hunger + 4.0).min(20.0);
                    inv.consume_selected();
                }
            } else if let Some(b) = as_block(stack.id) {
                if let Some((_, prev)) = raycast(&world, eye, dir, REACH) {
                    let cur = world.block(prev.x, prev.y, prev.z);
                    if (cur == AIR || cur == WATER)
                        && !block_intersects_player(player.pos, prev)
                        && prev.y >= 0
                        && prev.y < WORLD_HEIGHT
                    {
                        world.set_block(prev.x, prev.y, prev.z, b);
                        inv.consume_selected();
                    }
                }
            }
        }
    }

    let chunk_x = (player.pos.x.floor() as i32).div_euclid(16);
    let chunk_z = (player.pos.z.floor() as i32).div_euclid(16);
    if !world.column_loaded(chunk_x, chunk_z) {
        transform.translation = eye;
        transform.rotation = rot;
        return;
    }

    let dt = time.delta_secs().min(0.05);
    let feet_block = world.block(
        player.pos.x.floor() as i32,
        (player.pos.y + 0.4).floor() as i32,
        player.pos.z.floor() as i32,
    );
    let in_water = feet_block == WATER;

    let mut wish = Vec3::ZERO;
    let mut f = 0.0;
    let mut s = 0.0;
    if grabbed && !open.0 {
        if keys.pressed(KeyCode::KeyW) { f += 1.0; }
        if keys.pressed(KeyCode::KeyS) { f -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { s += 1.0; }
        if keys.pressed(KeyCode::KeyA) { s -= 1.0; }
        wish = forward * f + right * s;
        if wish.length_squared() > 0.0 {
            wish = wish.normalize();
        }
    }

    let sneak = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let sprint = (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
        && f > 0.0
        && !sneak
        && !in_water;
    player.sneak = sneak;

    let mut speed = if sneak {
        SNEAK
    } else if sprint {
        SPRINT
    } else {
        WALK
    };
    if in_water {
        speed *= 0.55;
    }
    player.vel.x = wish.x * speed;
    player.vel.z = wish.z * speed;

    if in_water {
        player.vel.y -= GRAVITY * 0.25 * dt;
        if player.vel.y < -2.5 {
            player.vel.y = -2.5;
        }
        if grabbed && keys.pressed(KeyCode::Space) {
            player.vel.y = 3.8;
        } else if player.vel.y < 0.0 {
            player.vel.y *= 0.9;
        }
        sur.fall_peak = player.pos.y;
    } else {
        player.vel.y -= GRAVITY * dt;
        if player.vel.y < TERMINAL {
            player.vel.y = TERMINAL;
        }
        if grabbed && keys.pressed(KeyCode::Space) && player.on_ground {
            player.vel.y = JUMP;
            player.on_ground = false;
        }
        if !player.on_ground && player.vel.y <= 0.0 {
            sur.fall_peak = sur.fall_peak.max(player.pos.y);
        }
    }

    // Move X.
    let mut feet = player.pos;
    feet.x += player.vel.x * dt;
    if collides(&world, feet) {
        if player.vel.x > 0.0 {
            feet.x = (feet.x + HALF).floor() - HALF - EPS;
        } else {
            feet.x = (feet.x - HALF).floor() + 1.0 + HALF + EPS;
        }
        player.vel.x = 0.0;
    }
    // Move Z.
    feet.z += player.vel.z * dt;
    if collides(&world, feet) {
        if player.vel.z > 0.0 {
            feet.z = (feet.z + HALF).floor() - HALF - EPS;
        } else {
            feet.z = (feet.z - HALF).floor() + 1.0 + HALF + EPS;
        }
        player.vel.z = 0.0;
    }
    // Move Y.
    player.on_ground = false;
    feet.y += player.vel.y * dt;
    if collides(&world, feet) {
        if player.vel.y <= 0.0 {
            feet.y = feet.y.floor() + 1.0 + EPS;
            player.on_ground = true;
            // Fall damage on landing.
            if !in_water {
                let fall = sur.fall_peak - feet.y;
                if fall > 3.0 {
                    sur.health -= (fall - 3.0).floor().max(1.0);
                }
            }
            sur.fall_peak = feet.y;
        } else {
            feet.y = (feet.y + BODY_H).floor() - BODY_H - EPS;
        }
        player.vel.y = 0.0;
    }
    if feet.y < -10.0 {
        feet = Vec3::new(0.5, surface_height(SEED, 0, 0) as f32 + 2.0, 0.5);
        player.vel = Vec3::ZERO;
    }

    player.pos = feet;
    let eye_h = if player.sneak { EYE_SNEAK } else { EYE };
    transform.translation = feet + Vec3::new(0.0, eye_h, 0.0);
    transform.rotation = rot;
}
