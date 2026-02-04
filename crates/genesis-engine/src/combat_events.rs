//! Combat event integration.
//!
//! This module provides:
//! - OnAttack: trigger sounds, particles, hitbox
//! - OnHit: apply damage, knockback, effects
//! - OnDeath: drop loot, play animation
//! - OnBlock: reduce damage, play sound

use genesis_common::EntityId;
use std::collections::VecDeque;
use tracing::{debug, info};

use crate::audio_assets::AudioCategory;
use crate::audio_integration::{AudioIntegration, SoundEvent};

/// Types of combat events.
#[derive(Debug, Clone)]
pub enum CombatEvent {
    /// Attack initiated.
    Attack(AttackEvent),
    /// Attack hit a target.
    Hit(HitEvent),
    /// Entity blocked an attack.
    Block(BlockEvent),
    /// Entity died.
    Death(DeathEvent),
    /// Entity took damage over time.
    DamageOverTime(DotEvent),
    /// Status effect applied.
    StatusApplied(StatusEvent),
    /// Status effect removed.
    StatusRemoved(StatusEvent),
    /// Projectile spawned.
    ProjectileSpawned(ProjectileEvent),
    /// Projectile hit.
    ProjectileHit(ProjectileEvent),
}

/// Event for an attack being initiated.
#[derive(Debug, Clone)]
pub struct AttackEvent {
    /// Entity performing the attack.
    pub attacker: EntityId,
    /// Target of the attack (entity or position).
    pub target: AttackTarget,
    /// Weapon ID used (if any).
    pub weapon_id: Option<u32>,
    /// Attack type (melee, ranged, etc.).
    pub attack_type: AttackCategory,
    /// Position of the attacker.
    pub position: (f32, f32),
    /// Direction of the attack (normalized).
    pub direction: (f32, f32),
}

/// Target types for attacks.
#[derive(Debug, Clone, Copy)]
pub enum AttackTarget {
    /// Target a specific entity.
    Entity(EntityId),
    /// Target a position in the world.
    Position(f32, f32),
    /// Target in a direction (for melee swings).
    Direction(f32, f32),
}

/// Categories of attacks for sound/visual selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackCategory {
    /// Melee weapon swing.
    MeleeSwing,
    /// Melee weapon thrust.
    MeleeThrust,
    /// Ranged bow shot.
    RangedBow,
    /// Ranged gun shot.
    RangedGun,
    /// Magic spell cast.
    MagicSpell,
    /// Unarmed punch/kick.
    Unarmed,
}

impl AttackCategory {
    /// Returns the sound name for this attack type.
    #[must_use]
    pub const fn sound_name(&self) -> &'static str {
        match self {
            Self::MeleeSwing => "combat/melee_swing",
            Self::MeleeThrust => "combat/melee_thrust",
            Self::RangedBow => "combat/bow_draw",
            Self::RangedGun => "combat/gunshot",
            Self::MagicSpell => "combat/spell_cast",
            Self::Unarmed => "combat/punch",
        }
    }
}

/// Event for when an attack hits a target.
#[derive(Debug, Clone)]
pub struct HitEvent {
    /// Entity that dealt the damage.
    pub attacker: EntityId,
    /// Entity that received the damage.
    pub target: EntityId,
    /// Damage dealt (after mitigation).
    pub damage: f32,
    /// Damage type.
    pub damage_type: DamageCategory,
    /// Position of the hit.
    pub position: (f32, f32),
    /// Knockback applied (if any).
    pub knockback: Option<(f32, f32)>,
    /// Whether this was a critical hit.
    pub critical: bool,
    /// Damage blocked by armor.
    pub blocked: f32,
    /// Damage resisted.
    pub resisted: f32,
}

/// Categories of damage for sound/visual selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageCategory {
    /// Physical damage (weapon, fall, etc.).
    Physical,
    /// Fire damage.
    Fire,
    /// Ice/cold damage.
    Ice,
    /// Electric/lightning damage.
    Electric,
    /// Poison damage.
    Poison,
    /// True damage (ignores defenses).
    True,
}

