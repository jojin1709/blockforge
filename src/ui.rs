use std::collections::HashMap;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::*;
use crate::items;
use crate::textures;

pub const HOTBAR_SLOTS: usize = 9;
const SLOT: f32 = 50.0;

#[derive(Component)]
pub struct SlotVis {
    pub idx: usize,
}

#[derive(Component)]
pub struct SlotCountText;

#[derive(Component)]
pub struct SlotIcon;

#[derive(Component)]
pub struct HeldStackIcon;

#[derive(Component)]
pub struct HeldStack;

#[derive(Component)]
pub struct InvPanel;

#[derive(Component)]
pub struct InvPanelChild;

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct HotbarSelBorder(pub usize);

#[derive(Component)]
pub struct BlockNameToast;

#[derive(Component)]
pub struct ToastTimer(f32);

/// Marks one of the 9 craft-grid cells by its grid position (row-major 3x3).
#[derive(Component)]
pub struct CraftCell(pub usize);

/// Sent when the player right-clicks a crafting table block.
#[derive(Message)]
pub struct OpenCraftTable;

fn inv_or_craft_changed(inv: Res<Inventory>, craft: Res<CraftGrid>, table: Res<TableCraftOpen>) -> bool {
    inv.is_changed() || craft.is_changed() || table.is_changed()
}

/// Per-item 16x16 icon images extracted from the procedural atlas.
#[derive(Resource)]
pub struct ItemIcons(pub HashMap<ItemId, Handle<Image>>);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<OpenCraftTable>()
            .add_systems(Startup, (setup_icons, setup_ui).chain())
            .add_systems(
                Update,
                (
                    hotbar_input,
                    toast_tick,
                    toggle_inventory,
                    open_table_ui,
                    update_craft_layout,
                    slot_clicks,
                    refresh_slots.run_if(inv_or_craft_changed),
                    update_held,
                )
                    .chain(),
            );
    }
}

fn setup_icons(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let data = textures::build_atlas_data();
    let mut map = HashMap::new();
    for id in 1..=16u16 {
        let img = textures::extract_tile_image(&data, textures::item_tile(id));
        map.insert(id, images.add(img));
    }
    for &id in items::ALL_ITEMS {
        let img = textures::extract_tile_image(&data, textures::item_tile(id));
        map.insert(id, images.add(img));
    }
    commands.insert_resource(ItemIcons(map));
}

fn spawn_slot(
    commands: &mut Commands,
    idx: usize,
    size: f32,
) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::new(Val::Px(2.0), Val::Px(2.0), Val::ZERO, Val::ZERO),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
            BorderColor::all(Color::srgb(0.25, 0.25, 0.28)),
            Interaction::default(),
            SlotVis { idx },
            Visibility::Inherited,
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(4.0),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.95)),
                SlotCountText,
                Visibility::Inherited,
            ));
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    margin: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                SlotIcon,
                Visibility::Inherited,
            ));
        })
        .id()
}

