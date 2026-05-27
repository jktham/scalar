use bevy::prelude::*;

use crate::{
    GameState::{self},
    buildings::{Building, FuelSlot, OutputSlot, Processing},
    inventory::Inventory,
    player::{OpenBuilding, Player},
};

#[derive(Component)]
pub struct BuildingMenu;

pub fn building_menu_interact(
    collect_button: Option<Single<&Interaction, With<CollectButton>>>,
    add_fuel_button: Option<Single<&Interaction, With<AddFuelButton>>>,
    exit_button: Single<&Interaction, With<ExitButton>>,
    mut inventory: Single<&mut Inventory, With<Player>>,
    interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    mut buildings: Query<(
        &Building,
        Option<&Processing>,
        Option<&mut OutputSlot>,
        Option<&mut FuelSlot>,
    )>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut open_building: ResMut<OpenBuilding>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut background_color) in interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.0));
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 1.0));
            }
        }
    }

    if let Some(collect_button) = collect_button
        && *collect_button == &Interaction::Pressed
    {
        if let Some(open) = open_building.0
            && let Ok((_, _, Some(mut output_slot), _)) = buildings.get_mut(open)
        {
            inventory.add(&output_slot.0.item, output_slot.0.count);
            output_slot.0.count = 0;
        }
    }

    if let Some(add_fuel_button) = add_fuel_button
        && *add_fuel_button == &Interaction::Pressed
        && mouse_input.just_pressed(MouseButton::Left)
    // only once
    {
        if let Some(open) = open_building.0
            && let Ok((_, _, _, Some(mut fuel_slot))) = buildings.get_mut(open)
        {
            if inventory.has(&fuel_slot.0.item, 1) {
                inventory.remove(&fuel_slot.0.item, 1);
                fuel_slot.0.count += 1;
            }
        }
    }

    if *exit_button == &Interaction::Pressed
        || keyboard_input.just_pressed(KeyCode::KeyE)
        || keyboard_input.just_pressed(KeyCode::Escape)
    {
        open_building.0 = None;
        next_state.set(GameState::Play);
    }
}

pub fn get_info_text(
    processing: &Processing,
    output_slot: &OutputSlot,
    fuel_slot: &FuelSlot,
) -> String {
    format!(
        "{:?}\nspeed: {}\nprogress: {}%\ncost: {}\nenergy: {}\noutput: ({:?} {})\nfuel: ({:?}, {})",
        processing.status,
        processing.speed,
        (processing.progress * 100.0).round(),
        processing.cost,
        processing.energy.round(),
        output_slot.0.item,
        output_slot.0.count,
        fuel_slot.0.item,
        fuel_slot.0.count,
    )
}

pub fn building_menu_update(
    buildings: Query<(
        &Building,
        Option<&Processing>,
        Option<&OutputSlot>,
        Option<&FuelSlot>,
    )>,
    mut info_text: Single<&mut Text, With<InfoText>>,
    open_building: Res<OpenBuilding>,
) {
    let mut building = None;
    if let Some(open) = open_building.0
        && let Ok(b) = buildings.get(open)
    {
        building = Some(b);
    }

    let info = match building {
        Some((_, Some(processing), Some(output_slot), Some(fuel_slot))) => {
            get_info_text(processing, output_slot, fuel_slot)
        }
        _ => String::from("No info"),
    };

    info_text.0 = info;
}

#[derive(Component)]
pub struct CollectButton;

#[derive(Component)]
pub struct AddFuelButton;

#[derive(Component)]
pub struct ExitButton;

#[derive(Component)]
pub struct InfoText;

pub fn show_building_menu(
    mut commands: Commands,
    buildings: Query<(
        &Building,
        Option<&Processing>,
        Option<&OutputSlot>,
        Option<&FuelSlot>,
    )>,
    open_building: Res<OpenBuilding>,
) {
    let mut building = None;
    if let Some(open) = open_building.0
        && let Ok(b) = buildings.get(open)
    {
        building = Some(b);
    }

    let title = match building {
        Some((b, _, _, _)) => b.name(),
        None => "None",
    };

    let info = match building {
        Some((_, Some(processing), Some(output_slot), Some(fuel_slot))) => {
            get_info_text(processing, output_slot, fuel_slot)
        }
        _ => String::from("No info"),
    };

    let buttons = match building {
        Some((Building::Miner, _, _, _)) => vec![
            commands
                .spawn((
                    CollectButton,
                    Button,
                    Node {
                        width: px(180),
                        height: px(60),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                    children![(
                        Text::new("Collect"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    )],
                ))
                .id(),
            commands
                .spawn((
                    AddFuelButton,
                    Button,
                    Node {
                        width: px(180),
                        height: px(60),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                    children![(
                        Text::new("Add 1 fuel"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    )],
                ))
                .id(),
        ],
        _ => vec![],
    };

    let menu = commands
        .spawn((
            BuildingMenu,
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .id();

    let header = commands
        .spawn((
            Node {
                width: px(360),
                // height: px(120),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(10),
                ..default()
            },
            children![
                (
                    Text::new(title),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ),
                (
                    InfoText,
                    Text::new(info),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                )
            ],
        ))
        .id();

    let exit_button = commands
        .spawn((
            ExitButton,
            Button,
            Node {
                width: px(180),
                height: px(60),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![(
                Text::new("[E] Exit"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )],
        ))
        .id();

    commands.get_entity(menu).unwrap().add_child(header);
    commands
        .get_entity(menu)
        .unwrap()
        .add_children(buttons.as_slice());
    commands.get_entity(menu).unwrap().add_child(exit_button);
}

pub fn hide_building_menu(
    mut commands: Commands,
    menu_entities: Query<Entity, With<BuildingMenu>>,
) {
    for e in menu_entities {
        commands.entity(e).despawn();
    }
}
