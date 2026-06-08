use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::{
    GameState,
    buildings::Building,
    controls::{Action, Controls},
    inventory::Inventory,
    player::{HeldBuilding, Player},
};

#[derive(Component)]
pub struct Menu;

pub fn interact(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    build_buttons: Query<(&Interaction, &BuildButton)>,
    exit_button: Single<&Interaction, With<ExitButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut held_building: Single<&mut HeldBuilding, With<Player>>,
    mut inventory: Single<&mut Inventory, With<Player>>,
    controls: Res<Controls>,
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

    for (interaction, build_button) in build_buttons {
        if *interaction == Interaction::Pressed {
            let cost = build_button.0.cost();
            if !cost.iter().fold(true, |acc, stack| {
                acc && inventory.has(&stack.item, stack.count)
            }) {
                // cant afford
                continue;
            }

            for stack in cost {
                inventory.remove(&stack.item, stack.count);
            }

            held_building.0 = Some(build_button.0);
            next_state.set(GameState::Play);
        }
    }

    if *exit_button == &Interaction::Pressed
        || keyboard_input.just_pressed(controls.get(Action::Cancel))
        || keyboard_input.just_pressed(controls.get(Action::Pause))
    {
        held_building.0 = None;

        if keyboard_input.just_pressed(controls.get(Action::Pause)) {
            next_state.set(GameState::PauseMenu);
        } else {
            next_state.set(GameState::Play);
        }
    }
}

#[derive(Component, Clone)]
pub struct BuildButton(pub Building);

#[derive(Component)]
pub struct ExitButton;

pub fn show(
    mut commands: Commands,
    inventory: Single<&Inventory, With<Player>>,
    controls: Res<Controls>,
) {
    let building_buttons = Building::iter()
        .map(|building| {
            (
                BuildButton(building),
                Button,
                Node {
                    width: px(180),
                    height: px(180),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    padding: UiRect::all(px(10)),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                children![
                    (
                        Text::new(building.name()),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    ),
                    (
                        Text::new(building.description()),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ),
                    (
                        Text::new(
                            building
                                .cost()
                                .iter()
                                .map(|stack| format!("{}", stack))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        TextColor(
                            if building.cost().iter().fold(true, |acc, stack| acc
                                && inventory.has(&stack.item, stack.count))
                            {
                                Color::srgb(0.6, 0.9, 0.6)
                            } else {
                                Color::srgb(0.9, 0.6, 0.6)
                            }
                        ),
                    )
                ],
            )
        })
        .map(|c| commands.spawn(c).id())
        .collect::<Vec<_>>();

    let menu = commands
        .spawn((
            Menu,
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                column_gap: px(10),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            children![(
                Node {
                    width: px(180),
                    height: px(120),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(px(-20)).with_top(px(20)),
                    ..default()
                },
                children![(
                    Text::new("Build"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )]
            )],
        ))
        .id();

    let building_list = commands
        .spawn((Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(10),
            column_gap: px(10),
            margin: UiRect::left(px(50)).with_right(px(50)),
            ..default()
        },))
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
                margin: UiRect::all(px(0)).with_top(auto()),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![(
                Text::new(format!("[{}] Cancel", controls.print(Action::Cancel),)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )],
        ))
        .id();

    commands.get_entity(menu).unwrap().add_child(building_list);
    commands
        .get_entity(building_list)
        .unwrap()
        .add_children(building_buttons.as_slice());
    commands.get_entity(menu).unwrap().add_child(exit_button);
}

pub fn hide(mut commands: Commands, menu_entities: Query<Entity, With<Menu>>) {
    for e in menu_entities {
        commands.entity(e).despawn();
    }
}
