mod audio;
mod blocks;
mod character;
mod daynight;
mod inventory;
mod items;
mod mesh;
mod mining;
mod pause;
mod particles;
mod player;
mod survival;
mod terrain;
mod textures;
mod ui;
mod visuals;
mod world;

use bevy::prelude::*;
use audio::SoundPlugin;
use character::CharacterPlugin;
use daynight::{DayNightPlugin, Sun};
use inventory::InventoryPlugin;
use mining::MiningPlugin;
use pause::PausePlugin;
use particles::ParticlesPlugin;
use player::PlayerPlugin;
use survival::SurvivalPlugin;
use ui::UiPlugin;
use visuals::VisualsPlugin;
use world::WorldPlugin;

/// Resolve the assets directory. Bevy resolves AssetPlugin.file_path relative to
/// the exe, so return an absolute path when the project-root assets dir is found
/// via CWD (cargo run case) or via exe-relative ../../assets (direct exe case).
fn asset_root() -> String {
    // 1. CWD/assets (works when launched via cargo run from project root).
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("assets");
        if p.exists() {
            return p.to_string_lossy().to_string();
        }
    }
    // 2. exe_dir/../../assets (works for target/release/blockforge.exe).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("../../assets");
            if p.exists() {
                return p.canonicalize().unwrap_or(p).to_string_lossy().to_string();
            }
        }
    }
    "assets".to_string()
}

fn main() {
    // Parse --skin <path> CLI override before the app starts.
    let args: Vec<String> = std::env::args().collect();
    let mut skin_override = character::SkinOverride::default();
    for i in 0..args.len() {
        if args[i] == "--skin" && i + 1 < args.len() {
            skin_override = character::SkinOverride(Some(args[i + 1].clone()));
        }
    }

    App::new()
        .insert_resource(skin_override)
        .insert_resource(ClearColor(Color::srgb(0.45, 0.70, 0.95)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Blockforge".into(),
                        resolution: (1280u32, 720u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_root(),
                    ..default()
                }),
        )
        .add_plugins((
            WorldPlugin,
            PlayerPlugin,
            InventoryPlugin,
            SurvivalPlugin,
            DayNightPlugin,
            UiPlugin,
            VisualsPlugin,
            PausePlugin,
            MiningPlugin,
            ParticlesPlugin,
            CharacterPlugin,
            SoundPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            shadow_depth_bias: 0.02,
            ..default()
        },
        Transform::from_xyz(60.0, 120.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun,
    ));

    commands.insert_resource(GlobalAmbientLight {
        brightness: 400.0,
        ..default()
    });
}
