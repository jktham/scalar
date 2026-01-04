use crate::inventory::{Item, ItemStack};
use bevy::prelude::*;
use rand::{Rng, rng};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct Map {
    pub tiles: HashMap<IVec2, Entity>,
    pub nodes: HashMap<IVec2, Entity>,
}

#[derive(Component, Default, Debug)]
pub struct Tile {
    pub tile_pos: IVec2,
}

#[derive(Component)]
pub struct ResourceNode {
    pub stack: ItemStack,
    pub tile_pos: IVec2,
}

const WORLD_SIZE: i32 = 21;

pub fn tile_to_world(tile_pos: &IVec2) -> Vec3 {
    vec3(tile_pos.x as f32, 0.0, tile_pos.y as f32)
}

pub fn world_to_tile(world_pos: &Vec3) -> IVec2 {
    world_pos.xz().round().as_ivec2()
}

fn generate_tiles() -> Vec<Tile> {
    let mut tiles = Vec::new();
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            let tile_pos = ivec2(x - WORLD_SIZE / 2, y - WORLD_SIZE / 2);
            tiles.push(Tile { tile_pos });
        }
    }
    tiles
}

fn generate_nodes() -> Vec<ResourceNode> {
    let mut rng = rng();
    let mut nodes = Vec::new();
    for _ in 0..100 {
        let tile_pos = ivec2(
            rng.random_range(0..WORLD_SIZE) - WORLD_SIZE / 2,
            rng.random_range(0..WORLD_SIZE) - WORLD_SIZE / 2,
        );
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
        nodes.push(ResourceNode { stack, tile_pos })
    }
    nodes
}

pub fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<Map>,
) {
    let tiles = generate_tiles();
    let nodes = generate_nodes();

    let mesh_handle_quad = meshes.add(Rectangle::new(1.0, 1.0));

    for tile in tiles {
        let tile_pos = tile.tile_pos;
        let transform = Transform::from_translation(tile_to_world(&tile_pos));
        let material_handle = materials.add(Color::srgb(0.0, rng().random_range(0.5..1.0), 0.0));

        let id = commands
            .spawn((
                tile,
                Mesh3d(mesh_handle_quad.clone()),
                MeshMaterial3d(material_handle),
                transform
                    * Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ))
            .id();
        map.tiles.insert(tile_pos, id);
    }

    let mesh_handle_cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

    for node in nodes {
        let tile_pos = node.tile_pos;
        let transform = Transform::from_translation(tile_to_world(&tile_pos) + vec3(0.0, 0.5, 0.0));
        let material_handle = materials.add(node.stack.item.color());

        let id = commands
            .spawn((
                node,
                Mesh3d(mesh_handle_cube.clone()),
                MeshMaterial3d(material_handle),
                transform,
            ))
            .id();
        map.nodes.insert(tile_pos, id);
    }
}
