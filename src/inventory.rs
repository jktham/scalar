use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Item {
    Iron,
    Copper,
    Wood,
}

impl Item {
    pub fn color(&self) -> Color {
        match self {
            Item::Iron => Color::srgb(0.0, 0.0, 1.0),
            Item::Copper => Color::srgb(1.0, 0.0, 0.0),
            Item::Wood => Color::srgb(0.5, 0.25, 0.0),
        }
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ItemStack {
    pub item: Item,
    pub count: i32,
}

#[derive(Resource, Default, Debug)]
pub struct Inventory {
    pub stacks: Vec<ItemStack>,
}

impl Inventory {
    pub fn get(&self, item: &Item) -> i32 {
        let stack = self.stacks.iter().find(|stack| stack.item == *item);
        if let Some(stack) = stack {
            stack.count
        } else {
            0
        }
    }

    pub fn set(&mut self, item: &Item, count: i32) {
        let stack = self.stacks.iter_mut().find(|stack| stack.item == *item);
        if let Some(stack) = stack {
            stack.count = count;
        } else {
            self.stacks.push(ItemStack {
                item: item.clone(),
                count,
            });
        }
    }

    pub fn add(&mut self, item: &Item, count: i32) {
        let current = self.get(item);
        self.set(item, current + count);
    }
}
