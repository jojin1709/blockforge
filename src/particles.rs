use bevy::prelude::*;
use rand::Rng;

use crate::blocks::def;
use crate::mining::BreakEvents;

const PARTICLE_COUNT: usize = 10;
const PARTICLE_LIFE: f32 = 0.5;
const GRAVITY: f32 = 12.0;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_break_particles, update_particles).chain());
    }
}

#[derive(Component)]
pub struct Particle {
    vel: Vec3,
    life: f32,
    max_life: f32,
}

fn spawn_break_particles(
    mut commands: Commands,
    mut events: ResMut<BreakEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if events.0.is_empty() {
        return;
    }
    let mut rng = rand::rng();
    let mesh = meshes.add(Cuboid::from_size(Vec3::splat(0.15)));

    for (pos, block_id) in events.0.drain(..) {
        let color = def(block_id).side;
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(color[0], color[1], color[2]),
            unlit: true,
            ..default()
        });

        for _ in 0..PARTICLE_COUNT {
            let offset = Vec3::new(
                rng.random_range(-0.3..0.3),
                rng.random_range(-0.3..0.3),
                rng.random_range(-0.3..0.3),
            );
            let vel = Vec3::new(
                rng.random_range(-2.0..2.0),
                rng.random_range(1.0..4.0),
                rng.random_range(-2.0..2.0),
            );
            let scale = rng.random_range(0.6..1.2);

            commands.spawn((
                Particle { vel, life: PARTICLE_LIFE, max_life: PARTICLE_LIFE },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat.clone()),
                Transform {
                    translation: pos.as_vec3() + Vec3::splat(0.5) + offset,
                    scale: Vec3::splat(scale),
                    ..default()
                },
            ));
        }
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut particle, mut tf) in q.iter_mut() {
        particle.life -= dt;
        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        particle.vel.y -= GRAVITY * dt;
        tf.translation += particle.vel * dt;
        let alpha = particle.life / particle.max_life;
        tf.scale *= 0.97;
        tf.scale *= 0.5 + alpha * 0.5;
    }
}
