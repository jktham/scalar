use crate::inventory::{Item, ItemStack};
use bevy::prelude::*;
use rand::{Rng, rng};

#[derive(Resource, Default)]
pub struct World {
    pub nodes: Vec<Entity>,
}

#[derive(Component)]
pub struct ResourceNode {
    pub stack: ItemStack,
    pub pos: Vec3,
}

const WORLD_SIZE: f32 = 100.0;

fn generate_nodes() -> Vec<ResourceNode> {
    let mut rng = rng();
    let mut nodes = Vec::new();
    for _ in 0..100 {
        let pos = vec3(
            rng.random::<f32>() * WORLD_SIZE - WORLD_SIZE / 2.0,
            0.5,
            rng.random::<f32>() * WORLD_SIZE - WORLD_SIZE / 2.0,
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
        nodes.push(ResourceNode { stack, pos })
    }
    nodes
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<World>,
    asset_server: Res<AssetServer>,
) {
    // let world_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::new(WORLD_SIZE, WORLD_SIZE)*2.0));

    let world_mesh = asset_server.load("world_mesh.obj");
    let world_texture = asset_server.load("world_mesh.png");

    commands.spawn((
        Mesh3d(world_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(world_texture),
            reflectance: 0.0,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));

    let mesh_handle_cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

    let nodes = generate_nodes();
    for node in nodes {
        let transform = Transform::from_translation(node.pos);
        let material_handle = materials.add(node.stack.item.color());

        let id = commands
            .spawn((
                node,
                Mesh3d(mesh_handle_cube.clone()),
                MeshMaterial3d(material_handle),
                transform,
            ))
            .id();
        world.nodes.push(id);
    }
}
