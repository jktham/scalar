use bevy::prelude::*;
use bevy_hanabi::EffectSpawner;
use strum_macros::EnumIter;

use crate::{inventory::ItemStack, world::ResourceNode};

#[derive(Component, Copy, Clone, EnumIter, Debug)]
pub enum Building {
    Miner,
    SatelliteDish,
}

impl Building {
    pub fn name(&self) -> &str {
        match self {
            Building::Miner => "Miner",
            Building::SatelliteDish => "Satellite Dish",
        }
    }

    pub fn asset(&self) -> &str {
        match self {
            Building::Miner => "miner.glb",
            Building::SatelliteDish => "satellite_dish.glb",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Building::Miner => "Can be placed on a resource node to automatically mine it",
            Building::SatelliteDish => "Sends images into the stars :)",
        }
    }
}

#[derive(Component)]
/// node the miner is attached to
pub struct MiningNode(pub Entity);

#[derive(Debug)]
pub enum ProcessingStatus {
    Idle,
    Running,
    OutOfEnergy,
}

#[derive(Component)]
pub struct Processing {
    /// status
    pub status: ProcessingStatus,
    /// operations per second
    pub speed: f32,
    /// progress of current operation, \[0, 1\]
    pub progress: f32,
    /// energy cost per second, W
    pub cost: f32,
    /// energy buffer, J
    pub energy: f32,
}

#[derive(Component)]
pub struct OutputSlot(pub ItemStack);

#[derive(Component)]
pub struct FuelSlot(pub ItemStack);

pub fn update_buildings(
    mut buildings: Query<(
        &Building,
        Option<&mut Processing>,
        Option<&mut OutputSlot>,
        Option<&mut FuelSlot>,
        Option<&MiningNode>,
    )>,
    mut nodes: Query<(&ResourceNode, &mut ItemStack), Without<Building>>,
    time: Res<Time>,
) {
    for (building, processing, output_slot, fuel_slot, attached_node) in buildings.iter_mut() {
        match building {
            Building::Miner => {
                if let Some(mut processing) = processing
                    && let Some(mut output_slot) = output_slot
                    && let Some(mut fuel_slot) = fuel_slot
                    && let Some(attached_node) = attached_node
                {
                    if let Some((_node, mut node_stack)) = nodes.get_mut(attached_node.0).ok()
                        && node_stack.count > 0
                    {
                        let delta_cost = processing.cost * time.delta_secs();

                        // burn fuel
                        if processing.energy < delta_cost && fuel_slot.0.count > 0 {
                            fuel_slot.0.count -= 1;
                            processing.energy += 1000.0;
                        }

                        // self fueling
                        if processing.energy < delta_cost
                            && fuel_slot.0.item == output_slot.0.item
                            && output_slot.0.count > 0
                        {
                            output_slot.0.count -= 1;
                            processing.energy += 1000.0;
                        }

                        if processing.energy >= delta_cost {
                            processing.status = ProcessingStatus::Running;
                            processing.progress += time.delta_secs() * processing.speed;
                            processing.energy -= delta_cost;

                            if processing.progress >= 1.0 {
                                let amount =
                                    i32::min(node_stack.count, processing.progress.floor() as i32);
                                output_slot.0.count += amount;
                                node_stack.count -= amount;
                                processing.progress = processing.progress.fract();
                            }
                        } else {
                            // energy empty, keep progress
                            processing.status = ProcessingStatus::OutOfEnergy;
                        }
                    } else {
                        // node empty, reset to clean
                        processing.status = ProcessingStatus::Idle;
                        processing.progress = 0.0;
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Component)]
/// animation to play when building runs
pub struct RunningAnimation(pub Handle<AnimationGraph>, pub AnimationNodeIndex);

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

                                    commands
                                        .entity(child)
                                        .try_insert_if_new(AnimationGraphHandle(
                                            running_animation.0.clone(),
                                        ));
                                }
                            }
                        }
                        _ => {
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

#[derive(Component)]
/// marker on child with particle effect spawner to activate when building runs
pub struct RunningParticles;

pub fn update_building_effects(
    mut buildings: Query<(Entity, &Building, Option<&Processing>)>,
    children: Query<&Children>,
    mut effect_spawners: Query<&mut EffectSpawner, With<RunningParticles>>,
) {
    for (entity, building, processing) in buildings.iter_mut() {
        match building {
            Building::Miner => {
                for child in children.iter_descendants(entity) {
                    if let Some(processing) = processing
                        && let Ok(mut spawner) = effect_spawners.get_mut(child)
                    {
                        match processing.status {
                            ProcessingStatus::Running => {
                                spawner.active = true;
                            }
                            _ => {
                                spawner.active = false;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
