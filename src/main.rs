use avian3d::prelude::*;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PresentMode, PrimaryWindow, WindowResolution},
};
use bevy_framepace::FramepacePlugin;
use bevy_hanabi::prelude::*;
use bevy_tnua::{TnuaControllerPlugin, TnuaUserControlsSystems};
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

mod build_menu;
mod building_menu;
mod buildings;
mod effects;
mod environment;
mod hud;
mod inventory;
mod pause_menu;
mod player;
mod world;
mod worldgen;

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Play,
    PauseMenu,
    BuildMenu,
    BuildingMenu,
}

fn cursor_grab(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

fn cursor_ungrab(
    mut cursor_options: Single<&mut CursorOptions>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let size = window.size();
    window.set_cursor_position(Some(size / 2.0));

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
            FramepacePlugin,
            HanabiPlugin,
        ))
        .init_state::<GameState>()
        .insert_resource(worldgen::WorldGen::generate())
        .insert_resource(effects::EffectMap::default())
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
                    player::update_hover_target,
                    player::update_hover_action,
                    player::update_interact,
                    player::place_held_building,
                )
                    .run_if(in_state(GameState::Play)),
                player::update_movement_noinput.run_if(not(in_state(GameState::Play))),
                hud::draw_inventory,
                buildings::update_building_animations,
                buildings::update_building_effects,
                world::update_world,
                (pause_menu::pause_menu_interact).run_if(in_state(GameState::PauseMenu)),
                (build_menu::build_menu_interact).run_if(in_state(GameState::BuildMenu)),
                (
                    building_menu::building_menu_interact,
                    building_menu::building_menu_update,
                )
                    .run_if(in_state(GameState::BuildingMenu)),
            ),
        )
        .add_systems(FixedUpdate, buildings::update_buildings)
        .add_systems(
            OnEnter(GameState::PauseMenu),
            (
                pause_time,
                cursor_ungrab,
                pause_menu::show_pause_menu,
                hud::hide_hud,
            ),
        )
        .add_systems(
            OnExit(GameState::PauseMenu),
            (
                unpause_time,
                cursor_grab,
                pause_menu::hide_pause_menu,
                hud::show_hud,
            ),
        )
        .add_systems(
            OnEnter(GameState::BuildMenu),
            (cursor_ungrab, build_menu::show_build_menu, hud::hide_hud),
        )
        .add_systems(
            OnExit(GameState::BuildMenu),
            (cursor_grab, build_menu::hide_build_menu, hud::show_hud),
        )
        .add_systems(
            OnEnter(GameState::BuildingMenu),
            (
                cursor_ungrab,
                building_menu::show_building_menu,
                hud::hide_hud,
            ),
        )
        .add_systems(
            OnExit(GameState::BuildingMenu),
            (
                cursor_grab,
                building_menu::hide_building_menu,
                hud::show_hud,
            ),
        )
        .run();
}
