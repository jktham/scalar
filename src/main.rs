use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
};
use rand::prelude::*;

#[derive(Debug)]
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
    const SIZE: i32 = 10;
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
    for tile in tiles {
        let transform = Transform::from_xyz(tile.pos.x as f32, 0.0, tile.pos.y as f32);
        let color = match tile.resource {
            TileResource::None => Color::srgb(0.0, 1.0, 0.0),
            TileResource::Iron => Color::srgb(0.0, 0.0, 1.0),
            TileResource::Copper => Color::srgb(1.0, 0.0, 0.0),
        };

        commands.spawn((
            tile,
            Mesh3d(meshes.add(Cuboid::new(1.0, 0.25, 1.0))),
            MeshMaterial3d(materials.add(color)),
            transform,
        ));
    }

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

}

fn list_tiles(query: Query<&Tile>) {
    for tile in &query {
        println!("{:?}", tile);
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
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
        .add_systems(Update, list_tiles)
        .run();
}