fn setup_ui(mut commands: Commands) {
    // Crosshair.
    let cross_color = Color::srgba(1.0, 1.0, 1.0, 0.85);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(18.0),
            height: Val::Px(2.0),
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect::new(Val::Px(-9.0), Val::ZERO, Val::Px(-1.0), Val::ZERO),
            ..default()
        },
        BackgroundColor(cross_color),
        Crosshair,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(2.0),
            height: Val::Px(18.0),
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect::new(Val::Px(-1.0), Val::ZERO, Val::Px(-9.0), Val::ZERO),
            ..default()
        },
        BackgroundColor(cross_color),
        Crosshair,
    ));

    // Hotbar slots (indices 0-8) with selection borders.
    let mut hotbar_slots = Vec::new();
    for i in 0..HOTBAR_SLOTS {
        let color = if i == 0 {
            Color::srgb(1.0, 1.0, 1.0)
        } else {
            Color::srgb(0.2, 0.2, 0.22)
        };
        let e = commands
            .spawn((
                Node {
                    width: Val::Px(SLOT),
                    height: Val::Px(SLOT),
                    border: UiRect::all(Val::Px(3.0)),
                    margin: UiRect::new(Val::Px(1.0), Val::Px(1.0), Val::ZERO, Val::ZERO),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
                BorderColor::all(color),
                Interaction::default(),
                SlotVis { idx: i },
                HotbarSelBorder(i),
                Visibility::Visible,
            ))
            .with_children(|p| {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(1.0),
                        right: Val::Px(4.0),
                        ..default()
                    },
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.95)),
                    SlotCountText,
                    Visibility::Hidden,
                ));
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        margin: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    ImageNode {
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    SlotIcon,
                    Visibility::Hidden,
                ));
            })
            .id();
        hotbar_slots.push(e);
    }

    // Full-width bottom bar centers hotbar properly.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(14.0),
                height: Val::Px(SLOT + 12.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::ZERO,
                ..default()
            },
        ))
        .add_children(&hotbar_slots);

    // Toast above hotbar.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(84.0),
            left: Val::Percent(50.0),
            margin: UiRect::new(Val::Px(-60.0), Val::ZERO, Val::ZERO, Val::ZERO),
            padding: UiRect::px(10.0, 10.0, 4.0, 4.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        BlockNameToast,
        ToastTimer(0.0),
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
    ));

    // Held (cursor) stack.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(44.0),
                height: Val::Px(44.0),
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
            BorderColor::all(Color::srgb(0.6, 0.6, 0.65)),
            HeldStack,
            Visibility::Hidden,
            ZIndex(10),
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    margin: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                HeldStackIcon,
                Visibility::Inherited,
            ));
        });

    // ---- Inventory panel ----
    let craft_cells: Vec<Entity> = (0..CRAFT_SLOTS)
        .map(|i| {
            commands
                .spawn((
                    Node {
                        width: Val::Px(44.0),
                        height: Val::Px(44.0),
                        border: UiRect::all(Val::Px(2.0)),
                        margin: UiRect::new(Val::Px(2.0), Val::Px(2.0), Val::ZERO, Val::ZERO),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
                    BorderColor::all(Color::srgb(0.25, 0.25, 0.28)),
                    Interaction::default(),
                    SlotVis { idx: INV_SLOTS + i },
                    CraftCell(i),
                    Visibility::Inherited,
                ))
                .with_children(|p| {
                    p.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(1.0),
                            right: Val::Px(4.0),
                            ..default()
                        },
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        SlotCountText,
                        Visibility::Hidden,
                    ));
                    p.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            right: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            margin: UiRect::all(Val::Px(3.0)),
                            ..default()
                        },
                        ImageNode {
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        SlotIcon,
                        Visibility::Hidden,
                    ));
                })
                .id()
        })
        .collect();
    let craft_out = spawn_slot(&mut commands, CRAFT_OUT, 52.0);
    let main_slots: Vec<Entity> = (0..27)
        .map(|i| spawn_slot(&mut commands, 9 + i, SLOT))
        .collect();
    let panel_hotbar: Vec<Entity> = (0..HOTBAR_SLOTS)
        .map(|i| spawn_slot(&mut commands, i, SLOT))
        .collect();

    let craft_row = commands
        .spawn((
            Node {
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .id();
    let craft_grid = commands
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::ZERO,
            ..default()
        },))
        .id();
    let craft_r0 = commands
        .spawn((Node {
            column_gap: Val::ZERO,
            ..default()
        },))
        .id();
    let craft_r1 = commands
        .spawn((Node {
            column_gap: Val::ZERO,
            ..default()
        },))
        .id();
    let craft_r2 = commands
        .spawn((Node {
            column_gap: Val::ZERO,
            ..default()
        },))
        .id();
    let arrow = commands
        .spawn((
            Node {
                width: Val::Px(40.0),
                height: Val::Px(4.0),
                margin: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::ZERO, Val::ZERO),
                ..default()
            },
            BackgroundColor(Color::srgb(0.7, 0.7, 0.75)),
        ))
        .id();

    let spacer1 = commands
        .spawn((Node {
            height: Val::Px(4.0),
            ..default()
        },))
        .id();
    let spacer2 = commands
        .spawn((Node {
            height: Val::Px(6.0),
            ..default()
        },))
        .id();

    let main_rows: Vec<Entity> = (0..3)
        .map(|_| {
            commands
                .spawn((Node {
                    column_gap: Val::ZERO,
                    justify_content: JustifyContent::Center,
                    ..default()
                },))
                .id()
        })
        .collect();
    let panel_hotbar_row = commands
        .spawn((Node {
            column_gap: Val::ZERO,
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .id();

    // Backdrop centers panel both axes. Starts fully transparent (alpha 0).
    let backdrop = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            InvPanel,
            ZIndex(5),
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.14, 0.97)),
            BorderColor::all(Color::srgba(0.4, 0.4, 0.45, 0.9)),
            InvPanelChild,
            Visibility::Hidden,
        ))
        .id();

    commands.entity(backdrop).add_children(&[panel]);
    commands.entity(craft_r0).add_children(&craft_cells[0..3]);
    commands.entity(craft_r1).add_children(&craft_cells[3..6]);
    commands.entity(craft_r2).add_children(&craft_cells[6..9]);
    commands
        .entity(craft_grid)
        .add_children(&[craft_r0, craft_r1, craft_r2]);
    commands
        .entity(craft_row)
        .add_children(&[craft_grid, arrow, craft_out]);

    for (row, e) in main_rows.iter().enumerate() {
        commands
            .entity(*e)
            .add_children(&main_slots[row * 9..(row + 1) * 9]);
    }
    commands
        .entity(panel_hotbar_row)
        .add_children(&panel_hotbar);

    commands.entity(panel).add_children(&[
        craft_row,
        spacer1,
        main_rows[0],
        main_rows[1],
        main_rows[2],
        spacer2,
        panel_hotbar_row,
    ]);
}

