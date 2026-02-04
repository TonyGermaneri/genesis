# Iteration 12: Infra Agent - Combat Integration

## Objective
Wire combat events, load weapon data, persist combat state, and profile performance.

## Tasks

### 1. Combat Event System (combat_events.rs)
- OnAttack: trigger sounds, particles, hitbox
- OnHit: apply damage, knockback, effects
- OnDeath: drop loot, play animation
- OnBlock: reduce damage, play sound

### 2. Weapon Data Loading (weapon_loader.rs)
- Load weapons from assets/weapons/*.toml
- WeaponData: damage, speed, reach, type
- Validate weapon stats on load
- Hot-reload for development

### 3. Combat State Persistence (combat_save.rs)
- Save player HP, stamina
- Save active status effects
- Save equipped weapon state
- Load combat state on game load

### 4. Combat Profiling (combat_profile.rs)
- Hitbox collision check timing
- Projectile update timing
- Combat event processing time
- Entity combat update batching

### 5. Update Engine Integration
- Add combat to game loop
- Wire input to attack actions
- Connect UI to combat stats
