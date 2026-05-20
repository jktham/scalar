use crate::{
    hud::TargetText,
    inventory::{Inventory, ItemStack},
    world::{Node, Stump, Tree, get_terrain_height},
};
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

#[derive(Component, Default)]
pub struct Player;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum ControlScheme {
    Jump(TnuaBuiltinJump),
}

pub fn setup_player(
    mut commands: Commands,
    mut control_scheme_configs: ResMut<Assets<ControlSchemeConfig>>,
) {
    let mut spawn_pos = Vec3::new(0.0, 0.0, 0.0);
    spawn_pos.y = get_terrain_height(spawn_pos.x, spawn_pos.z) + 2.0;

    commands.spawn((
        Player::default(),
        Inventory::default(),
        Transform::from_translation(spawn_pos),
        RigidBody::Dynamic,
        Collider::capsule(0.5, 0.5),
        TnuaController::<ControlScheme>::default(),
        TnuaConfig::<ControlScheme>(control_scheme_configs.add(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                float_height: 1.0,
                ..Default::default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 2.0,
                ..Default::default()
            },
        })),
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
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
        desired_forward: Some(Dir3::from_xyz_unchecked(front.x, 0.0, front.z)),
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

    camera_transform.translation = player_transform.translation + Vec3::new(0.0, 1.2, 0.0);
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
    nodes: Query<&ItemStack, (With<Node>, Without<Tree>)>,
    trees: Query<&ItemStack, (With<Tree>, Without<Node>)>,
    mut target_text: Single<&mut Text, With<TargetText>>,
) {
    let target = get_closest_hit(&camera_rayhits, vec![player.entity()]);

    target_text.0 = String::from("");
    if let Some(entity) = target {
        if let Ok(stack) = nodes.get(entity) {
            target_text.0 = String::from(format!("{:?} node ({})", &stack.item, &stack.count));
        } else if let Ok(stack) = trees.get(entity) {
            target_text.0 = String::from(format!("Tree ({})", &stack.count));
        }
    }
}

pub fn update_interact(
    camera_rayhits: Single<&RayHits, With<Camera>>,
    player: Single<Entity, With<Player>>,
    mut nodes: Query<&mut ItemStack, (With<Node>, Without<Tree>)>,
    mut trees: Query<(&Transform, &mut ItemStack), (With<Tree>, Without<Node>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut inventory: Single<&mut Inventory, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let target = get_closest_hit(&camera_rayhits, vec![player.entity()]);

    // mine resource if left mouse button is pressed
    if let Some(entity) = target {
        if let Ok(mut stack) = nodes.get_mut(entity) {
            if keyboard_input.just_pressed(KeyCode::KeyE) && stack.count > 0 {
                stack.count -= 1;
                inventory.add(&stack.item, 1);
            }
        } else if let Ok((transform, mut stack)) = trees.get_mut(entity) {
            if keyboard_input.just_pressed(KeyCode::KeyE) && stack.count > 0 {
                stack.count -= 1;
                inventory.add(&stack.item, 1);

                // if tree is depleted, despawn it and spawn a stump
                if stack.count <= 0 {
                    commands.entity(entity).despawn();
                    commands.spawn((
                        Stump,
                        SceneRoot(asset_server.load::<Scene>("stump.glb#Scene0")),
                        *transform,
                        Collider::cylinder(0.3, 2.0),
                    ));
                }
            }
        }
    }
}