fn toggle_inventory(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<InventoryOpen>,
    mut table: ResMut<TableCraftOpen>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut backdrop_q: Query<&mut BackgroundColor, With<InvPanel>>,
    mut panel_q: Query<&mut Visibility, (With<InvPanelChild>, Without<Crosshair>, Without<HeldStack>)>,
    mut cross_q: Query<&mut Visibility, (With<Crosshair>, Without<InvPanelChild>, Without<HeldStack>)>,
    mut held_q: Query<&mut Visibility, (With<HeldStack>, Without<InvPanelChild>, Without<Crosshair>)>,
    mut craft: ResMut<CraftGrid>,
    mut inv: ResMut<Inventory>,
) {
    let e = keys.just_pressed(KeyCode::KeyE);
    let esc = keys.just_pressed(KeyCode::Escape);
    if !e && !esc {
        return;
    }

    // Escape when inventory closed: defer to the pause menu (pause.rs handles it).
    if esc && !open.0 {
        return;
    }

    open.0 = !open.0;
    if open.0 {
        // E always opens the personal 2x2 grid, never the table.
        table.0 = false;
    }

    if let Ok(mut co) = cursor_q.single_mut() {
        if open.0 {
            co.grab_mode = CursorGrabMode::None;
            co.visible = true;
        } else {
            co.grab_mode = CursorGrabMode::Locked;
            co.visible = false;
            return_items(&mut inv, &mut craft);
            table.0 = false;
        }
    }

    if let Ok(mut bg) = backdrop_q.single_mut() {
        *bg = BackgroundColor(if open.0 {
            Color::srgba(0.0, 0.0, 0.0, 0.45)
        } else {
            Color::srgba(0.0, 0.0, 0.0, 0.0)
        });
    }
    if let Ok(mut vis) = panel_q.single_mut() {
        *vis = if open.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut vis in cross_q.iter_mut() {
        *vis = if open.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    for mut vis in held_q.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

fn return_items(inv: &mut Inventory, craft: &mut CraftGrid) {
    if let Some(s) = inv.cursor.take() {
        inv.add(s.id, s.count);
    }
    for s in craft.slots.iter_mut() {
        if let Some(st) = s.take() {
            inv.add(st.id, st.count);
        }
    }
}

/// Right-clicking a crafting table opens the inventory in 3x3 mode.
fn open_table_ui(
    mut ev: MessageReader<OpenCraftTable>,
    mut open: ResMut<InventoryOpen>,
    mut table: ResMut<TableCraftOpen>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut backdrop_q: Query<&mut BackgroundColor, With<InvPanel>>,
    mut panel_q: Query<&mut Visibility, (With<InvPanelChild>, Without<Crosshair>, Without<HeldStack>)>,
    mut cross_q: Query<&mut Visibility, (With<Crosshair>, Without<InvPanelChild>, Without<HeldStack>)>,
    mut held_q: Query<&mut Visibility, (With<HeldStack>, Without<InvPanelChild>, Without<Crosshair>)>,
) {
    if ev.read().next().is_none() || open.0 {
        return;
    }
    open.0 = true;
    table.0 = true;

    if let Ok(mut co) = cursor_q.single_mut() {
        co.grab_mode = CursorGrabMode::None;
        co.visible = true;
    }
    if let Ok(mut bg) = backdrop_q.single_mut() {
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45));
    }
    if let Ok(mut vis) = panel_q.single_mut() {
        *vis = Visibility::Visible;
    }
    for mut vis in cross_q.iter_mut() {
        *vis = Visibility::Hidden;
    }
    for mut vis in held_q.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

/// In personal mode only the 2x2 sub-grid (cells 0,1,3,4) is visible/clickable.
fn update_craft_layout(
    table: Res<TableCraftOpen>,
    mut cell_q: Query<(&CraftCell, &mut Visibility)>,
) {
    if !table.is_changed() {
        return;
    }
    for (cell, mut vis) in cell_q.iter_mut() {
        let shown = table.0 || matches!(cell.0, 0 | 1 | 3 | 4);
        *vis = if shown { Visibility::Inherited } else { Visibility::Hidden };
    }
}

fn hotbar_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    mut inv: ResMut<Inventory>,
    mut border_q: Query<(&HotbarSelBorder, &mut BorderColor)>,
    mut toast_q: Query<(&mut Text, &mut TextColor, &mut ToastTimer), With<BlockNameToast>>,
    open: Res<InventoryOpen>,
) {
    let mut changed = false;

    let digit_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    for (i, key) in digit_keys.iter().enumerate() {
        if keys.just_pressed(*key) {
            inv.selected = i;
            changed = true;
        }
    }

    if !open.0 {
        for ev in wheel.read() {
            if ev.y > 0.0 {
                inv.selected = (inv.selected + HOTBAR_SLOTS - 1) % HOTBAR_SLOTS;
                changed = true;
            } else if ev.y < 0.0 {
                inv.selected = (inv.selected + 1) % HOTBAR_SLOTS;
                changed = true;
            }
        }
    }

    if !changed {
        return;
    }

    for (sel, mut border) in border_q.iter_mut() {
        *border = BorderColor::all(if sel.0 == inv.selected {
            Color::srgb(1.0, 1.0, 1.0)
        } else {
            Color::srgb(0.2, 0.2, 0.22)
        });
    }

    if let Ok((mut text, mut color, mut timer)) = toast_q.single_mut() {
        let name = inv.slots[inv.selected]
            .map(|s| item_name(s.id))
            .unwrap_or("");
        text.0 = name.to_string();
        *color = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.95));
        timer.0 = 1.5;
    }
}

