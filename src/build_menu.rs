use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
pub struct BuildMenu;

pub fn build_menu_interact(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    build_buttons: Query<(&Interaction, &BuildButton)>,
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

    for (interaction, build_button) in build_buttons {
        match *interaction {
            Interaction::Pressed => {
                println!("Building {}", build_button.building);
                next_state.set(GameState::Play);
            }
            _ => (),
        }
    }
}

#[derive(Component, Clone)]
pub struct BuildButton {
    building: i32,
}

pub fn show_build_menu(mut commands: Commands) {
    let buildings = vec![0, 1, 2, 3, 4, 5, 6, 7];

    let building_buttons = buildings
        .into_iter()
        .map(|building| {
            (
                BuildButton { building },
                Button,
                Node {
                    width: px(180),
                    height: px(180),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                children![(
                    Text::new(format!("Building {}", building)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            )
        })
        .map(|c| commands.spawn(c).id())
        .collect::<Vec<_>>();

    let menu = commands
        .spawn((
            BuildMenu,
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

    commands.get_entity(menu).unwrap().add_child(building_list);
    commands
        .get_entity(building_list)
        .unwrap()
        .add_children(building_buttons.as_slice());
}

pub fn hide_build_menu(mut commands: Commands, menu_entities: Query<Entity, With<BuildMenu>>) {
    for e in menu_entities {
        commands.entity(e).despawn();
    }
}
