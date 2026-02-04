//! Damage number rendering system.
//!
//! Provides floating damage text with animations:
//! - Rising animation with fade-out
//! - Color coding (white=normal, yellow=crit, red=player damage)
//! - Batch rendering for multiple hits
//! - Text pooling for performance
//!
//! # Example
//!
//! ```
//! use genesis_kernel::damage_render::{
//!     DamageNumber, DamageType, DamageNumberManager,
//! };
//!
//! let mut manager = DamageNumberManager::new();
//!
//! // Spawn a critical hit at position (100.0, 200.0)
//! manager.spawn_damage(100.0, 200.0, 50, DamageType::Critical);
//!
//! // Update and get render data
//! manager.update(1.0 / 60.0);
//! let instances = manager.get_render_instances();
//! ```

/// Type of damage for color coding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum DamageType {
    /// Normal damage (white).
    #[default]
    Normal = 0,
    /// Critical hit (yellow/gold).
    Critical = 1,
    /// Player damage received (red).
    PlayerDamage = 2,
    /// Healing (green).
    Healing = 3,
    /// Poison/DoT (purple).
    Poison = 4,
    /// Fire damage (orange).
    Fire = 5,
    /// Ice damage (cyan).
    Ice = 6,
    /// Lightning damage (blue).
    Lightning = 7,
    /// Blocked/absorbed (gray).
    Blocked = 8,
    /// Miss/dodge (white, small).
    Miss = 9,
    /// Experience gain (gold).
    Experience = 10,
}

impl DamageType {
    /// Convert from u8.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Critical),
            2 => Some(Self::PlayerDamage),
            3 => Some(Self::Healing),
            4 => Some(Self::Poison),
            5 => Some(Self::Fire),
            6 => Some(Self::Ice),
            7 => Some(Self::Lightning),
            8 => Some(Self::Blocked),
            9 => Some(Self::Miss),
            10 => Some(Self::Experience),
            _ => None,
        }
    }

    /// Get the base color for this damage type.
    #[must_use]
    pub const fn color(&self) -> [f32; 4] {
        match self {
            Self::Normal => [1.0, 1.0, 1.0, 1.0],       // White
            Self::Critical => [1.0, 0.85, 0.0, 1.0],    // Gold
            Self::PlayerDamage => [1.0, 0.2, 0.2, 1.0], // Red
            Self::Healing => [0.2, 1.0, 0.2, 1.0],      // Green
            Self::Poison => [0.7, 0.2, 0.9, 1.0],       // Purple
            Self::Fire => [1.0, 0.5, 0.0, 1.0],         // Orange
            Self::Ice => [0.4, 0.9, 1.0, 1.0],          // Cyan
            Self::Lightning => [0.4, 0.6, 1.0, 1.0],    // Blue
            Self::Blocked => [0.6, 0.6, 0.6, 1.0],      // Gray
            Self::Miss => [0.8, 0.8, 0.8, 1.0],         // Light gray
            Self::Experience => [1.0, 0.9, 0.4, 1.0],   // Bright gold
        }
    }

    /// Get the scale multiplier for this damage type.
    #[must_use]
    pub const fn scale(&self) -> f32 {
        match self {
            Self::Critical => 1.5,
            Self::Miss => 0.7,
            Self::Blocked => 0.8,
            _ => 1.0,
        }
    }

    /// Check if this type should have outline.
    #[must_use]
    pub const fn has_outline(&self) -> bool {
        matches!(self, Self::Critical | Self::PlayerDamage | Self::Healing)
    }
}

/// Animation style for damage numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AnimationStyle {
    /// Rise up and fade.
    #[default]
    Rise = 0,
    /// Pop out and shrink.
    Pop = 1,
    /// Arc to the side.
    Arc = 2,
    /// Shake and fade.
    Shake = 3,
    /// Bounce up and settle.
    Bounce = 4,
}

impl AnimationStyle {
    /// Convert from u8.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Rise),
            1 => Some(Self::Pop),
            2 => Some(Self::Arc),
            3 => Some(Self::Shake),
            4 => Some(Self::Bounce),
            _ => None,
        }
    }
}

