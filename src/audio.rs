use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{
    AddAudioSource, AudioPlayer, ChannelCount, Decodable, PlaybackSettings, SampleRate, Source,
    Volume,
};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use rand::Rng;

use crate::blocks::{
    BlockId, AIR, BEDROCK, COAL_ORE, COBBLE, CRAFTING_TABLE, DIRT, GRAVEL, GRASS, IRON_ORE,
    LEAVES, LOG, PLANKS, SAND, SANDSTONE, SNOW, STONE, WATER,
};
use crate::inventory::InventoryOpen;
use crate::pause::PauseOpen;
use crate::player::Player;
use crate::survival::Survival;
use crate::world::World;

const SAMPLE_RATE: u32 = 44_100;

/// A code-synthesized sound clip: mono f32 samples at 44.1 kHz.
#[derive(Asset, TypePath)]
pub struct SynthAudio {
    samples: Arc<[f32]>,
}

pub struct SampleDecoder {
    samples: Arc<[f32]>,
    pos: usize,
}

impl Iterator for SampleDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let s = *self.samples.get(self.pos)?;
        self.pos += 1;
        Some(s)
    }
}

impl Source for SampleDecoder {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.samples.len().saturating_sub(self.pos))
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(1).unwrap()
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(SAMPLE_RATE).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.samples.len() as f64 / SAMPLE_RATE as f64,
        ))
    }
}

impl Decodable for SynthAudio {
    type Decoder = SampleDecoder;

    fn decoder(&self) -> Self::Decoder {
        SampleDecoder {
            samples: self.samples.clone(),
            pos: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoundEvent {
    Break(BlockId),
    Place(BlockId),
    Footstep(BlockId),
    Eat,
    Hurt,
    Splash,
}

#[derive(Resource, Default)]
pub struct SoundQueue(pub Vec<SoundEvent>);

#[derive(Resource)]
pub struct MasterVolume {
    pub volume: f32,
    pub muted: bool,
}

impl Default for MasterVolume {
    fn default() -> Self {
        MasterVolume {
            volume: 0.8,
            muted: false,
        }
    }
}

impl MasterVolume {
    fn effective(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume.clamp(0.0, 1.0)
        }
    }
}

#[derive(Resource)]
pub struct SoundBank {
    dig: [Handle<SynthAudio>; 5],
    place: Handle<SynthAudio>,
    footstep: Handle<SynthAudio>,
    eat: Handle<SynthAudio>,
    hurt: Handle<SynthAudio>,
    splash: Handle<SynthAudio>,
}

#[derive(Resource, Default)]
struct VolumeHudTimer(f32);

#[derive(Component)]
struct VolumeHudText;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundQueue>()
            .init_resource::<MasterVolume>()
            .init_resource::<VolumeHudTimer>()
            .add_audio_source::<SynthAudio>()
            .add_systems(Startup, (setup_sounds, setup_volume_hud))
            .add_systems(
                Update,
                (play_queued_sounds, gameplay_sounds, volume_keys, volume_hud).chain(),
            );
    }
}

#[derive(Clone, Copy)]
enum Material {
    Grass = 0,
    Stone,
    Wood,
    Sand,
    Leaves,
}

fn material_of(id: BlockId) -> Material {
    match id {
        GRASS | DIRT | SNOW => Material::Grass,
        STONE | COBBLE | COAL_ORE | IRON_ORE | BEDROCK | SANDSTONE | GRAVEL => Material::Stone,
        LOG | PLANKS | CRAFTING_TABLE => Material::Wood,
        SAND => Material::Sand,
        LEAVES => Material::Leaves,
        _ => Material::Grass,
    }
}

struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_f32(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
    }
}

