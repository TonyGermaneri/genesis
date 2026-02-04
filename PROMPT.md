# Iteration 12: Kernel Agent - Combat Infrastructure

## Objective
Implement hitbox/hurtbox collision, projectile physics, damage rendering, and combat effects.

## Tasks

### 1. Hitbox/Hurtbox Collision (combat_collision.rs)
- Hitbox struct with shape, offset, active frames
- Hurtbox for damageable entities
- Overlap detection between hitbox and hurtbox
- Layer masks (player vs enemy, friendly fire)

### 2. Projectile Physics (projectile.rs)
- Projectile struct: position, velocity, gravity, lifetime
- Arc trajectory for arrows
- Straight trajectory for spells
- Collision with terrain and entities

### 3. Damage Number Rendering (damage_render.rs)
- FloatingText: position, text, color, lifetime
- Rising animation with fade-out
- Color coding (white=normal, yellow=crit, red=player damage)
- Batch rendering for multiple hits

### 4. Combat Particle Effects (combat_particles.rs)
- HitSpark: position, direction, intensity
- BloodSplatter: for creature hits
- ImpactDust: for terrain hits
- Particle pooling for performance

### 5. Update lib.rs
Export: combat_collision, projectile, damage_render, combat_particles
