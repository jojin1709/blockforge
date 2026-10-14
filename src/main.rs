mod blocks;
mod daynight;
mod inventory;
mod mesh;
mod player;
mod survival;
mod terrain;
mod textures;
mod ui;
mod world;

use bevy::prelude::*;
use daynight::{DayNightPlugin, Sun};
use inventory::InventoryPlugin;
use player::PlayerPlugin;
use survival::SurvivalPlugin;
use ui::UiPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
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
                .set(AssetPlugin { ..default() }),
        )
        .add_plugins((
            WorldPlugin,
            PlayerPlugin,
            InventoryPlugin,
            SurvivalPlugin,
            DayNightPlugin,
            UiPlugin,
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
        brightness: 200.0,
        ..default()
    });
}
