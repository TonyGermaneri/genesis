//! Resolution management infrastructure.
//!
//! Handles runtime resolution changes and display configuration.
//! Features include:
//! - Resize all render targets
//! - Maintain aspect ratio
//! - Update projection matrices
//! - Fullscreen toggle support

use bytemuck::{Pod, Zeroable};

/// Minimum supported resolution width.
pub const MIN_RESOLUTION_WIDTH: u32 = 640;

/// Minimum supported resolution height.
pub const MIN_RESOLUTION_HEIGHT: u32 = 360;

/// Maximum supported resolution width.
pub const MAX_RESOLUTION_WIDTH: u32 = 7680;

/// Maximum supported resolution height.
pub const MAX_RESOLUTION_HEIGHT: u32 = 4320;

/// Default resolution width.
pub const DEFAULT_RESOLUTION_WIDTH: u32 = 1920;

/// Default resolution height.
pub const DEFAULT_RESOLUTION_HEIGHT: u32 = 1080;

/// Common 16:9 aspect ratio.
pub const ASPECT_16_9: f32 = 16.0 / 9.0;

/// Common 16:10 aspect ratio.
pub const ASPECT_16_10: f32 = 16.0 / 10.0;

/// Common 4:3 aspect ratio.
pub const ASPECT_4_3: f32 = 4.0 / 3.0;

/// Ultrawide 21:9 aspect ratio.
pub const ASPECT_21_9: f32 = 21.0 / 9.0;

/// Display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayMode {
    /// Windowed mode.
    #[default]
    Windowed,
    /// Borderless fullscreen (desktop resolution).
    BorderlessFullscreen,
    /// Exclusive fullscreen.
    ExclusiveFullscreen,
}

impl DisplayMode {
    /// Whether this mode is fullscreen.
    #[must_use]
    pub fn is_fullscreen(&self) -> bool {
        !matches!(self, Self::Windowed)
    }

    /// Whether this mode is exclusive.
    #[must_use]
    pub fn is_exclusive(&self) -> bool {
        matches!(self, Self::ExclusiveFullscreen)
    }
}

/// Scaling mode for non-native aspect ratios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScalingMode {
    /// Stretch to fill (may distort).
    Stretch,
    /// Maintain aspect ratio with letterboxing/pillarboxing.
    #[default]
    Letterbox,
    /// Crop to fill (may cut off edges).
    Crop,
    /// Integer scaling only (pixel-perfect).
    IntegerScale,
}

impl ScalingMode {
    /// Converts to GPU shader enum value.
    #[must_use]
    pub fn to_shader_value(&self) -> u32 {
        match self {
            Self::Stretch => 0,
            Self::Letterbox => 1,
            Self::Crop => 2,
            Self::IntegerScale => 3,
        }
    }
}

/// V-Sync mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VSyncMode {
    /// V-Sync disabled.
    Off,
    /// V-Sync enabled (wait for vertical blank).
    #[default]
    On,
    /// Adaptive V-Sync (disable on frame drops).
    Adaptive,
    /// Triple buffering.
    TripleBuffer,
}

impl VSyncMode {
    /// Whether V-Sync is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Off)
    }
}

/// A resolution preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        Self {
            width: DEFAULT_RESOLUTION_WIDTH,
            height: DEFAULT_RESOLUTION_HEIGHT,
        }
    }
}

impl Resolution {
    /// Creates a new resolution.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Creates a 720p resolution.
    #[must_use]
    pub const fn hd720() -> Self {
        Self::new(1280, 720)
    }

    /// Creates a 1080p resolution.
    #[must_use]
    pub const fn hd1080() -> Self {
        Self::new(1920, 1080)
    }

    /// Creates a 1440p resolution.
    #[must_use]
    pub const fn qhd() -> Self {
        Self::new(2560, 1440)
    }

    /// Creates a 4K resolution.
    #[must_use]
    pub const fn uhd4k() -> Self {
        Self::new(3840, 2160)
    }