fn toast_tick(
    time: Res<Time>,
    mut toast_q: Query<(&mut TextColor, &mut ToastTimer), With<BlockNameToast>>,
) {
    for (mut color, mut timer) in toast_q.iter_mut() {
        if timer.0 > 0.0 {
            timer.0 -= time.delta_secs();
            if timer.0 <= 0.0 {
                *color = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0));
            }
        }
    }
}

fn slot_clicks(
    mut clicks: Query<(&SlotVis, &Interaction), Changed<Interaction>>,
    mut inv: ResMut<Inventory>,
    mut craft: ResMut<CraftGrid>,
    open: Res<InventoryOpen>,
    table: Res<TableCraftOpen>,
) {
    if !open.0 {
        return;
    }
    for (slot, interaction) in clicks.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Personal mode: ignore clicks on hidden 3x3-only cells.
            if !table.0 && (INV_SLOTS..CRAFT_OUT).contains(&slot.idx) {
                let cell = slot.idx - INV_SLOTS;
                if !matches!(cell, 0 | 1 | 3 | 4) {
                    continue;
                }
            }
            click_slot(slot.idx, &mut inv, &mut craft, table.0);
        }
    }
}

fn stack_at(inv: &Inventory, craft: &CraftGrid, idx: usize, table: bool) -> Option<Stack> {
    if idx < INV_SLOTS {
        inv.slots[idx]
    } else if idx < CRAFT_OUT {
        craft.slots[idx - INV_SLOTS]
    } else {
        items::craft_result(&craft.slots, table).map(|(id, count, _)| Stack::new(id, count))
    }
}

