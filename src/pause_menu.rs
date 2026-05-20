use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
pub struct PauseMenu;

pub fn pause_menu_interact(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    continue_button: Single<&Interaction, With<ContinueButtonTag>>,
    settings_button: Single<&Interaction, With<SettingsButtonTag>>,
    quit_button: Single<&Interaction, With<QuitButtonTag>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut writer: MessageWriter<AppExit>,
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

    match *continue_button {
        Interaction::Pressed => next_state.set(GameState::Play),
        _ => (),
    }

    match *settings_button {
        _ => (),
    }

    match *quit_button {
        Interaction::Pressed => {
            writer.write(AppExit::Success);
        }
        _ => (),
    }
}

#[derive(Component)]
pub struct ContinueButtonTag;

#[derive(Component)]
pub struct SettingsButtonTag;

#[derive(Component)]
pub struct QuitButtonTag;

pub fn show_pause_menu(mut commands: Commands) {
    commands.spawn((
        PauseMenu,
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
        children![
            (
                Node {
                    width: px(180),
                    height: px(120),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    Text::new("Paused"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )]
            ),
            (
                ContinueButtonTag,
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
                    Text::new("Continue"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )]
            ),
            (
                SettingsButtonTag,
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
                    Text::new("Settings"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )]
            ),
            (
                QuitButtonTag,
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
                    Text::new("Quit"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )]
            ),
        ],
    ));
}

pub fn hide_pause_menu(mut commands: Commands, menu_entities: Query<Entity, With<PauseMenu>>) {
    for e in menu_entities {
        commands.entity(e).despawn();
    }
}