    /// Calculates the aspect ratio.
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    /// Checks if this is a valid resolution.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.width >= MIN_RESOLUTION_WIDTH
            && self.width <= MAX_RESOLUTION_WIDTH
            && self.height >= MIN_RESOLUTION_HEIGHT
            && self.height <= MAX_RESOLUTION_HEIGHT
    }

    /// Clamps to valid bounds.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            width: self.width.clamp(MIN_RESOLUTION_WIDTH, MAX_RESOLUTION_WIDTH),
            height: self
                .height
                .clamp(MIN_RESOLUTION_HEIGHT, MAX_RESOLUTION_HEIGHT),
        }
    }

    /// Gets the total pixel count.
    #[must_use]
    pub fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Checks if this is a 16:9 aspect ratio.
    #[must_use]
    pub fn is_16_9(&self) -> bool {
        let aspect = self.aspect_ratio();
        (aspect - ASPECT_16_9).abs() < 0.01
    }

    /// Checks if this is a 16:10 aspect ratio.
    #[must_use]
    pub fn is_16_10(&self) -> bool {
        let aspect = self.aspect_ratio();
        (aspect - ASPECT_16_10).abs() < 0.01
    }

    /// Scales by a factor while maintaining aspect ratio.
    #[must_use]
    pub fn scaled(self, factor: f32) -> Self {
        Self {
            width: ((self.width as f32 * factor) as u32).max(1),
            height: ((self.height as f32 * factor) as u32).max(1),
        }
    }
}

/// Common resolution presets.
pub const RESOLUTION_PRESETS: &[Resolution] = &[
    Resolution::new(1280, 720),   // 720p
    Resolution::new(1366, 768),   // Common laptop
    Resolution::new(1600, 900),   // 900p
    Resolution::new(1920, 1080),  // 1080p
    Resolution::new(2560, 1440),  // 1440p
    Resolution::new(3440, 1440),  // Ultrawide
    Resolution::new(3840, 2160),  // 4K
    Resolution::new(5120, 2880),  // 5K
    Resolution::new(7680, 4320),  // 8K
];

/// Viewport dimensions and position for letterboxing.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Viewport {
    /// X offset (for pillarboxing).
    pub x: f32,
    /// Y offset (for letterboxing).
    pub y: f32,
    /// Viewport width.
    pub width: f32,
    /// Viewport height.
    pub height: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: DEFAULT_RESOLUTION_WIDTH as f32,
            height: DEFAULT_RESOLUTION_HEIGHT as f32,
        }
    }
}

impl Viewport {
    /// Creates a new viewport.
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a full-screen viewport.
    #[must_use]
    pub fn full(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    /// Calculates a letterboxed viewport.
    #[must_use]
    pub fn letterboxed(
        window_width: f32,
        window_height: f32,
        target_width: f32,
        target_height: f32,
    ) -> Self {
        if window_width <= 0.0 || window_height <= 0.0 || target_width <= 0.0 || target_height <= 0.0
        {
            return Self::default();
        }

        let window_aspect = window_width / window_height;
        let target_aspect = target_width / target_height;

        if window_aspect > target_aspect {
            // Pillarbox (black bars on sides)
            let viewport_width = window_height * target_aspect;
            let x = (window_width - viewport_width) / 2.0;
            Self::new(x, 0.0, viewport_width, window_height)
        } else {
            // Letterbox (black bars on top/bottom)
            let viewport_height = window_width / target_aspect;
            let y = (window_height - viewport_height) / 2.0;
            Self::new(0.0, y, window_width, viewport_height)
        }
    }

    /// Calculates an integer-scaled viewport.
    #[must_use]
    pub fn integer_scaled(
        window_width: f32,
        window_height: f32,
        base_width: f32,
        base_height: f32,
    ) -> Self {
        if base_width <= 0.0 || base_height <= 0.0 {
            return Self::default();
        }

        let scale_x = (window_width / base_width).floor().max(1.0);
        let scale_y = (window_height / base_height).floor().max(1.0);
        let scale = scale_x.min(scale_y);

        let viewport_width = base_width * scale;
        let viewport_height = base_height * scale;

        let x = (window_width - viewport_width) / 2.0;
        let y = (window_height - viewport_height) / 2.0;

        Self::new(x, y, viewport_width, viewport_height)
    }

    /// Gets the aspect ratio.
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0.0 {
            1.0
        } else {
            self.width / self.height
        }
    }

