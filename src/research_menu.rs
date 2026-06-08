use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::{
    GameState,
    controls::{Action, Controls},
    inventory::Inventory,
    player::{Money, Player},
    unlocks::{Unlock, Unlocks},
};

#[derive(Component)]
pub struct ResearchMenu;

pub fn research_menu_interact(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    unlock_buttons: Query<(&Interaction, &UnlockButton)>,
    exit_button: Single<&Interaction, With<ExitButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut inventory: Single<&mut Inventory, With<Player>>,
    mut money: Single<&mut Money, With<Player>>,
    mut unlocks: Single<&mut Unlocks, With<Player>>,
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

    for (interaction, unlock_button) in unlock_buttons {
        if *interaction == Interaction::Pressed {
            if unlocks.0.contains(&unlock_button.0) {
                continue; // already unlocked
            }

            let cost = unlock_button.0.cost();
            if !(money.0 >= cost.0
                && cost.1.iter().fold(true, |acc, stack| {
                    acc && inventory.has(&stack.item, stack.count)
                }))
            {
                // cant afford
                continue;
            }

            money.0 -= cost.0;
            for stack in cost.1 {
                inventory.remove(&stack.item, stack.count);
            }

            unlocks.0.insert(unlock_button.0);

            next_state.set(GameState::Play);
        }
    }

    if *exit_button == &Interaction::Pressed
        || keyboard_input.just_pressed(controls.get(Action::Cancel))
        || keyboard_input.just_pressed(controls.get(Action::Pause))
    {
        if keyboard_input.just_pressed(controls.get(Action::Pause)) {
            next_state.set(GameState::PauseMenu);
        } else {
            next_state.set(GameState::Play);
        }
    }
}

#[derive(Component, Clone)]
pub struct UnlockButton(pub Unlock);

#[derive(Component)]
pub struct ExitButton;

pub fn show_research_menu(
    mut commands: Commands,
    inventory: Single<&Inventory, With<Player>>,
    money: Single<&Money, With<Player>>,
    unlocks: Single<&Unlocks, With<Player>>,
    controls: Res<Controls>,
) {
    let unlock_buttons = Unlock::iter()
        .map(|unlock| {
            (
                UnlockButton(unlock),
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
                        Text::new(unlock.name()),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        TextColor(if !unlocks.0.contains(&unlock) {
                            Color::srgb(1.0, 1.0, 1.0)
                        } else {
                            Color::srgb(0.6, 0.6, 0.6)
                        }),
                    ),
                    (
                        Text::new(unlock.description()),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        TextColor(if !unlocks.0.contains(&unlock) {
                            Color::srgb(0.8, 0.8, 0.8)
                        } else {
                            Color::srgb(0.4, 0.4, 0.4)
                        }),
                    ),
                    (
                        Text::new(
                            format!("${}, ", unlock.cost().0)
                                + unlock
                                    .cost()
                                    .1
                                    .iter()
                                    .map(|stack| format!("{}", stack))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                                    .as_str()
                        ),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
                        TextColor(if !unlocks.0.contains(&unlock) {
                            if money.0 >= unlock.cost().0
                                && unlock.cost().1.iter().fold(true, |acc, stack| {
                                    acc && inventory.has(&stack.item, stack.count)
                                })
                            {
                                Color::srgb(0.6, 0.9, 0.6)
                            } else {
                                Color::srgb(0.9, 0.6, 0.6)
                            }
                        } else {
                            Color::srgb(0.4, 0.4, 0.4)
                        }),
                    )
                ],
            )
        })
        .map(|c| commands.spawn(c).id())
        .collect::<Vec<_>>();

    let menu = commands
        .spawn((
            ResearchMenu,
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
                    Text::new("Research"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )]
            )],
        ))
        .id();

    let unlock_list = commands
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

    commands.get_entity(menu).unwrap().add_child(unlock_list);
    commands
        .get_entity(unlock_list)
        .unwrap()
        .add_children(unlock_buttons.as_slice());
    commands.get_entity(menu).unwrap().add_child(exit_button);
}

pub fn hide_research_menu(
    mut commands: Commands,
    menu_entities: Query<Entity, With<ResearchMenu>>,
) {
    for e in menu_entities {
        commands.entity(e).despawn();
    }
}
