use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
pub struct PauseMenu;

pub fn pause_menu_interact(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    continue_button: Single<&Interaction, With<ContinueButton>>,
    settings_button: Single<&Interaction, With<SettingsButton>>,
    quit_button: Single<&Interaction, With<QuitButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
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

    if *continue_button == &Interaction::Pressed || keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Play)
    }

    if *settings_button == &Interaction::Pressed {
        // todo
    }

    if *quit_button == &Interaction::Pressed {
        writer.write(AppExit::Success);
    }
}

#[derive(Component)]
pub struct ContinueButton;

#[derive(Component)]
pub struct SettingsButton;

#[derive(Component)]
pub struct QuitButton;

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
                ContinueButton,
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
                SettingsButton,
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
                QuitButton,
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