impl DamageCategory {
    /// Returns the hit sound name for this damage type.
    #[must_use]
    pub const fn hit_sound(&self) -> &'static str {
        match self {
            Self::Physical => "combat/hit_physical",
            Self::Fire => "combat/hit_fire",
            Self::Ice => "combat/hit_ice",
            Self::Electric => "combat/hit_electric",
            Self::Poison => "combat/hit_poison",
            Self::True => "combat/hit_magic",
        }
    }
}

/// Event for when an attack is blocked.
#[derive(Debug, Clone)]
pub struct BlockEvent {
    /// Entity that blocked.
    pub blocker: EntityId,
    /// Entity that attacked.
    pub attacker: EntityId,
    /// Damage that would have been dealt.
    pub damage_blocked: f32,
    /// Whether it was a perfect block (full damage negation).
    pub perfect_block: bool,
    /// Position of the block.
    pub position: (f32, f32),
    /// Stamina consumed by blocking.
    pub stamina_cost: f32,
}

/// Event for when an entity dies.
#[derive(Debug, Clone)]
pub struct DeathEvent {
    /// Entity that died.
    pub entity: EntityId,
    /// Entity that dealt the killing blow (if any).
    pub killer: Option<EntityId>,
    /// Position of death.
    pub position: (f32, f32),
    /// Cause of death.
    pub cause: DeathCause,
    /// Experience to award to killer.
    pub experience: u32,
}

/// Causes of death.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    /// Killed by another entity.
    Combat,
    /// Environmental damage (lava, fall, etc.).
    Environmental,
    /// Damage over time effect.
    DamageOverTime,
    /// Starvation/dehydration.
    Needs,
    /// Unknown/other cause.
    Unknown,
}

impl DeathCause {
    /// Returns the death sound name.
    #[must_use]
    pub const fn sound_name(&self) -> &'static str {
        match self {
            Self::Combat => "combat/death_combat",
            Self::Environmental => "combat/death_environmental",
            Self::DamageOverTime => "combat/death_dot",
            Self::Needs => "combat/death_needs",
            Self::Unknown => "combat/death_generic",
        }
    }
}

/// Event for damage over time.
#[derive(Debug, Clone)]
pub struct DotEvent {
    /// Entity receiving damage.
    pub target: EntityId,
    /// Source of the DoT (if any).
    pub source: Option<EntityId>,
    /// Damage dealt this tick.
    pub damage: f32,
    /// Type of damage.
    pub damage_type: DamageCategory,
    /// Name of the effect causing the DoT.
    pub effect_name: String,
}

/// Event for status effects.
#[derive(Debug, Clone)]
pub struct StatusEvent {
    /// Entity affected.
    pub target: EntityId,
    /// Entity that applied the effect (if any).
    pub source: Option<EntityId>,
    /// Status effect type.
    pub status: StatusEffect,
    /// Duration remaining (for apply) or 0 (for remove).
    pub duration: f32,
    /// Stack count.
    pub stacks: u32,
}

/// Types of status effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    /// Burning (fire DoT).
    Burning,
    /// Poisoned (poison DoT).
    Poisoned,
    /// Frozen (slowed/immobilized).
    Frozen,
    /// Stunned (can't act).
    Stunned,
    /// Bleeding (physical DoT).
    Bleeding,
    /// Weakened (reduced damage).
    Weakened,
    /// Strengthened (increased damage).
    Strengthened,
    /// Regenerating (healing over time).
    Regenerating,
    /// Shielded (damage absorption).
    Shielded,
}

impl StatusEffect {
    /// Returns the sound name for applying this effect.
    #[must_use]
    pub const fn apply_sound(&self) -> &'static str {
        match self {
            Self::Burning => "status/burning_apply",
            Self::Poisoned => "status/poison_apply",
            Self::Frozen => "status/frozen_apply",
            Self::Stunned => "status/stun_apply",
            Self::Bleeding => "status/bleed_apply",
            Self::Weakened => "status/debuff_apply",
            Self::Strengthened => "status/buff_apply",
            Self::Regenerating => "status/heal_apply",
            Self::Shielded => "status/shield_apply",
        }
    }
}

/// Event for projectile spawning/impact.
#[derive(Debug, Clone)]
pub struct ProjectileEvent {
    /// Entity that fired the projectile.
    pub owner: EntityId,
    /// Projectile type.
    pub projectile_type: ProjectileType,
    /// Current position.
    pub position: (f32, f32),
    /// Velocity.
    pub velocity: (f32, f32),
    /// Target entity (if homing or on hit).
    pub target: Option<EntityId>,
}

