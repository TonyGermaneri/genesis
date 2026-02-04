# Iteration 12: Gameplay Agent - Combat Logic

## Objective
Implement combat stats, melee/ranged attacks, and damage calculations.

## Tasks

### 1. Combat Stats System (combat_stats.rs)
- CombatStats: hp, max_hp, attack, defense, crit_chance, dodge
- Stat modifiers from equipment
- Temporary buffs/debuffs
- Stamina for attacks

### 2. Melee Attack Logic (melee_combat.rs)
- Attack timing windows (windup, active, recovery)
- Combo chains with timing bonuses
- Stamina cost per swing
- Weapon reach and arc

### 3. Ranged Attack Logic (ranged_combat.rs)
- Bow draw time and power scaling
- Arrow velocity based on draw
- Throwing weapons (instant)
- Ammo consumption

### 4. Damage Calculation (damage_calc.rs)
- Base damage from weapon + stats
- Defense reduction formula
- Critical hit multiplier
- Elemental resistances
- Damage types (physical, fire, ice, poison)

### 5. Update lib.rs
Export: combat_stats, melee_combat, ranged_combat, damage_calc
