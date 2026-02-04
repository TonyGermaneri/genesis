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

pub mod ai;
pub mod ambient;
pub mod biome;
pub mod collision_response;
pub mod combat;
pub mod crafting;
pub mod crafting_ui;
pub mod dialogue;
pub mod economy;
pub mod entity;
pub mod events;
pub mod faction;
pub mod game_state;
pub mod input;
pub mod interaction;
pub mod inventory;
pub mod inventory_ui;
pub mod music;
pub mod needs;
pub mod npc;
pub mod npc_spawning;
pub mod physics;
pub mod plants;
pub mod player;
pub mod quest;
pub mod save;
pub mod sound_events;
pub mod sound_triggers;
pub mod spawn;
pub mod terrain_manipulation;
pub mod time;
pub mod vehicle;
pub mod weather;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::ai::*;
    pub use crate::ambient::*;
    pub use crate::biome::*;
    pub use crate::collision_response::*;
    pub use crate::combat::*;
    pub use crate::crafting::*;
    pub use crate::crafting_ui::*;
    pub use crate::dialogue::*;
    pub use crate::economy::*;
    pub use crate::entity::*;
    pub use crate::events::*;
    pub use crate::faction::*;
    pub use crate::game_state::*;
    pub use crate::input::*;
    pub use crate::interaction::*;
    pub use crate::inventory::*;
    pub use crate::inventory_ui::*;
    pub use crate::music::*;
    pub use crate::needs::*;
    pub use crate::npc::*;
    pub use crate::npc_spawning::*;
    pub use crate::physics::*;
    pub use crate::plants::*;
    pub use crate::player::*;
    pub use crate::quest::*;
    pub use crate::save::*;
    pub use crate::sound_events::*;
    pub use crate::sound_triggers::*;
    pub use crate::spawn::*;
    pub use crate::terrain_manipulation::*;
    pub use crate::time::*;
    pub use crate::vehicle::*;
    pub use crate::weather::*;
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
