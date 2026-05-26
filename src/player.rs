use crate::buildings::MiningNode;
use crate::buildings::ProcessingStatus;
use crate::buildings::RunningAnimation;
use crate::effects::EffectMap;
use crate::world::ResourceNode;
use crate::world::Terrain;
use crate::worldgen::WorldGen;
use crate::{
    buildings::{Building, Processing},
    hud::{ActionText, TargetText},
    inventory::{Inventory, ItemStack},
};
use avian3d::collision::collider::ColliderConstructor;
use avian3d::collision::collider::ColliderConstructorHierarchy;
use avian3d::dynamics::rigid_body::Friction;
use avian3d::spatial_query::RayHitData;
use avian3d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LockedAxes, RigidBody},
    spatial_query::RayHits,
};
use bevy::prelude::*;
use bevy::{
    camera::Camera,
    ecs::{
        component::Component,
        query::With,
        system::{Res, Single},
    },
    input::{ButtonInput, keyboard::KeyCode, mouse::AccumulatedMouseMotion},
    math::{Quat, Vec3},
    transform::components::Transform,
};
use bevy_hanabi::ParticleEffect;
use bevy_tnua::{
    TnuaConfig, TnuaController, TnuaScheme,
    builtins::{TnuaBuiltinJump, TnuaBuiltinJumpConfig, TnuaBuiltinWalk, TnuaBuiltinWalkConfig},
};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerProcessing {
    pub speed: f32,
    pub progress: f32,
}

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum ControlScheme {
    Jump(TnuaBuiltinJump),
}

pub fn setup_player(
    mut commands: Commands,
    mut control_scheme_configs: ResMut<Assets<ControlSchemeConfig>>,
    worldgen: Res<WorldGen>,
) {
    let mut spawn_pos = Vec3::new(0.0, 0.0, 0.0);
    spawn_pos.y = worldgen.get_height(spawn_pos.x, spawn_pos.z) + 2.0;

    commands.spawn((
        Player,
        PlayerProcessing {
            speed: 0.25,
            progress: 0.0,
        },
        Inventory::default(),
        Transform::from_translation(spawn_pos),
        RigidBody::Dynamic,
        Collider::capsule(0.3, 2.0),
        Friction::new(0.1),
        TnuaController::<ControlScheme>::default(),
        TnuaConfig::<ControlScheme>(control_scheme_configs.add(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                float_height: 1.5,
                cling_distance: 0.0,
                acceleration: 120.0,
                air_acceleration: 60.0,
                spring_strength: 200.0,
                max_slope: f32::to_radians(60.0),
                ..Default::default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 2.0,
                ..Default::default()
            },
        })),
        TnuaAvian3dSensorShape(Collider::cylinder(0.2, 0.0)),
        LockedAxes::ROTATION_LOCKED,
    ));
}

pub fn update_movement(
    mut player_controller: Single<&mut TnuaController<ControlScheme>, With<Player>>,
    player_transform: Single<&mut Transform, With<Player>>,
    mut camera_transform: Single<&mut Transform, (With<Camera>, Without<Player>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    // calculate movement direction based on camera orientation and WASD input
    let left = Vec3::Y
        .cross(camera_transform.forward().as_vec3())
        .normalize();
    let front = left.cross(Vec3::Y).normalize(); // in plane

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

    const SPEED: f32 = 0.5;
    let mut speed = SPEED;
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        speed *= 2.0;
    }

    // update player controller
    player_controller.initiate_action_feeding();
    player_controller.basis = TnuaBuiltinWalk {
        desired_motion: movement.normalize_or_zero() * speed,
        desired_forward: Some(Dir3::new_unchecked(
            Vec3::new(front.x, 0.0, front.z).normalize(),
        )),
        ..Default::default()
    };
    if keyboard_input.pressed(KeyCode::Space) {
        player_controller.action(ControlScheme::Jump(Default::default()));
    }

    // update camera
    const SENSITIVITY: f32 = 0.001;
    camera_transform.rotation = Quat::from_rotation_y(-mouse_motion.delta.x * SENSITIVITY)
        * Quat::from_axis_angle(left, mouse_motion.delta.y * SENSITIVITY)
        * camera_transform.rotation; // TODO: prevent flipping over pole

    camera_transform.translation = player_transform.translation + Vec3::new(0.0, 0.0, 0.0);
}

