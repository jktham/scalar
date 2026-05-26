use bevy::prelude::*;
use bevy_hanabi::{
    Attribute, ColorOverLifetimeModifier, EffectAsset, ExprWriter, Gradient, SetAttributeModifier,
    SetPositionSphereModifier, ShapeDimension, SpawnerSettings,
};
use fxhash::FxHashMap;

#[derive(Resource, Default)]
pub struct EffectMap(pub FxHashMap<String, Handle<EffectAsset>>);

pub fn create_smoke_effect(
    mut effects: ResMut<Assets<EffectAsset>>,
    mut effect_map: ResMut<EffectMap>,
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0., 0., 0., 1.));
    gradient.add_key(1.0, Vec4::splat(0.));

    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.8).expr(),
        dimension: ShapeDimension::Surface,
    };

    // Initialize a random rotation around Y by setting particle frame axes.
    // axis_x = (cos(a), 0, -sin(a))
    // axis_y = (0, 1, 0)
    // axis_z = (sin(a), 0, cos(a))
    let angle = writer.lit(0.0).uniform(writer.lit(std::f32::consts::TAU));
    let cx = angle.clone().cos();
    let sx = angle.clone().sin();
    let zero = writer.lit(0.0);
    let axis_x = cx
        .clone()
        .vec3(zero.clone(), writer.lit(-1.0).mul(sx.clone()));
    let axis_y = zero.clone().vec3(writer.lit(1.0), zero.clone());
    let axis_z = sx.clone().vec3(zero.clone(), cx.clone());
    let init_rot_x = SetAttributeModifier::new(Attribute::AXIS_X, axis_x.expr());
    let init_rot_y = SetAttributeModifier::new(Attribute::AXIS_Y, axis_y.expr());
    let init_rot_z = SetAttributeModifier::new(Attribute::AXIS_Z, axis_z.expr());

    let y = writer.lit(3.0).uniform(writer.lit(6.0));
    let v = zero.clone().vec3(y, zero);
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, v.expr());

    let s = writer.lit(0.8);
    let init_scale = SetAttributeModifier::new(Attribute::SIZE, s.expr());

    let lifetime = writer.lit(10.).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let name = "smoke";
    let effect = EffectAsset::new(1000, SpawnerSettings::rate(10.0.into()), writer.finish())
        .with_name(name)
        .init(init_pos)
        .init(init_rot_x)
        .init(init_rot_y)
        .init(init_rot_z)
        .init(init_vel)
        .init(init_scale)
        .init(init_lifetime)
        .render(ColorOverLifetimeModifier {
            gradient,
            ..default()
        });

    // Insert into the asset system
    let handle = effects.add(effect);
    effect_map.0.insert(name.to_string(), handle);
}
