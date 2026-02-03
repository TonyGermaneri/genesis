//! Inventory system.

use genesis_common::ItemTypeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Inventory error types.
#[derive(Debug, Error)]
pub enum InventoryError {
    /// Not enough items
    #[error("Not enough items: need {needed}, have {have}")]
    NotEnough {
        /// Amount needed
        needed: u32,
        /// Amount available
        have: u32,
    },
    /// Inventory full
    #[error("Inventory full: capacity {capacity}")]
    Full {
        /// Inventory capacity
        capacity: u32,
    },
    /// Item not found
    #[error("Item not found")]
    NotFound,
}

/// Result type for inventory operations.
pub type InventoryResult<T> = Result<T, InventoryError>;

/// An inventory container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// Items and their quantities
    items: HashMap<ItemTypeId, u32>,
    /// Maximum unique item types
    capacity: u32,
}

impl Inventory {
    /// Creates a new inventory with the given capacity.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self {
            items: HashMap::new(),
            capacity,
        }
    }

    /// Returns the number of unique item types.
    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.items.len() as u32
    }

    /// Returns the capacity.
    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Returns the count of a specific item.
    #[must_use]
    pub fn count(&self, item: ItemTypeId) -> u32 {
        self.items.get(&item).copied().unwrap_or(0)
    }

    /// Checks if the inventory contains at least the given amount.
    #[must_use]
    pub fn has(&self, item: ItemTypeId, amount: u32) -> bool {
        self.count(item) >= amount
    }

    /// Adds items to the inventory.
    pub fn add(&mut self, item: ItemTypeId, amount: u32) -> InventoryResult<()> {
        let current = self.items.get(&item).copied().unwrap_or(0);
        if current == 0 && self.slot_count() >= self.capacity {
            return Err(InventoryError::Full {
                capacity: self.capacity,
            });
        }
        self.items.insert(item, current + amount);
        Ok(())
    }

    /// Removes items from the inventory.
    pub fn remove(&mut self, item: ItemTypeId, amount: u32) -> InventoryResult<()> {
        let current = self.count(item);
        if current < amount {
            return Err(InventoryError::NotEnough {
                needed: amount,
                have: current,
            });
        }
        if current == amount {
            self.items.remove(&item);
        } else {
            self.items.insert(item, current - amount);
        }
        Ok(())
    }

    /// Returns an iterator over all items.
    pub fn iter(&self) -> impl Iterator<Item = (ItemTypeId, u32)> + '_ {
        self.items.iter().map(|(&id, &count)| (id, count))
    }
}
