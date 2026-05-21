use crate::worldgen::get_terrain_height;
use std::f32::consts::PI;

use crate::inventory::{Item, ItemStack};
use avian3d::{
    collision::collider::{Collider, ColliderConstructor},
    dynamics::rigid_body::RigidBody,
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use rand::Rng;

#[derive(Component)]
pub struct Terrain;

#[derive(Component)]
pub struct ResourceNode;

#[derive(Component)]
pub struct Tree;

#[derive(Component)]
pub struct Stump;

#[derive(Component)]
pub struct Rock;

const N_CHUNKS: i32 = 9;
const N_TILES_X: i32 = 20; // should be even
const N_TILES_Z: i32 = 36;
const TILE_RADIUS: f32 = 1.0;

const CHUNK_SIZE_X: f32 = N_TILES_X as f32 * TILE_RADIUS * 3.0 / 2.0;
const CHUNK_SIZE_Z: f32 =
    N_TILES_Z as f32 * TILE_RADIUS - 1.34 * TILE_RADIUS * N_TILES_Z as f32 / 10.0;
const WORLD_SIZE_X: f32 = N_CHUNKS as f32 * CHUNK_SIZE_X;
const WORLD_SIZE_Z: f32 = N_CHUNKS as f32 * CHUNK_SIZE_Z;

pub fn generate_chunk_mesh(cx: f32, cz: f32) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();

    let initial_offset = Vec3::new(cx, 0.0, cz);
    let mut offset = initial_offset;

    for ix in 0..N_TILES_X {
        offset.x += 3.0 / 2.0 * TILE_RADIUS;
        offset.z = initial_offset.z;

        for iz in 0..N_TILES_Z {
            offset.z += f32::sin(2.0 / 3.0 * PI) * TILE_RADIUS;

            let odd = (ix + iz) % 2 == 1;
            let mut center = offset;
            if odd {
                center -= Vec3::new(TILE_RADIUS / 2.0, 0.0, 0.0);
            }

            let mut v0 =
                center + Vec3::new(1.0, 0.0, 0.0) * if odd { 1.0 } else { -1.0 } * TILE_RADIUS;
            let mut v1 = center
                + Vec3::new(f32::cos(2.0 / 3.0 * PI), 0.0, f32::sin(2.0 / 3.0 * PI))
                    * if odd { 1.0 } else { -1.0 }
                    * TILE_RADIUS;
            let mut v2 = center
                + Vec3::new(f32::cos(4.0 / 3.0 * PI), 0.0, f32::sin(4.0 / 3.0 * PI))
                    * if odd { 1.0 } else { -1.0 }
                    * TILE_RADIUS;

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

            let steepness = normal.dot(Vec3::Y).powf(10.0);

            let mut rng = rand::rng();
            let color = Vec4::new(
                rng.random::<f32>() * 0.1 + (1.0 - steepness) * 0.1,
                rng.random::<f32>() * 0.1 + steepness * 0.1 + 0.1,
                rng.random::<f32>() * 0.05 + (1.0 - steepness) * 0.05,
                1.0,
            );
            colors.push(color);
            colors.push(color);
            colors.push(color);
        }
    }

    let n_verts = vertices.len();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U16((0..n_verts).map(|i| i as u16).collect()));

    mesh
}

pub fn generate_terrain_meshes() -> Vec<Mesh> {
    let mut chunks = Vec::new();
    for icx in 0..N_CHUNKS {
        for icz in 0..N_CHUNKS {
            chunks.push(generate_chunk_mesh(
                icx as f32 * CHUNK_SIZE_X - WORLD_SIZE_X / 2.0,
                icz as f32 * CHUNK_SIZE_Z - WORLD_SIZE_Z / 2.0,
            ));
        }
    }
    chunks
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // terrain
    let chunk_meshes = generate_terrain_meshes();
    for mesh in chunk_meshes {
        let terrain_mesh = meshes.add(mesh.clone());
        let terrain_material = materials.add(StandardMaterial {
            reflectance: 0.0,
            ..default()
        });

        commands.spawn((
            Terrain,
            Mesh3d(terrain_mesh),
            MeshMaterial3d(terrain_material),
            RigidBody::Static,
            ColliderConstructor::TrimeshFromMesh,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    }

    // resource nodes
    let mut rng = rand::rng();
    for _ in 0..20 {
        let mut pos = vec3(
            rng.random::<f32>() * WORLD_SIZE_X - WORLD_SIZE_X / 2.0,
            0.0,
            rng.random::<f32>() * WORLD_SIZE_Z - WORLD_SIZE_Z / 2.0,
        );
        pos.y = get_terrain_height(pos.x, pos.z);
        let rot = Quat::from_rotation_y(rng.random::<f32>() * std::f32::consts::TAU);

        let stack = if rng.random::<f32>() < 0.5 {
            ItemStack {
                item: Item::Iron,
                count: rng.random_range(1000..10000),
            }
        } else {
            ItemStack {
                item: Item::Copper,
                count: rng.random_range(1000..10000),
            }
        };

        let transform = Transform::from_translation(pos).with_rotation(rot); // TODO: align with terrain normal
        let node = if stack.item == Item::Iron {
            asset_server.load::<Scene>("node_iron.glb#Scene0")
        } else {
            asset_server.load::<Scene>("node_copper.glb#Scene0")
        };
        commands.spawn((
            ResourceNode,
            stack,
            SceneRoot(node.clone()),
            transform,
            RigidBody::Static,
            Collider::sphere(0.5),
        ));
    }

    // trees
    for _ in 0..800 {
        let mut pos = vec3(
            rng.random::<f32>() * WORLD_SIZE_X - WORLD_SIZE_X / 2.0,
            0.0,
            rng.random::<f32>() * WORLD_SIZE_Z - WORLD_SIZE_Z / 2.0,
        );
        pos.y = get_terrain_height(pos.x, pos.z);
        let rot = Quat::from_rotation_y(rng.random::<f32>() * std::f32::consts::TAU);

        let stack = ItemStack {
            item: Item::Wood,
            count: 5,
        };

        let transform = Transform::from_translation(pos).with_rotation(rot);
        let node = asset_server.load::<Scene>("tree.glb#Scene0");
        commands.spawn((
            Tree,
            stack,
            SceneRoot(node.clone()),
            transform,
            RigidBody::Static,
            Collider::cylinder(0.3, 8.0),
        ));
    }
}

pub fn update_trees(
    trees: Query<(Entity, &Transform, &ItemStack), With<Tree>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (entity, transform, stack) in trees {
        // if tree is depleted, despawn it and spawn a stump
        if stack.count <= 0 {
            commands.entity(entity).despawn();
            commands.spawn((
                Stump,
                SceneRoot(asset_server.load::<Scene>("stump.glb#Scene0")),
                *transform,
                Collider::cylinder(0.5, 4.0),
            ));
        }
    }
}