/// Render `dur` seconds through `f(t)`, with a 4 ms fade-out to kill clicks.
fn render(dur: f32, mut f: impl FnMut(f32) -> f32) -> Vec<f32> {
    let n = (dur * SAMPLE_RATE as f32) as usize;
    let fade = (0.004 * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let mut s = f(t);
        if i + fade > n {
            s *= (n - i) as f32 / fade.max(1) as f32;
        }
        out.push(s.clamp(-1.0, 1.0));
    }
    out
}

fn mix_at(acc: &mut Vec<f32>, src: &[f32], offset_s: f32) {
    let off = (offset_s * SAMPLE_RATE as f32) as usize;
    if acc.len() < off + src.len() {
        acc.resize(off + src.len(), 0.0);
    }
    for (d, s) in acc[off..].iter_mut().zip(src) {
        *d += s;
    }
}

fn finalize(mut v: Vec<f32>) -> Vec<f32> {
    let peak = v.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
    if peak > 0.95 {
        for s in &mut v {
            *s *= 0.95 / peak;
        }
    }
    v
}

fn sine_burst(dur: f32, f0: f32, f1: f32, decay_k: f32, amp: f32) -> Vec<f32> {
    render(dur, |t| {
        let u = t / dur;
        let phase =
            std::f32::consts::TAU * (f0 * t + (f1 - f0) * t * t / (2.0 * dur));
        phase.sin() * (-decay_k * u).exp() * amp
    })
}

fn clip_dig(material: Material) -> Vec<f32> {
    let mut rng = Lcg::new(0xC0FFEE ^ (material as u64));
    match material {
        Material::Grass => {
            let mut lp = 0.0f32;
            let noise = render(0.18, |t| {
                let env = (-4.5 * t / 0.18).exp();
                lp += 0.12 * (rng.next_f32() - lp);
                lp * env * 0.75
            });
            let thump = sine_burst(0.08, 95.0, 70.0, 5.0, 0.5);
            let mut acc = noise;
            mix_at(&mut acc, &thump, 0.0);
            finalize(acc)
        }
        Material::Stone => {
            let mut lp = 0.0f32;
            let noise = render(0.2, |t| {
                let env = (-6.5 * t / 0.2).exp();
                lp += 0.55 * (rng.next_f32() - lp);
                lp * env * 0.7
            });
            let thump = sine_burst(0.07, 170.0, 120.0, 6.0, 0.45);
            let mut acc = noise;
            mix_at(&mut acc, &thump, 0.0);
            finalize(acc)
        }
        Material::Wood => {
            let (mut f1, mut f2) = (0.0f32, 0.0f32);
            let noise = render(0.16, |t| {
                let env = (-5.0 * t / 0.16).exp();
                let x = rng.next_f32();
                f1 += 0.25 * (x - f1);
                f2 += 0.03 * (f1 - f2);
                (f1 - f2) * env * 1.2
            });
            let tone = sine_burst(0.1, 210.0, 150.0, 5.5, 0.4);
            let mut acc = noise;
            mix_at(&mut acc, &tone, 0.0);
            finalize(acc)
        }
        Material::Sand => {
            let mut lp = 0.0f32;
            let noise = render(0.14, |t| {
                let env = (-5.5 * t / 0.14).exp();
                let x = rng.next_f32();
                lp += 0.35 * (x - lp);
                (x - lp) * env * 0.55
            });
            finalize(noise)
        }
        Material::Leaves => {
            let mut lp = 0.0f32;
            let noise = render(0.12, |t| {
                let env = (-7.0 * t / 0.12).exp();
                let x = rng.next_f32();
                lp += 0.5 * (x - lp);
                (x - lp) * env * 0.5
            });
            finalize(noise)
        }
    }
}

fn clip_place() -> Vec<f32> {
    let mut rng = Lcg::new(0xBEEF);
    let tone = sine_burst(0.1, 95.0, 65.0, 6.0, 0.85);
    let mut lp = 0.0f32;
    let noise = render(0.08, |t| {
        let env = (-9.0 * t / 0.08).exp();
        lp += 0.3 * (rng.next_f32() - lp);
        lp * env * 0.4
    });
    let mut acc = tone;
    mix_at(&mut acc, &noise, 0.0);
    finalize(acc)
}