    /// Converts window coordinates to viewport coordinates.
    #[must_use]
    pub fn window_to_viewport(&self, window_x: f32, window_y: f32) -> (f32, f32) {
        let vx = (window_x - self.x) / self.width;
        let vy = (window_y - self.y) / self.height;
        (vx, vy)
    }

    /// Checks if a point is inside the viewport.
    #[must_use]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Orthographic projection matrix for 2D rendering.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct OrthoProjection {
    /// 4x4 projection matrix (column-major).
    pub matrix: [f32; 16],
}

impl Default for OrthoProjection {
    fn default() -> Self {
        Self::new(0.0, DEFAULT_RESOLUTION_WIDTH as f32, DEFAULT_RESOLUTION_HEIGHT as f32, 0.0)
    }
}

impl OrthoProjection {
    /// Creates an orthographic projection matrix.
    #[must_use]
    pub fn new(left: f32, right: f32, bottom: f32, top: f32) -> Self {
        let near = -1.0;
        let far = 1.0;

        let width = right - left;
        let height = top - bottom;
        let depth = far - near;

        // Handle zero dimensions
        let tx = if width == 0.0 {
            0.0
        } else {
            -(right + left) / width
        };
        let ty = if height == 0.0 {
            0.0
        } else {
            -(top + bottom) / height
        };
        let tz = if depth == 0.0 {
            0.0
        } else {
            -(far + near) / depth
        };

        let sx = if width == 0.0 { 1.0 } else { 2.0 / width };
        let sy = if height == 0.0 { 1.0 } else { 2.0 / height };
        let sz = if depth == 0.0 { 1.0 } else { -2.0 / depth };

        Self {
            matrix: [
                sx, 0.0, 0.0, 0.0, // Column 0
                0.0, sy, 0.0, 0.0, // Column 1
                0.0, 0.0, sz, 0.0, // Column 2
                tx, ty, tz, 1.0, // Column 3
            ],
        }
    }

    /// Creates a projection for the given resolution.
    #[must_use]
    pub fn for_resolution(width: f32, height: f32) -> Self {
        Self::new(0.0, width, height, 0.0)
    }

    /// Creates a projection for the given viewport.
    #[must_use]
    pub fn for_viewport(viewport: &Viewport) -> Self {
        Self::new(0.0, viewport.width, viewport.height, 0.0)
    }
}

/// GPU-ready uniform buffer for resolution/viewport data.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct ResolutionUniforms {
    /// Current window width.
    pub window_width: f32,
    /// Current window height.
    pub window_height: f32,
    /// Render target width.
    pub render_width: f32,
    /// Render target height.
    pub render_height: f32,
    /// Viewport X offset.
    pub viewport_x: f32,
    /// Viewport Y offset.
    pub viewport_y: f32,
    /// Viewport width.
    pub viewport_width: f32,
    /// Viewport height.
    pub viewport_height: f32,
    /// Scale factor.
    pub scale: f32,
    /// Scaling mode.
    pub scaling_mode: u32,
    /// Padding for alignment.
    _pad: [u32; 2],
}

