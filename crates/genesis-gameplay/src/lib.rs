//! # Genesis Gameplay
//!
//! Gameplay systems for Project Genesis.
//!
//! This crate provides the CPU-side entity layer and all RPG systems:
//! - Entities (player, NPCs, vehicles)
//! - Player controller with movement and physics
//! - Input handling system
//! - World interaction (dig/place)
//! - Inventory system
//! - Item crafting system
//! - Building crafting system
//! - Economy (prices, wallet, trade)
//! - Factions and reputation
//! - Needs (food, water)
//! - Event bus for inter-system communication

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

pub mod crafting;
pub mod crafting_ui;
pub mod economy;
pub mod entity;
pub mod events;
pub mod faction;
pub mod input;
pub mod interaction;
pub mod inventory;
pub mod inventory_ui;
pub mod needs;
pub mod physics;
pub mod player;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::crafting::*;
    pub use crate::crafting_ui::*;
    pub use crate::economy::*;
    pub use crate::entity::*;
    pub use crate::events::*;
    pub use crate::faction::*;
    pub use crate::input::*;
    pub use crate::interaction::*;
    pub use crate::inventory::*;
    pub use crate::inventory_ui::*;
    pub use crate::needs::*;
    pub use crate::physics::*;
    pub use crate::player::*;
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_add_remove() {
        use genesis_common::ItemTypeId;

        let mut inv = Inventory::new(10);
        let item_id = ItemTypeId::new(1);

        assert!(inv.add(item_id, 5).is_ok());
        assert_eq!(inv.count(item_id), 5);

        assert!(inv.remove(item_id, 3).is_ok());
        assert_eq!(inv.count(item_id), 2);
    }

    #[test]
    fn test_wallet_transactions() {
        let mut wallet = Wallet::new(1000);

        assert!(wallet.spend(500).is_ok());
        assert_eq!(wallet.balance(), 500);

        wallet.earn(200);
        assert_eq!(wallet.balance(), 700);
    }

    #[test]
    fn test_faction_reputation() {
        use genesis_common::FactionId;

        let mut rep = ReputationTracker::new();
        let faction = FactionId::new(1);

        rep.modify(faction, 50);
        assert_eq!(rep.standing(faction), ReputationStanding::Allied);

        rep.modify(faction, -100);
        assert_eq!(rep.standing(faction), ReputationStanding::Hostile);
    }

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(EntityType::Player);
        assert!(entity.id().is_valid());
        assert_eq!(entity.entity_type(), EntityType::Player);
    }
}
