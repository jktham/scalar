use crate::inventory::{Item, ItemStack};
use bevy::prelude::*;
use rand::{Rng, rng};

#[derive(Component)]
pub struct ResourceNode;

const WORLD_SIZE: f32 = 100.0;

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // terrain
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

    // resource nodes
    let mesh_handle_cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

    let mut rng = rng();
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

        let transform = Transform::from_translation(pos);
        let material_handle = materials.add(stack.item.color());

        commands.spawn((
            ResourceNode,
            stack,
            Mesh3d(mesh_handle_cube.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}
