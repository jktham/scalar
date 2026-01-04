use crate::{
    inventory::Inventory,
    map::{Map, ResourceNode, Tile, setup_map, world_to_tile},
    ui::{setup_ui, update_ui},
};
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    render::view::NoIndirectDrawing,
    window::{PresentMode, WindowResolution},
};

mod inventory;
mod map;
mod ui;

#[derive(Resource, Default)]
pub struct HoverPos(IVec2);

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            illuminance: 2000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        NoIndirectDrawing,
        Transform::from_xyz(-10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn update_camera(
    mut camera_transform: Single<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let left = Vec3::Y
        .cross(camera_transform.forward().as_vec3())
        .normalize();
    let front = left.cross(Vec3::Y).normalize();

    let mut movement = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement += left;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement -= left;
    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        movement += front;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement -= front;
    }

    let mut speed = 10.0;
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        speed *= 2.0;
    }

    camera_transform.translation += movement * speed * time.delta().as_secs_f32();
}

fn update_hover(
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    tiles_query: Query<(&mut Tile, &MeshMaterial3d<StandardMaterial>)>,
    mut nodes_query: Query<&mut ResourceNode>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hover_pos: ResMut<HoverPos>,
    map: Res<Map>,
    mut inventory: ResMut<Inventory>,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
        && let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
    {
        let point = ray.get_point(distance);
        let current_hover_pos: IVec2 = world_to_tile(&point);
        let prev_hover_pos: IVec2 = hover_pos.0;
        hover_pos.0 = current_hover_pos;

        if let Some(tile_id) = map.tiles.get(&prev_hover_pos)
            && let Ok((_tile, material_handle)) = tiles_query.get(*tile_id)
            && let Some(material) = materials.get_mut(&material_handle.0)
        {
            material.emissive = LinearRgba::BLACK;
        }

        if let Some(tile_id) = map.tiles.get(&current_hover_pos)
            && let Ok((_tile, material_handle)) = tiles_query.get(*tile_id)
            && let Some(material) = materials.get_mut(&material_handle.0)
        {
            material.emissive = LinearRgba::WHITE;
        }

        if mouse_input.just_pressed(MouseButton::Left)
            && let Some(node_id) = map.nodes.get(&current_hover_pos)
            && let Ok(mut node) = nodes_query.get_mut(*node_id)
            && node.stack.count > 0
        {
            node.stack.count -= 1;
            inventory.add(&node.stack.item, 1);
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "scalar".into(),
                    resolution: WindowResolution::new(960, 540),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
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
                        target_fps: 60.0,
                    },
                },
            },
        ))
        .insert_resource(HoverPos::default())
        .insert_resource(Map::default())
        .insert_resource(Inventory::default())
        .add_systems(Startup, (setup, setup_map, setup_ui))
        .add_systems(Update, (update_camera, update_hover, update_ui))
        .run();
}
