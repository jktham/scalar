use bevy::prelude::*;
use strum_macros::EnumIter;

use crate::{inventory::ItemStack, player::AttachedNode, world::ResourceNode};

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
pub struct ProcessingStatus {
    /// operations per second
    pub speed: f32,
    /// progress of current operation, \[0, 1\]
    pub progress: f32,
}

pub fn update_buildings(
    mut buildings: Query<(
        &Building,
        Option<&mut ProcessingStatus>,
        Option<&mut ItemStack>,
        Option<&AttachedNode>,
    )>,
    mut nodes: Query<(&ResourceNode, &mut ItemStack), Without<Building>>,
    time: Res<Time>,
) {
    for (building, status, building_stack, attached_node) in buildings.iter_mut() {
        match building {
            Building::Miner => {
                if let Some(mut status) = status
                    && let Some(mut building_stack) = building_stack
                    && let Some(attached_node) = attached_node
                    && let Some((_node, mut node_stack)) = nodes.get_mut(attached_node.0).ok()
                {
                    if node_stack.count > 0 {
                        status.progress += time.delta_secs() * status.speed;
                        if status.progress >= 1.0 {
                            let amount = i32::min(node_stack.count, status.progress.floor() as i32);
                            building_stack.count += amount;
                            node_stack.count -= amount;
                            status.progress = status.progress.fract();
                        }
                    } else {
                        // node empty
                        status.progress = 0.0;
                    }
                }
            }
            _ => {}
        }
    }
}