const RANGE: f32 = 6.0;
/// get closest hit within range, ignoring specified entities
fn get_closest_hit(rayhits: &RayHits, ignored: Vec<Entity>) -> Option<RayHitData> {
    let mut closest_hit = None;
    for hit in rayhits.iter_sorted() {
        if hit.distance > RANGE || ignored.contains(&hit.entity) {
            continue;
        }
        closest_hit = Some(hit);
        break;
    }
    closest_hit
}

pub fn update_hover_target(
    camera_rayhits: Single<&RayHits, With<Camera>>,
    player: Single<Entity, With<Player>>,
    mut target_text: Single<&mut Text, With<TargetText>>,
    nodes: Query<(&ResourceNode, &ItemStack)>,
    buildings: Query<(&Building, Option<&Processing>, Option<&ItemStack>), (With<Building>,)>,
    parent_query: Query<&ChildOf>,
) {
    target_text.0 = String::from("");

    let closest_hit = get_closest_hit(&camera_rayhits, vec![player.entity()]);
    if closest_hit.is_none() {
        return; // no target
    }
    let hit = closest_hit.unwrap();

    // traverse parents, since colliders triggering hit may be on children
    let mut entity = hit.entity;
    while let Some(parent) = parent_query.get(entity).ok() {
        entity = parent.0;
    }

    if let Ok((node, stack)) = nodes.get(entity) {
        // resource node
        target_text.0 = String::from(format!("{:?} ({:?}, {})", node, &stack.item, &stack.count));
    } else if let Ok((building, status, stack)) = buildings.get(entity) {
        // building
        if let Some(status) = status
            && let Some(stack) = stack
        {
            target_text.0 = String::from(format!(
                "{} ({:?}, {}), {:0>2.0}%",
                &building.name(),
                &stack.item,
                &stack.count,
                status.progress * 100.0
            ));
        } else {
            target_text.0 = String::from(format!("{}", &building.name(),));
        }
    }
}

pub fn update_hover_action(
    camera_rayhits: Single<&RayHits, With<Camera>>,
    player: Single<Entity, With<Player>>,
    player_status: Single<&PlayerProcessing, With<Player>>,
    mut action_text: Single<&mut Text, (With<ActionText>, Without<TargetText>)>,
    nodes: Query<&ResourceNode>,
    buildings: Query<&Building>,
    parent_query: Query<&ChildOf>,
    held_building: Res<HeldBuilding>,
) {
    if held_building.0.is_some() {
        return; // only update action text if player is not holding a building, otherwise it should show building placement instructions
    }

    action_text.0 = String::from("");

    let closest_hit = get_closest_hit(&camera_rayhits, vec![player.entity()]);
    if closest_hit.is_none() {
        return;
    }
    let hit = closest_hit.unwrap();

    // traverse parents, since colliders triggering hit may be on children
    let mut entity = hit.entity;
    while let Some(parent) = parent_query.get(entity).ok() {
        entity = parent.0;
    }

    if let Ok(_node) = nodes.get(entity) {
        // resource node
        if player_status.progress == 0.0 {
            action_text.0 = String::from(format!("[E] Mine"));
        } else {
            action_text.0 = String::from(format!(
                "[E] Mine, {:0>2.0}%",
                player_status.progress * 100.0
            ));
        }
    } else if let Ok(_building) = buildings.get(entity) {
        // building
        action_text.0 = String::from("[E] Open");
    }
}

pub fn update_interact(
    camera_rayhits: Single<&RayHits, With<Camera>>,
    player: Single<Entity, With<Player>>,
    mut player_status: Single<&mut PlayerProcessing, With<Player>>,
    mut inventory: Single<&mut Inventory, With<Player>>,
    mut nodes: Query<(&ResourceNode, &mut ItemStack)>,
    mut buildings: Query<&Building>,
    parent_query: Query<&ChildOf>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    held_building: Res<HeldBuilding>,
    time: Res<Time>,
) {
    if held_building.0.is_some() {
        player_status.progress = 0.0;
        return; // if player is holding a building, don't interact with world
    }

    let closest_hit = get_closest_hit(&camera_rayhits, vec![player.entity()]);
    if closest_hit.is_none() {
        player_status.progress = 0.0;
        return;
    }
    let hit = closest_hit.unwrap();

    // traverse parents, since colliders triggering hit may be on children
    let mut entity = hit.entity;
    while let Some(parent) = parent_query.get(entity).ok() {
        entity = parent.0;
    }

    // mine resource
    if let Ok((_node, mut stack)) = nodes.get_mut(entity)
        && keyboard_input.pressed(KeyCode::KeyE)
        && stack.count > 0
    {
        player_status.progress += time.delta_secs() * player_status.speed;
        if player_status.progress >= 1.0 {
            let amount = i32::min(stack.count, player_status.progress.floor() as i32);
            stack.count -= amount;
            inventory.add(&stack.item, amount);
            player_status.progress = player_status.progress.fract();
        }
    } else {
        player_status.progress = 0.0;
    }

    // open building menu
    if let Ok(building) = buildings.get_mut(entity) {
        if keyboard_input.just_pressed(KeyCode::KeyE) {
            match building {
                Building::Miner => {}
                _ => {}
            }
        }
    }
}

