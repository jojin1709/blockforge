use bevy::prelude::*;

use crate::inventory::InventoryOpen;
use crate::player::Player;
use crate::terrain::{surface_height, SEA_LEVEL};
use crate::world::SEED;

pub const MAX_HP: f32 = 20.0;
pub const MAX_FOOD: f32 = 20.0;

#[derive(Resource)]
pub struct Survival {
    pub health: f32,
    pub hunger: f32,
    pub fall_peak: f32,
}

impl Default for Survival {
    fn default() -> Self {
        Survival {
            health: MAX_HP,
            hunger: MAX_FOOD,
            fall_peak: 0.0,
        }
    }
}

#[derive(Component)]
pub struct HeartIcon(pub usize);

#[derive(Component)]
pub struct FoodIcon(pub usize);

fn sur_or_open_changed(sur: Res<Survival>, open: Res<InventoryOpen>) -> bool {
    sur.is_changed() || open.is_changed()
}

pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Survival>()
            .add_systems(Startup, setup_bars)
            .add_systems(
                Update,
                (survival_tick, refresh_bars.run_if(sur_or_open_changed)),
            );
    }
}

fn icon_node() -> Node {
    Node {
        width: Val::Px(16.0),
        height: Val::Px(16.0),
        margin: UiRect::new(Val::Px(1.0), Val::Px(1.0), Val::ZERO, Val::ZERO),
        border_radius: BorderRadius::all(Val::Px(3.0)),
        ..default()
    }
}

fn setup_bars(mut commands: Commands) {
    let mut heart_kids = Vec::new();
    for i in 0..10 {
        heart_kids.push(
            commands
                .spawn((
                    icon_node(),
                    BackgroundColor(Color::srgb(0.15, 0.05, 0.05)),
                    HeartIcon(i),
                ))
                .id(),
        );
    }
    let mut food_kids = Vec::new();
    for i in 0..10 {
        food_kids.push(
            commands
                .spawn((
                    icon_node(),
                    BackgroundColor(Color::srgb(0.16, 0.12, 0.08)),
                    FoodIcon(i),
                ))
                .id(),
        );
    }

    // Full-width bar above hotbar, hearts left half + food right half.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(84.0),
                height: Val::Px(18.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(80.0),
                ..default()
            },
            ZIndex(10),
        ))
        .with_children(|bar| {
            bar.spawn((Node { column_gap: Val::ZERO, ..default() },))
                .add_children(&heart_kids);
            bar.spawn((Node { column_gap: Val::ZERO, ..default() },))
                .add_children(&food_kids);
        });
}

fn refresh_bars(
    sur: Res<Survival>,
    open: Res<InventoryOpen>,
    mut heart_q: Query<(&HeartIcon, &mut BackgroundColor), Without<FoodIcon>>,
    mut food_q: Query<(&FoodIcon, &mut BackgroundColor), Without<HeartIcon>>,
) {
    for (h, mut bg) in heart_q.iter_mut() {
        let v = sur.health - h.0 as f32 * 2.0;
        *bg = BackgroundColor(if v >= 2.0 {
            Color::srgb(0.85, 0.1, 0.12)
        } else if v >= 1.0 {
            Color::srgb(0.7, 0.14, 0.14)
        } else {
            Color::srgb(0.18, 0.08, 0.08)
        });
    }
    for (f, mut bg) in food_q.iter_mut() {
        let v = sur.hunger - f.0 as f32 * 2.0;
        *bg = BackgroundColor(if v >= 2.0 {
            Color::srgb(0.75, 0.45, 0.15)
        } else if v >= 1.0 {
            Color::srgb(0.62, 0.38, 0.16)
        } else {
            Color::srgb(0.16, 0.12, 0.08)
        });
    }
    let _ = open.0;
}

fn survival_tick(
    time: Res<Time>,
    mut sur: ResMut<Survival>,
    mut player_q: Query<&mut Player>,
) {
    let dt = time.delta_secs().min(0.1);
    sur.hunger = (sur.hunger - dt * 0.04).max(0.0);
    if sur.hunger > 17.0 && sur.health < MAX_HP {
        sur.health = (sur.health + dt * 0.45).min(MAX_HP);
        sur.hunger = (sur.hunger - dt * 0.10).max(0.0);
    }
    if sur.hunger <= 0.0 {
        sur.health = (sur.health - dt * 0.35).max(0.0);
    }
    if sur.health <= 0.0 {
        if let Ok(mut p) = player_q.single_mut() {
            let h = surface_height(SEED, 0, 0).max(SEA_LEVEL) as f32;
            p.pos = Vec3::new(0.5, h + 1.05, 0.5);
            p.vel = Vec3::ZERO;
        }
        sur.health = MAX_HP;
        sur.hunger = MAX_FOOD;
        sur.fall_peak = 0.0;
    }
}
