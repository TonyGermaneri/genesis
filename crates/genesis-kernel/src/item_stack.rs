//! Item Stack Management
//!
//! This module provides low-level item stack operations for inventory and
//! crafting systems. It handles:
//!
//! - Stack combining and splitting
//! - Durability tracking
//! - Metadata storage (enchantments, custom data)
//! - Serialization for persistence
//!
//! # Architecture
//!
//! ```text
//! ┌────────────┐     ┌──────────────┐     ┌─────────────────┐
//! │ ItemStack  │────▶│ StackResult  │────▶│ Inventory/Grid  │
//! │ (core data)│     │ (operation)  │     │ (storage)       │
//! └────────────┘     └──────────────┘     └─────────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use genesis_kernel::item_stack::{ItemStack, StackResult};
//!
//! // Create stacks
//! let mut stack1 = ItemStack::new(1, 32); // 32 of item ID 1
//! let mut stack2 = ItemStack::new(1, 20); // 20 of item ID 1
//!
//! // Combine stacks (max 64)
//! let result = stack1.try_combine(&mut stack2, 64);
//! assert_eq!(stack1.count(), 52);
//! assert_eq!(stack2.count(), 0);
//!
//! // Split a stack
//! let split = stack1.split(10);
//! assert_eq!(stack1.count(), 42);
//! assert_eq!(split.count(), 10);
//! ```

use std::collections::HashMap;

use tracing::trace;

/// Item ID type (matches CraftingGrid::ItemId).
pub type ItemId = u32;

/// Durability value type.
pub type Durability = u16;

/// Metadata key type.
pub type MetadataKey = u16;

/// Maximum stack size constant.
pub const DEFAULT_MAX_STACK: u32 = 64;

/// Infinite durability marker.
pub const INFINITE_DURABILITY: Durability = Durability::MAX;

/// No durability (tools that don't break).
pub const NO_DURABILITY: Durability = 0;

/// Result of a stack operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackResult {
    /// Operation completed fully.
    Complete,
    /// Operation partially completed with remainder.
    Partial(u32),
    /// Operation failed (incompatible items).
    Incompatible,
    /// Source stack is empty.
    Empty,
}

impl StackResult {
    /// Check if operation was at least partially successful.
    #[must_use]
    pub const fn success(&self) -> bool {
        matches!(self, Self::Complete | Self::Partial(_))
    }

    /// Get remainder count if partial.
    #[must_use]
    pub const fn remainder(&self) -> Option<u32> {
        match self {
            Self::Partial(n) => Some(*n),
            _ => None,
        }
    }
}

/// Item metadata storage.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemMetadata {
    /// Key-value metadata pairs.
    data: HashMap<MetadataKey, i64>,
    /// Custom tag data (for complex metadata).
    tag: Option<Box<[u8]>>,
}

impl ItemMetadata {
    /// Create empty metadata.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a metadata value.
    pub fn set(&mut self, key: MetadataKey, value: i64) {
        self.data.insert(key, value);
    }

    /// Get a metadata value.
    #[must_use]
    pub fn get(&self, key: MetadataKey) -> Option<i64> {
        self.data.get(&key).copied()
    }

    /// Remove a metadata value.
    pub fn remove(&mut self, key: MetadataKey) -> Option<i64> {
        self.data.remove(&key)
    }

    /// Check if metadata is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.tag.is_none()
    }

    /// Set custom tag data.
    pub fn set_tag(&mut self, tag: &[u8]) {
        self.tag = Some(tag.into());
    }

    /// Get custom tag data.
    #[must_use]
    pub fn tag(&self) -> Option<&[u8]> {
        self.tag.as_deref()
    }

    /// Clear all metadata.
    pub fn clear(&mut self) {
        self.data.clear();
        self.tag = None;
    }

    /// Clone metadata (for stack splitting).
    #[must_use]
    pub fn clone_meta(&self) -> Self {
        self.clone()
    }
}

