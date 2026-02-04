//! Egui integration layer for in-game UI rendering.
//!
//! Provides a clean abstraction over egui-wgpu-winit for rendering
//! UI elements in the game engine.

use egui::{Context, FullOutput, ViewportId};

/// Egui integration for wgpu and winit.
///
/// Handles the setup and lifecycle of egui rendering within the game engine.
pub struct EguiIntegration {
    /// The egui context.
    context: Context,
    /// Egui-winit state for event handling.
    state: egui_winit::State,
    /// Egui-wgpu renderer.
    renderer: egui_wgpu::Renderer,
}

impl std::fmt::Debug for EguiIntegration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiIntegration")
            .field("context", &"<egui::Context>")
            .field("state", &"<egui_winit::State>")
            .field("renderer", &"<egui_wgpu::Renderer>")
            .finish()
    }
}

impl EguiIntegration {
    /// Create a new egui integration.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `output_format` - The texture format for the render target
    /// * `window` - The winit window
    /// * `msaa_samples` - Number of MSAA samples (1 for no MSAA)
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        window: &winit::window::Window,
        msaa_samples: u32,
    ) -> Self {
        let context = Context::default();

        // Set up pixels per point based on window scale factor
        context.set_pixels_per_point(window.scale_factor() as f32);

        let state = egui_winit::State::new(
            context.clone(),
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let renderer = egui_wgpu::Renderer::new(device, output_format, None, msaa_samples, false);

        Self {
            context,
            state,
            renderer,
        }
    }

    /// Handle a winit window event.
    ///
    /// Returns `true` if egui consumed the event (i.e., the game should ignore it).
    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    /// Begin a new frame.
    ///
    /// Call this before any UI code.
    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_pass(raw_input);
    }

    /// End the current frame and get the output.
    ///
    /// Call this after all UI code.
    pub fn end_frame(&mut self, window: &winit::window::Window) -> FullOutput {
        let output = self.context.end_pass();
        self.state
            .handle_platform_output(window, output.platform_output.clone());
        output
    }

    /// Prepare egui for rendering and return paint jobs.
    ///
    /// Call this before creating the render pass. Then call `render_to_pass`.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `queue` - The wgpu queue
    /// * `encoder` - The command encoder
    /// * `screen_descriptor` - Screen size and scale information
    /// * `output` - The egui output from `end_frame`
    ///
    /// Returns the paint jobs to be passed to `render_to_pass`.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        output: FullOutput,
    ) -> Vec<egui::ClippedPrimitive> {
        // Upload textures
        for (id, image_delta) in &output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        // Convert shapes to paint jobs
        let paint_jobs = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);

        // Update buffers
        self.renderer
            .update_buffers(device, queue, encoder, &paint_jobs, screen_descriptor);

        // Free textures
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        paint_jobs
    }

    /// Render egui to an existing render pass.
    ///
    /// Call this after `prepare` and after creating your render pass.
    ///
    /// # Safety Note
    /// The `render_pass` must have a `'static` lifetime. Use
    /// `render_pass.forget_lifetime()` to convert a borrowed render pass.
    /// This is safe as long as you don't use the encoder until the render
    /// pass is dropped.
    ///
    /// # Example
    /// ```ignore
    /// let mut render_pass = encoder.begin_render_pass(&desc);
    /// egui_integration.render(&mut render_pass.forget_lifetime(), &paint_jobs, &screen_desc);
    /// drop(render_pass);
    /// ```
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass<'static>,
        paint_jobs: &[egui::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        self.renderer
            .render(render_pass, paint_jobs, screen_descriptor);
    }

    /// Get a reference to the underlying egui-wgpu renderer for advanced usage.
    #[must_use]
    pub fn renderer(&self) -> &egui_wgpu::Renderer {
        &self.renderer
    }

    /// Get a mutable reference to the underlying egui-wgpu renderer.
    #[must_use]
    pub fn renderer_mut(&mut self) -> &mut egui_wgpu::Renderer {
        &mut self.renderer
    }

    /// Get the egui context for UI code.
    #[must_use]
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Set pixels per point (scale factor).
    pub fn set_pixels_per_point(&mut self, pixels_per_point: f32) {
        self.context.set_pixels_per_point(pixels_per_point);
    }

    /// Get current pixels per point.
    #[must_use]
    pub fn pixels_per_point(&self) -> f32 {
        self.context.pixels_per_point()
    }

    /// Create a screen descriptor from window dimensions.
    #[must_use]
    pub fn screen_descriptor(&self, width: u32, height: u32) -> egui_wgpu::ScreenDescriptor {
        egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: self.pixels_per_point(),
        }
    }
}

