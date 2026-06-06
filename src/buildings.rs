use core::fmt;

use bevy::prelude::*;
use bevy_hanabi::EffectSpawner;
use fastrand::Rng;
use strum_macros::EnumIter;

use crate::{
    inventory::{Item, ItemStack},
    player::{Money, Player},
    world::ResourceNode,
};

#[derive(Component, Copy, Clone, EnumIter, Debug)]
/// dynamic enum for building impl
pub enum Building {
    Miner,
    Processor,
    SatelliteDish,
}

#[derive(Component)]
/// static marker
pub struct MinerStatic;

#[derive(Component)]
/// static marker
pub struct ProcessorStatic;

#[derive(Component)]
/// static marker
pub struct SatelliteDishStatic;

impl Building {
    pub fn name(&self) -> &str {
        match self {
            Building::Miner => "Miner",
            Building::Processor => "Processor",
            Building::SatelliteDish => "Satellite Dish",
        }
    }

    pub fn asset(&self) -> &str {
        match self {
            Building::Miner => "models/miner.glb",
            Building::Processor => "models/processor.glb",
            Building::SatelliteDish => "models/satellite_dish.glb",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Building::Miner => {
                "Mines resources from an ore vein. Self fueling when placed on coal."
            }
            Building::Processor => "Generates image data. Powered by clean coal.",
            Building::SatelliteDish => "Sends images into the stars :). You get money in exchange.",
        }
    }

    pub fn cost(&self) -> Vec<ItemStack> {
        match self {
            Building::Miner => vec![
                ItemStack {
                    item: Item::Stone,
                    count: 1,
                },
                ItemStack {
                    item: Item::Wood,
                    count: 5,
                },
                ItemStack {
                    item: Item::Iron,
                    count: 10,
                },
            ],
            Building::Processor => vec![
                ItemStack {
                    item: Item::Iron,
                    count: 10,
                },
                ItemStack {
                    item: Item::Copper,
                    count: 10,
                },
            ],
            Building::SatelliteDish => vec![
                ItemStack {
                    item: Item::Wood,
                    count: 10,
                },
                ItemStack {
                    item: Item::Iron,
                    count: 20,
                },
                ItemStack {
                    item: Item::Copper,
                    count: 20,
                },
            ],
        }
    }
}

#[derive(Component)]
/// node the miner is attached to
pub struct MiningNode(pub Entity);

#[derive(Debug, Default)]
pub enum ProcessingStatus {
    #[default]
    Idle,
    Running,
    OutOfEnergy,
    OutputFull,
}

impl fmt::Display for ProcessingStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingStatus::Idle => write!(f, "Idle"),
            ProcessingStatus::Running => write!(f, "Running"),
            ProcessingStatus::OutOfEnergy => write!(f, "Out of energy"),
            ProcessingStatus::OutputFull => write!(f, "Output full"),
        }
    }
}

#[derive(Component, Default)]
pub struct Processing {
    /// status
    pub status: ProcessingStatus,
    /// operations per second
    pub speed: f32,
    /// progress of current operation, \[0, 1\]
    pub progress: f32,
    /// energy cost per second, W
    pub consumption: f32,
    /// energy buffer, J
    pub energy: f32,
}

#[derive(Component)]
pub struct OutputSlot {
    pub stack: ItemStack,
    pub limit: i32,
}

#[derive(Component)]
pub struct FuelSlot {
    pub stack: ItemStack,
    pub limit: i32,
}

#[derive(Component)]
pub struct ImageData {
    pub count: i32,
    pub limit: i32,
}