/// An item stack with count, durability, and metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    /// Item type ID (0 = empty/air).
    item_id: ItemId,
    /// Number of items in the stack.
    count: u32,
    /// Current durability (0 = destroyed, MAX = infinite).
    durability: Durability,
    /// Maximum durability for this item.
    max_durability: Durability,
    /// Item variant/damage value.
    variant: u16,
    /// Additional metadata.
    metadata: ItemMetadata,
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::empty()
    }
}

impl ItemStack {
    /// Create an empty stack.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            item_id: 0,
            count: 0,
            durability: NO_DURABILITY,
            max_durability: NO_DURABILITY,
            variant: 0,
            metadata: ItemMetadata::new(),
        }
    }

    /// Create a new item stack.
    #[must_use]
    pub fn new(item_id: ItemId, count: u32) -> Self {
        Self {
            item_id,
            count,
            durability: NO_DURABILITY,
            max_durability: NO_DURABILITY,
            variant: 0,
            metadata: ItemMetadata::new(),
        }
    }

    /// Create a stack with durability.
    #[must_use]
    pub fn with_durability(item_id: ItemId, count: u32, max_durability: Durability) -> Self {
        Self {
            item_id,
            count,
            durability: max_durability,
            max_durability,
            variant: 0,
            metadata: ItemMetadata::new(),
        }
    }

    /// Create a stack with a variant.
    #[must_use]
    pub fn with_variant(item_id: ItemId, count: u32, variant: u16) -> Self {
        Self {
            item_id,
            count,
            durability: NO_DURABILITY,
            max_durability: NO_DURABILITY,
            variant,
            metadata: ItemMetadata::new(),
        }
    }

    /// Get the item ID.
    #[must_use]
    pub const fn item_id(&self) -> ItemId {
        self.item_id
    }

    /// Get the stack count.
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.count
    }

    /// Get the variant.
    #[must_use]
    pub const fn variant(&self) -> u16 {
        self.variant
    }

    /// Set the variant.
    pub fn set_variant(&mut self, variant: u16) {
        self.variant = variant;
    }

    /// Get current durability.
    #[must_use]
    pub const fn durability(&self) -> Durability {
        self.durability
    }

    /// Get maximum durability.
    #[must_use]
    pub const fn max_durability(&self) -> Durability {
        self.max_durability
    }

    /// Get durability as a ratio (0.0 to 1.0).
    #[must_use]
    pub fn durability_ratio(&self) -> f32 {
        if self.max_durability == NO_DURABILITY || self.max_durability == INFINITE_DURABILITY {
            1.0
        } else {
            self.durability as f32 / self.max_durability as f32
        }
    }

    /// Check if the stack is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.item_id == 0 || self.count == 0
    }

    /// Check if the item has durability tracking.
    #[must_use]
    pub const fn has_durability(&self) -> bool {
        self.max_durability != NO_DURABILITY
    }

    /// Check if durability is infinite.
    #[must_use]
    pub const fn is_infinite_durability(&self) -> bool {
        self.max_durability == INFINITE_DURABILITY
    }

    /// Check if the item is broken (durability depleted).
    #[must_use]
    pub const fn is_broken(&self) -> bool {
        self.has_durability() && !self.is_infinite_durability() && self.durability == 0
    }

    /// Get mutable metadata reference.
    pub fn metadata_mut(&mut self) -> &mut ItemMetadata {
        &mut self.metadata
    }

    /// Get metadata reference.
    #[must_use]
    pub const fn metadata(&self) -> &ItemMetadata {
        &self.metadata
    }

    /// Check if this stack can combine with another.
    #[must_use]
    pub fn can_combine(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return true; // Empty stacks can always "combine"
        }

        self.item_id == other.item_id
            && self.variant == other.variant
            && self.durability == other.durability
            && self.max_durability == other.max_durability
            && self.metadata == other.metadata
    }

    /// Try to combine another stack into this one.
    ///
    /// Returns the result of the operation and modifies both stacks.
    pub fn try_combine(&mut self, source: &mut Self, max_stack: u32) -> StackResult {
        if source.is_empty() {
            return StackResult::Empty;
        }

        if self.is_empty() {
            // Take all from source
            *self = source.clone();
            let taken = source.count.min(max_stack);
            self.count = taken;
            source.count -= taken;
            if source.count == 0 {
                *source = Self::empty();
            }
            return if source.is_empty() {
                StackResult::Complete
            } else {
                StackResult::Partial(source.count)
            };
        }

        if !self.can_combine(source) {
            return StackResult::Incompatible;
        }

        let space = max_stack.saturating_sub(self.count);
        let transfer = source.count.min(space);

        self.count += transfer;
        source.count -= transfer;

        if source.count == 0 {
            *source = Self::empty();
            StackResult::Complete
        } else {
            StackResult::Partial(source.count)
        }
    }

    /// Split off a portion of this stack.
    ///
    /// Returns the split-off portion.
    #[must_use]
    pub fn split(&mut self, amount: u32) -> Self {
        if self.is_empty() || amount == 0 {
            return Self::empty();
        }

        let take = amount.min(self.count);
        self.count -= take;

        let mut split = self.clone();
        split.count = take;

        if self.count == 0 {
            *self = Self::empty();
        }

        trace!("Split {} items, {} remaining", take, self.count);
        split
    }

    /// Split the stack in half.
    #[must_use]
    pub fn split_half(&mut self) -> Self {
        let half = self.count.div_ceil(2);
        self.split(half)
    }

    /// Take one item from the stack.
    #[must_use]
    pub fn take_one(&mut self) -> Self {
        self.split(1)
    }

    /// Add items to the stack (unchecked).
    pub fn add(&mut self, amount: u32) {
        self.count = self.count.saturating_add(amount);
    }

    /// Remove items from the stack.
    ///
    /// Returns the actual amount removed.
    pub fn remove(&mut self, amount: u32) -> u32 {
        let removed = amount.min(self.count);
        self.count -= removed;

        if self.count == 0 {
            *self = Self::empty();
        }

        removed
    }

    /// Set the count directly.
    pub fn set_count(&mut self, count: u32) {
        self.count = count;
        if count == 0 {
            *self = Self::empty();
        }
    }

    /// Damage the item (reduce durability).
    ///
    /// Returns true if the item broke.
    pub fn damage(&mut self, amount: Durability) -> bool {
        if !self.has_durability() || self.is_infinite_durability() {
            return false;
        }

        self.durability = self.durability.saturating_sub(amount);
        let broke = self.durability == 0;

        if broke {
            trace!("Item {:08x} broke", self.item_id);
        }

        broke
    }

    /// Repair the item.
    ///
    /// Returns the amount actually repaired.
    pub fn repair(&mut self, amount: Durability) -> Durability {
        if !self.has_durability() || self.is_infinite_durability() {
            return 0;
        }

        let before = self.durability;
        self.durability = self.durability.saturating_add(amount).min(self.max_durability);
        self.durability - before
    }

    /// Fully repair the item.
    pub fn repair_full(&mut self) {
        if self.has_durability() && !self.is_infinite_durability() {
            self.durability = self.max_durability;
        }
    }

    /// Clone the stack with a new count.
    #[must_use]
    pub fn clone_with_count(&self, count: u32) -> Self {
        let mut cloned = self.clone();
        cloned.count = count;
        if count == 0 {
            Self::empty()
        } else {
            cloned
        }
    }

    /// Swap this stack with another.
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }

    /// Transfer items from another stack.
    ///
    /// Returns (transferred, remaining in source).
    pub fn transfer_from(&mut self, source: &mut Self, max_transfer: u32, max_stack: u32) -> (u32, u32) {
        if source.is_empty() {
            return (0, 0);
        }

        if self.is_empty() {
            let take = source.count.min(max_transfer).min(max_stack);
            *self = source.clone_with_count(take);
            source.count -= take;
            if source.count == 0 {
                *source = Self::empty();
            }
            return (take, source.count);
        }

        if !self.can_combine(source) {
            return (0, source.count);
        }

        let space = max_stack.saturating_sub(self.count);
        let transfer = source.count.min(max_transfer).min(space);

        self.count += transfer;
        source.count -= transfer;

        if source.count == 0 {
            *source = Self::empty();
        }

        (transfer, source.count)
    }

    /// Serialize to bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);

        // Item ID (4 bytes)
        bytes.extend_from_slice(&self.item_id.to_le_bytes());
        // Count (4 bytes)
        bytes.extend_from_slice(&self.count.to_le_bytes());
        // Durability (2 bytes)
        bytes.extend_from_slice(&self.durability.to_le_bytes());
        // Max durability (2 bytes)
        bytes.extend_from_slice(&self.max_durability.to_le_bytes());
        // Variant (2 bytes)
        bytes.extend_from_slice(&self.variant.to_le_bytes());
        // Metadata count (2 bytes)
        let meta_count = self.metadata.data.len() as u16;
        bytes.extend_from_slice(&meta_count.to_le_bytes());

        // Metadata entries
        for (&key, &value) in &self.metadata.data {
            bytes.extend_from_slice(&key.to_le_bytes());
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        // Tag data
        if let Some(tag) = &self.metadata.tag {
            bytes.extend_from_slice(&(tag.len() as u32).to_le_bytes());
            bytes.extend_from_slice(tag);
        } else {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }

        bytes
    }

    /// Deserialize from bytes.
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }

        let item_id = u32::from_le_bytes(data[0..4].try_into().ok()?);
        let count = u32::from_le_bytes(data[4..8].try_into().ok()?);
        let durability = u16::from_le_bytes(data[8..10].try_into().ok()?);
        let max_durability = u16::from_le_bytes(data[10..12].try_into().ok()?);
        let variant = u16::from_le_bytes(data[12..14].try_into().ok()?);
        let meta_count = u16::from_le_bytes(data[14..16].try_into().ok()?) as usize;

        let mut offset = 16;
        let mut metadata = ItemMetadata::new();

        // Read metadata entries
        for _ in 0..meta_count {
            if offset + 10 > data.len() {
                return None;
            }
            let key = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?);
            let value = i64::from_le_bytes(data[offset + 2..offset + 10].try_into().ok()?);
            metadata.set(key, value);
            offset += 10;
        }

        // Read tag data
        if offset + 4 <= data.len() {
            let tag_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
            offset += 4;
            if tag_len > 0 && offset + tag_len <= data.len() {
                metadata.set_tag(&data[offset..offset + tag_len]);
            }
        }

        Some(Self {
            item_id,
            count,
            durability,
            max_durability,
            variant,
            metadata,
        })
    }
}

