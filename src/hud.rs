use crate::{inventory::Inventory, player::Player};
use bevy::prelude::*;

#[derive(Component)]
pub struct InventoryText;

#[derive(Component)]
pub struct TargetText;

#[derive(Component)]
pub struct Crosshair;

pub fn setup_hud(mut commands: Commands) {
    commands.spawn((
        InventoryText,
        Text::new("Inventory"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(60),
            ..default()
        },
    ));

    commands.spawn((
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
    ));

    commands.spawn((
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
                top: auto(),
                bottom: Val::Percent(20.0),
            },
            ..default()
        },
    ));
}

pub fn draw_inventory(
    inventory: Single<&Inventory, With<Player>>,
    mut inventory_text: Single<&mut Text, With<InventoryText>>,
) {
    let mut text = String::from("Inventory\n");
    for stack in &inventory.stacks {
        text += &format!("{:?}: {:?}\n", stack.item, stack.count);
    }
    inventory_text.0 = text;
}
