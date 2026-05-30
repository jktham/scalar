use avian3d::spatial_query::{RayCaster, RayHits};
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    light::{CascadeShadowConfigBuilder, NotShadowCaster},
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
};

pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sky_color = Color::srgb(0.35, 0.48, 0.66);
    let sun_color = Color::srgb(0.98, 0.95, 0.82);
    let fog_color = Color::srgba(0.35, 0.48, 0.66, 1.0);

    // skybox
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: sky_color,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_scale(Vec3::splat(1000.0)),
        NotShadowCaster,
    ));

    // sun
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            color: sun_color,
            ..default()
        },
        Transform::from_translation(Vec3::new(3.0, 4.0, 2.0) * 1000.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            maximum_distance: 500.0,
            ..default()
        }
        .build(),
    ));

    // camera (here for fog config :p)
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 80.0_f32.to_radians(),
            ..default()
        }),
        // NoIndirectDrawing,
        ScreenSpaceAmbientOcclusion::default(),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::new(1.0, 0.0, 0.0), Vec3::Y),
        RayCaster::new(Vec3::ZERO, -Dir3::Z).with_max_distance(10.0),
        RayHits::default(),
        DistanceFog {
            color: fog_color,
            directional_light_color: sun_color,
            directional_light_exponent: 200.0,
            falloff: FogFalloff::from_visibility_squared(500.0),
        },
    ));
}