/// Builder for creating item stacks with complex configuration.
#[derive(Debug, Clone)]
pub struct ItemStackBuilder {
    stack: ItemStack,
}

impl ItemStackBuilder {
    /// Create a new builder for an item.
    #[must_use]
    pub fn new(item_id: ItemId) -> Self {
        Self {
            stack: ItemStack::new(item_id, 1),
        }
    }

    /// Set the count.
    #[must_use]
    pub const fn count(mut self, count: u32) -> Self {
        self.stack.count = count;
        self
    }

    /// Set the variant.
    #[must_use]
    pub const fn variant(mut self, variant: u16) -> Self {
        self.stack.variant = variant;
        self
    }

    /// Set durability.
    #[must_use]
    pub const fn durability(mut self, current: Durability, max: Durability) -> Self {
        self.stack.durability = current;
        self.stack.max_durability = max;
        self
    }

    /// Set maximum durability (current = max).
    #[must_use]
    pub const fn max_durability(mut self, max: Durability) -> Self {
        self.stack.durability = max;
        self.stack.max_durability = max;
        self
    }

    /// Set infinite durability.
    #[must_use]
    pub const fn infinite_durability(mut self) -> Self {
        self.stack.durability = INFINITE_DURABILITY;
        self.stack.max_durability = INFINITE_DURABILITY;
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn metadata(mut self, key: MetadataKey, value: i64) -> Self {
        self.stack.metadata.set(key, value);
        self
    }

    /// Add tag data.
    #[must_use]
    pub fn tag(mut self, data: &[u8]) -> Self {
        self.stack.metadata.set_tag(data);
        self
    }

    /// Build the item stack.
    #[must_use]
    pub fn build(self) -> ItemStack {
        self.stack
    }
}

/// Compact stack representation for GPU/network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct CompactStack {
    /// Item ID.
    pub item_id: u32,
    /// Count and variant packed.
    pub count_variant: u32,
    /// Durability packed.
    pub durability: u32,
}

