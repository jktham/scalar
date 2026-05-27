use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Item {
    Iron,
    Copper,
    Wood,
    Stone,
    Coal,
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ItemStack {
    pub item: Item,
    pub count: i32,
}

#[derive(Component, Default, Debug)]
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
            self.stacks.push(ItemStack { item: *item, count });
        }
        self.stacks.retain(|s| s.count > 0);
    }

    pub fn add(&mut self, item: &Item, count: i32) {
        if count == 0 {
            return;
        }
        let current = self.get(item);
        self.set(item, current + count);
    }

    pub fn has(&mut self, item: &Item, count: i32) -> bool {
        let current = self.get(item);
        current >= count
    }

    pub fn remove(&mut self, item: &Item, count: i32) {
        if !self.has(item, count) {
            warn!("invalid inventory remove!");
            return;
        }
        let current = self.get(item);
        self.set(item, current - count);
    }
}