/// A floating damage number.
#[derive(Debug, Clone)]
pub struct DamageNumber {
    /// Display value (can be negative for healing).
    pub value: i32,
    /// Current position.
    pub position: (f32, f32),
    /// Velocity for animation.
    pub velocity: (f32, f32),
    /// Damage type (determines color).
    pub damage_type: DamageType,
    /// Animation style.
    pub animation: AnimationStyle,
    /// Current lifetime.
    pub lifetime: f32,
    /// Maximum lifetime.
    pub max_lifetime: f32,
    /// Current scale.
    pub scale: f32,
    /// Target scale (for pop animation).
    pub target_scale: f32,
    /// Current alpha.
    pub alpha: f32,
    /// Current rotation.
    pub rotation: f32,
    /// Custom text (if not using value).
    pub custom_text: Option<String>,
    /// Whether this number is active.
    pub active: bool,
    /// Random offset for variation.
    random_offset: f32,
}

impl Default for DamageNumber {
    fn default() -> Self {
        Self {
            value: 0,
            position: (0.0, 0.0),
            velocity: (0.0, -80.0), // Rise upward
            damage_type: DamageType::Normal,
            animation: AnimationStyle::Rise,
            lifetime: 0.0,
            max_lifetime: 1.5,
            scale: 1.0,
            target_scale: 1.0,
            alpha: 1.0,
            rotation: 0.0,
            custom_text: None,
            active: true,
            random_offset: 0.0,
        }
    }
}

impl DamageNumber {
    /// Create a new damage number.
    #[must_use]
    pub fn new(value: i32, position: (f32, f32), damage_type: DamageType) -> Self {
        // Add slight random horizontal offset
        let random_offset = (position.0 * 17.0 + position.1 * 31.0).sin() * 10.0;
        Self {
            value,
            position: (position.0 + random_offset, position.1),
            damage_type,
            scale: damage_type.scale(),
            target_scale: damage_type.scale(),
            random_offset,
            ..Default::default()
        }
    }

    /// Create a miss indicator.
    #[must_use]
    pub fn miss(position: (f32, f32)) -> Self {
        Self {
            value: 0,
            position,
            damage_type: DamageType::Miss,
            custom_text: Some("MISS".to_string()),
            ..Default::default()
        }
    }

    /// Create a blocked indicator.
    #[must_use]
    pub fn blocked(value: i32, position: (f32, f32)) -> Self {
        Self {
            value,
            position,
            damage_type: DamageType::Blocked,
            custom_text: Some(format!("({value})")),
            ..Default::default()
        }
    }

    /// Create an experience gain indicator.
    #[must_use]
    pub fn experience(value: i32, position: (f32, f32)) -> Self {
        Self {
            value,
            position,
            damage_type: DamageType::Experience,
            custom_text: Some(format!("+{value} XP")),
            max_lifetime: 2.0,
            ..Default::default()
        }
    }

    /// Set animation style.
    #[must_use]
    pub const fn with_animation(mut self, animation: AnimationStyle) -> Self {
        self.animation = animation;
        self
    }

    /// Set lifetime.
    #[must_use]
    pub const fn with_lifetime(mut self, lifetime: f32) -> Self {
        self.max_lifetime = lifetime;
        self
    }

    /// Set custom text.
    #[must_use]
    pub fn with_text(mut self, text: String) -> Self {
        self.custom_text = Some(text);
        self
    }

    /// Get display text.
    #[must_use]
    pub fn text(&self) -> String {
        if let Some(ref custom) = self.custom_text {
            custom.clone()
        } else if self.value < 0 {
            format!("+{}", -self.value) // Healing
        } else {
            self.value.to_string()
        }
    }

    /// Get current color with alpha.
    #[must_use]
    pub fn color(&self) -> [f32; 4] {
        let mut c = self.damage_type.color();
        c[3] *= self.alpha;
        c
    }

