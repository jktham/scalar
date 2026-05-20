use bevy::prelude::*;
use strum_macros::EnumIter;

#[derive(Component, Copy, Clone, EnumIter, Debug)]
pub enum Building {
    Miner,
    Pumpjack,
    Smelter,
    Generator,
}