impl Default for ResolutionUniforms {
    fn default() -> Self {
        Self {
            window_width: DEFAULT_RESOLUTION_WIDTH as f32,
            window_height: DEFAULT_RESOLUTION_HEIGHT as f32,
            render_width: DEFAULT_RESOLUTION_WIDTH as f32,
            render_height: DEFAULT_RESOLUTION_HEIGHT as f32,
            viewport_x: 0.0,
            viewport_y: 0.0,
            viewport_width: DEFAULT_RESOLUTION_WIDTH as f32,
            viewport_height: DEFAULT_RESOLUTION_HEIGHT as f32,
            scale: 1.0,
            scaling_mode: ScalingMode::Letterbox.to_shader_value(),
            _pad: [0; 2],
        }
    }
}

impl ResolutionUniforms {
    /// Creates uniforms for the given resolution.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            window_width: width,
            window_height: height,
            render_width: width,
            render_height: height,
            viewport_x: 0.0,
            viewport_y: 0.0,
            viewport_width: width,
            viewport_height: height,
            ..Default::default()
        }
    }

    /// Updates from viewport.
    pub fn update_viewport(&mut self, viewport: &Viewport) {
        self.viewport_x = viewport.x;
        self.viewport_y = viewport.y;
        self.viewport_width = viewport.width;
        self.viewport_height = viewport.height;
    }

    /// Sets the scaling mode.
    pub fn set_scaling_mode(&mut self, mode: ScalingMode) {
        self.scaling_mode = mode.to_shader_value();
    }
}

/// Resolution change request.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub struct ResolutionChangeRequest {
    /// New resolution.
    pub resolution: Resolution,
    /// New display mode.
    pub display_mode: DisplayMode,
    /// New scaling mode.
    pub scaling_mode: ScalingMode,
    /// New V-Sync mode.
    pub vsync: VSyncMode,
}


impl ResolutionChangeRequest {
    /// Creates a resolution change request.
    #[must_use]
    pub fn new(resolution: Resolution) -> Self {
        Self {
            resolution,
            ..Default::default()
        }
    }

    /// Sets the display mode.
    #[must_use]
    pub fn with_display_mode(mut self, mode: DisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    /// Sets the scaling mode.
    #[must_use]
    pub fn with_scaling_mode(mut self, mode: ScalingMode) -> Self {
        self.scaling_mode = mode;
        self
    }

    /// Sets the V-Sync mode.
    #[must_use]
    pub fn with_vsync(mut self, vsync: VSyncMode) -> Self {
        self.vsync = vsync;
        self
    }

    /// Creates a fullscreen toggle request.
    #[must_use]
    pub fn fullscreen(resolution: Resolution) -> Self {
        Self {
            resolution,
            display_mode: DisplayMode::BorderlessFullscreen,
            ..Default::default()
        }
    }

    /// Creates a windowed request.
    #[must_use]
    pub fn windowed(resolution: Resolution) -> Self {
        Self {
            resolution,
            display_mode: DisplayMode::Windowed,
            ..Default::default()
        }
    }
}

/// Resolution manager for handling display configuration.
#[derive(Debug, Clone)]
pub struct ResolutionManager {
    /// Current window resolution.
    current_resolution: Resolution,
    /// Target render resolution.
    render_resolution: Resolution,
    /// Current display mode.
    display_mode: DisplayMode,
    /// Current scaling mode.
    scaling_mode: ScalingMode,
    /// Current V-Sync mode.
    vsync: VSyncMode,
    /// Current viewport.
    viewport: Viewport,
    /// Projection matrix.
    projection: OrthoProjection,
    /// GPU uniforms.
    uniforms: ResolutionUniforms,
    /// Pending resolution change.
    pending_change: Option<ResolutionChangeRequest>,
    /// Whether render targets need recreation.
    targets_dirty: bool,
}

impl Default for ResolutionManager {
    fn default() -> Self {
        let resolution = Resolution::default();
        let viewport = Viewport::full(resolution.width as f32, resolution.height as f32);
        Self {
            current_resolution: resolution,
            render_resolution: resolution,
            display_mode: DisplayMode::default(),
            scaling_mode: ScalingMode::default(),
            vsync: VSyncMode::default(),
            viewport,
            projection: OrthoProjection::for_viewport(&viewport),
            uniforms: ResolutionUniforms::default(),
            pending_change: None,
            targets_dirty: false,
        }
    }
}

impl ResolutionManager {
    /// Creates a new resolution manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a resolution manager with the given initial resolution.
    #[must_use]
    pub fn with_resolution(resolution: Resolution) -> Self {
        let mut manager = Self::default();
        manager.set_resolution(resolution);
        manager
    }