    /// Check if still active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Get progress (0-1).
    #[must_use]
    pub fn progress(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }

    /// Update animation.
    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }

        self.lifetime += dt;

        if self.lifetime >= self.max_lifetime {
            self.active = false;
            return;
        }

        let progress = self.progress();

        match self.animation {
            AnimationStyle::Rise => {
                // Simple rise with deceleration
                let decel = 1.0 - progress;
                self.position.0 += self.velocity.0 * dt * decel;
                self.position.1 += self.velocity.1 * dt * decel;
                // Fade out in last 30%
                if progress > 0.7 {
                    self.alpha = 1.0 - ((progress - 0.7) / 0.3);
                }
            },
            AnimationStyle::Pop => {
                // Start big, shrink to target
                if progress < 0.2 {
                    self.scale = self.target_scale * (1.5 - progress * 2.5);
                } else {
                    self.scale = self.target_scale;
                }
                // Rise slightly
                self.position.1 += self.velocity.1 * dt * 0.3;
                // Fade out
                if progress > 0.5 {
                    self.alpha = 1.0 - ((progress - 0.5) / 0.5);
                }
            },
            AnimationStyle::Arc => {
                // Arc to the side
                let dir = if self.random_offset > 0.0 { 1.0 } else { -1.0 };
                self.position.0 += dir * 50.0 * dt * (1.0 - progress);
                self.position.1 += self.velocity.1 * dt * (1.0 - progress * 0.5);
                // Fade out
                if progress > 0.6 {
                    self.alpha = 1.0 - ((progress - 0.6) / 0.4);
                }
            },
            AnimationStyle::Shake => {
                // Shake horizontally
                let shake = (self.lifetime * 30.0).sin() * (1.0 - progress) * 5.0;
                self.position.0 += shake * dt * 10.0;
                // Slow rise
                self.position.1 += self.velocity.1 * dt * 0.5;
                // Fade out
                if progress > 0.5 {
                    self.alpha = 1.0 - ((progress - 0.5) / 0.5);
                }
            },
            AnimationStyle::Bounce => {
                // Bounce physics
                let bounce_height = 30.0;
                let bounces = 3.0;
                let decay = (1.0 - progress).powi(2);
                let y_offset =
                    (progress * std::f32::consts::PI * bounces).sin().abs() * bounce_height * decay;
                self.position.1 -= y_offset * dt * 5.0;
                // Fade out
                if progress > 0.7 {
                    self.alpha = 1.0 - ((progress - 0.7) / 0.3);
                }
            },
        }
    }
}

/// GPU-friendly damage number instance for rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DamageNumberInstance {
    /// Position (x, y).
    pub position: [f32; 2],
    /// Scale.
    pub scale: f32,
    /// Rotation.
    pub rotation: f32,
    /// Color with alpha.
    pub color: [f32; 4],
    /// Glyph offset in atlas (start).
    pub glyph_start: u32,
    /// Number of glyphs.
    pub glyph_count: u32,
    /// Outline color.
    pub outline_color: [f32; 4],
}