fn refresh_slots(
    inv: Res<Inventory>,
    craft: Res<CraftGrid>,
    table: Res<TableCraftOpen>,
    icons: Res<ItemIcons>,
    mut slot_q: Query<(&SlotVis, &mut BackgroundColor, &Children)>,
    mut text_q: Query<(&mut Text, &mut Visibility, &mut Node), With<SlotCountText>>,
    mut icon_q: Query<
        (&mut ImageNode, &mut Visibility, &mut Node),
        (With<SlotIcon>, Without<SlotCountText>),
    >,
) {
    for (slot, mut bg, children) in slot_q.iter_mut() {
        let stack = stack_at(&inv, &craft, slot.idx, table.0);
        let (r, g, b) = match stack {
            Some(s) => {
                let c = item_color(s.id);
                (c[0], c[1], c[2])
            }
            None => (0.12, 0.12, 0.14),
        };
        *bg = BackgroundColor(Color::srgb(r, g, b));

        let mut text_done = false;
        let mut icon_done = false;
        for child in children.iter() {
            if !text_done {
                if let Ok((mut text, mut vis, mut node)) = text_q.get_mut(child) {
                    match stack {
                        Some(s) if s.count > 1 => {
                            text.0 = s.count.to_string();
                            *vis = Visibility::Inherited;
                            node.display = Display::Flex;
                        }
                        // Tools show remaining durability in the corner.
                        Some(s) if s.dura > 0 => {
                            text.0 = s.dura.to_string();
                            *vis = Visibility::Inherited;
                            node.display = Display::Flex;
                        }
                        _ => {
                            *vis = Visibility::Inherited;
                            node.display = Display::None;
                        }
                    }
                    text_done = true;
                    continue;
                }
            }
            if !icon_done {
                if let Ok((mut img, mut vis, mut node)) = icon_q.get_mut(child) {
                    match stack {
                        Some(s) => {
                            if let Some(h) = icons.0.get(&s.id) {
                                img.image = h.clone();
                            }
                            *vis = Visibility::Inherited;
                            node.display = Display::Flex;
                        }
                        None => {
                            *vis = Visibility::Inherited;
                            node.display = Display::None;
                        }
                    }
                    icon_done = true;
                }
            }
        }
    }
}

fn update_held(
    inv: Res<Inventory>,
    open: Res<InventoryOpen>,
    icons: Res<ItemIcons>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut held_q: Query<(&mut Node, &mut Visibility, &mut BackgroundColor), With<HeldStack>>,
    mut icon_q: Query<
        (&mut ImageNode, &mut Node),
        (With<HeldStackIcon>, Without<HeldStack>),
    >,
) {
    let Ok((mut node, mut vis, mut bg)) = held_q.single_mut() else {
        return;
    };
    let Ok((mut icon, mut inode)) = icon_q.single_mut() else {
        return;
    };
    if !open.0 {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
        return;
    }
    if let Some(pos) = windows.single().ok().and_then(|w| w.cursor_position()) {
        node.left = Val::Px(pos.x + 8.0);
        node.top = Val::Px(pos.y + 8.0);
    }
    match inv.cursor {
        Some(s) => {
            let c = item_color(s.id);
            *bg = BackgroundColor(Color::srgb(c[0], c[1], c[2]));
            *vis = Visibility::Visible;
            if let Some(h) = icons.0.get(&s.id) {
                icon.image = h.clone();
            }
            inode.display = Display::Flex;
        }
        None => {
            *vis = Visibility::Hidden;
            inode.display = Display::None;
        }
    }
}
