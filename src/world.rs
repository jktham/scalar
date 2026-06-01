use crate::{player::GameLayer, worldgen::WorldGen};
use core::fmt;
use std::f32::consts::PI;

use crate::inventory::{Item, ItemStack};
use avian3d::{
    collision::collider::{ColliderConstructor, ColliderConstructorHierarchy, CollisionLayers},
    dynamics::rigid_body::RigidBody,
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use fastrand::Rng;

#[derive(Component)]
pub struct Terrain;

#[derive(Component, Debug)]
pub enum ResourceNode {
    Ore,
    Tree,
    Rock,
}

impl fmt::Display for ResourceNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResourceNode::Ore => write!(f, "Ore"),
            ResourceNode::Tree => write!(f, "Tree"),
            ResourceNode::Rock => write!(f, "Rock"),
        }
    }
}

pub const N_CHUNKS: i32 = 24;
pub const N_TILES_X: i32 = 40; // should be even
pub const N_TILES_Z: i32 = 70;
pub const TILE_RADIUS: f32 = 1.0;

pub const CHUNK_SIZE_X: f32 = N_TILES_X as f32 * TILE_RADIUS * 3.0 / 2.0;
pub const CHUNK_SIZE_Z: f32 =
    N_TILES_Z as f32 * TILE_RADIUS - 1.34 * TILE_RADIUS * N_TILES_Z as f32 / 10.0;
pub const WORLD_SIZE_X: f32 = N_CHUNKS as f32 * CHUNK_SIZE_X;
pub const WORLD_SIZE_Z: f32 = N_CHUNKS as f32 * CHUNK_SIZE_Z;