/// update processing state and function of each type of building
pub fn update_buildings(
    mut miners: Query<
        (&mut Processing, &mut OutputSlot, &mut FuelSlot, &MiningNode),
        With<MinerStatic>,
    >,
    mut nodes: Query<(&ResourceNode, &mut ItemStack), Without<Building>>,
    mut processors: Query<
        (&mut Processing, &mut ImageData, &mut FuelSlot),
        (With<ProcessorStatic>, Without<MinerStatic>),
    >,
    mut satellite_dishes: Query<
        (&mut Processing, &mut FuelSlot),
        (
            With<SatelliteDishStatic>,
            Without<ProcessorStatic>,
            Without<MinerStatic>,
        ),
    >,
    mut money: Single<&mut Money, With<Player>>,
    time: Res<Time>,
) {
    // miner
    for (mut processing, mut output_slot, mut fuel_slot, mining_node) in miners.iter_mut() {
        if let Some((_node, mut node_stack)) = nodes.get_mut(mining_node.0).ok()
            && node_stack.count > 0
        {
            let mut dt: f32 = time.delta_secs();
            let mut delta_consumption = processing.consumption * dt;

            // subtick delta, when there is enough energy for a single operation but not enough for an entire ticks consumption rate
            if processing.energy < delta_consumption && processing.energy > 0.0 {
                dt = dt * (processing.energy / delta_consumption);
                delta_consumption = processing.consumption * dt;
            }

            // burn fuel
            if processing.energy < delta_consumption && fuel_slot.stack.count > 0 {
                fuel_slot.stack.count -= 1;
                processing.energy += 1000.0;
            }

            // self fueling
            if processing.energy < delta_consumption
                && fuel_slot.stack.item == output_slot.stack.item
                && output_slot.stack.count > 0
            {
                output_slot.stack.count -= 1;
                processing.energy += 1000.0;
            }

            if output_slot.stack.count < output_slot.limit {
                // process
                if processing.energy >= delta_consumption {
                    processing.status = ProcessingStatus::Running;
                    processing.progress += dt * processing.speed;
                    processing.energy -= delta_consumption;

                    if processing.progress >= 1.0 {
                        let amount = i32::min(node_stack.count, processing.progress.floor() as i32)
                            .min(output_slot.limit - output_slot.stack.count);
                        output_slot.stack.count += amount;
                        node_stack.count -= amount;
                        processing.progress -= amount as f32; // progress can be above 1 if we hit output limit and speed is high enough
                    }
                } else {
                    // energy empty, keep progress
                    processing.status = ProcessingStatus::OutOfEnergy;
                }
            } else {
                // output full, keep progress
                processing.status = ProcessingStatus::OutputFull;
            }
        } else {
            // node empty, reset progress
            processing.status = ProcessingStatus::Idle;
            processing.progress = 0.0;
        }
    }

    // processor
    for (mut processing, mut image_data, mut fuel_slot) in processors.iter_mut() {
        let mut dt: f32 = time.delta_secs();
        let mut delta_consumption = processing.consumption * dt;

        // subtick delta, when there is enough energy for a single operation but not enough for an entire ticks consumption rate
        if processing.energy < delta_consumption && processing.energy > 0.0 {
            dt = dt * (processing.energy / delta_consumption);
            delta_consumption = processing.consumption * dt;
        }

        // burn fuel
        if processing.energy < delta_consumption && fuel_slot.stack.count > 0 {
            fuel_slot.stack.count -= 1;
            processing.energy += 1000.0;
        }

        if image_data.count < image_data.limit {
            // process
            if processing.energy >= delta_consumption {
                processing.status = ProcessingStatus::Running;
                processing.progress += dt * processing.speed;
                processing.energy -= delta_consumption;

                if processing.progress >= 1.0 {
                    let amount = i32::min(
                        processing.progress.floor() as i32,
                        image_data.limit - image_data.count,
                    );
                    image_data.count += amount;
                    processing.progress -= amount as f32;
                }
            } else {
                // energy empty, keep progress
                processing.status = ProcessingStatus::OutOfEnergy;
            }
        } else {
            // output full, keep progress
            processing.status = ProcessingStatus::OutputFull;
        }
    }

    // satellite dish
    for (mut processing, mut fuel_slot) in satellite_dishes.iter_mut() {
        let mut dt: f32 = time.delta_secs();
        let mut delta_consumption = processing.consumption * dt;

        let total_data = processors.iter().fold(0, |acc, p| acc + p.1.count);
        let mut data_sources = processors
            .iter_mut()
            .map(|p| p.1)
            .filter(|i| i.count > 0)
            .collect::<Vec<_>>();

        if total_data <= 0 || data_sources.len() == 0 {
            processing.status = ProcessingStatus::Idle;
            return;
        }

        // subtick delta, when there is enough energy for a single operation but not enough for an entire ticks consumption rate
        if processing.energy < delta_consumption && processing.energy > 0.0 {
            dt = dt * (processing.energy / delta_consumption);
            delta_consumption = processing.consumption * dt;
        }

        // burn fuel
        if processing.energy < delta_consumption && fuel_slot.stack.count > 0 {
            fuel_slot.stack.count -= 1;
            processing.energy += 1000.0;
        }

        // process
        if processing.energy >= delta_consumption {
            processing.status = ProcessingStatus::Running;
            processing.progress += dt * processing.speed;
            processing.energy -= delta_consumption;

            if processing.progress >= 1.0 {
                let amount = i32::min(total_data, processing.progress.floor() as i32);
                money.0 += amount;
                processing.progress -= amount as f32;

                // subtract data from available sources, picking randomly each time
                let mut rng = Rng::default();
                let mut remaining = amount;

                while remaining > 0 {
                    let random_index = rng.usize(0..data_sources.len());
                    if data_sources[random_index].count <= 0 {
                        continue;
                    }

                    data_sources[random_index].count -= 1;
                    remaining -= 1;
                }
            }
        } else {
            // energy empty, keep progress
            processing.status = ProcessingStatus::OutOfEnergy;
        }
    }
}

#[derive(Component)]
/// animation to play when building runs
pub struct RunningAnimation(pub Handle<AnimationGraph>, pub AnimationNodeIndex);

/// play animations on any building with Processing and RunningAnimation components
pub fn update_building_animations(
    mut commands: Commands,
    buildings: Query<(Entity, &Processing, &RunningAnimation)>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (entity, processing, running_animation) in buildings.iter() {
        for child in children.iter_descendants(entity) {
            if let Ok(mut player) = players.get_mut(child) {
                player.play(running_animation.1).repeat();

                commands
                    .entity(child)
                    .try_insert_if_new(AnimationGraphHandle(running_animation.0.clone()));

                match processing.status {
                    ProcessingStatus::Running => {
                        player.play(running_animation.1).resume();
                    }
                    _ => {
                        player.play(running_animation.1).pause();
                    }
                }
            }
        }
    }
}

#[derive(Component)]
/// marker on child with particle effect spawner to activate when building runs
pub struct RunningParticles;

/// spawn particle effects on any building with Processing and RunningParticles components
pub fn update_building_effects(
    buildings: Query<(Entity, &Processing)>,
    children: Query<&Children>,
    mut effect_spawners: Query<&mut EffectSpawner, With<RunningParticles>>,
) {
    for (entity, processing) in buildings.iter() {
        for child in children.iter_descendants(entity) {
            if let Ok(mut spawner) = effect_spawners.get_mut(child) {
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
}