/// Types of projectiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileType {
    /// Arrow from bow.
    Arrow,
    /// Bolt from crossbow.
    Bolt,
    /// Bullet from gun.
    Bullet,
    /// Magic projectile.
    MagicBolt,
    /// Thrown weapon.
    ThrownWeapon,
}

impl ProjectileType {
    /// Returns the spawn sound name.
    #[must_use]
    pub const fn spawn_sound(&self) -> &'static str {
        match self {
            Self::Arrow => "combat/arrow_fire",
            Self::Bolt => "combat/bolt_fire",
            Self::Bullet => "combat/bullet_fire",
            Self::MagicBolt => "combat/magic_fire",
            Self::ThrownWeapon => "combat/throw",
        }
    }

    /// Returns the impact sound name.
    #[must_use]
    pub const fn impact_sound(&self) -> &'static str {
        match self {
            Self::Arrow => "combat/arrow_impact",
            Self::Bolt => "combat/bolt_impact",
            Self::Bullet => "combat/bullet_impact",
            Self::MagicBolt => "combat/magic_impact",
            Self::ThrownWeapon => "combat/throw_impact",
        }
    }
}

/// Statistics tracked for combat.
#[derive(Debug, Clone, Default)]
pub struct CombatStats {
    /// Total attacks made.
    pub attacks_made: u64,
    /// Total hits landed.
    pub hits_landed: u64,
    /// Total damage dealt.
    pub damage_dealt: f64,
    /// Total damage taken.
    pub damage_taken: f64,
    /// Total damage blocked.
    pub damage_blocked: f64,
    /// Total kills.
    pub kills: u64,
    /// Total deaths.
    pub deaths: u64,
    /// Critical hits landed.
    pub critical_hits: u64,
    /// Perfect blocks performed.
    pub perfect_blocks: u64,
}

impl CombatStats {
    /// Creates new empty stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an attack.
    pub fn record_attack(&mut self) {
        self.attacks_made += 1;
    }

    /// Records a hit.
    pub fn record_hit(&mut self, damage: f32, critical: bool) {
        self.hits_landed += 1;
        self.damage_dealt += f64::from(damage);
        if critical {
            self.critical_hits += 1;
        }
    }

    /// Records damage taken.
    pub fn record_damage_taken(&mut self, damage: f32) {
        self.damage_taken += f64::from(damage);
    }

    /// Records a block.
    pub fn record_block(&mut self, damage: f32, perfect: bool) {
        self.damage_blocked += f64::from(damage);
        if perfect {
            self.perfect_blocks += 1;
        }
    }

    /// Records a kill.
    pub fn record_kill(&mut self) {
        self.kills += 1;
    }

    /// Records a death.
    pub fn record_death(&mut self) {
        self.deaths += 1;
    }

    /// Returns hit rate (hits / attacks).
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        if self.attacks_made == 0 {
            0.0
        } else {
            self.hits_landed as f64 / self.attacks_made as f64
        }
    }

    /// Returns kill/death ratio.
    #[must_use]
    pub fn kd_ratio(&self) -> f64 {
        if self.deaths == 0 {
            self.kills as f64
        } else {
            self.kills as f64 / self.deaths as f64
        }
    }
}

/// Handler for combat events.
pub struct CombatEventHandler {
    /// Pending events to process.
    event_queue: VecDeque<CombatEvent>,
    /// Combat statistics.
    stats: CombatStats,
    /// Pending loot drops (entity_id, position, items).
    pending_loot: Vec<(EntityId, (f32, f32), Vec<LootItem>)>,
}

/// A loot item to drop.
#[derive(Debug, Clone)]
pub struct LootItem {
    /// Item ID.
    pub item_id: u32,
    /// Quantity.
    pub quantity: u32,
    /// Drop chance (0.0 - 1.0).
    pub chance: f32,
}