    /// Gets the current window resolution.
    #[must_use]
    pub fn current_resolution(&self) -> Resolution {
        self.current_resolution
    }

    /// Gets the render resolution.
    #[must_use]
    pub fn render_resolution(&self) -> Resolution {
        self.render_resolution
    }

    /// Gets the current display mode.
    #[must_use]
    pub fn display_mode(&self) -> DisplayMode {
        self.display_mode
    }

    /// Gets the current scaling mode.
    #[must_use]
    pub fn scaling_mode(&self) -> ScalingMode {
        self.scaling_mode
    }

    /// Gets the current V-Sync mode.
    #[must_use]
    pub fn vsync(&self) -> VSyncMode {
        self.vsync
    }

    /// Gets the current viewport.
    #[must_use]
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Gets the projection matrix.
    #[must_use]
    pub fn projection(&self) -> &OrthoProjection {
        &self.projection
    }

    /// Gets the GPU uniforms.
    #[must_use]
    pub fn uniforms(&self) -> &ResolutionUniforms {
        &self.uniforms
    }

    /// Checks if render targets need recreation.
    #[must_use]
    pub fn targets_dirty(&self) -> bool {
        self.targets_dirty
    }

    /// Clears the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.targets_dirty = false;
    }

    /// Requests a resolution change.
    pub fn request_change(&mut self, request: ResolutionChangeRequest) {
        self.pending_change = Some(request);
    }

    /// Checks if there's a pending change.
    #[must_use]
    pub fn has_pending_change(&self) -> bool {
        self.pending_change.is_some()
    }

    /// Takes the pending change request.
    #[must_use]
    pub fn take_pending_change(&mut self) -> Option<ResolutionChangeRequest> {
        self.pending_change.take()
    }

