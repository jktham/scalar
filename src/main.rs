use avian3d::prelude::*;
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PresentMode, WindowResolution},
};
use bevy_obj::ObjPlugin;
use bevy_tnua::{TnuaControllerPlugin, TnuaUserControlsSystems};
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

mod build_menu;
mod hud;
mod inventory;
mod pause_menu;
mod player;
mod world;

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Play,
    PauseMenu,
    BuildMenu,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 2000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        // NoIndirectDrawing,
        ScreenSpaceAmbientOcclusion::default(),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::new(1.0, 0.0, 0.0), Vec3::Y),
        RayCaster::new(Vec3::ZERO, -Dir3::Z).with_max_distance(10.0),
        RayHits::default(),
    ));
}

fn cursor_grab(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

fn cursor_ungrab(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
}

fn handle_menu_keys(
    state: ResMut<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let next = match state.get() {
            GameState::Play => GameState::PauseMenu,
            GameState::PauseMenu | GameState::BuildMenu => GameState::Play,
        };
        next_state.set(next);
    }

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        let next = match state.get() {
            GameState::Play => GameState::BuildMenu,
            GameState::BuildMenu => GameState::Play,
            _ => return,
        };
        next_state.set(next);
    }
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
                        present_mode: PresentMode::AutoVsync,
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
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: true,
                        min_fps: 30.0,
                        target_fps: 120.0,
                    },
                },
            },
            ObjPlugin,
            PhysicsPlugins::default(),
            TnuaControllerPlugin::<player::ControlScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .init_state::<GameState>()
        .add_systems(
            Startup,
            (
                setup,
                cursor_grab,
                player::setup_player,
                world::setup_world,
                hud::setup_hud,
            ),
        )
        .add_systems(
            Update,
            (
                (
                    player::update_movement.in_set(TnuaUserControlsSystems),
                    player::update_hover,
                    player::update_interact,
                    hud::draw_inventory,
                )
                    .run_if(in_state(GameState::Play)),
                (pause_menu::pause_menu_interact).run_if(in_state(GameState::PauseMenu)),
                (build_menu::build_menu_interact).run_if(in_state(GameState::BuildMenu)),
                handle_menu_keys,
            ),
        )
        .add_systems(
            OnEnter(GameState::PauseMenu),
            (pause_time, cursor_ungrab, pause_menu::show_pause_menu),
        )
        .add_systems(
            OnExit(GameState::PauseMenu),
            (unpause_time, cursor_grab, pause_menu::hide_pause_menu),
        )
        .add_systems(
            OnEnter(GameState::BuildMenu),
            (pause_time, cursor_ungrab, build_menu::show_build_menu),
        )
        .add_systems(
            OnExit(GameState::BuildMenu),
            (unpause_time, cursor_grab, build_menu::hide_build_menu),
        )
        .run();
}