impl Default for CombatEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatEventHandler {
    /// Creates a new combat event handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            stats: CombatStats::new(),
            pending_loot: Vec::new(),
        }
    }

    /// Queues a combat event for processing.
    pub fn queue_event(&mut self, event: CombatEvent) {
        self.event_queue.push_back(event);
    }

    /// Processes all pending events.
    pub fn process_events(
        &mut self,
        mut audio: Option<&mut AudioIntegration>,
    ) -> ProcessedCombatEvents {
        let mut result = ProcessedCombatEvents::default();

        while let Some(event) = self.event_queue.pop_front() {
            match event {
                CombatEvent::Attack(attack) => {
                    debug!(
                        "Attack: {:?} attacking {:?} with {:?}",
                        attack.attacker, attack.target, attack.attack_type
                    );
                    self.stats.record_attack();

                    // Play attack sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound =
                            SoundEvent::new(AudioCategory::Sfx, attack.attack_type.sound_name())
                                .at_position(attack.position.0, attack.position.1);
                        audio.queue_sound(sound);
                    }

                    result.attacks.push(attack);
                },
                CombatEvent::Hit(hit) => {
                    debug!(
                        "Hit: {:?} dealt {} damage to {:?}",
                        hit.attacker, hit.damage, hit.target
                    );
                    self.stats.record_hit(hit.damage, hit.critical);

                    // Play hit sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound =
                            SoundEvent::new(AudioCategory::Sfx, hit.damage_type.hit_sound())
                                .at_position(hit.position.0, hit.position.1);
                        audio.queue_sound(sound);

                        // Play critical hit sound if applicable
                        if hit.critical {
                            let crit_sound =
                                SoundEvent::new(AudioCategory::Sfx, "combat/critical_hit")
                                    .at_position(hit.position.0, hit.position.1);
                            audio.queue_sound(crit_sound);
                        }
                    }

                    result.hits.push(hit);
                },
                CombatEvent::Block(block) => {
                    debug!(
                        "Block: {:?} blocked {} damage from {:?}",
                        block.blocker, block.damage_blocked, block.attacker
                    );
                    self.stats
                        .record_block(block.damage_blocked, block.perfect_block);

                    // Play block sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound_name = if block.perfect_block {
                            "combat/perfect_block"
                        } else {
                            "combat/block"
                        };
                        let sound = SoundEvent::new(AudioCategory::Sfx, sound_name)
                            .at_position(block.position.0, block.position.1);
                        audio.queue_sound(sound);
                    }

                    result.blocks.push(block);
                },
                CombatEvent::Death(death) => {
                    info!("Death: {:?} killed by {:?}", death.entity, death.killer);
                    self.stats.record_death();

                    if death.killer.is_some() {
                        self.stats.record_kill();
                    }

                    // Play death sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound = SoundEvent::new(AudioCategory::Sfx, death.cause.sound_name())
                            .at_position(death.position.0, death.position.1);
                        audio.queue_sound(sound);
                    }

                    result.deaths.push(death);
                },
                CombatEvent::DamageOverTime(dot) => {
                    debug!(
                        "DoT: {:?} took {} {} damage from {}",
                        dot.target, dot.damage, dot.damage_type as u8, dot.effect_name
                    );
                    self.stats.record_damage_taken(dot.damage);
                    result.dots.push(dot);
                },
                CombatEvent::StatusApplied(status) => {
                    debug!(
                        "Status applied: {:?} on {:?} for {}s",
                        status.status, status.target, status.duration
                    );

                    // Play status apply sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound =
                            SoundEvent::new(AudioCategory::Sfx, status.status.apply_sound());
                        audio.queue_sound(sound);
                    }

                    result.statuses_applied.push(status);
                },
                CombatEvent::StatusRemoved(status) => {
                    debug!(
                        "Status removed: {:?} from {:?}",
                        status.status, status.target
                    );
                    result.statuses_removed.push(status);
                },
                CombatEvent::ProjectileSpawned(proj) => {
                    debug!(
                        "Projectile spawned: {:?} from {:?} at {:?}",
                        proj.projectile_type, proj.owner, proj.position
                    );

                    // Play projectile spawn sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound =
                            SoundEvent::new(AudioCategory::Sfx, proj.projectile_type.spawn_sound())
                                .at_position(proj.position.0, proj.position.1);
                        audio.queue_sound(sound);
                    }

                    result.projectiles_spawned.push(proj);
                },
                CombatEvent::ProjectileHit(proj) => {
                    debug!(
                        "Projectile hit: {:?} at {:?}",
                        proj.projectile_type, proj.position
                    );

                    // Play projectile impact sound
                    if let Some(audio) = audio.as_deref_mut() {
                        let sound = SoundEvent::new(
                            AudioCategory::Sfx,
                            proj.projectile_type.impact_sound(),
                        )
                        .at_position(proj.position.0, proj.position.1);
                        audio.queue_sound(sound);
                    }

                    result.projectiles_hit.push(proj);
                },
            }
        }

        // Move pending loot to result
        result.loot_drops = std::mem::take(&mut self.pending_loot);

        result
    }

    /// Registers a loot drop for a death event.
    pub fn register_loot_drop(
        &mut self,
        entity_id: EntityId,
        position: (f32, f32),
        items: Vec<LootItem>,
    ) {
        self.pending_loot.push((entity_id, position, items));
    }

    /// Returns combat statistics.
    #[must_use]
    pub fn stats(&self) -> &CombatStats {
        &self.stats
    }

    /// Returns mutable statistics (for loading saves).
    pub fn stats_mut(&mut self) -> &mut CombatStats {
        &mut self.stats
    }

    /// Creates an attack event.
    #[must_use]
    pub fn make_attack_event(
        attacker: EntityId,
        target: AttackTarget,
        attack_type: AttackCategory,
        position: (f32, f32),
        direction: (f32, f32),
    ) -> CombatEvent {
        CombatEvent::Attack(AttackEvent {
            attacker,
            target,
            weapon_id: None,
            attack_type,
            position,
            direction,
        })
    }

    /// Creates a hit event.
    #[must_use]
    pub fn make_hit_event(
        attacker: EntityId,
        target: EntityId,
        damage: f32,
        damage_type: DamageCategory,
        position: (f32, f32),
    ) -> CombatEvent {
        CombatEvent::Hit(HitEvent {
            attacker,
            target,
            damage,
            damage_type,
            position,
            knockback: None,
            critical: false,
            blocked: 0.0,
            resisted: 0.0,
        })
    }

    /// Creates a death event.
    #[must_use]
    pub fn make_death_event(
        entity: EntityId,
        killer: Option<EntityId>,
        position: (f32, f32),
        cause: DeathCause,
        experience: u32,
    ) -> CombatEvent {
        CombatEvent::Death(DeathEvent {
            entity,
            killer,
            position,
            cause,
            experience,
        })
    }
}

