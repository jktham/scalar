use crate::inventory::Inventory;
use bevy::prelude::*;

#[derive(Component)]
pub struct InventoryText;

pub fn setup_ui(mut commands: Commands) {
    commands.spawn((
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
        InventoryText,
    ));
}

pub fn update_ui(
    inventory: Res<Inventory>,
    mut inventory_text: Single<&mut Text, With<InventoryText>>,
) {
    let mut text = String::from("Inventory\n");
    for stack in &inventory.stacks {
        text += &format!("{:?}: {:?}\n", stack.item, stack.count);
    }
    inventory_text.0 = text;
}