/// Configuration for egui integration.
#[derive(Debug, Clone)]
pub struct EguiConfig {
    /// Initial pixels per point (scale factor).
    pub pixels_per_point: Option<f32>,
    /// Number of MSAA samples.
    pub msaa_samples: u32,
    /// Maximum texture dimension.
    pub max_texture_side: usize,
}

impl Default for EguiConfig {
    fn default() -> Self {
        Self {
            pixels_per_point: None,
            msaa_samples: 1,
            max_texture_side: 2048,
        }
    }
}

/// Helper for tracking frame timing.
#[derive(Debug, Clone)]
pub struct FrameTimer {
    /// Last frame time in seconds.
    last_frame_time: f32,
    /// Accumulated time for FPS calculation.
    accumulated_time: f32,
    /// Frame count for FPS calculation.
    frame_count: u32,
    /// Calculated FPS.
    fps: f32,
    /// Update interval for FPS calculation.
    update_interval: f32,
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameTimer {
    /// Create a new frame timer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_frame_time: 0.0,
            accumulated_time: 0.0,
            frame_count: 0,
            fps: 0.0,
            update_interval: 0.5, // Update FPS every 0.5 seconds
        }
    }

    /// Create with custom update interval.
    #[must_use]
    pub fn with_interval(update_interval: f32) -> Self {
        Self {
            last_frame_time: 0.0,
            accumulated_time: 0.0,
            frame_count: 0,
            fps: 0.0,
            update_interval,
        }
    }

    /// Update the timer with the current frame time.
    ///
    /// Returns the current FPS.
    pub fn update(&mut self, delta_time: f32) -> f32 {
        self.last_frame_time = delta_time;
        self.accumulated_time += delta_time;
        self.frame_count += 1;

        if self.accumulated_time >= self.update_interval {
            self.fps = self.frame_count as f32 / self.accumulated_time;
            self.accumulated_time = 0.0;
            self.frame_count = 0;
        }

        self.fps
    }

    /// Get the current FPS.
    #[must_use]
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Get the last frame time in seconds.
    #[must_use]
    pub fn frame_time(&self) -> f32 {
        self.last_frame_time
    }

    /// Get the last frame time in milliseconds.
    #[must_use]
    pub fn frame_time_ms(&self) -> f32 {
        self.last_frame_time * 1000.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_egui_config_defaults() {
        let config = EguiConfig::default();
        assert_eq!(config.msaa_samples, 1);
        assert!(config.pixels_per_point.is_none());
        assert_eq!(config.max_texture_side, 2048);
    }

    #[test]
    fn test_frame_timer_new() {
        let timer = FrameTimer::new();
        assert_eq!(timer.fps(), 0.0);
        assert_eq!(timer.frame_time(), 0.0);
    }

    #[test]
    fn test_frame_timer_with_interval() {
        let timer = FrameTimer::with_interval(1.0);
        assert_eq!(timer.update_interval, 1.0);
    }

    #[test]
    fn test_frame_timer_update() {
        let mut timer = FrameTimer::with_interval(0.1);

        // Simulate 60fps (16.67ms per frame)
        for _ in 0..10 {
            timer.update(0.01667);
        }

        // Should have updated FPS after interval
        assert!(timer.fps() > 50.0);
        assert!(timer.fps() < 70.0);
    }

    #[test]
    fn test_frame_timer_frame_time_ms() {
        let mut timer = FrameTimer::new();
        timer.update(0.016);
        assert!((timer.frame_time_ms() - 16.0).abs() < 0.1);
    }
}
