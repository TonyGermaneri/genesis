//! 2D camera system for viewing the world.
//!
//! This module provides a camera for panning and zooming the world view,
//! with coordinate transforms between screen and world space.

use bytemuck::{Pod, Zeroable};

/// Minimum zoom level (zoomed out).
pub const MIN_ZOOM: f32 = 0.25;

/// Maximum zoom level (zoomed in).
pub const MAX_ZOOM: f32 = 20.0;

/// Default zoom level.
pub const DEFAULT_ZOOM: f32 = 1.0;

/// 2D camera for viewing the world.
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world coordinates (center of view).
    pub position: (f32, f32),
    /// Zoom level (1.0 = 1:1 pixel mapping).
    pub zoom: f32,
    /// Viewport size in pixels (width, height).
    pub viewport_size: (u32, u32),
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            zoom: DEFAULT_ZOOM,
            viewport_size: (1280, 720),
        }
    }
}

impl Camera {
    /// Creates a new camera with the given viewport size.
    #[must_use]
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            position: (0.0, 0.0),
            zoom: DEFAULT_ZOOM,
            viewport_size: (viewport_width, viewport_height),
        }
    }

    /// Move camera by delta in world units.
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position.0 += dx;
        self.position.1 += dy;
    }

    /// Zoom in/out by factor (clamped to reasonable range).
    ///
    /// Factor > 1.0 zooms in, < 1.0 zooms out.
    pub fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Set absolute zoom level (clamped).
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Center camera on world position.
    pub fn center_on(&mut self, world_x: f32, world_y: f32) {
        self.position = (world_x, world_y);
    }

    /// Set the viewport size.
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_size = (width, height);
    }

    /// Get the viewport size.
    #[must_use]
    pub const fn viewport(&self) -> (u32, u32) {
        self.viewport_size
    }

    /// Convert screen coordinates to world coordinates.
    ///
    /// Screen origin is top-left, Y increases downward.
    /// World Y also increases downward (matching screen convention).
    #[must_use]
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        let half_width = self.viewport_size.0 as f32 / 2.0;
        let half_height = self.viewport_size.1 as f32 / 2.0;

        // Offset from screen center, scaled by zoom
        let world_x = self.position.0 + (screen_x - half_width) / self.zoom;
        let world_y = self.position.1 + (screen_y - half_height) / self.zoom;

        (world_x, world_y)
    }

    /// Convert world coordinates to screen coordinates.
    ///
    /// Returns screen position where world point would appear.
    #[must_use]
    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        let half_width = self.viewport_size.0 as f32 / 2.0;
        let half_height = self.viewport_size.1 as f32 / 2.0;

        // Offset from camera center, scaled by zoom, then offset to screen center
        let screen_x = (world_x - self.position.0) * self.zoom + half_width;
        let screen_y = (world_y - self.position.1) * self.zoom + half_height;

        (screen_x, screen_y)
    }

    /// Get visible world bounds (min_x, min_y, max_x, max_y).
    ///
    /// Returns the rectangular region of the world visible in the viewport.
    #[must_use]
    pub fn visible_bounds(&self) -> (f32, f32, f32, f32) {
        let half_width = (self.viewport_size.0 as f32 / 2.0) / self.zoom;
        let half_height = (self.viewport_size.1 as f32 / 2.0) / self.zoom;

        (
            self.position.0 - half_width,  // min_x
            self.position.1 - half_height, // min_y
            self.position.0 + half_width,  // max_x
            self.position.1 + half_height, // max_y
        )
    }

    /// Get visible world size (width, height) in world units.
    #[must_use]
    pub fn visible_size(&self) -> (f32, f32) {
        (
            self.viewport_size.0 as f32 / self.zoom,
            self.viewport_size.1 as f32 / self.zoom,
        )
    }

    /// Check if a world point is visible on screen.
    #[must_use]
    pub fn is_visible(&self, world_x: f32, world_y: f32) -> bool {
        let (min_x, min_y, max_x, max_y) = self.visible_bounds();
        world_x >= min_x && world_x <= max_x && world_y >= min_y && world_y <= max_y
    }

    /// Check if a world rectangle intersects the visible area.
    #[must_use]
    pub fn is_rect_visible(
        &self,
        rect_min_x: f32,
        rect_min_y: f32,
        rect_max_x: f32,
        rect_max_y: f32,
    ) -> bool {
        let (min_x, min_y, max_x, max_y) = self.visible_bounds();
        rect_max_x >= min_x && rect_min_x <= max_x && rect_max_y >= min_y && rect_min_y <= max_y
    }

    /// Get uniform data for GPU shaders.
    #[must_use]
    pub fn as_uniform(&self) -> CameraUniform {
        // Create orthographic projection matrix
        let (min_x, min_y, max_x, max_y) = self.visible_bounds();

        // Orthographic projection: maps world coords to NDC [-1, 1]
        let width = max_x - min_x;
        let height = max_y - min_y;

        // Avoid division by zero
        let width = if width.abs() < f32::EPSILON {
            1.0
        } else {
            width
        };
        let height = if height.abs() < f32::EPSILON {
            1.0
        } else {
            height
        };

        // Orthographic projection matrix (row-major for WGSL)
        // Maps [min_x, max_x] -> [-1, 1] and [min_y, max_y] -> [-1, 1]
        let view_proj = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height, 0.0, 0.0], // Flip Y for screen coords
            [0.0, 0.0, 1.0, 0.0],
            [-(max_x + min_x) / width, (max_y + min_y) / height, 0.0, 1.0],
        ];

        CameraUniform {
            view_proj,
            position: [self.position.0, self.position.1],
            zoom: self.zoom,
            _padding: 0.0,
        }
    }

    /// Smoothly interpolate camera toward target position.
    pub fn lerp_to(&mut self, target_x: f32, target_y: f32, t: f32) {
        let t = t.clamp(0.0, 1.0);
        self.position.0 += (target_x - self.position.0) * t;
        self.position.1 += (target_y - self.position.1) * t;
    }

    /// Smoothly interpolate zoom toward target zoom.
    pub fn lerp_zoom(&mut self, target_zoom: f32, t: f32) {
        let t = t.clamp(0.0, 1.0);
        let target = target_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self.zoom += (target - self.zoom) * t;
    }
}

