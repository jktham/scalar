use crate::world::ResourceNode;
use crate::world::Terrain;
use crate::worldgen::WorldGen;
use crate::{
    buildings::{Building, ProcessingStatus},
    hud::{ActionText, TargetText},
    inventory::{Inventory, ItemStack},
};
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
use bevy_tnua::{
    TnuaConfig, TnuaController, TnuaScheme,
    builtins::{TnuaBuiltinJump, TnuaBuiltinJumpConfig, TnuaBuiltinWalk, TnuaBuiltinWalkConfig},
};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerStatus {
    pub mining_speed: f32,
    pub mining_progress: f32,
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
        PlayerStatus {
            mining_speed: 0.25,
            mining_progress: 0.0,
        },
        Inventory::default(),
        Transform::from_translation(spawn_pos),
        RigidBody::Dynamic,
        Collider::capsule(0.3, 3.0),
        Friction::new(0.1),
        TnuaController::<ControlScheme>::default(),
        TnuaConfig::<ControlScheme>(control_scheme_configs.add(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                float_height: 2.0,
                cling_distance: 0.0,
                acceleration: 120.0,
                air_acceleration: 60.0,
                max_slope: f32::to_radians(80.0),
                ..Default::default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 2.0,
                ..Default::default()
            },
        })),
        TnuaAvian3dSensorShape(Collider::cylinder(0.25, 0.0)),
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
    buildings: Query<(&Building, Option<&ProcessingStatus>, Option<&ItemStack>), (With<Building>,)>,
) {
    target_text.0 = String::from("");

    let closest_hit = get_closest_hit(&camera_rayhits, vec![player.entity()]);
    if closest_hit.is_none() {
        return; // no target
    }
    let hit = closest_hit.unwrap();

    if let Ok((node, stack)) = nodes.get(hit.entity) {
        // resource node
        target_text.0 = String::from(format!("{:?} ({:?}, {})", node, &stack.item, &stack.count));
    } else if let Ok((building, status, stack)) = buildings.get(hit.entity) {
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
    player_status: Single<&PlayerStatus, With<Player>>,
    mut action_text: Single<&mut Text, (With<ActionText>, Without<TargetText>)>,
    nodes: Query<&ResourceNode>,
    buildings: Query<&Building>,
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

    if let Ok(_node) = nodes.get(hit.entity) {
        // resource node
        if player_status.mining_progress == 0.0 {
            action_text.0 = String::from(format!("[E] Mine"));
        } else {
            action_text.0 = String::from(format!(
                "[E] Mine, {:0>2.0}%",
                player_status.mining_progress * 100.0
            ));
        }
    } else if let Ok(_building) = buildings.get(hit.entity) {
        // building
        action_text.0 = String::from("[E] Open");
    }
}

pub fn update_interact(
    camera_rayhits: Single<&RayHits, With<Camera>>,
    player: Single<Entity, With<Player>>,
    mut player_status: Single<&mut PlayerStatus, With<Player>>,
    mut inventory: Single<&mut Inventory, With<Player>>,
    mut nodes: Query<(&ResourceNode, &mut ItemStack)>,
    mut buildings: Query<&Building>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    held_building: Res<HeldBuilding>,
    time: Res<Time>,
) {
    if held_building.0.is_some() {
        player_status.mining_progress = 0.0;
        return; // if player is holding a building, don't interact with world
    }

    let closest_hit = get_closest_hit(&camera_rayhits, vec![player.entity()]);
    if closest_hit.is_none() {
        player_status.mining_progress = 0.0;
        return;
    }
    let hit = closest_hit.unwrap();

    // mine resource
    if let Ok((_node, mut stack)) = nodes.get_mut(hit.entity)
        && keyboard_input.pressed(KeyCode::KeyE)
        && stack.count > 0
    {
        player_status.mining_progress += time.delta_secs() * player_status.mining_speed;
        if player_status.mining_progress >= 1.0 {
            let amount = i32::min(stack.count, player_status.mining_progress.floor() as i32);
            stack.count -= amount;
            inventory.add(&stack.item, amount);
            player_status.mining_progress = player_status.mining_progress.fract();
        }
    } else {
        player_status.mining_progress = 0.0;
    }

    // open building menu
    if let Ok(building) = buildings.get_mut(hit.entity) {
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

#[derive(Component)]
/// node the miner is attached to
pub struct AttachedNode(pub Entity);

pub fn place_held_building(
    mut commands: Commands,
    camera_rayhits: Single<&RayHits, With<Camera>>,
    camera_transform: Single<&Transform, With<Camera>>,
    player: Single<Entity, With<Player>>,
    mut action_text: Single<&mut Text, With<ActionText>>,
    nodes: Query<(&ResourceNode, &Transform, &ItemStack)>,
    terrain: Query<&Terrain>,
    mut held_building: ResMut<HeldBuilding>,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    worldgen: Res<WorldGen>,
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

    match building {
        Building::SatelliteDish => {
            // only placeable on terrain
            if let Some(_terrain) = terrain.get(hit.entity).ok() {
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
                        ProcessingStatus {
                            speed: 1.0,
                            progress: 0.0,
                        },
                        SceneRoot(
                            asset_server.load::<Scene>(building.asset().to_owned() + "#Scene0"),
                        ),
                        Transform::from_translation(pos).with_rotation(rot),
                        Collider::compound(vec![
                            (
                                Vec3::new(0.0, 3.5, 0.0),
                                Quat::default(),
                                Collider::sphere(1.0),
                            ),
                            (
                                Vec3::new(0.0, 0.0, 0.0),
                                Quat::default(),
                                Collider::cylinder(0.2, 5.0),
                            ),
                        ]),
                    ));
                    held_building.0 = None;
                }
            }
        }
        Building::Miner { .. } => {
            // only placeable on ore
            if let Some((node, transform, stack)) = nodes.get(hit.entity).ok() {
                match node {
                    ResourceNode::Ore => {
                        action_text.0 =
                            String::from(format!("[E] Place {}\n[Q] Cancel", building.name()));
                        if keyboard_input.just_pressed(KeyCode::KeyE) {
                            commands.spawn((
                                Building::Miner,
                                ProcessingStatus {
                                    speed: 1.0,
                                    progress: 0.0,
                                },
                                AttachedNode(hit.entity),
                                ItemStack {
                                    item: stack.item,
                                    count: 0,
                                },
                                SceneRoot(
                                    asset_server
                                        .load::<Scene>(building.asset().to_owned() + "#Scene0"),
                                ),
                                transform.clone(),
                                Collider::compound(vec![(
                                    Vec3::new(0.0, 2.6, 0.0),
                                    Quat::default(),
                                    Collider::cylinder(1.1, 2.6),
                                )]),
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