    /// Sets the resolution directly.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        let resolution = resolution.clamped();
        if resolution != self.current_resolution {
            self.current_resolution = resolution;
            self.targets_dirty = true;
            self.update_viewport();
            self.update_uniforms();
        }
    }

    /// Sets the render resolution (internal resolution for upscaling).
    pub fn set_render_resolution(&mut self, resolution: Resolution) {
        let resolution = resolution.clamped();
        if resolution != self.render_resolution {
            self.render_resolution = resolution;
            self.targets_dirty = true;
            self.update_uniforms();
        }
    }

    /// Sets the display mode.
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }

    /// Sets the scaling mode.
    pub fn set_scaling_mode(&mut self, mode: ScalingMode) {
        if mode != self.scaling_mode {
            self.scaling_mode = mode;
            self.update_viewport();
            self.update_uniforms();
        }
    }

    /// Sets the V-Sync mode.
    pub fn set_vsync(&mut self, vsync: VSyncMode) {
        self.vsync = vsync;
    }

    /// Toggles fullscreen mode.
    pub fn toggle_fullscreen(&mut self) {
        self.display_mode = if self.display_mode.is_fullscreen() {
            DisplayMode::Windowed
        } else {
            DisplayMode::BorderlessFullscreen
        };
    }

    /// Updates the viewport based on current settings.
    fn update_viewport(&mut self) {
        let window_width = self.current_resolution.width as f32;
        let window_height = self.current_resolution.height as f32;
        let render_width = self.render_resolution.width as f32;
        let render_height = self.render_resolution.height as f32;

        self.viewport = match self.scaling_mode {
            ScalingMode::Stretch | ScalingMode::Crop => Viewport::full(window_width, window_height),
            ScalingMode::Letterbox => {
                Viewport::letterboxed(window_width, window_height, render_width, render_height)
            }
            ScalingMode::IntegerScale => {
                Viewport::integer_scaled(window_width, window_height, render_width, render_height)
            }
        };

        self.projection = OrthoProjection::for_viewport(&self.viewport);
    }

    /// Updates the GPU uniforms.
    fn update_uniforms(&mut self) {
        self.uniforms = ResolutionUniforms {
            window_width: self.current_resolution.width as f32,
            window_height: self.current_resolution.height as f32,
            render_width: self.render_resolution.width as f32,
            render_height: self.render_resolution.height as f32,
            viewport_x: self.viewport.x,
            viewport_y: self.viewport.y,
            viewport_width: self.viewport.width,
            viewport_height: self.viewport.height,
            scale: self.viewport.width / self.render_resolution.width as f32,
            scaling_mode: self.scaling_mode.to_shader_value(),
            _pad: [0; 2],
        };
    }

    /// Handles a window resize event.
    pub fn on_resize(&mut self, width: u32, height: u32) {
        self.set_resolution(Resolution::new(width, height));
    }

    /// Gets the available resolution presets.
    #[must_use]
    pub fn available_presets(&self) -> &'static [Resolution] {
        RESOLUTION_PRESETS
    }

    /// Finds the closest preset to the given resolution.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn closest_preset(&self, resolution: Resolution) -> Resolution {
        let target_pixels = resolution.pixel_count();
        RESOLUTION_PRESETS
            .iter()
            .min_by_key(|preset| {
                (preset.pixel_count() as i64 - target_pixels as i64).unsigned_abs()
            })
            .copied()
            .unwrap_or(resolution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mode_fullscreen() {
        assert!(!DisplayMode::Windowed.is_fullscreen());
        assert!(DisplayMode::BorderlessFullscreen.is_fullscreen());
        assert!(DisplayMode::ExclusiveFullscreen.is_fullscreen());
    }

    #[test]
    fn test_display_mode_exclusive() {
        assert!(!DisplayMode::Windowed.is_exclusive());
        assert!(!DisplayMode::BorderlessFullscreen.is_exclusive());
        assert!(DisplayMode::ExclusiveFullscreen.is_exclusive());
    }

    #[test]
    fn test_scaling_mode_shader_value() {
        assert_eq!(ScalingMode::Stretch.to_shader_value(), 0);
        assert_eq!(ScalingMode::Letterbox.to_shader_value(), 1);
        assert_eq!(ScalingMode::Crop.to_shader_value(), 2);
        assert_eq!(ScalingMode::IntegerScale.to_shader_value(), 3);
    }

    #[test]
    fn test_vsync_enabled() {
        assert!(!VSyncMode::Off.is_enabled());
        assert!(VSyncMode::On.is_enabled());
        assert!(VSyncMode::Adaptive.is_enabled());
    }

    #[test]
    fn test_resolution_default() {
        let res = Resolution::default();
        assert_eq!(res.width, DEFAULT_RESOLUTION_WIDTH);
        assert_eq!(res.height, DEFAULT_RESOLUTION_HEIGHT);
    }

    #[test]
    fn test_resolution_presets() {
        assert_eq!(Resolution::hd720().width, 1280);
        assert_eq!(Resolution::hd1080().width, 1920);
        assert_eq!(Resolution::qhd().width, 2560);
        assert_eq!(Resolution::uhd4k().width, 3840);
    }

    #[test]
    fn test_resolution_aspect_ratio() {
        let res = Resolution::hd1080();
        let aspect = res.aspect_ratio();
        assert!((aspect - ASPECT_16_9).abs() < 0.01);
    }

    #[test]
    fn test_resolution_is_valid() {
        assert!(Resolution::hd1080().is_valid());
        assert!(!Resolution::new(100, 100).is_valid());
        assert!(!Resolution::new(10000, 10000).is_valid());
    }

    #[test]
    fn test_resolution_clamped() {
        let too_small = Resolution::new(100, 100);
        let clamped = too_small.clamped();
        assert_eq!(clamped.width, MIN_RESOLUTION_WIDTH);
        assert_eq!(clamped.height, MIN_RESOLUTION_HEIGHT);
    }

    #[test]
    fn test_resolution_pixel_count() {
        let res = Resolution::hd1080();
        assert_eq!(res.pixel_count(), 1920 * 1080);
    }

    #[test]
    fn test_resolution_is_16_9() {
        assert!(Resolution::hd1080().is_16_9());
        assert!(!Resolution::new(1600, 1000).is_16_9());
    }

    #[test]
    fn test_resolution_scaled() {
        let res = Resolution::hd1080().scaled(0.5);
        assert_eq!(res.width, 960);
        assert_eq!(res.height, 540);
    }

    #[test]
    fn test_viewport_default() {
        let vp = Viewport::default();
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 0.0);
    }

    #[test]
    fn test_viewport_full() {
        let vp = Viewport::full(1920.0, 1080.0);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.width, 1920.0);
    }

    #[test]
    fn test_viewport_letterboxed_pillarbox() {
        // 21:9 window with 16:9 content = pillarbox
        let vp = Viewport::letterboxed(2560.0, 1080.0, 1920.0, 1080.0);
        assert!(vp.x > 0.0); // Has horizontal offset
        assert_eq!(vp.y, 0.0); // No vertical offset
    }

    #[test]
    fn test_viewport_letterboxed_letterbox() {
        // 4:3 window with 16:9 content = letterbox
        let vp = Viewport::letterboxed(1024.0, 768.0, 1920.0, 1080.0);
        assert_eq!(vp.x, 0.0); // No horizontal offset
        assert!(vp.y > 0.0); // Has vertical offset
    }

    #[test]
    fn test_viewport_integer_scaled() {
        let vp = Viewport::integer_scaled(1920.0, 1080.0, 320.0, 180.0);
        // Should scale by 6x (1920/320 = 6, 1080/180 = 6)
        assert_eq!(vp.width, 1920.0);
        assert_eq!(vp.height, 1080.0);
    }

    #[test]
    fn test_viewport_contains() {
        let vp = Viewport::new(100.0, 100.0, 200.0, 200.0);
        assert!(vp.contains(150.0, 150.0));
        assert!(!vp.contains(50.0, 50.0));
        assert!(!vp.contains(350.0, 350.0));
    }

    #[test]
    fn test_viewport_window_to_viewport() {
        let vp = Viewport::new(100.0, 100.0, 200.0, 200.0);
        let (vx, vy) = vp.window_to_viewport(200.0, 200.0);
        assert_eq!(vx, 0.5);
        assert_eq!(vy, 0.5);
    }

    #[test]
    fn test_ortho_projection_default() {
        let proj = OrthoProjection::default();
        // First element should be 2/width
        assert!((proj.matrix[0] - 2.0 / DEFAULT_RESOLUTION_WIDTH as f32).abs() < 0.0001);
    }

    #[test]
    fn test_ortho_projection_for_resolution() {
        let proj = OrthoProjection::for_resolution(1920.0, 1080.0);
        assert!((proj.matrix[0] - 2.0 / 1920.0).abs() < 0.0001);
    }

    #[test]
    fn test_resolution_uniforms_default() {
        let uniforms = ResolutionUniforms::default();
        assert_eq!(uniforms.window_width, DEFAULT_RESOLUTION_WIDTH as f32);
        assert_eq!(uniforms.scaling_mode, ScalingMode::Letterbox.to_shader_value());
    }

    #[test]
    fn test_resolution_uniforms_update_viewport() {
        let mut uniforms = ResolutionUniforms::default();
        let vp = Viewport::new(10.0, 20.0, 100.0, 100.0);
        uniforms.update_viewport(&vp);
        assert_eq!(uniforms.viewport_x, 10.0);
        assert_eq!(uniforms.viewport_y, 20.0);
    }

    #[test]
    fn test_resolution_change_request() {
        let request = ResolutionChangeRequest::new(Resolution::hd720())
            .with_display_mode(DisplayMode::BorderlessFullscreen)
            .with_scaling_mode(ScalingMode::IntegerScale)
            .with_vsync(VSyncMode::Off);

        assert_eq!(request.resolution.width, 1280);
        assert_eq!(request.display_mode, DisplayMode::BorderlessFullscreen);
        assert_eq!(request.scaling_mode, ScalingMode::IntegerScale);
        assert_eq!(request.vsync, VSyncMode::Off);
    }

    #[test]
    fn test_resolution_change_request_fullscreen() {
        let request = ResolutionChangeRequest::fullscreen(Resolution::hd1080());
        assert_eq!(request.display_mode, DisplayMode::BorderlessFullscreen);
    }

    #[test]
    fn test_resolution_manager_default() {
        let manager = ResolutionManager::new();
        assert_eq!(manager.current_resolution(), Resolution::default());
        assert_eq!(manager.display_mode(), DisplayMode::Windowed);
    }

    #[test]
    fn test_resolution_manager_with_resolution() {
        let manager = ResolutionManager::with_resolution(Resolution::hd720());
        assert_eq!(manager.current_resolution().width, 1280);
    }

    #[test]
    fn test_resolution_manager_set_resolution() {
        let mut manager = ResolutionManager::new();
        manager.set_resolution(Resolution::hd720());
        assert_eq!(manager.current_resolution().width, 1280);
        assert!(manager.targets_dirty());
    }

    #[test]
    fn test_resolution_manager_clear_dirty() {
        let mut manager = ResolutionManager::new();
        manager.set_resolution(Resolution::hd720());
        manager.clear_dirty();
        assert!(!manager.targets_dirty());
    }

    #[test]
    fn test_resolution_manager_request_change() {
        let mut manager = ResolutionManager::new();
        manager.request_change(ResolutionChangeRequest::default());
        assert!(manager.has_pending_change());
        let taken = manager.take_pending_change();
        assert!(taken.is_some());
        assert!(!manager.has_pending_change());
    }

    #[test]
    fn test_resolution_manager_toggle_fullscreen() {
        let mut manager = ResolutionManager::new();
        assert_eq!(manager.display_mode(), DisplayMode::Windowed);
        manager.toggle_fullscreen();
        assert_eq!(manager.display_mode(), DisplayMode::BorderlessFullscreen);
        manager.toggle_fullscreen();
        assert_eq!(manager.display_mode(), DisplayMode::Windowed);
    }

    #[test]
    fn test_resolution_manager_on_resize() {
        let mut manager = ResolutionManager::new();
        manager.on_resize(1280, 720);
        assert_eq!(manager.current_resolution().width, 1280);
    }

    #[test]
    fn test_resolution_manager_closest_preset() {
        let manager = ResolutionManager::new();
        let closest = manager.closest_preset(Resolution::new(1900, 1070));
        assert_eq!(closest.width, 1920);
        assert_eq!(closest.height, 1080);
    }

    #[test]
    fn test_resolution_uniforms_size() {
        assert_eq!(std::mem::size_of::<ResolutionUniforms>(), 48);
    }

    #[test]
    fn test_viewport_size() {
        assert_eq!(std::mem::size_of::<Viewport>(), 16);
    }

    #[test]
    fn test_ortho_projection_size() {
        assert_eq!(std::mem::size_of::<OrthoProjection>(), 64);
    }

    #[test]
    fn test_resolution_presets_count() {
        assert!(!RESOLUTION_PRESETS.is_empty());
        assert!(RESOLUTION_PRESETS.len() >= 5);
    }
}