pub fn generate_chunk_mesh(cx: f32, cz: f32, worldgen: &Res<WorldGen>, rng: &mut Rng) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();

    let global_offset = Vec3::new(cx, 0.0, cz);
    let mut offset = Vec3::ZERO;

    for ix in 0..N_TILES_X {
        offset.x += 3.0 / 2.0 * TILE_RADIUS;
        offset.z = 0.0;

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

            v0.y = worldgen.get_height(global_offset.x + v0.x, global_offset.z + v0.z);
            v1.y = worldgen.get_height(global_offset.x + v1.x, global_offset.z + v1.z);
            v2.y = worldgen.get_height(global_offset.x + v2.x, global_offset.z + v2.z);

            vertices.push(v1);
            vertices.push(v0);
            vertices.push(v2);

            let normal = (v2 - v0).cross(v1 - v0).normalize();
            normals.push(normal);
            normals.push(normal);
            normals.push(normal);

            let steepness = normal.dot(Vec3::Y).powf(10.0);

            let color = Vec4::new(
                rng.f32() * 0.1 + (1.0 - steepness) * 0.1,
                rng.f32() * 0.1 + steepness * 0.1 + 0.1,
                rng.f32() * 0.05 + (1.0 - steepness) * 0.05,
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

/// mesh and chunk pos
pub fn generate_terrain_chunk_meshes(worldgen: &Res<WorldGen>, rng: &mut Rng) -> Vec<(Mesh, Vec3)> {
    let mut chunks = Vec::new();
    for icx in 0..N_CHUNKS {
        for icz in 0..N_CHUNKS {
            let cx = icx as f32 * CHUNK_SIZE_X - WORLD_SIZE_X / 2.0;
            let cz = icz as f32 * CHUNK_SIZE_Z - WORLD_SIZE_Z / 2.0;
            chunks.push((
                generate_chunk_mesh(cx, cz, worldgen, rng),
                Vec3::new(cx, 0.0, cz),
            ));
        }
    }
    chunks
}

#[derive(Component)]
/// entities with this marker are hidden by player::update_active_entities
pub struct CullDistance(pub f32);

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    worldgen: Res<WorldGen>,
) {
    let mut rng = Rng::with_seed(67);

    // terrain
    let chunk_meshes = generate_terrain_chunk_meshes(&worldgen, &mut rng);
    let mut terrain_batch = Vec::new();
    for mesh in chunk_meshes {
        let terrain_mesh = meshes.add(mesh.0);
        let terrain_material = materials.add(StandardMaterial {
            reflectance: 0.0,
            ..default()
        });

        terrain_batch.push((
            Terrain,
            Mesh3d(terrain_mesh),
            MeshMaterial3d(terrain_material),
            RigidBody::Static,
            ColliderConstructor::TrimeshFromMesh,
            Transform::from_translation(mesh.1),
            CullDistance(300.0),
            CollisionLayers::new(GameLayer::Terrain, [GameLayer::Player]),
        ));
    }
    commands.spawn_batch(terrain_batch);

    // ore
    let mut ore_batch = Vec::new();
    for _ in 0..500 {
        let mut pos = vec3(
            rng.f32() * WORLD_SIZE_X - WORLD_SIZE_X / 2.0,
            0.0,
            rng.f32() * WORLD_SIZE_Z - WORLD_SIZE_Z / 2.0,
        );
        pos.y = worldgen.get_height(pos.x, pos.z);
        let rot = Quat::from_rotation_y(rng.f32() * std::f32::consts::TAU);

        let normal = worldgen.get_normal(pos.x, pos.z);
        let normal_rot =
            Quat::from_axis_angle(normal.cross(Vec3::Y), -f32::acos(normal.dot(Vec3::Y)));

        let variant = rng.i32(0..3);
        let stack = match variant {
            0 => ItemStack {
                item: Item::Iron,
                count: rng.i32(100..1000),
            },
            1 => ItemStack {
                item: Item::Copper,
                count: rng.i32(100..1000),
            },
            _ => ItemStack {
                item: Item::Coal,
                count: rng.i32(100..1000),
            },
        };

        let transform = Transform::from_translation(pos).with_rotation(normal_rot * rot);
        let ore_scene = match variant {
            0 => asset_server.load::<Scene>("node_iron.glb#Scene0"),
            1 => asset_server.load::<Scene>("node_copper.glb#Scene0"),
            _ => asset_server.load::<Scene>("node_coal.glb#Scene0"),
        };

        ore_batch.push((
            ResourceNode::Ore,
            stack,
            SceneRoot(ore_scene),
            transform,
            RigidBody::Static,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
                .with_default_layers(CollisionLayers::new(GameLayer::Object, [GameLayer::Player])),
            CullDistance(300.0),
        ));
    }
    commands.spawn_batch(ore_batch);

    // trees
    let mut tree_batch = Vec::new();
    for _ in 0..20000 {
        let mut pos = vec3(
            rng.f32() * WORLD_SIZE_X - WORLD_SIZE_X / 2.0,
            0.0,
            rng.f32() * WORLD_SIZE_Z - WORLD_SIZE_Z / 2.0,
        );
        pos.y = worldgen.get_height(pos.x, pos.z);
        let rot = Quat::from_rotation_y(rng.f32() * std::f32::consts::TAU);

        let stack = ItemStack {
            item: Item::Wood,
            count: 5,
        };

        let transform = Transform::from_translation(pos).with_rotation(rot);
        let tree_scene = asset_server.load::<Scene>("tree.glb#Scene0");

        tree_batch.push((
            ResourceNode::Tree,
            stack,
            SceneRoot(tree_scene),
            transform,
            RigidBody::Static,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
                .with_default_layers(CollisionLayers::new(GameLayer::Object, [GameLayer::Player])),
            CullDistance(300.0),
        ));
    }
    commands.spawn_batch(tree_batch);

    // rocks
    let mut rock_batch = Vec::new();
    for _ in 0..4000 {
        let mut pos = vec3(
            rng.f32() * WORLD_SIZE_X - WORLD_SIZE_X / 2.0,
            0.0,
            rng.f32() * WORLD_SIZE_Z - WORLD_SIZE_Z / 2.0,
        );
        pos.y = worldgen.get_height(pos.x, pos.z);
        let rot = Quat::from_rotation_y(rng.f32() * std::f32::consts::TAU);

        let normal = worldgen.get_normal(pos.x, pos.z);
        let normal_rot =
            Quat::from_axis_angle(normal.cross(Vec3::Y), -f32::acos(normal.dot(Vec3::Y)));

        let stack = ItemStack {
            item: Item::Stone,
            count: 1,
        };

        let transform = Transform::from_translation(pos).with_rotation(normal_rot * rot);
        let variant = rng.i32(0..3);
        let rock_scene = asset_server.load::<Scene>(format!("rock_{}.glb#Scene0", variant));

        rock_batch.push((
            ResourceNode::Rock,
            stack,
            SceneRoot(rock_scene),
            transform,
            RigidBody::Static,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
                .with_default_layers(CollisionLayers::new(GameLayer::Object, [GameLayer::Player])),
            CullDistance(300.0),
        ));
    }
    commands.spawn_batch(rock_batch);
}

pub fn update_world(
    mut commands: Commands,
    nodes: Query<(&ResourceNode, Entity, &Transform, &ItemStack)>,
    asset_server: Res<AssetServer>,
) {
    for (node, entity, transform, stack) in nodes {
        if stack.count <= 0 {
            commands.entity(entity).despawn();

            match node {
                ResourceNode::Tree => {
                    commands.spawn((
                        SceneRoot(asset_server.load::<Scene>("stump.glb#Scene0")),
                        *transform,
                        RigidBody::Static,
                        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
                            .with_default_layers(CollisionLayers::new(
                                GameLayer::Object,
                                [GameLayer::Player],
                            )),
                        CullDistance(300.0),
                    ));
                }
                _ => {}
            }
        }
    }
}
