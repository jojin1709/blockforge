use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::InventoryOpen;

#[derive(Resource, Default)]
pub struct PauseOpen(pub bool);

#[derive(Component)]
pub struct PauseRoot;

#[derive(Component)]
pub enum PauseBtn {
    Resume,
    Settings,
    Statistics,
    Advancements,
    OpenLan,
    Feedback,
    ReportBugs,
    SaveQuit,
}

#[derive(Component)]
struct PauseBtnStyle {
    normal: Color,
    hover: Color,
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PauseOpen>()
            .add_systems(Startup, setup_pause_menu)
            .add_systems(Update, (pause_input, pause_visibility, pause_hover, pause_buttons).chain());
    }
}

const BTN_W: f32 = 420.0;
const BTN_H: f32 = 44.0;
const GAP: f32 = 8.0;

fn setup_pause_menu(mut commands: Commands) {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            PauseRoot,
            ZIndex(20),
            Visibility::Hidden,
        ))
        .id();

    // Title + subtitle.
    let title_block = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(2.0),
                margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(24.0), Val::Px(28.0)),
                ..default()
            },
        ))
        .id();
    let title = commands
        .spawn((
            Text::new("Game Menu"),
            TextFont {
                font_size: FontSize::Px(32.0),
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ))
        .id();
    let subtitle = commands
        .spawn((
            Text::new("Blockforge"),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgba(0.75, 0.75, 0.78, 0.9)),
        ))
        .id();
    commands.entity(title_block).add_children(&[title, subtitle]);

    // Buttons column.
    let col = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(GAP),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();

    let grey = Color::srgb(0.22, 0.22, 0.25);
    let grey_border = Color::srgb(0.38, 0.38, 0.42);
    let grey_hover = Color::srgb(0.30, 0.30, 0.34);

    let resume = spawn_btn(
        &mut commands,
        "Resume Game",
        PauseBtn::Resume,
        Color::srgb(0.25, 0.46, 0.20),
        Color::srgb(0.35, 0.58, 0.28),
        Color::srgb(0.32, 0.54, 0.26),
        true,
    );

    let row1 = spawn_row(&mut commands);
    let settings = spawn_btn(&mut commands, "Settings", PauseBtn::Settings, grey, grey_border, grey_hover, false);
    let statistics = spawn_btn(&mut commands, "Statistics", PauseBtn::Statistics, grey, grey_border, grey_hover, false);
    commands.entity(row1).add_children(&[settings, statistics]);

    let row2 = spawn_row(&mut commands);
    let advancements = spawn_btn(&mut commands, "Advancements", PauseBtn::Advancements, grey, grey_border, grey_hover, false);
    let lan = spawn_btn(
        &mut commands,
        "Open to LAN",
        PauseBtn::OpenLan,
        Color::srgb(0.15, 0.15, 0.17),
        Color::srgb(0.22, 0.22, 0.25),
        Color::srgb(0.18, 0.18, 0.20),
        false,
    );
    commands.entity(row2).add_children(&[advancements, lan]);

    let row3 = spawn_row(&mut commands);
    let feedback = spawn_btn(&mut commands, "Give Feedback", PauseBtn::Feedback, grey, grey_border, grey_hover, false);
    let bugs = spawn_btn(&mut commands, "Report Bugs", PauseBtn::ReportBugs, grey, grey_border, grey_hover, false);
    commands.entity(row3).add_children(&[feedback, bugs]);

    let save_quit = spawn_btn(
        &mut commands,
        "Save and Quit to Title",
        PauseBtn::SaveQuit,
        Color::srgb(0.42, 0.12, 0.12),
        Color::srgb(0.55, 0.18, 0.18),
        Color::srgb(0.50, 0.15, 0.15),
        true,
    );

    commands
        .entity(col)
        .add_children(&[resume, row1, row2, row3, save_quit]);
    commands.entity(root).add_children(&[title_block, col]);
}

fn spawn_row(commands: &mut Commands) -> Entity {
    commands
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(GAP),
            ..default()
        },))
        .id()
}

fn spawn_btn(
    commands: &mut Commands,
    label: &str,
    kind: PauseBtn,
    bg: Color,
    border: Color,
    hover: Color,
    full_width: bool,
) -> Entity {
    let width = if full_width {
        Val::Px(BTN_W)
    } else {
        Val::Px((BTN_W - GAP) / 2.0)
    };
    commands
        .spawn((
            Node {
                width,
                height: Val::Px(BTN_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::all(border),
            Interaction::default(),
            kind,
            PauseBtnStyle { normal: bg, hover },
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.92, 0.94)),
            ));
        })
        .id()
}

fn pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    inv: Res<InventoryOpen>,
    mut pause: ResMut<PauseOpen>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if inv.0 {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        pause.0 = !pause.0;
        if let Ok(mut co) = cursor_q.single_mut() {
            if pause.0 {
                co.grab_mode = CursorGrabMode::None;
                co.visible = true;
            } else {
                co.grab_mode = CursorGrabMode::Locked;
                co.visible = false;
            }
        }
    }
}

fn pause_visibility(
    pause: Res<PauseOpen>,
    mut root_q: Query<&mut Visibility, With<PauseRoot>>,
) {
    if !pause.is_changed() {
        return;
    }
    if let Ok(mut vis) = root_q.single_mut() {
        *vis = if pause.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn pause_hover(mut q: Query<(&PauseBtnStyle, &Interaction, &mut BackgroundColor), Changed<Interaction>>) {
    for (style, interaction, mut bg) in q.iter_mut() {
        *bg = BackgroundColor(match interaction {
            Interaction::Hovered => style.hover,
            _ => style.normal,
        });
    }
}

fn pause_buttons(
    mut clicks: Query<(&PauseBtn, &Interaction), (Changed<Interaction>, With<PauseBtnStyle>)>,
    mut pause: ResMut<PauseOpen>,
    mut exit: MessageWriter<AppExit>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for (btn, interaction) in clicks.iter_mut() {
        if *interaction != Interaction::Pressed || !pause.0 {
            continue;
        }
        match btn {
            PauseBtn::Resume => {
                pause.0 = false;
                if let Ok(mut co) = cursor_q.single_mut() {
                    co.grab_mode = CursorGrabMode::Locked;
                    co.visible = false;
                }
            }
            PauseBtn::SaveQuit => {
                exit.write(AppExit::Success);
            }
            _ => {}
        }
    }
}
