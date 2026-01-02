use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig}, prelude::*, render::view::NoIndirectDrawing, window::{PresentMode, WindowResolution}
};
use rand::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq)]
enum TileResource {
    None,
    Iron,
    Copper,
}

impl Default for TileResource {
    fn default() -> Self {
        TileResource::None
    }
}

impl Distribution<TileResource> for rand::distr::StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileResource {
        match rng.random_range(0..100) {
            0..10 => TileResource::Iron,
            10..20 => TileResource::Copper,
            _ => TileResource::None,
        }
    }
}

#[derive(Component, Default, Debug)]
struct Tile {
    pos: IVec2,
    resource: TileResource,
}

fn generate_tiles() -> Vec<Tile> {
    const SIZE: i32 = 21;
    let mut rng = rand::rng();

    let mut tiles = Vec::new();
    for x in 0..SIZE {
        for y in 0..SIZE {
            tiles.push(
                Tile {
                    pos: IVec2 { x: x - SIZE/2, y: y - SIZE/2 },
                    resource: rng.random::<TileResource>(),
                }
            );
        }
    }
    return tiles;
}

fn setup (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tiles = generate_tiles();
    let mesh_handle_quad = meshes.add(Rectangle::new(1.0, 1.0));

    for tile in tiles {
        let transform = Transform::from_xyz(tile.pos.x as f32, 0.0, tile.pos.y as f32);
        let material_handle = match tile.resource {
            TileResource::None => materials.add(Color::srgb(0.0, 1.0, 0.0)),
            TileResource::Iron => materials.add(Color::srgb(0.0, 0.0, 1.0)),
            TileResource::Copper => materials.add(Color::srgb(1.0, 0.0, 0.0)),
        };

        commands.spawn((
            tile,
            Mesh3d(mesh_handle_quad.clone()),
            MeshMaterial3d(material_handle),
            transform * Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ));
    }

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        NoIndirectDrawing,
        Transform::from_xyz(-10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));

}

fn update_camera (
    mut camera: Single<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let left = Vec3::Y.cross(camera.forward().as_vec3()).normalize();
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

    camera.translation += movement * speed * time.delta().as_secs_f32();
}

fn update_cursor (
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    tiles_query: Query<(&Tile, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
        && let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
    {
        let point = ray.get_point(distance);

        let tile_coords: IVec2 = point.xz().round().as_ivec2();
        for (tile, material_handle) in tiles_query {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                if tile.pos == tile_coords {
                    material.emissive = LinearRgba::WHITE;
                } else {
                    material.emissive = LinearRgba::BLACK;
                }
            }
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
                        font_size: 20.0,
                        ..default()
                    },
                    text_color: Color::srgb(0.0, 1.0, 0.0),
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
        .add_systems(Startup, setup)
        .add_systems(Update, (update_camera, update_cursor).chain())
        .run();
}
