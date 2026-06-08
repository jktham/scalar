use bevy::prelude::*;
use fxhash::FxHashSet;
use strum_macros::EnumIter;

use crate::inventory::{Item, ItemStack};

#[derive(Component, Default)]
pub struct Unlocks(pub FxHashSet<Unlock>);

#[derive(EnumIter, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Unlock {
    PlayerMineSpeed,
}

impl Unlock {
    pub fn name(&self) -> &str {
        match self {
            Unlock::PlayerMineSpeed => "Pickaxe",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Unlock::PlayerMineSpeed => "Increases player mining speed.",
        }
    }

    /// (money, items)
    pub fn cost(&self) -> (i32, Vec<ItemStack>) {
        match self {
            Unlock::PlayerMineSpeed => (
                100,
                vec![
                    ItemStack {
                        item: Item::Stone,
                        count: 5,
                    },
                    ItemStack {
                        item: Item::Wood,
                        count: 10,
                    },
                ],
            ),
        }
    }
}
