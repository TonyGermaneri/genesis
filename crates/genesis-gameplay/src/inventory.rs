//! Inventory system with stacking and atomic transfers.

use genesis_common::ItemTypeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Default maximum stack size for items.
pub const DEFAULT_MAX_STACK: u32 = 999;

/// Inventory error types.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InventoryError {
    /// Not enough items
    #[error("Not enough items: need {needed}, have {have}")]
    NotEnough {
        /// Amount needed
        needed: u32,
        /// Amount available
        have: u32,
    },
    /// Inventory full (no free slots)
    #[error("Inventory full: capacity {capacity}")]
    Full {
        /// Inventory capacity
        capacity: u32,
    },
    /// Stack size exceeded
    #[error("Stack limit exceeded: max {max}, would be {would_be}")]
    StackOverflow {
        /// Maximum stack size
        max: u32,
        /// What the stack would become
        would_be: u32,
    },
    /// Item not found
    #[error("Item not found")]
    NotFound,
    /// Transfer would fail atomically
    #[error("Transfer failed: {reason}")]
    TransferFailed {
        /// Reason for failure
        reason: String,
    },
}

/// Result type for inventory operations.
pub type InventoryResult<T> = Result<T, InventoryError>;

/// A stack of items in an inventory slot.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ItemStack {
    /// The item type
    pub item_type: ItemTypeId,
    /// Quantity in this stack
    pub quantity: u32,
}

impl ItemStack {
    /// Creates a new item stack.
    #[must_use]
    pub const fn new(item_type: ItemTypeId, quantity: u32) -> Self {
        Self {
            item_type,
            quantity,
        }
    }
}

/// An inventory container with stacking support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// Items and their quantities
    items: HashMap<ItemTypeId, u32>,
    /// Maximum unique item types (slots)
    capacity: u32,
    /// Maximum stack size per item type
    max_stack: u32,
}