/// GPU-compatible camera uniform data.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform {
    /// View-projection matrix (4x4, row-major).
    pub view_proj: [[f32; 4]; 4],
    /// Camera position in world coordinates.
    pub position: [f32; 2],
    /// Zoom level.
    pub zoom: f32,
    /// Padding for 16-byte alignment.
    _padding: f32,
}

impl Default for CameraUniform {
    fn default() -> Self {
        Camera::default().as_uniform()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(1920, 1080);
        assert_eq!(camera.viewport_size, (1920, 1080));
        assert!((camera.zoom - DEFAULT_ZOOM).abs() < f32::EPSILON);
    }

    #[test]
    fn test_camera_translate() {
        let mut camera = Camera::default();
        camera.translate(100.0, 50.0);
        assert!((camera.position.0 - 100.0).abs() < f32::EPSILON);
        assert!((camera.position.1 - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_camera_zoom_clamp() {
        let mut camera = Camera::default();

        // Zoom in beyond max
        camera.set_zoom(10.0);
        assert!((camera.zoom - MAX_ZOOM).abs() < f32::EPSILON);

        // Zoom out beyond min
        camera.set_zoom(0.01);
        assert!((camera.zoom - MIN_ZOOM).abs() < f32::EPSILON);
    }

    #[test]
    fn test_camera_zoom_by() {
        let mut camera = Camera::default();
        camera.set_zoom(1.0);

        camera.zoom_by(2.0);
        assert!((camera.zoom - 2.0).abs() < f32::EPSILON);

        camera.zoom_by(0.5);
        assert!((camera.zoom - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_camera_center_on() {
        let mut camera = Camera::default();
        camera.center_on(500.0, 300.0);
        assert!((camera.position.0 - 500.0).abs() < f32::EPSILON);
        assert!((camera.position.1 - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_screen_to_world_center() {
        let camera = Camera::new(800, 600);
        // Screen center should map to world position (camera center)
        let (world_x, world_y) = camera.screen_to_world(400.0, 300.0);
        assert!((world_x - 0.0).abs() < f32::EPSILON);
        assert!((world_y - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_world_to_screen_center() {
        let camera = Camera::new(800, 600);
        // Camera center should map to screen center
        let (screen_x, screen_y) = camera.world_to_screen(0.0, 0.0);
        assert!((screen_x - 400.0).abs() < f32::EPSILON);
        assert!((screen_y - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_screen_world_roundtrip() {
        let mut camera = Camera::new(1280, 720);
        camera.center_on(1000.0, 500.0);
        camera.set_zoom(2.0);

        let screen_pos = (640.0, 360.0); // Screen center
        let world_pos = camera.screen_to_world(screen_pos.0, screen_pos.1);
        let back_to_screen = camera.world_to_screen(world_pos.0, world_pos.1);

        assert!((back_to_screen.0 - screen_pos.0).abs() < 0.01);
        assert!((back_to_screen.1 - screen_pos.1).abs() < 0.01);
    }

    #[test]
    fn test_visible_bounds() {
        let mut camera = Camera::new(800, 600);
        camera.center_on(100.0, 100.0);
        camera.set_zoom(1.0);

        let (min_x, min_y, max_x, max_y) = camera.visible_bounds();

        // With zoom=1, visible area is viewport size centered on position
        assert!((min_x - (100.0 - 400.0)).abs() < f32::EPSILON);
        assert!((min_y - (100.0 - 300.0)).abs() < f32::EPSILON);
        assert!((max_x - (100.0 + 400.0)).abs() < f32::EPSILON);
        assert!((max_y - (100.0 + 300.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_visible_bounds_zoomed() {
        let mut camera = Camera::new(800, 600);
        camera.center_on(0.0, 0.0);
        camera.set_zoom(2.0);

        let (min_x, min_y, max_x, max_y) = camera.visible_bounds();

        // With zoom=2, visible area is half the viewport size in world units
        assert!((min_x - (-200.0)).abs() < f32::EPSILON);
        assert!((max_x - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_visible() {
        let mut camera = Camera::new(800, 600);
        camera.center_on(0.0, 0.0);
        camera.set_zoom(1.0);

        // Center is visible
        assert!(camera.is_visible(0.0, 0.0));

        // Point within bounds
        assert!(camera.is_visible(100.0, 100.0));

        // Point outside bounds
        assert!(!camera.is_visible(1000.0, 1000.0));
    }

    #[test]
    fn test_is_rect_visible() {
        let mut camera = Camera::new(800, 600);
        camera.center_on(0.0, 0.0);
        camera.set_zoom(1.0);

        // Rect fully inside
        assert!(camera.is_rect_visible(-100.0, -100.0, 100.0, 100.0));

        // Rect partially overlapping
        assert!(camera.is_rect_visible(300.0, 200.0, 600.0, 500.0));

        // Rect fully outside
        assert!(!camera.is_rect_visible(1000.0, 1000.0, 1200.0, 1200.0));
    }

    #[test]
    fn test_camera_uniform_size() {
        // Ensure proper GPU alignment
        assert_eq!(std::mem::size_of::<CameraUniform>(), 80);
    }

    #[test]
    fn test_lerp_to() {
        let mut camera = Camera::default();
        camera.center_on(0.0, 0.0);

        camera.lerp_to(100.0, 100.0, 0.5);
        assert!((camera.position.0 - 50.0).abs() < f32::EPSILON);
        assert!((camera.position.1 - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lerp_zoom() {
        let mut camera = Camera::default();
        camera.set_zoom(1.0);

        camera.lerp_zoom(2.0, 0.5);
        assert!((camera.zoom - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_visible_size() {
        let mut camera = Camera::new(800, 600);
        camera.set_zoom(2.0);

        let (w, h) = camera.visible_size();
        assert!((w - 400.0).abs() < f32::EPSILON);
        assert!((h - 300.0).abs() < f32::EPSILON);
    }
}
