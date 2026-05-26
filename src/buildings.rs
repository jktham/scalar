use bevy::prelude::*;
use bevy_hanabi::EffectSpawner;
use strum_macros::EnumIter;

use crate::{inventory::ItemStack, world::ResourceNode};

#[derive(Component, Copy, Clone, EnumIter, Debug)]
pub enum Building {
    SatelliteDish,
    Miner,
}

impl Building {
    pub fn name(&self) -> &str {
        match self {
            Building::SatelliteDish => "Satellite Dish",
            Building::Miner => "Miner",
        }
    }

    pub fn asset(&self) -> &str {
        match self {
            Building::SatelliteDish => "satellite_dish.glb",
            Building::Miner => "miner.glb",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Building::SatelliteDish => "Sends images into the stars :)",
            Building::Miner => "Can be placed on a resource node to automatically mine it",
        }
    }
}

#[derive(Component)]
/// node the miner is attached to
pub struct MiningNode(pub Entity);

#[derive(Component)]
/// animation to play when building runs
pub struct RunningAnimation(pub Handle<AnimationGraph>, pub AnimationNodeIndex);

pub enum ProcessingStatus {
    Idle,
    Running,
}

#[derive(Component)]
pub struct Processing {
    /// status
    pub status: ProcessingStatus,
    /// operations per second
    pub speed: f32,
    /// progress of current operation, \[0, 1\]
    pub progress: f32,
}

pub fn update_buildings(
    mut buildings: Query<(
        &Building,
        Option<&mut Processing>,
        Option<&mut ItemStack>,
        Option<&MiningNode>,
    )>,
    mut nodes: Query<(&ResourceNode, &mut ItemStack), Without<Building>>,
    time: Res<Time>,
) {
    for (building, processing, building_stack, attached_node) in buildings.iter_mut() {
        match building {
            Building::Miner => {
                if let Some(mut processing) = processing
                    && let Some(mut building_stack) = building_stack
                    && let Some(attached_node) = attached_node
                {
                    if let Some((_node, mut node_stack)) = nodes.get_mut(attached_node.0).ok()
                        && node_stack.count > 0
                    {
                        processing.status = ProcessingStatus::Running;
                        processing.progress += time.delta_secs() * processing.speed;
                        if processing.progress >= 1.0 {
                            let amount =
                                i32::min(node_stack.count, processing.progress.floor() as i32);
                            building_stack.count += amount;
                            node_stack.count -= amount;
                            processing.progress = processing.progress.fract();
                        }
                    } else {
                        // node empty
                        processing.status = ProcessingStatus::Idle;
                        processing.progress = 0.0;
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn update_building_animations(
    mut commands: Commands,
    mut buildings: Query<(
        Entity,
        &Building,
        Option<&Processing>,
        Option<&RunningAnimation>,
    )>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (entity, building, processing, running_animation) in buildings.iter_mut() {
        match building {
            Building::Miner => {
                if let Some(processing) = processing
                    && let Some(running_animation) = running_animation
                {
                    match processing.status {
                        ProcessingStatus::Running => {
                            for child in children.iter_descendants(entity) {
                                if let Ok(mut player) = players.get_mut(child) {
                                    player.play(running_animation.1).repeat();

                                    commands.entity(child).insert_if_new(AnimationGraphHandle(
                                        running_animation.0.clone(),
                                    ));
                                }
                            }
                        }
                        ProcessingStatus::Idle => {
                            for child in children.iter_descendants(entity) {
                                if let Ok(mut player) = players.get_mut(child) {
                                    player.play(running_animation.1).pause();
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn update_building_effects(
    mut buildings: Query<(&Building, Option<&Processing>, Option<&mut EffectSpawner>)>,
) {
    for (building, processing, effect_spawner) in buildings.iter_mut() {
        match building {
            Building::Miner => {
                if let Some(processing) = processing
                    && let Some(mut effect_spawner) = effect_spawner
                {
                    match processing.status {
                        ProcessingStatus::Running => {
                            effect_spawner.active = true;
                        }
                        ProcessingStatus::Idle => {
                            effect_spawner.active = false;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
