use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    time::common_conditions::on_timer,
    window::{CursorGrabMode, CursorOptions, PresentMode, WindowResolution},
};
// use bevy_framepace::FramepacePlugin;
use bevy_hanabi::prelude::*;
use bevy_tnua::{TnuaControllerPlugin, TnuaUserControlsSystems};
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

mod buildings;
mod controls;
mod effects;
mod environment;
mod hud;
mod inventory;
mod menus;
mod player;
mod unlocks;
mod world;
mod worldgen;

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Play,
    PauseMenu,
    BuildMenu,
    BuildingMenu,
    MapMenu,
    ResearchMenu,
}

fn cursor_grab(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

fn cursor_ungrab(
    mut cursor_options: Single<&mut CursorOptions>,
    // mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    // let size = window.size();
    // window.set_cursor_position(Some(size / 2.0)); // broken on wayland

    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
}

fn pause_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn unpause_time(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "scalar".into(),
                        resolution: WindowResolution::new(960, 540),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    text_color: Color::WHITE,
                    refresh_interval: core::time::Duration::from_millis(500),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: true,
                        min_fps: 60.0,
                        target_fps: 120.0,
                    },
                },
            },
            PhysicsPlugins::default(),
            TnuaControllerPlugin::<player::ControlScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            // FramepacePlugin,
            HanabiPlugin,
        ))
        .init_state::<GameState>()
        .insert_resource(worldgen::WorldGen::generate())
        .insert_resource(effects::Effects::default())
        .insert_resource(controls::Controls::default())
        .add_systems(
            Startup,
            (
                cursor_grab,
                player::setup_player,
                world::setup_world,
                environment::setup_environment,
                hud::setup_hud,
                effects::create_smoke_effect,
            ),
        )
        .add_systems(
            Update,
            (
                (
                    player::update_movement.in_set(TnuaUserControlsSystems),
                    player::update_target_text,
                    player::update_action_text,
                    player::interact,
                    player::place_held_building,
                    world::cull_visibility.run_if(on_timer(Duration::from_secs_f32(1.0))),
                    world::insert_colliders.run_if(on_timer(Duration::from_secs_f32(1.0))),
                )
                    .run_if(in_state(GameState::Play)),
                player::update_movement_noinput.run_if(not(in_state(GameState::Play))),
                hud::update_inventory,
                hud::update_money,
                buildings::update_building_animations,
                buildings::update_building_effects,
                world::remove_depleted_nodes,
                (menus::pause::interact).run_if(in_state(GameState::PauseMenu)),
                (menus::build::interact).run_if(in_state(GameState::BuildMenu)),
                (menus::building::interact, menus::building::update)
                    .run_if(in_state(GameState::BuildingMenu)),
                (menus::map::interact).run_if(in_state(GameState::MapMenu)),
                (menus::research::interact).run_if(in_state(GameState::ResearchMenu)),
            ),
        )
        .add_systems(FixedUpdate, buildings::update_buildings)
        .add_systems(
            OnEnter(GameState::PauseMenu),
            (pause_time, cursor_ungrab, menus::pause::show, hud::hide_hud),
        )
        .add_systems(
            OnExit(GameState::PauseMenu),
            (unpause_time, cursor_grab, menus::pause::hide, hud::show_hud),
        )
        .add_systems(
            OnEnter(GameState::BuildMenu),
            (cursor_ungrab, menus::build::show, hud::hide_hud),
        )
        .add_systems(
            OnExit(GameState::BuildMenu),
            (cursor_grab, menus::build::hide, hud::show_hud),
        )
        .add_systems(
            OnEnter(GameState::BuildingMenu),
            (cursor_ungrab, menus::building::show, hud::hide_hud),
        )
        .add_systems(
            OnExit(GameState::BuildingMenu),
            (cursor_grab, menus::building::hide, hud::show_hud),
        )
        .add_systems(
            OnEnter(GameState::MapMenu),
            (cursor_ungrab, menus::map::show, hud::hide_hud),
        )
        .add_systems(
            OnExit(GameState::MapMenu),
            (cursor_grab, menus::map::hide, hud::show_hud),
        )
        .add_systems(
            OnEnter(GameState::ResearchMenu),
            (cursor_ungrab, menus::research::show, hud::hide_hud),
        )
        .add_systems(
            OnExit(GameState::ResearchMenu),
            (cursor_grab, menus::research::hide, hud::show_hud),
        )
        .run();
}
