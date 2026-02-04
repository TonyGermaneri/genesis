//! Menu backdrop rendering infrastructure.
//!
//! Provides animated and static background rendering for menus.
//! Features include:
//! - Slowly moving clouds/particles
//! - Parallax effect with multiple layers
//! - Day/night cycle ambient effect
//! - Static fallback image support

use bytemuck::{Pod, Zeroable};

/// Default parallax speed for the background layer.
pub const DEFAULT_PARALLAX_SPEED: f32 = 0.02;

/// Default parallax speed for the midground layer.
pub const DEFAULT_MIDGROUND_SPEED: f32 = 0.05;

/// Default parallax speed for the foreground layer.
pub const DEFAULT_FOREGROUND_SPEED: f32 = 0.1;

/// Duration of a full day/night cycle in seconds.
pub const DAY_NIGHT_CYCLE_DURATION: f32 = 120.0;

/// Maximum number of cloud particles.
pub const MAX_CLOUD_PARTICLES: usize = 64;

/// Maximum number of ambient particles.
pub const MAX_AMBIENT_PARTICLES: usize = 128;

/// A single cloud particle in the backdrop.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct CloudParticle {
    /// X position (normalized 0.0-1.0).
    pub x: f32,
    /// Y position (normalized 0.0-1.0).
    pub y: f32,
    /// Horizontal velocity.
    pub velocity_x: f32,
    /// Scale factor.
    pub scale: f32,
    /// Opacity (0.0-1.0).
    pub opacity: f32,
    /// Cloud type variant (0-3).
    pub variant: u32,
    /// Depth layer (0=far, 1=mid, 2=near).
    pub layer: u32,
    /// Padding for alignment.
    _pad: u32,
}

impl Default for CloudParticle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.5,
            velocity_x: DEFAULT_PARALLAX_SPEED,
            scale: 1.0,
            opacity: 0.8,
            variant: 0,
            layer: 0,
            _pad: 0,
        }
    }
}

impl CloudParticle {
    /// Creates a new cloud particle at the given position.
    #[must_use]
    pub fn new(x: f32, y: f32, layer: u32) -> Self {
        let speed = match layer {
            0 => DEFAULT_PARALLAX_SPEED,
            1 => DEFAULT_MIDGROUND_SPEED,
            _ => DEFAULT_FOREGROUND_SPEED,
        };
        Self {
            x,
            y,
            velocity_x: speed,
            scale: 1.0 - (layer as f32 * 0.2),
            opacity: 0.6 + (layer as f32 * 0.1),
            variant: 0,
            layer,
            _pad: 0,
        }
    }

    /// Updates the particle position based on delta time.
    pub fn update(&mut self, dt: f32) {
        self.x += self.velocity_x * dt;
        // Wrap around when off-screen
        if self.x > 1.5 {
            self.x = -0.5;
        }
    }

    /// Checks if the particle is visible on screen.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.x > -0.5 && self.x < 1.5 && self.opacity > 0.0
    }
}

/// An ambient particle (dust, pollen, fireflies, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct AmbientParticle {
    /// X position (normalized 0.0-1.0).
    pub x: f32,
    /// Y position (normalized 0.0-1.0).
    pub y: f32,
    /// Horizontal velocity.
    pub velocity_x: f32,
    /// Vertical velocity.
    pub velocity_y: f32,
    /// Particle size in pixels.
    pub size: f32,
    /// Opacity (0.0-1.0).
    pub opacity: f32,
    /// Color tint (packed RGBA).
    pub color: u32,
    /// Lifetime remaining in seconds.
    pub lifetime: f32,
}

impl Default for AmbientParticle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            velocity_x: 0.01,
            velocity_y: 0.005,
            size: 2.0,
            opacity: 0.5,
            color: 0xFFFF_FFFF,
            lifetime: 5.0,
        }
    }
}

impl AmbientParticle {
    /// Creates a new ambient particle.
    #[must_use]
    pub fn new(x: f32, y: f32, size: f32, color: u32) -> Self {
        Self {
            x,
            y,
            velocity_x: 0.01,
            velocity_y: 0.005,
            size,
            opacity: 0.5,
            color,
            lifetime: 5.0,
        }
    }

    /// Updates the particle position and lifetime.
    pub fn update(&mut self, dt: f32) {
        self.x += self.velocity_x * dt;
        self.y += self.velocity_y * dt;
        self.lifetime -= dt;
    }