fn clip_footstep() -> Vec<f32> {
    let mut rng = Lcg::new(0xF00D);
    let mut lp = 0.0f32;
    let noise = render(0.09, |t| {
        let env = (-8.0 * t / 0.09).exp();
        lp += 0.15 * (rng.next_f32() - lp);
        lp * env * 0.8
    });
    let thump = sine_burst(0.06, 75.0, 55.0, 7.0, 0.55);
    let mut acc = noise;
    mix_at(&mut acc, &thump, 0.0);
    finalize(acc)
}

fn clip_eat() -> Vec<f32> {
    let mut acc = Vec::new();
    for i in 0..3u32 {
        let f = 340.0 - i as f32 * 40.0;
        let blip = sine_burst(0.09, f, f * 0.75, 6.0, 0.65);
        mix_at(&mut acc, &blip, 0.04 + i as f32 * 0.11);
    }
    finalize(acc)
}

fn clip_hurt() -> Vec<f32> {
    let mut rng = Lcg::new(0xDEAD);
    let tone = sine_burst(0.22, 300.0, 140.0, 4.5, 0.65);
    let mut lp = 0.0f32;
    let noise = render(0.18, |t| {
        let env = (-5.0 * t / 0.18).exp();
        lp += 0.2 * (rng.next_f32() - lp);
        lp * env * 0.35
    });
    let mut acc = tone;
    mix_at(&mut acc, &noise, 0.0);
    finalize(acc)
}

fn clip_splash() -> Vec<f32> {
    let mut rng = Lcg::new(0xA11A);
    let mut lp = 0.0f32;
    let noise = render(0.3, |t| {
        let env = (-3.5 * t / 0.3).exp();
        let x = rng.next_f32();
        lp += 0.4 * (x - lp);
        (x - lp * 0.5) * env * 0.6
    });
    let sweep = sine_burst(0.25, 420.0, 130.0, 3.0, 0.35);
    let mut acc = noise;
    mix_at(&mut acc, &sweep, 0.0);
    finalize(acc)
}

fn setup_sounds(mut commands: Commands, mut assets: ResMut<Assets<SynthAudio>>) {
    let clip = |v: Vec<f32>| SynthAudio { samples: v.into() };
    let mut dig = Vec::with_capacity(5);
    for m in [
        Material::Grass,
        Material::Stone,
        Material::Wood,
        Material::Sand,
        Material::Leaves,
    ] {
        dig.push(assets.add(clip(clip_dig(m))));
    }
    commands.insert_resource(SoundBank {
        dig: dig.try_into().unwrap(),
        place: assets.add(clip(clip_place())),
        footstep: assets.add(clip(clip_footstep())),
        eat: assets.add(clip(clip_eat())),
        hurt: assets.add(clip(clip_hurt())),
        splash: assets.add(clip(clip_splash())),
    });
}

fn setup_volume_hud(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(14.0),
            bottom: Val::Px(10.0),
            ..default()
        },
        ZIndex(30),
        Visibility::Hidden,
        VolumeHudText,
    ));
}

