use crate::world::Rock;
use crate::worldgen::WorldGen;
use crate::{
    buildings::{Building, BuildingProperties},
    hud::{ActionText, TargetText},
    inventory::{Inventory, ItemStack},
    world::{ResourceNode, Tree},
};
use avian3d::dynamics::rigid_body::Friction;
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
pub struct PlayerProperties {
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
        PlayerProperties {
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
fn get_closest_hit(rayhits: &RayHits, ignored: Vec<Entity>) -> Option<Entity> {
    let mut target: Option<Entity> = None;
    for hit in rayhits.iter_sorted() {
        if hit.distance > RANGE || ignored.contains(&hit.entity) {
            continue;
        }
        target = Some(hit.entity);
        break;
    }
    target
}

pub fn update_hover(
    camera_rayhits: Single<&RayHits, With<Camera>>,
    player: Single<Entity, With<Player>>,
    player_props: Single<&PlayerProperties, With<Player>>,
    nodes: Query<&ItemStack, (With<ResourceNode>, Without<Tree>)>,
    trees: Query<&ItemStack, (With<Tree>, Without<ResourceNode>)>,
    rocks: Query<&ItemStack, (With<Rock>, Without<Tree>, Without<ResourceNode>)>,
    buildings: Query<
        (&Building, &BuildingProperties, &ItemStack),
        (With<Building>, Without<ResourceNode>, Without<Tree>),
    >,
    mut target_text: Single<&mut Text, With<TargetText>>,
    mut action_text: Single<&mut Text, (With<ActionText>, Without<TargetText>)>,
    held_building: Res<HeldBuilding>,
) {
    target_text.0 = String::from("");
    if held_building.0.is_none() {
        action_text.0 = String::from(""); // only update action text if player is not holding a building, otherwise it should show building placement instructions
    }

    let target = get_closest_hit(&camera_rayhits, vec![player.entity()]);
    if let Some(entity) = target {
        if let Ok(stack) = nodes.get(entity) {
            target_text.0 = String::from(format!(
                "Resource node ({:?}, {})",
                &stack.item, &stack.count
            ));
            if held_building.0.is_none() {
                if player_props.mining_progress == 0.0 {
                    action_text.0 = String::from(format!("[E] Mine"));
                } else {
                    action_text.0 = String::from(format!(
                        "[E] Mine, {:0>2.0}%",
                        player_props.mining_progress * 100.0
                    ));
                }
            }
        } else if let Ok(stack) = trees.get(entity) {
            target_text.0 = String::from(format!("Tree ({:?}, {})", &stack.item, &stack.count));
            if held_building.0.is_none() {
                if player_props.mining_progress == 0.0 {
                    action_text.0 = String::from(format!("[E] Mine"));
                } else {
                    action_text.0 = String::from(format!(
                        "[E] Mine, {:0>2.0}%",
                        player_props.mining_progress * 100.0
                    ));
                }
            }
        } else if let Ok(stack) = rocks.get(entity) {
            target_text.0 = String::from(format!("Rock ({:?}, {})", &stack.item, &stack.count));
            if held_building.0.is_none() {
                if player_props.mining_progress == 0.0 {
                    action_text.0 = String::from(format!("[E] Mine"));
                } else {
                    action_text.0 = String::from(format!(
                        "[E] Mine, {:0>2.0}%",
                        player_props.mining_progress * 100.0
                    ));
                }
            }
        } else if let Ok((building, props, stack)) = buildings.get(entity) {
            target_text.0 = String::from(format!(
                "{:?} ({:?}, {}), {:0>2.0}%",
                &building,
                &stack.item,
                &stack.count,
                props.progress * 100.0
            ));
            if held_building.0.is_none() {
                action_text.0 = String::from("[E] Collect");
            }
        }
    }
}

pub fn update_interact(
    camera_rayhits: Single<&RayHits, With<Camera>>,
    player: Single<Entity, With<Player>>,
    mut player_props: Single<&mut PlayerProperties, With<Player>>,
    mut nodes: Query<&mut ItemStack, (With<ResourceNode>, Without<Tree>)>,
    mut trees: Query<&mut ItemStack, (With<Tree>, Without<ResourceNode>)>,
    mut rocks: Query<&mut ItemStack, (With<Rock>, Without<Tree>, Without<ResourceNode>)>,
    mut buildings: Query<
        (&Building, &mut ItemStack),
        (
            With<Building>,
            Without<ResourceNode>,
            Without<Tree>,
            Without<Rock>,
        ),
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut inventory: Single<&mut Inventory, With<Player>>,
    held_building: Res<HeldBuilding>,
    time: Res<Time>,
) {
    if held_building.0.is_some() {
        return; // if player is holding a building, don't interact with world
    }

    let target = get_closest_hit(&camera_rayhits, vec![player.entity()]);

    // mine resource
    if let Some(entity) = target
        && let Ok(mut stack) = nodes
            .get_mut(entity)
            .or(trees.get_mut(entity))
            .or(rocks.get_mut(entity))
        && keyboard_input.pressed(KeyCode::KeyE)
        && stack.count > 0
    {
        player_props.mining_progress += time.delta_secs() * player_props.mining_speed;
        if player_props.mining_progress >= 1.0 {
            let amount = i32::min(stack.count, player_props.mining_progress.floor() as i32);
            stack.count -= amount;
            inventory.add(&stack.item, amount);
            player_props.mining_progress = player_props.mining_progress.fract();
        }
    } else {
        player_props.mining_progress = 0.0;
    }

    // interact with building
    if let Some(entity) = target
        && let Ok((building, mut stack)) = buildings.get_mut(entity)
    {
        if keyboard_input.just_pressed(KeyCode::KeyE) && stack.count > 0 {
            match building {
                Building::Miner { .. } => {
                    inventory.add(&stack.item, stack.count);
                    stack.count = 0;
                }
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
    mut held_building: ResMut<HeldBuilding>,
    camera_rayhits: Single<&RayHits, With<Camera>>,
    mut nodes: Query<(&Transform, &ItemStack), (With<ResourceNode>, Without<Tree>)>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut action_text: Single<&mut Text, With<ActionText>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        held_building.0 = None; // cancel building placement if Q is pressed
        return;
    }

    if let Some(building) = held_building.0 {
        action_text.0 = String::from(format!("[E] Can't place {:?} here\n[Q] Cancel", building));

        let target: Option<Entity> = get_closest_hit(&camera_rayhits, vec![player.entity()]);

        match building {
            Building::Miner { .. } => {
                if let Some(entity) = target {
                    if let Some((transform, stack)) = nodes.get_mut(entity).ok() {
                        action_text.0 =
                            String::from(format!("[E] Place {:?}\n[Q] Cancel", building));
                        if keyboard_input.just_pressed(KeyCode::KeyE) {
                            commands.spawn((
                                Building::Miner,
                                BuildingProperties {
                                    speed: 1.0,
                                    progress: 0.0,
                                },
                                AttachedNode(entity),
                                ItemStack {
                                    item: stack.item,
                                    count: 0,
                                },
                                SceneRoot(asset_server.load::<Scene>(
                                    format!("{:?}.glb", building).to_lowercase() + "#Scene0",
                                )),
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
                }
            }
            _ => {}
        }
    }
}