impl DamageNumberInstance {
    /// Size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

/// Manager for damage numbers.
#[derive(Debug, Default)]
pub struct DamageNumberManager {
    numbers: Vec<DamageNumber>,
    pool: Vec<DamageNumber>,
    max_numbers: usize,
    combo_window: f32,
    combo_position: Option<(f32, f32)>,
    combo_damage: i32,
    combo_timer: f32,
}

impl DamageNumberManager {
    /// Create a new damage number manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            numbers: Vec::with_capacity(64),
            pool: Vec::with_capacity(32),
            max_numbers: 100,
            combo_window: 0.1, // 100ms to combine hits
            combo_position: None,
            combo_damage: 0,
            combo_timer: 0.0,
        }
    }

    /// Set maximum number of simultaneous damage numbers.
    pub fn set_max_numbers(&mut self, max: usize) {
        self.max_numbers = max;
    }

    /// Set combo window (time to combine rapid hits).
    pub fn set_combo_window(&mut self, window: f32) {
        self.combo_window = window;
    }

    /// Spawn a damage number.
    pub fn spawn(&mut self, number: DamageNumber) {
        if self.numbers.len() >= self.max_numbers {
            // Remove oldest
            if let Some(oldest) = self.numbers.iter().position(|n| !n.is_active()) {
                self.numbers.remove(oldest);
            } else {
                self.numbers.remove(0);
            }
        }
        self.numbers.push(number);
    }

    /// Spawn damage at a position with automatic combo combining.
    pub fn spawn_damage(&mut self, x: f32, y: f32, value: i32, damage_type: DamageType) {
        // Check for combo
        if let Some(combo_pos) = self.combo_position {
            let dx = x - combo_pos.0;
            let dy = y - combo_pos.1;
            if dx * dx + dy * dy < 900.0 && self.combo_timer > 0.0 {
                // Same target, combine
                self.combo_damage += value;
                self.combo_timer = self.combo_window;
                return;
            }
        }

        // Flush previous combo if any
        self.flush_combo();

        // Start new combo
        self.combo_position = Some((x, y));
        self.combo_damage = value;
        self.combo_timer = self.combo_window;

        // For crits and special types, spawn immediately
        if damage_type != DamageType::Normal {
            let number = DamageNumber::new(value, (x, y), damage_type);
            self.spawn(number);
            self.combo_position = None;
            self.combo_damage = 0;
            self.combo_timer = 0.0;
        }
    }

    /// Flush accumulated combo damage.
    fn flush_combo(&mut self) {
        if self.combo_damage > 0 {
            if let Some(pos) = self.combo_position {
                let number = DamageNumber::new(self.combo_damage, pos, DamageType::Normal);
                self.spawn(number);
            }
        }
        self.combo_position = None;
        self.combo_damage = 0;
        self.combo_timer = 0.0;
    }

    /// Spawn a critical hit.
    pub fn spawn_crit(&mut self, x: f32, y: f32, value: i32) {
        let number = DamageNumber::new(value, (x, y), DamageType::Critical)
            .with_animation(AnimationStyle::Pop);
        self.spawn(number);
    }

    /// Spawn healing number.
    pub fn spawn_heal(&mut self, x: f32, y: f32, value: i32) {
        let number = DamageNumber::new(-value, (x, y), DamageType::Healing);
        self.spawn(number);
    }

    /// Spawn a miss indicator.
    pub fn spawn_miss(&mut self, x: f32, y: f32) {
        self.spawn(DamageNumber::miss((x, y)));
    }

    /// Spawn experience gain.
    pub fn spawn_experience(&mut self, x: f32, y: f32, value: i32) {
        self.spawn(DamageNumber::experience(value, (x, y)));
    }

    /// Update all damage numbers.
    pub fn update(&mut self, dt: f32) {
        // Update combo timer
        if self.combo_timer > 0.0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 {
                self.flush_combo();
            }
        }

        // Update existing numbers
        for number in &mut self.numbers {
            number.update(dt);
        }

        // Move inactive to pool and remove
        let mut i = 0;
        while i < self.numbers.len() {
            if self.numbers[i].is_active() {
                i += 1;
            } else {
                let mut removed = self.numbers.swap_remove(i);
                removed.active = false;
                if self.pool.len() < 32 {
                    self.pool.push(removed);
                }
            }
        }
    }

    /// Get render instances.
    #[must_use]
    pub fn get_render_instances(&self) -> Vec<(String, DamageNumberInstance)> {
        self.numbers
            .iter()
            .filter(|n| n.is_active())
            .map(|n| {
                let instance = DamageNumberInstance {
                    position: [n.position.0, n.position.1],
                    scale: n.scale,
                    rotation: n.rotation,
                    color: n.color(),
                    glyph_start: 0,
                    glyph_count: 0,
                    outline_color: if n.damage_type.has_outline() {
                        [0.0, 0.0, 0.0, n.alpha * 0.8]
                    } else {
                        [0.0, 0.0, 0.0, 0.0]
                    },
                };
                (n.text(), instance)
            })
            .collect()
    }

    /// Get number of active damage numbers.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.numbers.iter().filter(|n| n.is_active()).count()
    }

    /// Clear all damage numbers.
    pub fn clear(&mut self) {
        self.numbers.clear();
        self.combo_position = None;
        self.combo_damage = 0;
        self.combo_timer = 0.0;
    }
}