fn play_queued_sounds(
    mut commands: Commands,
    bank: Option<Res<SoundBank>>,
    master: Res<MasterVolume>,
    mut queue: ResMut<SoundQueue>,
) {
    if queue.0.is_empty() {
        return;
    }
    let Some(bank) = bank else {
        queue.0.clear();
        return;
    };
    let master_lin = master.effective();
    if master_lin <= 0.0 {
        queue.0.clear();
        return;
    }
    let mut rng = rand::rng();
    for ev in queue.0.drain(..).take(12) {
        let (handle, base_vol, jitter) = match ev {
            SoundEvent::Break(id) => {
                let v = match material_of(id) {
                    Material::Grass => 0.55,
                    Material::Stone => 0.6,
                    Material::Wood => 0.55,
                    Material::Sand => 0.45,
                    Material::Leaves => 0.35,
                };
                (&bank.dig[material_of(id) as usize], v, 0.9..1.1)
            }
            SoundEvent::Place(_) => (&bank.place, 0.5, 0.92..1.08),
            SoundEvent::Footstep(_) => (&bank.footstep, 0.28, 0.85..1.15),
            SoundEvent::Eat => (&bank.eat, 0.45, 0.95..1.05),
            SoundEvent::Hurt => (&bank.hurt, 0.55, 0.95..1.05),
            SoundEvent::Splash => (&bank.splash, 0.5, 0.9..1.1),
        };
        let speed = rng.random_range(jitter);
        commands.spawn((
            AudioPlayer(handle.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(master_lin * base_vol))
                .with_speed(speed),
        ));
    }
}

fn volume_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut master: ResMut<MasterVolume>,
    mut hud_timer: ResMut<VolumeHudTimer>,
) {
    let mut changed = false;
    if keys.just_pressed(KeyCode::KeyM) {
        master.muted = !master.muted;
        changed = true;
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        master.volume = (master.volume - 0.1).max(0.0);
        master.muted = false;
        changed = true;
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        master.volume = (master.volume + 0.1).min(1.0);
        master.muted = false;
        changed = true;
    }
    if changed {
        hud_timer.0 = 1.5;
    }
}

fn gameplay_sounds(
    time: Res<Time>,
    pause: Res<PauseOpen>,
    open: Res<InventoryOpen>,
    world: Res<World>,
    sur: Res<Survival>,
    player_q: Query<&Player>,
    mut queue: ResMut<SoundQueue>,
    mut step_dist: Local<f32>,
    mut was_in_water: Local<bool>,
    mut prev_health: Local<f32>,
) {
    if pause.0 || open.0 {
        return;
    }
    let Ok(player) = player_q.single() else { return };

    let feet_block = world.block(
        player.pos.x.floor() as i32,
        (player.pos.y + 0.4).floor() as i32,
        player.pos.z.floor() as i32,
    );
    let in_water = feet_block == WATER;
    if in_water && !*was_in_water {
        queue.0.push(SoundEvent::Splash);
    }
    *was_in_water = in_water;

    // Footsteps: accumulate horizontal distance while grounded on land.
    let horiz = Vec3::new(player.vel.x, 0.0, player.vel.z).length();
    if player.on_ground && !in_water && horiz > 0.5 {
        *step_dist += horiz * time.delta_secs();
        let stride = if player.sneak { 1.4 } else { 2.1 };
        if *step_dist >= stride {
            *step_dist = 0.0;
            let under = world.block(
                player.pos.x.floor() as i32,
                (player.pos.y - 0.2).floor() as i32,
                player.pos.z.floor() as i32,
            );
            if under != AIR {
                queue.0.push(SoundEvent::Footstep(under));
            }
        }
    } else {
        *step_dist = 0.0;
    }

    // Hurt: any health decrease triggers a sound (fall/starvation).
    if sur.health < *prev_health - 0.01 {
        queue.0.push(SoundEvent::Hurt);
    }
    *prev_health = sur.health;
}

fn volume_hud(
    time: Res<Time>,
    mut hud_timer: ResMut<VolumeHudTimer>,
    master: Res<MasterVolume>,
    mut q: Query<(&mut Visibility, &mut Text), With<VolumeHudText>>,
) {
    if hud_timer.0 > 0.0 {
        hud_timer.0 -= time.delta_secs();
    }
    let Ok((mut vis, mut text)) = q.single_mut() else {
        return;
    };
    if hud_timer.0 > 0.0 {
        *text = Text::new(if master.muted {
            "Muted".to_string()
        } else {
            format!("Volume {}%", (master.effective() * 100.0).round())
        });
        *vis = Visibility::Visible;
    } else {
        *vis = Visibility::Hidden;
    }
}
