use std::f32::consts::PI;

use crate::inventory::{Item, ItemStack};
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use rand::{Rng, rng};

#[derive(Component)]
pub struct Node;

#[derive(Component)]
pub struct Tree;

#[derive(Component)]
pub struct Stump;

pub fn get_terrain_height(x: f32, z: f32) -> f32 {
    f32::sin(x * 0.3) * f32::cos(z * 0.3) * 1.0
}

const WORLD_SIZE: f32 = 100.0;

pub fn generate_terrain() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();

    const TRIANGLE_RADIUS: f32 = 1.0;
    const N_X: i32 = (WORLD_SIZE / (TRIANGLE_RADIUS * 1.5)) as i32;
    const N_Z: i32 = (N_X as f32 * 1.8) as i32;
    const INITIAL_OFFSET: Vec3 = Vec3::new(
        -N_X as f32 * 3.0 / 2.0 * TRIANGLE_RADIUS / 2.0,
        0.0,
        -N_Z as f32 * 0.866 * TRIANGLE_RADIUS / 2.0,
    );

    let mut offset = INITIAL_OFFSET;
    for ix in 0..N_X {
        offset.x += 3.0 / 2.0 * TRIANGLE_RADIUS;
        offset.z = INITIAL_OFFSET.z;

        for iz in 0..N_Z {
            offset.z += f32::sin(2.0 / 3.0 * PI) * TRIANGLE_RADIUS;

            let odd = (ix + iz) % 2 == 1;
            let mut center = offset;
            if odd {
                center -= Vec3::new(TRIANGLE_RADIUS / 2.0, 0.0, 0.0);
            }

            let mut v0 =
                center + Vec3::new(1.0, 0.0, 0.0) * if odd { 1.0 } else { -1.0 } * TRIANGLE_RADIUS;
            let mut v1 = center
                + Vec3::new(f32::cos(2.0 / 3.0 * PI), 0.0, f32::sin(2.0 / 3.0 * PI))
                    * if odd { 1.0 } else { -1.0 }
                    * TRIANGLE_RADIUS;
            let mut v2 = center
                + Vec3::new(f32::cos(4.0 / 3.0 * PI), 0.0, f32::sin(4.0 / 3.0 * PI))
                    * if odd { 1.0 } else { -1.0 }
                    * TRIANGLE_RADIUS;

            v0.y = get_terrain_height(v0.x, v0.z);
            v1.y = get_terrain_height(v1.x, v1.z);
            v2.y = get_terrain_height(v2.x, v2.z);

            vertices.push(v1);
            vertices.push(v0);
            vertices.push(v2);

            let normal = (v2 - v0).cross(v1 - v0).normalize();
            normals.push(normal);
            normals.push(normal);
            normals.push(normal);

            let mut rng = rand::rng();
            let color = Vec4::new(
                rng.random::<f32>() * 0.1,
                rng.random::<f32>() * 0.5 + 0.2,
                rng.random::<f32>() * 0.1,
                1.0,
            );
            colors.push(color);
            colors.push(color);
            colors.push(color);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    mesh
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // terrain
    let terrain_mesh = meshes.add(generate_terrain());
    let terrain_material = materials.add(StandardMaterial {
        reflectance: 0.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(terrain_mesh),
        MeshMaterial3d(terrain_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));

    // resource nodes
    let mut rng = rng();
    for _ in 0..100 {
        let mut pos = vec3(
            rng.random::<f32>() * WORLD_SIZE - WORLD_SIZE / 2.0,
            0.0,
            rng.random::<f32>() * WORLD_SIZE - WORLD_SIZE / 2.0,
        );
        pos.y = get_terrain_height(pos.x, pos.z);
        let rot = Quat::from_rotation_y(rng.random::<f32>() * std::f32::consts::TAU);

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

        let transform = Transform::from_translation(pos).with_rotation(rot);
        let node = if stack.item == Item::Iron {
            asset_server.load::<Scene>("node_iron.glb#Scene0")
        } else {
            asset_server.load::<Scene>("node_copper.glb#Scene0")
        };
        commands.spawn((Node, stack, SceneRoot(node.clone()), transform));
    }

    // trees
    for _ in 0..100 {
        let mut pos = vec3(
            rng.random::<f32>() * WORLD_SIZE - WORLD_SIZE / 2.0,
            0.0,
            rng.random::<f32>() * WORLD_SIZE - WORLD_SIZE / 2.0,
        );
        pos.y = get_terrain_height(pos.x, pos.z) - 0.1;
        let rot = Quat::from_rotation_y(rng.random::<f32>() * std::f32::consts::TAU);

        let stack = ItemStack {
            item: Item::Wood,
            count: 10,
        };

        let transform = Transform::from_translation(pos).with_rotation(rot);
        let node = asset_server.load::<Scene>("tree.glb#Scene0");
        commands.spawn((Tree, stack, SceneRoot(node.clone()), transform));
    }
}