impl Inventory {
    /// Creates a new inventory with the given slot capacity.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self {
            items: HashMap::new(),
            capacity,
            max_stack: DEFAULT_MAX_STACK,
        }
    }

    /// Creates a new inventory with custom stack limit.
    #[must_use]
    pub fn with_stack_limit(capacity: u32, max_stack: u32) -> Self {
        Self {
            items: HashMap::new(),
            capacity,
            max_stack,
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
    ///
    /// Checks both slot capacity and stack limits.
    pub fn add(&mut self, item: ItemTypeId, amount: u32) -> InventoryResult<()> {
        let current = self.items.get(&item).copied().unwrap_or(0);

        // Check if this would exceed stack limit
        let new_total = current + amount;
        if new_total > self.max_stack {
            return Err(InventoryError::StackOverflow {
                max: self.max_stack,
                would_be: new_total,
            });
        }

        // Check slot capacity only when adding a new item type
        if current == 0 && self.slot_count() >= self.capacity {
            return Err(InventoryError::Full {
                capacity: self.capacity,
            });
        }

        self.items.insert(item, new_total);
        Ok(())
    }

    /// Checks if items can be added without errors.
    #[must_use]
    pub fn can_add(&self, item: ItemTypeId, amount: u32) -> bool {
        let current = self.items.get(&item).copied().unwrap_or(0);
        let new_total = current + amount;

        // Check stack limit
        if new_total > self.max_stack {
            return false;
        }

        // Check slot capacity for new items
        if current == 0 && self.slot_count() >= self.capacity {
            return false;
        }

        true
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

    /// Returns an iterator over all items as ItemStacks.
    pub fn stacks(&self) -> impl Iterator<Item = ItemStack> + '_ {
        self.items
            .iter()
            .map(|(&item_type, &quantity)| ItemStack::new(item_type, quantity))
    }

    /// Returns the maximum stack size.
    #[must_use]
    pub const fn max_stack(&self) -> u32 {
        self.max_stack
    }

    /// Returns the total number of items across all stacks.
    #[must_use]
    pub fn total_items(&self) -> u32 {
        self.items.values().sum()
    }

    /// Checks if the inventory is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clears all items from the inventory.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Returns free slots available.
    #[must_use]
    pub fn free_slots(&self) -> u32 {
        self.capacity.saturating_sub(self.slot_count())
    }
}

/// Atomically transfers items between inventories.
///
/// This operation either succeeds completely or fails without modifying
/// either inventory (atomic/transactional behavior).
pub fn transfer(
    from: &mut Inventory,
    to: &mut Inventory,
    item: ItemTypeId,
    amount: u32,
) -> InventoryResult<()> {
    // First validate the entire operation
    let available = from.count(item);
    if available < amount {
        return Err(InventoryError::TransferFailed {
            reason: format!(
                "Source has {} of {:?}, need {}",
                available,
                item.raw(),
                amount
            ),
        });
    }

    if !to.can_add(item, amount) {
        // Determine the specific reason
        let current = to.count(item);
        let new_total = current + amount;
        if new_total > to.max_stack {
            return Err(InventoryError::TransferFailed {
                reason: format!(
                    "Would exceed stack limit: max {}, would be {}",
                    to.max_stack, new_total
                ),
            });
        }
        return Err(InventoryError::TransferFailed {
            reason: format!("Destination inventory full (capacity {})", to.capacity),
        });
    }

    // Now perform the transfer - both operations should succeed
    from.remove(item, amount)
        .expect("Validated: source should have enough items");
    to.add(item, amount)
        .expect("Validated: destination should accept items");

    Ok(())
}

/// Transfers all items of a type between inventories.
///
/// Returns the amount transferred.
pub fn transfer_all(
    from: &mut Inventory,
    to: &mut Inventory,
    item: ItemTypeId,
) -> InventoryResult<u32> {
    let amount = from.count(item);
    if amount == 0 {
        return Ok(0);
    }
    transfer(from, to, item, amount)?;
    Ok(amount)
}

/// Transfers as many items as possible between inventories.
///
/// Returns the amount actually transferred.
pub fn transfer_max(from: &mut Inventory, to: &mut Inventory, item: ItemTypeId) -> u32 {
    let available = from.count(item);
    if available == 0 {
        return 0;
    }

    let to_current = to.count(item);
    let to_space = to.max_stack.saturating_sub(to_current);

    // Check if we need a new slot
    let effective_space = if to_current == 0 && to.free_slots() == 0 {
        0
    } else {
        to_space
    };

    let transfer_amount = available.min(effective_space);
    if transfer_amount == 0 {
        return 0;
    }

    // Perform the transfer
    let _ = from.remove(item, transfer_amount);
    let _ = to.add(item, transfer_amount);

    transfer_amount
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_new() {
        let inv = Inventory::new(10);
        assert_eq!(inv.capacity(), 10);
        assert_eq!(inv.slot_count(), 0);
        assert!(inv.is_empty());
    }

    #[test]
    fn test_inventory_with_stack_limit() {
        let inv = Inventory::with_stack_limit(10, 64);
        assert_eq!(inv.max_stack(), 64);
    }

    #[test]
    fn test_add_and_count() {
        let mut inv = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(inv.add(item, 5).is_ok());
        assert_eq!(inv.count(item), 5);
        assert_eq!(inv.slot_count(), 1);
    }

    #[test]
    fn test_stacking() {
        let mut inv = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(inv.add(item, 50).is_ok());
        assert!(inv.add(item, 25).is_ok());
        assert_eq!(inv.count(item), 75);
        assert_eq!(inv.slot_count(), 1); // Still only one slot
    }

    #[test]
    fn test_stack_overflow() {
        let mut inv = Inventory::with_stack_limit(10, 100);
        let item = ItemTypeId::new(1);

        assert!(inv.add(item, 50).is_ok());
        let result = inv.add(item, 60); // Would exceed 100 limit
        assert!(matches!(
            result,
            Err(InventoryError::StackOverflow {
                max: 100,
                would_be: 110
            })
        ));
        assert_eq!(inv.count(item), 50); // Unchanged
    }

    #[test]
    fn test_slot_capacity() {
        let mut inv = Inventory::new(2);

        let item1 = ItemTypeId::new(1);
        let item2 = ItemTypeId::new(2);
        let item3 = ItemTypeId::new(3);

        assert!(inv.add(item1, 10).is_ok());
        assert!(inv.add(item2, 10).is_ok());
        let result = inv.add(item3, 10);
        assert!(matches!(result, Err(InventoryError::Full { capacity: 2 })));
    }

    #[test]
    fn test_remove() {
        let mut inv = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(inv.add(item, 10).is_ok());
        assert!(inv.remove(item, 3).is_ok());
        assert_eq!(inv.count(item), 7);
    }

    #[test]
    fn test_remove_all_frees_slot() {
        let mut inv = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(inv.add(item, 10).is_ok());
        assert!(inv.remove(item, 10).is_ok());
        assert_eq!(inv.count(item), 0);
        assert_eq!(inv.slot_count(), 0);
    }

    #[test]
    fn test_remove_not_enough() {
        let mut inv = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(inv.add(item, 5).is_ok());
        let result = inv.remove(item, 10);
        assert!(matches!(
            result,
            Err(InventoryError::NotEnough {
                needed: 10,
                have: 5
            })
        ));
        assert_eq!(inv.count(item), 5); // Unchanged
    }

    #[test]
    fn test_has() {
        let mut inv = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(!inv.has(item, 1));
        assert!(inv.add(item, 10).is_ok());
        assert!(inv.has(item, 5));
        assert!(inv.has(item, 10));
        assert!(!inv.has(item, 11));
    }

    #[test]
    fn test_can_add() {
        let mut inv = Inventory::with_stack_limit(2, 100);
        let item1 = ItemTypeId::new(1);
        let item2 = ItemTypeId::new(2);
        let item3 = ItemTypeId::new(3);

        assert!(inv.can_add(item1, 50));
        assert!(inv.add(item1, 50).is_ok());

        // Can add more to existing stack
        assert!(inv.can_add(item1, 40));
        // Cannot exceed stack limit
        assert!(!inv.can_add(item1, 60));

        assert!(inv.add(item2, 10).is_ok());
        // No more slots
        assert!(!inv.can_add(item3, 10));
    }

    #[test]
    fn test_transfer_success() {
        let mut from = Inventory::new(10);
        let mut to = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(from.add(item, 50).is_ok());
        assert!(transfer(&mut from, &mut to, item, 30).is_ok());

        assert_eq!(from.count(item), 20);
        assert_eq!(to.count(item), 30);
    }

    #[test]
    fn test_transfer_not_enough() {
        let mut from = Inventory::new(10);
        let mut to = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(from.add(item, 20).is_ok());
        let result = transfer(&mut from, &mut to, item, 30);
        assert!(result.is_err());

        // Atomic: neither inventory changed
        assert_eq!(from.count(item), 20);
        assert_eq!(to.count(item), 0);
    }

    #[test]
    fn test_transfer_destination_full() {
        let mut from = Inventory::new(10);
        let mut to = Inventory::new(1);
        let item1 = ItemTypeId::new(1);
        let item2 = ItemTypeId::new(2);

        assert!(from.add(item1, 50).is_ok());
        assert!(to.add(item2, 10).is_ok()); // Fill the only slot

        let result = transfer(&mut from, &mut to, item1, 30);
        assert!(result.is_err());

        // Atomic: neither inventory changed
        assert_eq!(from.count(item1), 50);
        assert_eq!(to.count(item1), 0);
    }

    #[test]
    fn test_transfer_stack_overflow() {
        let mut from = Inventory::new(10);
        let mut to = Inventory::with_stack_limit(10, 50);
        let item = ItemTypeId::new(1);

        assert!(from.add(item, 100).is_ok());
        assert!(to.add(item, 30).is_ok());

        // Would exceed stack limit (30 + 30 = 60 > 50)
        let result = transfer(&mut from, &mut to, item, 30);
        assert!(result.is_err());

        // Atomic: neither inventory changed
        assert_eq!(from.count(item), 100);
        assert_eq!(to.count(item), 30);
    }

    #[test]
    fn test_transfer_all() {
        let mut from = Inventory::new(10);
        let mut to = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(from.add(item, 50).is_ok());
        let transferred = transfer_all(&mut from, &mut to, item).expect("Should succeed");

        assert_eq!(transferred, 50);
        assert_eq!(from.count(item), 0);
        assert_eq!(to.count(item), 50);
    }

    #[test]
    fn test_transfer_max() {
        let mut from = Inventory::new(10);
        let mut to = Inventory::with_stack_limit(10, 50);
        let item = ItemTypeId::new(1);

        assert!(from.add(item, 100).is_ok());
        assert!(to.add(item, 30).is_ok());

        // Can only transfer 20 more (50 - 30)
        let transferred = transfer_max(&mut from, &mut to, item);
        assert_eq!(transferred, 20);
        assert_eq!(from.count(item), 80);
        assert_eq!(to.count(item), 50);
    }

    #[test]
    fn test_transfer_max_no_slot() {
        let mut from = Inventory::new(10);
        let mut to = Inventory::new(1);
        let item1 = ItemTypeId::new(1);
        let item2 = ItemTypeId::new(2);

        assert!(from.add(item1, 100).is_ok());
        assert!(to.add(item2, 10).is_ok()); // Fill the only slot

        // Cannot transfer - no slot available
        let transferred = transfer_max(&mut from, &mut to, item1);
        assert_eq!(transferred, 0);
        assert_eq!(from.count(item1), 100);
    }

    #[test]
    fn test_iter_and_stacks() {
        let mut inv = Inventory::new(10);
        let item1 = ItemTypeId::new(1);
        let item2 = ItemTypeId::new(2);

        assert!(inv.add(item1, 10).is_ok());
        assert!(inv.add(item2, 20).is_ok());

        let items: Vec<_> = inv.iter().collect();
        assert_eq!(items.len(), 2);

        let stacks: Vec<_> = inv.stacks().collect();
        assert_eq!(stacks.len(), 2);
    }

    #[test]
    fn test_total_items() {
        let mut inv = Inventory::new(10);
        let item1 = ItemTypeId::new(1);
        let item2 = ItemTypeId::new(2);

        assert!(inv.add(item1, 10).is_ok());
        assert!(inv.add(item2, 20).is_ok());

        assert_eq!(inv.total_items(), 30);
    }

    #[test]
    fn test_clear() {
        let mut inv = Inventory::new(10);
        let item = ItemTypeId::new(1);

        assert!(inv.add(item, 50).is_ok());
        inv.clear();

        assert!(inv.is_empty());
        assert_eq!(inv.slot_count(), 0);
    }

    #[test]
    fn test_free_slots() {
        let mut inv = Inventory::new(5);
        let item1 = ItemTypeId::new(1);
        let item2 = ItemTypeId::new(2);

        assert_eq!(inv.free_slots(), 5);

        assert!(inv.add(item1, 10).is_ok());
        assert_eq!(inv.free_slots(), 4);

        assert!(inv.add(item2, 10).is_ok());
        assert_eq!(inv.free_slots(), 3);
    }
}