    /// Checks if the particle is still alive.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }
}

/// Time of day phases for ambient effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeOfDay {
    /// Dawn (warm orange glow).
    Dawn,
    /// Daytime (bright, clear).
    #[default]
    Day,
    /// Dusk (warm red/purple).
    Dusk,
    /// Night (cool blue, stars).
    Night,
}

impl TimeOfDay {
    /// Gets the time of day from a cycle position (0.0-1.0).
    #[must_use]
    pub fn from_cycle(cycle: f32) -> Self {
        let normalized = cycle.fract();
        match normalized {
            x if x < 0.2 => Self::Dawn,
            x if x < 0.45 => Self::Day,
            x if x < 0.55 => Self::Dusk,
            _ => Self::Night,
        }
    }

    /// Gets the ambient color for this time of day (packed RGBA).
    #[must_use]
    pub fn ambient_color(&self) -> u32 {
        match self {
            Self::Dawn => 0xFF99_66FF,  // Warm orange
            Self::Day => 0xFFFF_FFFF,   // White
            Self::Dusk => 0xFF66_66CC,  // Purple-red
            Self::Night => 0xFF44_3366, // Dark blue
        }
    }

    /// Gets the sky gradient top color.
    #[must_use]
    pub fn sky_top_color(&self) -> u32 {
        match self {
            Self::Dawn => 0xFF44_88BB,  // Light blue with orange
            Self::Day => 0xFF33_99DD,   // Sky blue
            Self::Dusk => 0xFF22_4477,  // Deep blue
            Self::Night => 0xFF11_1133, // Very dark blue
        }
    }

    /// Gets the sky gradient bottom color.
    #[must_use]
    pub fn sky_bottom_color(&self) -> u32 {
        match self {
            Self::Dawn => 0xFFBB_7744,  // Orange
            Self::Day => 0xFF88_CCEE,   // Light blue
            Self::Dusk => 0xFF99_3355,  // Purple-red
            Self::Night => 0xFF22_2244, // Dark blue
        }
    }
}

/// Day/night cycle state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayNightCycle {
    /// Current cycle position (0.0-1.0).
    pub cycle_position: f32,
    /// Cycle duration in seconds.
    pub cycle_duration: f32,
    /// Whether the cycle is paused.
    pub paused: bool,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            cycle_position: 0.3, // Start at mid-morning
            cycle_duration: DAY_NIGHT_CYCLE_DURATION,
            paused: false,
        }
    }
}

impl DayNightCycle {
    /// Creates a new day/night cycle.
    #[must_use]
    pub fn new(duration: f32) -> Self {
        Self {
            cycle_position: 0.3,
            cycle_duration: duration,
            paused: false,
        }
    }

    /// Updates the cycle based on delta time.
    pub fn update(&mut self, dt: f32) {
        if !self.paused {
            self.cycle_position += dt / self.cycle_duration;
            if self.cycle_position >= 1.0 {
                self.cycle_position -= 1.0;
            }
        }
    }

    /// Gets the current time of day.
    #[must_use]
    pub fn time_of_day(&self) -> TimeOfDay {
        TimeOfDay::from_cycle(self.cycle_position)
    }

    /// Sets a specific time of day.
    pub fn set_time(&mut self, time: TimeOfDay) {
        self.cycle_position = match time {
            TimeOfDay::Dawn => 0.1,
            TimeOfDay::Day => 0.3,
            TimeOfDay::Dusk => 0.5,
            TimeOfDay::Night => 0.75,
        };
    }

    /// Pauses the cycle.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the cycle.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Interpolates between current and next time period colors.
    #[must_use]
    pub fn interpolated_ambient_color(&self) -> u32 {
        let current = self.time_of_day();
        current.ambient_color()
    }
}

/// Parallax layer for background rendering.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct ParallaxLayer {
    /// Horizontal scroll offset.
    pub scroll_x: f32,
    /// Vertical scroll offset.
    pub scroll_y: f32,
    /// Scroll speed multiplier.
    pub speed: f32,
    /// Layer opacity.
    pub opacity: f32,
    /// Texture index for this layer.
    pub texture_index: u32,
    /// Scale factor.
    pub scale: f32,
    /// Padding for alignment.
    _pad: [u32; 2],
}

