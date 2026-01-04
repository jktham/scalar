use crate::inventory::{Item, ItemStack};
use bevy::prelude::*;
use rand::{Rng, rng};

#[derive(Component, Default, Debug)]
pub struct Tile {
    pub tile_pos: IVec2,
    pub world_pos: Vec3,
}

#[derive(Component)]
pub struct ResourceNode {
    pub stack: ItemStack,
    pub tile_pos: IVec2,
    pub world_pos: Vec3,
}

const WORLD_SIZE: i32 = 21;

fn generate_tiles() -> Vec<Tile> {
    let mut tiles = Vec::new();
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            let tile_pos = ivec2(x - WORLD_SIZE / 2, y - WORLD_SIZE / 2);
            let world_pos = vec3(tile_pos.x as f32, 0.0, tile_pos.y as f32);
            tiles.push(Tile {
                tile_pos,
                world_pos,
            });
        }
    }
    tiles
}

fn generate_resources() -> Vec<ResourceNode> {
    let mut rng = rng();
    let mut nodes = Vec::new();
    for _ in 0..100 {
        let tile_pos = ivec2(
            rng.random_range(0..WORLD_SIZE) - WORLD_SIZE / 2,
            rng.random_range(0..WORLD_SIZE) - WORLD_SIZE / 2,
        );
        let world_pos = vec3(tile_pos.x as f32, 0.5, tile_pos.y as f32);
        let stack = if rng.random::<f32>() < 0.5 {
            ItemStack {
                item: Item::Iron,
                count: rng.random_range(0..100),
            }
        } else {
            ItemStack {
                item: Item::Copper,
                count: rng.random_range(0..100),
            }
        };
        nodes.push(ResourceNode {
            stack,
            tile_pos,
            world_pos,
        })
    }
    nodes
}

pub fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tiles = generate_tiles();
    let nodes = generate_resources();

    let mesh_handle_quad = meshes.add(Rectangle::new(1.0, 1.0));

    for tile in tiles {
        let transform = Transform::from_translation(tile.world_pos);
        let material_handle = materials.add(Color::srgb(0.0, rng().random_range(0.5..1.0), 0.0));

        commands.spawn((
            tile,
            Mesh3d(mesh_handle_quad.clone()),
            MeshMaterial3d(material_handle),
            transform
                * Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ));
    }

    let mesh_handle_cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

    for node in nodes {
        let transform = Transform::from_translation(node.world_pos);
        let material_handle = materials.add(node.stack.item.color());

        commands.spawn((
            node,
            Mesh3d(mesh_handle_cube.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}
