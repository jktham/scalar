use crate::{
    inventory::Inventory,
    player::{Money, Player},
};
use bevy::prelude::*;

#[derive(Component)]
pub struct HideHud;

#[derive(Component)]
pub struct InventoryText;

#[derive(Component)]
pub struct MoneyText;

#[derive(Component)]
pub struct TargetText;

#[derive(Component)]
pub struct ActionText;

#[derive(Component)]
pub struct Crosshair;

pub fn setup_hud(mut commands: Commands) {
    commands.spawn((
        // HideHud,
        InventoryText,
        Text::new("Inventory"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextLayout {
            justify: Justify::Right,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(32),
            right: px(10),
            ..default()
        },
        ZIndex(-10),
    ));

    commands.spawn((
        // HideHud,
        MoneyText,
        Text::new("$0"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            right: px(10),
            ..default()
        },
        ZIndex(-10),
    ));

    commands.spawn((
        HideHud,
        Crosshair,
        Text::new("+"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        Node {
            margin: UiRect::all(auto()),
            ..default()
        },
        ZIndex(-10),
    ));

    commands.spawn((
        HideHud,
        TargetText,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        Node {
            margin: UiRect {
                left: auto(),
                right: auto(),
                top: percent(35),
                bottom: auto(),
            },
            ..default()
        },
        ZIndex(-10),
    ));

    commands.spawn((
        HideHud,
        ActionText,
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        Node {
            margin: UiRect {
                left: auto(),
                right: auto(),
                top: percent(40),
                bottom: auto(),
            },
            ..default()
        },
        ZIndex(-10),
    ));
}

pub fn hide_hud(mut commands: Commands, mut hud_entities: Query<Entity, With<HideHud>>) {
    for entity in hud_entities.iter_mut() {
        commands.entity(entity).insert(Visibility::Hidden);
    }
}

pub fn show_hud(mut commands: Commands, mut hud_entities: Query<Entity, With<HideHud>>) {
    for entity in hud_entities.iter_mut() {
        commands.entity(entity).insert(Visibility::Visible);
    }
}

pub fn update_inventory(
    inventory: Single<&Inventory, With<Player>>,
    mut inventory_text: Single<&mut Text, With<InventoryText>>,
) {
    let mut text = String::from("Inventory\n");
    for stack in &inventory.stacks {
        text += &format!("{:?}: {:?}\n", stack.item, stack.count);
    }
    inventory_text.0 = text;
}

pub fn update_money(
    money: Single<&Money, With<Player>>,
    mut money_text: Single<&mut Text, With<MoneyText>>,
) {
    money_text.0 = format!("${}", money.0);
}