/// Batch renderer for damage numbers.
#[derive(Debug, Default)]
pub struct DamageNumberBatch {
    /// Vertex data for all numbers.
    vertices: Vec<DamageNumberVertex>,
    /// Number of vertices.
    vertex_count: usize,
}

/// Vertex for damage number rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DamageNumberVertex {
    /// Position.
    pub position: [f32; 2],
    /// UV coordinates.
    pub uv: [f32; 2],
    /// Color.
    pub color: [f32; 4],
}

impl DamageNumberVertex {
    /// Size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

impl DamageNumberBatch {
    /// Create a new batch.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            vertex_count: 0,
        }
    }

    /// Clear the batch.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.vertex_count = 0;
    }

    /// Add a damage number to the batch.
    pub fn add(
        &mut self,
        text: &str,
        position: (f32, f32),
        scale: f32,
        color: [f32; 4],
        glyph_lookup: impl Fn(char) -> Option<(f32, f32, f32, f32)>, // (u, v, w, h)
    ) {
        let char_width = 12.0 * scale;
        let char_height = 16.0 * scale;
        let total_width = text.len() as f32 * char_width;
        let start_x = position.0 - total_width * 0.5;

        for (i, ch) in text.chars().enumerate() {
            if let Some((u, v, w, h)) = glyph_lookup(ch) {
                let x = start_x + i as f32 * char_width;
                let y = position.1;

                // Two triangles per character
                let verts = [
                    // Triangle 1
                    DamageNumberVertex {
                        position: [x, y],
                        uv: [u, v],
                        color,
                    },
                    DamageNumberVertex {
                        position: [x + char_width, y],
                        uv: [u + w, v],
                        color,
                    },
                    DamageNumberVertex {
                        position: [x, y + char_height],
                        uv: [u, v + h],
                        color,
                    },
                    // Triangle 2
                    DamageNumberVertex {
                        position: [x + char_width, y],
                        uv: [u + w, v],
                        color,
                    },
                    DamageNumberVertex {
                        position: [x + char_width, y + char_height],
                        uv: [u + w, v + h],
                        color,
                    },
                    DamageNumberVertex {
                        position: [x, y + char_height],
                        uv: [u, v + h],
                        color,
                    },
                ];
                self.vertices.extend_from_slice(&verts);
                self.vertex_count += 6;
            }
        }
    }

    /// Get vertex data.
    #[must_use]
    pub fn vertices(&self) -> &[DamageNumberVertex] {
        &self.vertices
    }

    /// Get vertex count.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_type_colors() {
        let normal_color = DamageType::Normal.color();
        assert_eq!(normal_color, [1.0, 1.0, 1.0, 1.0]);

        let crit_color = DamageType::Critical.color();
        assert!((crit_color[0] - 1.0).abs() < 0.01);
        assert!((crit_color[1] - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_damage_type_scale() {
        assert_eq!(DamageType::Normal.scale(), 1.0);
        assert_eq!(DamageType::Critical.scale(), 1.5);
        assert_eq!(DamageType::Miss.scale(), 0.7);
    }

    #[test]
    fn test_damage_number_creation() {
        let num = DamageNumber::new(100, (50.0, 100.0), DamageType::Normal);
        assert_eq!(num.value, 100);
        assert!(num.is_active());
        assert_eq!(num.text(), "100");
    }

    #[test]
    fn test_damage_number_miss() {
        let miss = DamageNumber::miss((0.0, 0.0));
        assert_eq!(miss.text(), "MISS");
        assert_eq!(miss.damage_type, DamageType::Miss);
    }

    #[test]
    fn test_damage_number_experience() {
        let xp = DamageNumber::experience(50, (0.0, 0.0));
        assert_eq!(xp.text(), "+50 XP");
    }

    #[test]
    fn test_damage_number_update() {
        let mut num = DamageNumber::new(100, (0.0, 0.0), DamageType::Normal).with_lifetime(1.0);

        num.update(0.5);
        assert!(num.is_active());
        assert!(num.position.1 < 0.0); // Should have risen

        num.update(0.6);
        assert!(!num.is_active()); // Should have expired
    }

    #[test]
    fn test_damage_number_alpha_fade() {
        let mut num = DamageNumber::new(100, (0.0, 0.0), DamageType::Normal).with_lifetime(1.0);

        num.update(0.6);
        assert!(num.alpha > 0.9); // Not yet fading

        num.update(0.2);
        assert!(num.alpha < 1.0); // Should be fading
    }

    #[test]
    fn test_damage_number_manager() {
        let mut manager = DamageNumberManager::new();
        manager.spawn(DamageNumber::new(100, (0.0, 0.0), DamageType::Normal));

        assert_eq!(manager.active_count(), 1);

        manager.update(0.1);
        assert_eq!(manager.active_count(), 1);
    }

    #[test]
    fn test_damage_number_manager_combo() {
        let mut manager = DamageNumberManager::new();

        // Rapid hits at same position should combine
        manager.spawn_damage(100.0, 100.0, 10, DamageType::Normal);
        manager.spawn_damage(100.0, 100.0, 10, DamageType::Normal);
        manager.spawn_damage(100.0, 100.0, 10, DamageType::Normal);

        // Flush by updating past combo window
        manager.update(0.2);

        // Should have combined into one number with value 30
        assert_eq!(manager.active_count(), 1);
    }

    #[test]
    fn test_damage_number_manager_crit() {
        let mut manager = DamageNumberManager::new();
        manager.spawn_crit(0.0, 0.0, 200);

        assert_eq!(manager.active_count(), 1);
    }

    #[test]
    fn test_damage_number_instance_size() {
        assert_eq!(DamageNumberInstance::SIZE, 56);
    }

    #[test]
    fn test_damage_number_batch() {
        let mut batch = DamageNumberBatch::new();

        // Mock glyph lookup
        let lookup = |_: char| Some((0.0, 0.0, 0.1, 0.1));

        batch.add("123", (0.0, 0.0), 1.0, [1.0, 1.0, 1.0, 1.0], lookup);

        assert_eq!(batch.vertex_count(), 18); // 3 chars * 6 verts each
    }

    #[test]
    fn test_animation_style_from_u8() {
        assert_eq!(AnimationStyle::from_u8(0), Some(AnimationStyle::Rise));
        assert_eq!(AnimationStyle::from_u8(1), Some(AnimationStyle::Pop));
        assert_eq!(AnimationStyle::from_u8(99), None);
    }

    #[test]
    fn test_damage_type_from_u8() {
        assert_eq!(DamageType::from_u8(0), Some(DamageType::Normal));
        assert_eq!(DamageType::from_u8(1), Some(DamageType::Critical));
        assert_eq!(DamageType::from_u8(99), None);
    }

    #[test]
    fn test_damage_number_healing_text() {
        let heal = DamageNumber::new(-50, (0.0, 0.0), DamageType::Healing);
        assert_eq!(heal.text(), "+50");
    }

    #[test]
    fn test_damage_number_vertex_size() {
        assert_eq!(DamageNumberVertex::SIZE, 32);
    }

    #[test]
    fn test_damage_number_pop_animation() {
        let mut num = DamageNumber::new(100, (0.0, 0.0), DamageType::Critical)
            .with_animation(AnimationStyle::Pop)
            .with_lifetime(1.0);

        let initial_scale = num.scale;
        num.update(0.05);
        // Pop animation starts bigger
        assert!(num.scale > initial_scale * 0.9 || num.scale > 1.0);
    }
}
