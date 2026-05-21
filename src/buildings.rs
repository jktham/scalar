use bevy::prelude::*;
use strum_macros::EnumIter;

use crate::{inventory::ItemStack, player::AttachedNode, world::ResourceNode};

#[derive(Component, Copy, Clone, EnumIter, Debug)]
pub enum Building {
    Miner,
    Pumpjack,
    Smelter,
    Generator,
}

#[derive(Component)]
pub struct BuildingProperties {
    /// operations per second
    pub speed: f32,
    /// progress of current operation, \[0, 1\]
    pub progress: f32,
}

pub fn update_buildings(
    mut buildings: Query<(
        &Building,
        &mut BuildingProperties,
        &mut ItemStack,
        &AttachedNode,
    )>,
    mut nodes: Query<&mut ItemStack, (With<ResourceNode>, Without<Building>)>,
    time: Res<Time>,
) {
    for (building, mut props, mut building_stack, attached_node) in buildings.iter_mut() {
        match building {
            Building::Miner => {
                if let Some(mut node_stack) = nodes.get_mut(attached_node.0).ok() {
                    if node_stack.count > 0 {
                        props.progress += time.delta_secs() * props.speed;
                        if props.progress >= 1.0 {
                            let amount = i32::min(node_stack.count, props.progress.floor() as i32);
                            building_stack.count += amount;
                            node_stack.count -= amount;
                            props.progress = props.progress.fract();
                        }
                    } else {
                        props.progress = 0.0;
                    }
                }
            }
            _ => {}
        }
    }
}
