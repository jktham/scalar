use crate::{
    inventory::{Inventory, ItemStack},
    ui::{ActionText, setup_ui, update_ui},
    world::{ResourceNode, setup_world},
};
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    input::mouse::AccumulatedMouseMotion,
    math::bounding::{BoundingSphere, RayCast3d},
    prelude::*,
    render::view::NoIndirectDrawing,
    window::{CursorGrabMode, CursorOptions, PresentMode, WindowResolution},
};
use bevy_obj::ObjPlugin;

mod inventory;
mod ui;
mod world;

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            illuminance: 2000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        NoIndirectDrawing,
        Transform::from_xyz(0.0, 2.0, 0.0).looking_to(Vec3::new(1.0, 0.0, 0.0), Vec3::Y),
    ));
}

fn cursor_grab(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::Confined;
    cursor_options.visible = false;
}

fn cursor_ungrab(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
}

fn update_camera(
    mut camera_transform: Single<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
) {
    let left = Vec3::Y
        .cross(camera_transform.forward().as_vec3())
        .normalize();
    let front = left.cross(Vec3::Y).normalize(); // in plane

    const SENSITIVITY: f32 = 0.001;
    camera_transform.rotation = Quat::from_rotation_y(-mouse_motion.delta.x * SENSITIVITY)
        * Quat::from_axis_angle(left, mouse_motion.delta.y * SENSITIVITY)
        * camera_transform.rotation; // TODO: prevent flipping over pole

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

    const SPEED: f32 = 8.0;
    let mut speed = SPEED;
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        speed *= 2.0;
    }

    camera_transform.translation += movement * speed * time.delta().as_secs_f32();
}

fn mine_node(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut nodes: Query<(&Transform, &mut ItemStack), With<ResourceNode>>,
    mut inventory: ResMut<Inventory>,
    mut action_text: Single<&mut Text, With<ActionText>>,
) {
    let (_camera, camera_transform) = *camera_query;

    const RANGE: f32 = 3.0;
    let ray = RayCast3d::new(
        camera_transform.translation(),
        camera_transform.forward(),
        RANGE,
    );

    let mut min_dist = RANGE;
    let mut target = None;

    for (transform, stack) in nodes.iter_mut() {
        let bound = BoundingSphere::new(transform.translation, 0.5);

        if let Some(dist) = ray.sphere_intersection_at(&bound) {
            if dist < min_dist {
                min_dist = dist;
                target = Some(stack);
            }
        }
    }

    if let Some(mut stack) = target {
        action_text.0 = String::from(format!("{:?} node ({})", &stack.item, &stack.count));
        if mouse_input.just_pressed(MouseButton::Left) && stack.count > 0 {
            stack.count -= 1;
            inventory.add(&stack.item, 1);
        }
    } else {
        action_text.0 = String::from("");
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "scalar".into(),
                        resolution: WindowResolution::new(960, 540),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    text_color: Color::WHITE,
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: true,
                        min_fps: 30.0,
                        target_fps: 60.0,
                    },
                },
            },
            ObjPlugin,
        ))
        .insert_resource(Inventory::default())
        .add_systems(Startup, (setup, setup_world, setup_ui, cursor_grab))
        .add_systems(Update, (update_camera, mine_node, update_ui))
        .run();
}