impl Default for ParallaxLayer {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            speed: 1.0,
            opacity: 1.0,
            texture_index: 0,
            scale: 1.0,
            _pad: [0; 2],
        }
    }
}

impl ParallaxLayer {
    /// Creates a new parallax layer with the given speed.
    #[must_use]
    pub fn new(speed: f32, texture_index: u32) -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            speed,
            opacity: 1.0,
            texture_index,
            scale: 1.0,
            _pad: [0; 2],
        }
    }

    /// Updates the layer scroll based on delta time.
    pub fn update(&mut self, dt: f32, base_speed: f32) {
        self.scroll_x += base_speed * self.speed * dt;
        // Wrap scroll position
        if self.scroll_x > 1.0 {
            self.scroll_x -= 1.0;
        }
    }
}

/// Static fallback image descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub struct StaticBackdrop {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Whether the image is loaded.
    pub loaded: bool,
    /// Texture index.
    pub texture_index: u32,
}


impl StaticBackdrop {
    /// Creates a new static backdrop descriptor.
    #[must_use]
    pub fn new(width: u32, height: u32, texture_index: u32) -> Self {
        Self {
            width,
            height,
            loaded: true,
            texture_index,
        }
    }

    /// Checks if the backdrop is ready to use.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.loaded && self.width > 0 && self.height > 0
    }
}

/// Menu backdrop configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackdropMode {
    /// Animated backdrop with particles and parallax.
    #[default]
    Animated,
    /// Static image backdrop.
    Static,
    /// Solid color backdrop.
    SolidColor,
}

/// GPU-ready uniform buffer for backdrop rendering.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct BackdropUniforms {
    /// Screen width.
    pub screen_width: f32,
    /// Screen height.
    pub screen_height: f32,
    /// Current time for animations.
    pub time: f32,
    /// Day/night cycle position.
    pub cycle_position: f32,
    /// Sky top color (packed).
    pub sky_top_color: u32,
    /// Sky bottom color (packed).
    pub sky_bottom_color: u32,
    /// Ambient color (packed).
    pub ambient_color: u32,
    /// Backdrop mode (0=animated, 1=static, 2=solid).
    pub mode: u32,
    /// Base scroll speed.
    pub base_scroll_speed: f32,
    /// Cloud opacity multiplier.
    pub cloud_opacity: f32,
    /// Particle opacity multiplier.
    pub particle_opacity: f32,
    /// Padding for 16-byte alignment.
    _pad: u32,
}

impl Default for BackdropUniforms {
    fn default() -> Self {
        let time_of_day = TimeOfDay::Day;
        Self {
            screen_width: 1920.0,
            screen_height: 1080.0,
            time: 0.0,
            cycle_position: 0.3,
            sky_top_color: time_of_day.sky_top_color(),
            sky_bottom_color: time_of_day.sky_bottom_color(),
            ambient_color: time_of_day.ambient_color(),
            mode: 0,
            base_scroll_speed: DEFAULT_PARALLAX_SPEED,
            cloud_opacity: 1.0,
            particle_opacity: 1.0,
            _pad: 0,
        }
    }
}

impl BackdropUniforms {
    /// Creates uniforms for the given screen size.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            screen_width: width,
            screen_height: height,
            ..Default::default()
        }
    }

    /// Updates uniforms from a day/night cycle.
    pub fn update_from_cycle(&mut self, cycle: &DayNightCycle) {
        let time_of_day = cycle.time_of_day();
        self.cycle_position = cycle.cycle_position;
        self.sky_top_color = time_of_day.sky_top_color();
        self.sky_bottom_color = time_of_day.sky_bottom_color();
        self.ambient_color = time_of_day.ambient_color();
    }

    /// Sets the backdrop mode.
    pub fn set_mode(&mut self, mode: BackdropMode) {
        self.mode = match mode {
            BackdropMode::Animated => 0,
            BackdropMode::Static => 1,
            BackdropMode::SolidColor => 2,
        };
    }
}

/// Complete backdrop state for CPU-side management.
#[derive(Debug, Clone)]
pub struct BackdropState {
    /// Active backdrop mode.
    pub mode: BackdropMode,
    /// Day/night cycle state.
    pub day_night: DayNightCycle,
    /// Parallax layers (up to 4).
    pub layers: [ParallaxLayer; 4],
    /// Cloud particles.
    pub clouds: Vec<CloudParticle>,
    /// Ambient particles.
    pub ambient_particles: Vec<AmbientParticle>,
    /// Static backdrop fallback.
    pub static_backdrop: StaticBackdrop,
    /// Total elapsed time.
    pub time: f32,
    /// GPU uniform buffer data.
    pub uniforms: BackdropUniforms,
}

