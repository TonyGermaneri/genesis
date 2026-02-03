//! Event bus for inter-system communication.

use crossbeam_channel::{bounded, Receiver, Sender};
use serde::{Deserialize, Serialize};

use genesis_common::EntityId;

/// Event types that can be sent through the event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    /// Entity spawned
    EntitySpawned {
        /// Entity ID
        entity_id: EntityId,
    },
    /// Entity destroyed
    EntityDestroyed {
        /// Entity ID
        entity_id: EntityId,
    },
    /// Entity took damage
    EntityDamaged {
        /// Entity ID
        entity_id: EntityId,
        /// Damage amount
        damage: i32,
        /// Source entity (if any)
        source: Option<EntityId>,
    },
    /// Item picked up
    ItemPickedUp {
        /// Entity that picked up
        entity_id: EntityId,
        /// Item type
        item_type: u32,
        /// Quantity
        quantity: u32,
    },
    /// Item crafted
    ItemCrafted {
        /// Entity that crafted
        entity_id: EntityId,
        /// Recipe used
        recipe_id: u32,
    },
    /// Building placed
    BuildingPlaced {
        /// Entity that placed
        entity_id: EntityId,
        /// Building type
        building_type: u32,
        /// World X
        x: i64,
        /// World Y
        y: i64,
    },
    /// Transaction completed
    Transaction {
        /// Buyer entity
        buyer: EntityId,
        /// Seller entity
        seller: EntityId,
        /// Amount
        amount: u64,
    },
    /// Reputation changed
    ReputationChanged {
        /// Entity affected
        entity_id: EntityId,
        /// Faction ID
        faction_id: u16,
        /// New value
        new_value: i32,
    },
    /// Custom mod event
    Custom {
        /// Event name
        name: String,
        /// JSON payload
        payload: String,
    },
}

/// Event bus for broadcasting events to subscribers.
#[derive(Debug)]
pub struct EventBus {
    /// Sender for broadcasting events
    sender: Sender<GameEvent>,
    /// Receiver for collecting events
    receiver: Receiver<GameEvent>,
    /// Channel capacity
    capacity: usize,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl EventBus {
    /// Creates a new event bus with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self {
            sender,
            receiver,
            capacity,
        }
    }

    /// Publishes an event to the bus.
    pub fn publish(&self, event: GameEvent) {
        // Non-blocking send - if full, event is dropped
        let _ = self.sender.try_send(event);
    }

    /// Drains all pending events.
    pub fn drain(&self) -> Vec<GameEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }

    /// Returns the number of pending events.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.receiver.len()
    }

    /// Returns the channel capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Creates a new sender handle for publishing events.
    #[must_use]
    pub fn sender(&self) -> Sender<GameEvent> {
        self.sender.clone()
    }
}

/// Typed event handler trait.
pub trait EventHandler: Send + Sync {
    /// Handles an event.
    fn handle(&self, event: &GameEvent);
}