/// Result of processing combat events.
#[derive(Debug, Default)]
pub struct ProcessedCombatEvents {
    /// Attack events processed.
    pub attacks: Vec<AttackEvent>,
    /// Hit events processed.
    pub hits: Vec<HitEvent>,
    /// Block events processed.
    pub blocks: Vec<BlockEvent>,
    /// Death events processed.
    pub deaths: Vec<DeathEvent>,
    /// DoT events processed.
    pub dots: Vec<DotEvent>,
    /// Status effects applied.
    pub statuses_applied: Vec<StatusEvent>,
    /// Status effects removed.
    pub statuses_removed: Vec<StatusEvent>,
    /// Projectiles spawned.
    pub projectiles_spawned: Vec<ProjectileEvent>,
    /// Projectiles that hit.
    pub projectiles_hit: Vec<ProjectileEvent>,
    /// Loot drops to spawn (entity_id, position, items).
    pub loot_drops: Vec<(EntityId, (f32, f32), Vec<LootItem>)>,
}

impl ProcessedCombatEvents {
    /// Returns true if any events were processed.
    #[must_use]
    pub fn has_events(&self) -> bool {
        !self.attacks.is_empty()
            || !self.hits.is_empty()
            || !self.blocks.is_empty()
            || !self.deaths.is_empty()
            || !self.dots.is_empty()
    }

    /// Returns total damage dealt this frame.
    #[must_use]
    pub fn total_damage_dealt(&self) -> f32 {
        self.hits.iter().map(|h| h.damage).sum()
    }