impl Default for BackdropState {
    fn default() -> Self {
        Self {
            mode: BackdropMode::Animated,
            day_night: DayNightCycle::default(),
            layers: [
                ParallaxLayer::new(DEFAULT_PARALLAX_SPEED, 0),
                ParallaxLayer::new(DEFAULT_MIDGROUND_SPEED, 1),
                ParallaxLayer::new(DEFAULT_FOREGROUND_SPEED, 2),
                ParallaxLayer::default(),
            ],
            clouds: Vec::new(),
            ambient_particles: Vec::new(),
            static_backdrop: StaticBackdrop::default(),
            time: 0.0,
            uniforms: BackdropUniforms::default(),
        }
    }
}

impl BackdropState {
    /// Creates a new animated backdrop state.
    #[must_use]
    pub fn new_animated() -> Self {
        let mut state = Self::default();
        state.initialize_clouds();
        state
    }

    /// Creates a new static backdrop state.
    #[must_use]
    pub fn new_static(width: u32, height: u32, texture_index: u32) -> Self {
        Self {
            mode: BackdropMode::Static,
            static_backdrop: StaticBackdrop::new(width, height, texture_index),
            uniforms: {
                let mut u = BackdropUniforms::default();
                u.set_mode(BackdropMode::Static);
                u
            },
            ..Default::default()
        }
    }

    /// Initializes cloud particles across the screen.
    pub fn initialize_clouds(&mut self) {
        self.clouds.clear();
        let clouds_per_layer = MAX_CLOUD_PARTICLES / 3;

        for layer in 0..3u32 {
            for i in 0..clouds_per_layer {
                let x = (i as f32 / clouds_per_layer as f32) * 2.0 - 0.5;
                let y = 0.2 + (layer as f32 * 0.15) + ((i % 3) as f32 * 0.1);
                let mut cloud = CloudParticle::new(x, y, layer);
                cloud.variant = (i % 4) as u32;
                self.clouds.push(cloud);
            }
        }
    }

    /// Updates all backdrop elements.
    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        // Update day/night cycle
        self.day_night.update(dt);

        // Update parallax layers
        for layer in &mut self.layers {
            layer.update(dt, self.uniforms.base_scroll_speed);
        }

        // Update clouds
        for cloud in &mut self.clouds {
            cloud.update(dt);
        }

        // Update ambient particles
        self.ambient_particles.retain_mut(|p| {
            p.update(dt);
            p.is_alive()
        });