impl CompactStack {
    /// Create from an item stack.
    #[must_use]
    pub fn from_stack(stack: &ItemStack) -> Self {
        Self {
            item_id: stack.item_id,
            count_variant: (stack.count & 0xFFFF) | ((stack.variant as u32) << 16),
            durability: (stack.durability as u32) | ((stack.max_durability as u32) << 16),
        }
    }

    /// Convert to an item stack (without metadata).
    #[must_use]
    pub fn to_stack(&self) -> ItemStack {
        ItemStack {
            item_id: self.item_id,
            count: self.count_variant & 0xFFFF,
            variant: (self.count_variant >> 16) as u16,
            durability: (self.durability & 0xFFFF) as u16,
            max_durability: (self.durability >> 16) as u16,
            metadata: ItemMetadata::new(),
        }
    }

    /// Check if empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.item_id == 0 || self.count_variant.trailing_zeros() >= 16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_stack_creation() {
        let stack = ItemStack::new(42, 10);
        assert_eq!(stack.item_id(), 42);
        assert_eq!(stack.count(), 10);
        assert!(!stack.is_empty());
    }

    #[test]
    fn test_empty_stack() {
        let empty = ItemStack::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.item_id(), 0);
        assert_eq!(empty.count(), 0);
    }

    #[test]
    fn test_stack_with_durability() {
        let mut stack = ItemStack::with_durability(1, 1, 100);
        assert!(stack.has_durability());
        assert_eq!(stack.durability(), 100);
        assert_eq!(stack.max_durability(), 100);
        assert!((stack.durability_ratio() - 1.0).abs() < f32::EPSILON);

        // Damage the item
        assert!(!stack.damage(30));
        assert_eq!(stack.durability(), 70);
        assert!((stack.durability_ratio() - 0.7).abs() < f32::EPSILON);

        // Break the item
        assert!(stack.damage(100));
        assert!(stack.is_broken());
    }

    #[test]
    fn test_stack_combine() {
        let mut dest = ItemStack::new(1, 32);
        let mut source = ItemStack::new(1, 20);

        let result = dest.try_combine(&mut source, 64);
        assert_eq!(result, StackResult::Complete);
        assert_eq!(dest.count(), 52);
        assert!(source.is_empty());
    }

    #[test]
    fn test_stack_combine_partial() {
        let mut dest = ItemStack::new(1, 50);
        let mut source = ItemStack::new(1, 30);

        let result = dest.try_combine(&mut source, 64);
        assert_eq!(result, StackResult::Partial(16));
        assert_eq!(dest.count(), 64);
        assert_eq!(source.count(), 16);
    }

    #[test]
    fn test_stack_combine_incompatible() {
        let mut dest = ItemStack::new(1, 32);
        let mut source = ItemStack::new(2, 20);

        let result = dest.try_combine(&mut source, 64);
        assert_eq!(result, StackResult::Incompatible);
        assert_eq!(dest.count(), 32);
        assert_eq!(source.count(), 20);
    }

    #[test]
    fn test_stack_combine_empty_dest() {
        let mut dest = ItemStack::empty();
        let mut source = ItemStack::new(1, 20);

        let result = dest.try_combine(&mut source, 64);
        assert_eq!(result, StackResult::Complete);
        assert_eq!(dest.item_id(), 1);
        assert_eq!(dest.count(), 20);
        assert!(source.is_empty());
    }

    #[test]
    fn test_stack_split() {
        let mut stack = ItemStack::new(1, 30);
        let split = stack.split(10);

        assert_eq!(stack.count(), 20);
        assert_eq!(split.count(), 10);
        assert_eq!(split.item_id(), 1);
    }

    #[test]
    fn test_stack_split_half() {
        let mut stack = ItemStack::new(1, 31);
        let split = stack.split_half();

        assert_eq!(stack.count(), 15);
        assert_eq!(split.count(), 16);
    }

    #[test]
    fn test_stack_transfer() {
        let mut dest = ItemStack::new(1, 20);
        let mut source = ItemStack::new(1, 50);

        let (transferred, remaining) = dest.transfer_from(&mut source, 30, 64);
        assert_eq!(transferred, 30);
        assert_eq!(remaining, 20);
        assert_eq!(dest.count(), 50);
        assert_eq!(source.count(), 20);
    }

    #[test]
    fn test_stack_metadata() {
        let mut stack = ItemStack::new(1, 1);
        stack.metadata_mut().set(1, 42);
        stack.metadata_mut().set(2, -100);

        assert_eq!(stack.metadata().get(1), Some(42));
        assert_eq!(stack.metadata().get(2), Some(-100));
        assert_eq!(stack.metadata().get(3), None);
    }

    #[test]
    fn test_stack_serialization() {
        let mut stack = ItemStackBuilder::new(42)
            .count(10)
            .variant(5)
            .max_durability(100)
            .metadata(1, 12345)
            .build();
        stack.damage(30);

        let bytes = stack.to_bytes();
        let restored = ItemStack::from_bytes(&bytes).expect("should deserialize");

        assert_eq!(restored.item_id(), 42);
        assert_eq!(restored.count(), 10);
        assert_eq!(restored.variant(), 5);
        assert_eq!(restored.durability(), 70);
        assert_eq!(restored.max_durability(), 100);
        assert_eq!(restored.metadata().get(1), Some(12345));
    }

    #[test]
    fn test_stack_builder() {
        let stack = ItemStackBuilder::new(100)
            .count(32)
            .variant(3)
            .max_durability(500)
            .metadata(10, 999)
            .build();

        assert_eq!(stack.item_id(), 100);
        assert_eq!(stack.count(), 32);
        assert_eq!(stack.variant(), 3);
        assert_eq!(stack.durability(), 500);
        assert_eq!(stack.metadata().get(10), Some(999));
    }

    #[test]
    fn test_compact_stack() {
        let stack = ItemStackBuilder::new(42)
            .count(30)
            .variant(5)
            .durability(70, 100)
            .build();

        let compact = CompactStack::from_stack(&stack);
        let restored = compact.to_stack();

        assert_eq!(restored.item_id(), 42);
        assert_eq!(restored.count(), 30);
        assert_eq!(restored.variant(), 5);
        assert_eq!(restored.durability(), 70);
        assert_eq!(restored.max_durability(), 100);
    }

    #[test]
    fn test_infinite_durability() {
        let stack = ItemStackBuilder::new(1)
            .infinite_durability()
            .build();

        assert!(stack.is_infinite_durability());
        assert!(!stack.is_broken());
    }

    #[test]
    fn test_repair() {
        let mut stack = ItemStack::with_durability(1, 1, 100);
        stack.damage(60);
        assert_eq!(stack.durability(), 40);

        let repaired = stack.repair(30);
        assert_eq!(repaired, 30);
        assert_eq!(stack.durability(), 70);

        // Can't repair past max
        let repaired = stack.repair(100);
        assert_eq!(repaired, 30);
        assert_eq!(stack.durability(), 100);
    }

    #[test]
    fn test_stack_swap() {
        let mut a = ItemStack::new(1, 10);
        let mut b = ItemStack::new(2, 20);

        a.swap(&mut b);

        assert_eq!(a.item_id(), 2);
        assert_eq!(a.count(), 20);
        assert_eq!(b.item_id(), 1);
        assert_eq!(b.count(), 10);
    }
}