#[derive(Resource)]
/// The building the player is currently holding and about to place, if any
pub struct HeldBuilding(pub Option<Building>);

pub fn place_held_building(
    mut commands: Commands,
    camera_rayhits: Single<&RayHits, With<Camera>>,
    camera_transform: Single<&Transform, With<Camera>>,
    player: Single<Entity, With<Player>>,
    mut action_text: Single<&mut Text, With<ActionText>>,
    nodes: Query<(&ResourceNode, &Transform, &ItemStack)>,
    terrain: Query<&Terrain>,
    parent_query: Query<&ChildOf>,
    mut held_building: ResMut<HeldBuilding>,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    worldgen: Res<WorldGen>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    effect_map: Res<EffectMap>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        held_building.0 = None; // cancel building placement if Q is pressed
        return;
    }

    if held_building.0.is_none() {
        return; // not placing a building
    }
    let building = held_building.0.unwrap();

    action_text.0 = String::from(format!(
        "[E] Can't place {} here\n[Q] Cancel",
        building.name()
    ));

    let closest_hit = get_closest_hit(&camera_rayhits, vec![player.entity()]);
    if closest_hit.is_none() {
        return;
    }
    let hit = closest_hit.unwrap();

    // traverse parents, since colliders triggering hit may be on children
    let mut entity = hit.entity;
    while let Some(parent) = parent_query.get(entity).ok() {
        entity = parent.0;
    }

    match building {
        Building::SatelliteDish => {
            // only placeable on terrain
            if let Some(_terrain) = terrain.get(entity).ok() {
                action_text.0 = String::from(format!("[E] Place {}\n[Q] Cancel", building.name()));

                if keyboard_input.just_pressed(KeyCode::KeyE) {
                    let pos =
                        camera_transform.translation + camera_transform.forward() * hit.distance;
                    let normal = worldgen.get_normal(pos.x, pos.z);
                    let rot = Quat::from_axis_angle(
                        normal.cross(Vec3::Y),
                        -f32::acos(normal.dot(Vec3::Y)),
                    );

                    commands.spawn((
                        Building::SatelliteDish,
                        Processing {
                            status: ProcessingStatus::Idle,
                            speed: 1.0,
                            progress: 0.0,
                        },
                        SceneRoot(
                            asset_server.load::<Scene>(building.asset().to_owned() + "#Scene0"),
                        ),
                        Transform::from_translation(pos).with_rotation(rot),
                        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
                    ));
                    held_building.0 = None;
                }
            }
        }
        Building::Miner { .. } => {
            // only placeable on ore
            if let Some((node, transform, stack)) = nodes.get(entity).ok() {
                match node {
                    ResourceNode::Ore => {
                        action_text.0 =
                            String::from(format!("[E] Place {}\n[Q] Cancel", building.name()));
                        if keyboard_input.just_pressed(KeyCode::KeyE) {
                            let (graph, index) =
                                AnimationGraph::from_clip(asset_server.load::<AnimationClip>(
                                    building.asset().to_owned() + "#Animation0",
                                ));
                            let graph_handle = graphs.add(graph);

                            let smoke_handle = effect_map.0.get("smoke").unwrap().clone();

                            commands.spawn((
                                Building::Miner,
                                Processing {
                                    status: ProcessingStatus::Idle,
                                    speed: 0.5,
                                    progress: 0.0,
                                },
                                MiningNode(entity),
                                ItemStack {
                                    item: stack.item,
                                    count: 0,
                                },
                                SceneRoot(
                                    asset_server
                                        .load::<Scene>(building.asset().to_owned() + "#Scene0"),
                                ),
                                RunningAnimation(graph_handle, index),
                                ParticleEffect::new(smoke_handle),
                                transform.clone(),
                                ColliderConstructorHierarchy::new(
                                    ColliderConstructor::TrimeshFromMesh,
                                ),
                            ));
                            held_building.0 = None;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