        // Update uniforms
        self.uniforms.time = self.time;
        self.uniforms.update_from_cycle(&self.day_night);
    }

    /// Gets the current number of visible clouds.
    #[must_use]
    pub fn visible_cloud_count(&self) -> usize {
        self.clouds.iter().filter(|c| c.is_visible()).count()
    }

    /// Gets the current number of alive ambient particles.
    #[must_use]
    pub fn ambient_particle_count(&self) -> usize {
        self.ambient_particles.len()
    }

    /// Spawns a new ambient particle at random position.
    pub fn spawn_ambient_particle(&mut self, x: f32, y: f32, size: f32, color: u32) {
        if self.ambient_particles.len() < MAX_AMBIENT_PARTICLES {
            self.ambient_particles.push(AmbientParticle::new(x, y, size, color));
        }
    }

    /// Sets the screen size for the backdrop.
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.uniforms.screen_width = width;
        self.uniforms.screen_height = height;
    }

    /// Checks if the backdrop is ready for rendering.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        match self.mode {
            BackdropMode::Animated | BackdropMode::SolidColor => true,
            BackdropMode::Static => self.static_backdrop.is_ready(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_particle_default() {
        let cloud = CloudParticle::default();
        assert_eq!(cloud.x, 0.0);
        assert_eq!(cloud.y, 0.5);
        assert_eq!(cloud.velocity_x, DEFAULT_PARALLAX_SPEED);
        assert_eq!(cloud.layer, 0);
    }

    #[test]
    fn test_cloud_particle_new() {
        let cloud = CloudParticle::new(0.5, 0.3, 1);
        assert_eq!(cloud.x, 0.5);
        assert_eq!(cloud.y, 0.3);
        assert_eq!(cloud.layer, 1);
        assert_eq!(cloud.velocity_x, DEFAULT_MIDGROUND_SPEED);
    }

    #[test]
    fn test_cloud_particle_update() {
        let mut cloud = CloudParticle::default();
        cloud.update(1.0);
        assert!(cloud.x > 0.0);
    }

    #[test]
    fn test_cloud_particle_wrap() {
        let mut cloud = CloudParticle::default();
        cloud.x = 1.6;
        cloud.update(0.0);
        assert_eq!(cloud.x, -0.5);
    }

    #[test]
    fn test_cloud_particle_visibility() {
        let cloud = CloudParticle::default();
        assert!(cloud.is_visible());

        let mut hidden = CloudParticle::default();
        hidden.x = -0.6;
        assert!(!hidden.is_visible());
    }

    #[test]
    fn test_ambient_particle_default() {
        let particle = AmbientParticle::default();
        assert_eq!(particle.x, 0.0);
        assert_eq!(particle.y, 0.0);
        assert_eq!(particle.lifetime, 5.0);
    }

    #[test]
    fn test_ambient_particle_update() {
        let mut particle = AmbientParticle::default();
        particle.update(1.0);
        assert!(particle.x > 0.0);
        assert!(particle.y > 0.0);
        assert_eq!(particle.lifetime, 4.0);
    }

    #[test]
    fn test_ambient_particle_alive() {
        let particle = AmbientParticle::default();
        assert!(particle.is_alive());

        let mut dead = AmbientParticle::default();
        dead.lifetime = 0.0;
        assert!(!dead.is_alive());
    }

    #[test]
    fn test_time_of_day_from_cycle() {
        assert_eq!(TimeOfDay::from_cycle(0.1), TimeOfDay::Dawn);
        assert_eq!(TimeOfDay::from_cycle(0.3), TimeOfDay::Day);
        assert_eq!(TimeOfDay::from_cycle(0.5), TimeOfDay::Dusk);
        assert_eq!(TimeOfDay::from_cycle(0.75), TimeOfDay::Night);
    }

    #[test]
    fn test_time_of_day_colors() {
        let day = TimeOfDay::Day;
        assert_eq!(day.ambient_color(), 0xFFFFFFFF);
        assert_ne!(day.sky_top_color(), day.sky_bottom_color());
    }

    #[test]
    fn test_day_night_cycle_default() {
        let cycle = DayNightCycle::default();
        assert_eq!(cycle.cycle_position, 0.3);
        assert_eq!(cycle.cycle_duration, DAY_NIGHT_CYCLE_DURATION);
        assert!(!cycle.paused);
    }

    #[test]
    fn test_day_night_cycle_update() {
        let mut cycle = DayNightCycle::default();
        let initial = cycle.cycle_position;
        cycle.update(1.0);
        assert!(cycle.cycle_position > initial);
    }

    #[test]
    fn test_day_night_cycle_pause() {
        let mut cycle = DayNightCycle::default();
        cycle.pause();
        let pos = cycle.cycle_position;
        cycle.update(1.0);
        assert_eq!(cycle.cycle_position, pos);
    }

    #[test]
    fn test_day_night_cycle_resume() {
        let mut cycle = DayNightCycle::default();
        cycle.pause();
        cycle.resume();
        assert!(!cycle.paused);
    }

    #[test]
    fn test_day_night_cycle_set_time() {
        let mut cycle = DayNightCycle::default();
        cycle.set_time(TimeOfDay::Night);
        assert_eq!(cycle.time_of_day(), TimeOfDay::Night);
    }

    #[test]
    fn test_day_night_cycle_wrap() {
        let mut cycle = DayNightCycle::default();
        cycle.cycle_position = 0.99;
        cycle.update(cycle.cycle_duration * 0.02);
        assert!(cycle.cycle_position < 0.5);
    }

    #[test]
    fn test_parallax_layer_default() {
        let layer = ParallaxLayer::default();
        assert_eq!(layer.scroll_x, 0.0);
        assert_eq!(layer.speed, 1.0);
    }

    #[test]
    fn test_parallax_layer_update() {
        let mut layer = ParallaxLayer::new(0.5, 0);
        layer.update(1.0, 0.1);
        assert!(layer.scroll_x > 0.0);
    }

    #[test]
    fn test_parallax_layer_wrap() {
        let mut layer = ParallaxLayer::default();
        layer.scroll_x = 1.1;
        layer.update(0.0, 0.0);
        assert!(layer.scroll_x < 1.0);
    }

    #[test]
    fn test_static_backdrop_default() {
        let backdrop = StaticBackdrop::default();
        assert!(!backdrop.is_ready());
    }

    #[test]
    fn test_static_backdrop_new() {
        let backdrop = StaticBackdrop::new(1920, 1080, 0);
        assert!(backdrop.is_ready());
        assert_eq!(backdrop.width, 1920);
        assert_eq!(backdrop.height, 1080);
    }

    #[test]
    fn test_backdrop_uniforms_default() {
        let uniforms = BackdropUniforms::default();
        assert_eq!(uniforms.screen_width, 1920.0);
        assert_eq!(uniforms.screen_height, 1080.0);
        assert_eq!(uniforms.mode, 0);
    }

    #[test]
    fn test_backdrop_uniforms_set_mode() {
        let mut uniforms = BackdropUniforms::default();
        uniforms.set_mode(BackdropMode::Static);
        assert_eq!(uniforms.mode, 1);
        uniforms.set_mode(BackdropMode::SolidColor);
        assert_eq!(uniforms.mode, 2);
    }

    #[test]
    fn test_backdrop_uniforms_update_from_cycle() {
        let mut uniforms = BackdropUniforms::default();
        let mut cycle = DayNightCycle::default();
        cycle.set_time(TimeOfDay::Night);
        uniforms.update_from_cycle(&cycle);
        assert_eq!(uniforms.sky_top_color, TimeOfDay::Night.sky_top_color());
    }

    #[test]
    fn test_backdrop_state_default() {
        let state = BackdropState::default();
        assert_eq!(state.mode, BackdropMode::Animated);
        assert!(state.clouds.is_empty());
    }

    #[test]
    fn test_backdrop_state_new_animated() {
        let state = BackdropState::new_animated();
        assert!(!state.clouds.is_empty());
        assert!(state.is_ready());
    }

    #[test]
    fn test_backdrop_state_new_static() {
        let state = BackdropState::new_static(1920, 1080, 0);
        assert_eq!(state.mode, BackdropMode::Static);
        assert!(state.is_ready());
    }

    #[test]
    fn test_backdrop_state_update() {
        let mut state = BackdropState::new_animated();
        let initial_time = state.time;
        state.update(0.016);
        assert!(state.time > initial_time);
    }

    #[test]
    fn test_backdrop_state_screen_size() {
        let mut state = BackdropState::default();
        state.set_screen_size(1280.0, 720.0);
        assert_eq!(state.uniforms.screen_width, 1280.0);
        assert_eq!(state.uniforms.screen_height, 720.0);
    }

    #[test]
    fn test_backdrop_state_spawn_ambient() {
        let mut state = BackdropState::default();
        state.spawn_ambient_particle(0.5, 0.5, 2.0, 0xFFFFFFFF);
        assert_eq!(state.ambient_particle_count(), 1);
    }

    #[test]
    fn test_backdrop_state_ambient_limit() {
        let mut state = BackdropState::default();
        for _ in 0..MAX_AMBIENT_PARTICLES + 10 {
            state.spawn_ambient_particle(0.5, 0.5, 2.0, 0xFFFFFFFF);
        }
        assert_eq!(state.ambient_particle_count(), MAX_AMBIENT_PARTICLES);
    }

    #[test]
    fn test_backdrop_state_visible_clouds() {
        let state = BackdropState::new_animated();
        assert!(state.visible_cloud_count() > 0);
    }

    #[test]
    fn test_cloud_particle_size() {
        assert_eq!(std::mem::size_of::<CloudParticle>(), 32);
    }

    #[test]
    fn test_ambient_particle_size() {
        assert_eq!(std::mem::size_of::<AmbientParticle>(), 32);
    }

    #[test]
    fn test_parallax_layer_size() {
        assert_eq!(std::mem::size_of::<ParallaxLayer>(), 32);
    }

    #[test]
    fn test_backdrop_uniforms_size() {
        assert_eq!(std::mem::size_of::<BackdropUniforms>(), 48);
    }
}
