# Iteration 12: Tools Agent - Combat UI

## Objective
Create health/stamina bars, combat HUD, equipment stats, and debug overlays.

## Tasks

### 1. Health/Stamina Bars (ui/health_bars.rs)
- Player health bar (top-left)
- Stamina bar below health
- Target health bar (when locked on)
- Smooth interpolation on damage

### 2. Combat HUD (ui/combat_hud.rs)
- Combo counter display
- Damage taken flash indicator
- Low health warning effect
- Status effect icons

### 3. Equipment Stats Panel (ui/equipment_stats.rs)
- Weapon damage display
- Armor/defense values
- Stat comparison on hover
- DPS calculation display

### 4. Combat Debug Overlay (ui/combat_debug.rs)
- Hitbox/hurtbox visualization
- Damage log scrolling list
- Combat frame data display
- Invincibility frame indicator

### 5. Update ui/mod.rs
Export: health_bars, combat_hud, equipment_stats, combat_debug