    /// Returns total damage blocked this frame.
    #[must_use]
    pub fn total_damage_blocked(&self) -> f32 {
        self.blocks.iter().map(|b| b.damage_blocked).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_stats_new() {
        let stats = CombatStats::new();
        assert_eq!(stats.attacks_made, 0);
        assert_eq!(stats.hits_landed, 0);
        assert_eq!(stats.kills, 0);
    }

    #[test]
    fn test_combat_stats_record_attack() {
        let mut stats = CombatStats::new();
        stats.record_attack();
        stats.record_attack();
        assert_eq!(stats.attacks_made, 2);
    }

    #[test]
    fn test_combat_stats_record_hit() {
        let mut stats = CombatStats::new();
        stats.record_hit(50.0, false);
        stats.record_hit(75.0, true);
        assert_eq!(stats.hits_landed, 2);
        assert!((stats.damage_dealt - 125.0).abs() < 0.01);
        assert_eq!(stats.critical_hits, 1);
    }

    #[test]
    fn test_combat_stats_hit_rate() {
        let mut stats = CombatStats::new();
        stats.attacks_made = 10;
        stats.hits_landed = 7;
        assert!((stats.hit_rate() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_combat_stats_kd_ratio() {
        let mut stats = CombatStats::new();
        stats.kills = 10;
        stats.deaths = 2;
        assert!((stats.kd_ratio() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_combat_event_handler_queue() {
        let mut handler = CombatEventHandler::new();

        let event = CombatEventHandler::make_attack_event(
            EntityId::from_raw(1),
            AttackTarget::Entity(EntityId::from_raw(2)),
            AttackCategory::MeleeSwing,
            (0.0, 0.0),
            (1.0, 0.0),
        );
        handler.queue_event(event);

        let result = handler.process_events(None);
        assert_eq!(result.attacks.len(), 1);
        assert_eq!(handler.stats().attacks_made, 1);
    }

    #[test]
    fn test_combat_event_handler_hit() {
        let mut handler = CombatEventHandler::new();

        let event = CombatEventHandler::make_hit_event(
            EntityId::from_raw(1),
            EntityId::from_raw(2),
            50.0,
            DamageCategory::Physical,
            (10.0, 20.0),
        );
        handler.queue_event(event);

        let result = handler.process_events(None);
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.total_damage_dealt(), 50.0);
    }

    #[test]
    fn test_combat_event_handler_death() {
        let mut handler = CombatEventHandler::new();

        let event = CombatEventHandler::make_death_event(
            EntityId::from_raw(2),
            Some(EntityId::from_raw(1)),
            (10.0, 20.0),
            DeathCause::Combat,
            100,
        );
        handler.queue_event(event);

        let result = handler.process_events(None);
        assert_eq!(result.deaths.len(), 1);
        assert_eq!(handler.stats().deaths, 1);
        assert_eq!(handler.stats().kills, 1);
    }

    #[test]
    fn test_attack_category_sounds() {
        assert_eq!(
            AttackCategory::MeleeSwing.sound_name(),
            "combat/melee_swing"
        );
        assert_eq!(AttackCategory::RangedBow.sound_name(), "combat/bow_draw");
        assert_eq!(AttackCategory::MagicSpell.sound_name(), "combat/spell_cast");
    }

    #[test]
    fn test_damage_category_hit_sounds() {
        assert_eq!(DamageCategory::Physical.hit_sound(), "combat/hit_physical");
        assert_eq!(DamageCategory::Fire.hit_sound(), "combat/hit_fire");
    }

    #[test]
    fn test_projectile_type_sounds() {
        assert_eq!(ProjectileType::Arrow.spawn_sound(), "combat/arrow_fire");
        assert_eq!(ProjectileType::Arrow.impact_sound(), "combat/arrow_impact");
    }

    #[test]
    fn test_processed_events_has_events() {
        let empty = ProcessedCombatEvents::default();
        assert!(!empty.has_events());

        let mut with_hit = ProcessedCombatEvents::default();
        with_hit.hits.push(HitEvent {
            attacker: EntityId::from_raw(1),
            target: EntityId::from_raw(2),
            damage: 10.0,
            damage_type: DamageCategory::Physical,
            position: (0.0, 0.0),
            knockback: None,
            critical: false,
            blocked: 0.0,
            resisted: 0.0,
        });
        assert!(with_hit.has_events());
    }
}
