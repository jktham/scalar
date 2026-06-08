use std::f32::consts::PI;

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    math::ops::atan2,
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    GameState,
    controls::{Action, Controls},
    player::Player,
    worldgen::TERRAIN_SIZE,
};

#[derive(Component)]
pub struct Menu;

#[derive(Component)]
pub struct Map;

#[derive(Component)]
pub struct PlayerMarker;

pub fn interact(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    mut map: Single<&mut Node, With<Map>>,
    mut player_marker: Single<(&mut Node, &mut UiTransform), (With<PlayerMarker>, Without<Map>)>,
    exit_button: Single<&Interaction, With<ExitButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    player_transform: Single<&Transform, With<Player>>,
    camera_transform: Single<&Transform, With<Camera>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut next_state: ResMut<NextState<GameState>>,
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

    // todo: this is a mess
    let map_width = map
        .width
        .resolve(1.0, window.width(), window.size())
        .unwrap_or(1.0);
    let map_position = Vec2::new(
        map.left
            .resolve(1.0, window.width(), window.size())
            .unwrap_or(1.0),
        map.top
            .resolve(1.0, window.width(), window.size())
            .unwrap_or(1.0),
    );

    let mut map_offset = Vec2::new(
        match map.left {
            Val::Px(p) => p,
            _ => 0.0,
        },
        match map.top {
            Val::Px(p) => p,
            _ => 0.0,
        },
    );
    let mut map_scale = match map.width {
        Val::Percent(s) => s / 100.0,
        _ => 1.00,
    };

    let player_offset =
        player_transform.translation.zx() * Vec2::new(1.0, -1.0) * map_width / TERRAIN_SIZE;
    let player_rotation = {
        let up = Vec3::Y;
        let front = camera_transform.forward().as_vec3();
        let flattened = front - front.dot(up) * up;
        let angle = atan2(flattened.x, flattened.z);
        (angle - PI / 2.0).to_degrees()
    };

    if map_offset == Vec2::ZERO {
        map_offset -= player_offset; // center on player on first open
    }
    let mut marker_offset = player_offset + map_offset;

    if mouse_button.pressed(MouseButton::Left) {
        let motion = mouse_motion.delta;

        map_offset += motion;
        marker_offset = player_offset + map_offset + motion;
    }

    if mouse_scroll.delta.y != 0.0 {
        let delta = match mouse_scroll.unit {
            MouseScrollUnit::Line => mouse_scroll.delta.y * 10.0,
            MouseScrollUnit::Pixel => mouse_scroll.delta.y * 1.0,
        };

        let multiplier = if delta > 0.0 {
            1.0 + delta.abs() * 0.01
        } else {
            1.0 / (1.0 + delta.abs() * 0.01)
        };

        map_scale *= multiplier;

        // offset to zoom on cursor
        let cursor_position = window.cursor_position().unwrap_or(Vec2::ZERO);
        let cursor_offset = cursor_position - (map_position + window.size() / 2.0);

        map_offset -= cursor_offset / map_width * (map_width * multiplier - map_width);

        marker_offset = player_offset * multiplier + map_offset;
    }

    map.left = px(map_offset.x);
    map.top = px(map_offset.y);
    map.width = percent(map_scale * 100.0);

    player_marker.0.left = px(marker_offset.x);
    player_marker.0.top = px(marker_offset.y);
    player_marker.1.rotation = Rot2::degrees(-player_rotation);

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

#[derive(Component)]
pub struct ExitButton;

pub fn show(mut commands: Commands, controls: Res<Controls>, asset_server: Res<AssetServer>) {
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
                    Text::new("Map"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )]
            )],
        ))
        .id();

    let map_container = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ZIndex(-2),
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.)),
        ))
        .id();

    let map_image = asset_server.load::<Image>("map/relief.png");
    let map = commands
        .spawn((
            Map,
            Node {
                top: px(0),
                left: px(0),
                width: percent(300),
                aspect_ratio: Some(1.0),
                ..default()
            },
            ImageNode {
                image: map_image.clone(),
                ..default()
            },
            ZIndex(-2),
        ))
        .id();

    let player_marker_container = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ZIndex(-2),
        ))
        .id();

    let player_marker_image = asset_server.load::<Image>("textures/player_marker.png");
    let player_marker = commands
        .spawn((
            PlayerMarker,
            Node {
                top: px(0),
                left: px(0),
                width: percent(2),
                aspect_ratio: Some(1.0),
                ..default()
            },
            ImageNode {
                image: player_marker_image.clone(),
                color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                ..default()
            },
            UiTransform::from_rotation(Rot2::degrees(0.0)),
            ZIndex(-1),
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
                margin: UiRect::all(px(0)).with_top(auto()),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![(
                Text::new(format!("[{}] Exit", controls.print(Action::Cancel),)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )],
        ))
        .id();

    commands.get_entity(map_container).unwrap().add_child(map);
    commands.get_entity(menu).unwrap().add_child(map_container);
    commands
        .get_entity(player_marker_container)
        .unwrap()
        .add_child(player_marker);
    commands
        .get_entity(menu)
        .unwrap()
        .add_child(player_marker_container);
    commands.get_entity(menu).unwrap().add_child(exit_button);
}

pub fn hide(mut commands: Commands, menu_entities: Query<Entity, With<Menu>>) {
    for e in menu_entities {
        commands.entity(e).despawn();
    }
}
