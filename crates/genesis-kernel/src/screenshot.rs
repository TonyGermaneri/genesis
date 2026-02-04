//! Screenshot capture infrastructure.
//!
//! Provides framebuffer capture and thumbnail generation.
//! Features include:
//! - Capture framebuffer to CPU buffer
//! - Downsample to 256x144 thumbnail
//! - Return as PNG-ready byte array
//! - Async operation to avoid frame stalls

use bytemuck::{Pod, Zeroable};

/// Default thumbnail width.
pub const THUMBNAIL_WIDTH: u32 = 256;

/// Default thumbnail height.
pub const THUMBNAIL_HEIGHT: u32 = 144;

/// Maximum screenshot width.
pub const MAX_SCREENSHOT_WIDTH: u32 = 7680;

/// Maximum screenshot height.
pub const MAX_SCREENSHOT_HEIGHT: u32 = 4320;

/// Bytes per pixel (RGBA).
pub const BYTES_PER_PIXEL: usize = 4;

/// Screenshot capture status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptureStatus {
    /// No capture in progress.
    #[default]
    Idle,
    /// Waiting for GPU readback.
    Pending,
    /// Readback complete, processing.
    Processing,
    /// Screenshot ready.
    Ready,
    /// Capture failed.
    Failed,
}

impl CaptureStatus {
    /// Whether a capture is in progress.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Pending | Self::Processing)
    }

    /// Whether the capture is ready.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Whether the capture failed.
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

/// Screenshot output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenshotFormat {
    /// Raw RGBA bytes.
    #[default]
    RawRgba,
    /// PNG-ready (includes header info).
    Png,
    /// JPEG-ready (includes header info).
    Jpeg,
}

/// Screenshot quality setting for compressed formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenshotQuality {
    /// Low quality, small file size.
    Low,
    /// Medium quality.
    #[default]
    Medium,
    /// High quality, larger file size.
    High,
    /// Maximum quality.
    Maximum,
}

impl ScreenshotQuality {
    /// Gets the JPEG quality value (0-100).
    #[must_use]
    pub fn jpeg_quality(&self) -> u8 {
        match self {
            Self::Low => 50,
            Self::Medium => 75,
            Self::High => 90,
            Self::Maximum => 100,
        }
    }

    /// Gets the PNG compression level (0-9).
    #[must_use]
    pub fn png_compression(&self) -> u8 {
        match self {
            Self::Low => 9,
            Self::Medium => 6,
            Self::High => 3,
            Self::Maximum => 0,
        }
    }
}

/// Configuration for screenshot capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureConfig {
    /// Whether to generate a thumbnail.
    pub generate_thumbnail: bool,
    /// Thumbnail width.
    pub thumbnail_width: u32,
    /// Thumbnail height.
    pub thumbnail_height: u32,
    /// Output format.
    pub format: ScreenshotFormat,
    /// Quality setting.
    pub quality: ScreenshotQuality,
    /// Whether to include UI in the capture.
    pub include_ui: bool,
    /// Whether to flip vertically (for OpenGL).
    pub flip_vertical: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            generate_thumbnail: true,
            thumbnail_width: THUMBNAIL_WIDTH,
            thumbnail_height: THUMBNAIL_HEIGHT,
            format: ScreenshotFormat::RawRgba,
            quality: ScreenshotQuality::Medium,
            include_ui: true,
            flip_vertical: false,
        }
    }
}

impl CaptureConfig {
    /// Creates a configuration for thumbnail-only capture.
    #[must_use]
    pub fn thumbnail_only() -> Self {
        Self {
            generate_thumbnail: true,
            ..Default::default()
        }
    }

    /// Creates a configuration for full-resolution capture.
    #[must_use]
    pub fn full_resolution() -> Self {
        Self {
            generate_thumbnail: false,
            ..Default::default()
        }
    }

    /// Sets the thumbnail dimensions.
    #[must_use]
    pub fn with_thumbnail_size(mut self, width: u32, height: u32) -> Self {
        self.thumbnail_width = width.min(MAX_SCREENSHOT_WIDTH);
        self.thumbnail_height = height.min(MAX_SCREENSHOT_HEIGHT);
        self
    }

    /// Sets the output format.
    #[must_use]
    pub fn with_format(mut self, format: ScreenshotFormat) -> Self {
        self.format = format;
        self
    }

    /// Sets the quality.
    #[must_use]
    pub fn with_quality(mut self, quality: ScreenshotQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Sets whether to include UI.
    #[must_use]
    pub fn with_ui(mut self, include: bool) -> Self {
        self.include_ui = include;
        self
    }

    /// Calculates the expected buffer size for full resolution.
    #[must_use]
    pub fn full_buffer_size(&self, width: u32, height: u32) -> usize {
        (width as usize) * (height as usize) * BYTES_PER_PIXEL
    }

    /// Calculates the expected thumbnail buffer size.
    #[must_use]
    pub fn thumbnail_buffer_size(&self) -> usize {
        (self.thumbnail_width as usize) * (self.thumbnail_height as usize) * BYTES_PER_PIXEL
    }
}

/// GPU-side capture request descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct CaptureRequest {
    /// Source X offset.
    pub src_x: u32,
    /// Source Y offset.
    pub src_y: u32,
    /// Source width.
    pub src_width: u32,
    /// Source height.
    pub src_height: u32,
    /// Destination width.
    pub dst_width: u32,
    /// Destination height.
    pub dst_height: u32,
    /// Flags (bit 0: flip vertical, bit 1: include UI).
    pub flags: u32,
    /// Padding.
    _pad: u32,
}

impl Default for CaptureRequest {
    fn default() -> Self {
        Self {
            src_x: 0,
            src_y: 0,
            src_width: 1920,
            src_height: 1080,
            dst_width: THUMBNAIL_WIDTH,
            dst_height: THUMBNAIL_HEIGHT,
            flags: 0,
            _pad: 0,
        }
    }
}

impl CaptureRequest {
    /// Creates a capture request for the full framebuffer.
    #[must_use]
    pub fn full_framebuffer(width: u32, height: u32) -> Self {
        Self {
            src_x: 0,
            src_y: 0,
            src_width: width,
            src_height: height,
            dst_width: width,
            dst_height: height,
            flags: 0,
            _pad: 0,
        }
    }

    /// Creates a capture request for a thumbnail.
    #[must_use]
    pub fn thumbnail(src_width: u32, src_height: u32) -> Self {
        Self {
            src_x: 0,
            src_y: 0,
            src_width,
            src_height,
            dst_width: THUMBNAIL_WIDTH,
            dst_height: THUMBNAIL_HEIGHT,
            flags: 0,
            _pad: 0,
        }
    }

    /// Creates a capture request for a region.
    #[must_use]
    pub fn region(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            src_x: x,
            src_y: y,
            src_width: width,
            src_height: height,
            dst_width: width,
            dst_height: height,
            flags: 0,
            _pad: 0,
        }
    }

    /// Sets the vertical flip flag.
    #[must_use]
    pub fn with_flip(mut self, flip: bool) -> Self {
        if flip {
            self.flags |= 1;
        } else {
            self.flags &= !1;
        }
        self
    }

    /// Sets the include UI flag.
    #[must_use]
    pub fn with_ui(mut self, include: bool) -> Self {
        if include {
            self.flags |= 2;
        } else {
            self.flags &= !2;
        }
        self
    }

    /// Whether vertical flip is enabled.
    #[must_use]
    pub fn is_flipped(&self) -> bool {
        self.flags & 1 != 0
    }

    /// Whether UI capture is enabled.
    #[must_use]
    pub fn includes_ui(&self) -> bool {
        self.flags & 2 != 0
    }

    /// Calculates the output buffer size.
    #[must_use]
    pub fn output_size(&self) -> usize {
        (self.dst_width as usize) * (self.dst_height as usize) * BYTES_PER_PIXEL
    }

    /// Validates the request parameters.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.src_width > 0
            && self.src_height > 0
            && self.dst_width > 0
            && self.dst_height > 0
            && self.src_width <= MAX_SCREENSHOT_WIDTH
            && self.src_height <= MAX_SCREENSHOT_HEIGHT
            && self.dst_width <= MAX_SCREENSHOT_WIDTH
            && self.dst_height <= MAX_SCREENSHOT_HEIGHT
    }
}

/// Screenshot result containing image data.
#[derive(Debug, Clone)]
pub struct ScreenshotData {
    /// Image width.
    pub width: u32,
    /// Image height.
    pub height: u32,
    /// Raw RGBA pixel data.
    pub pixels: Vec<u8>,
    /// Format of the data.
    pub format: ScreenshotFormat,
}

impl ScreenshotData {
    /// Creates new screenshot data.
    #[must_use]
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            width,
            height,
            pixels,
            format: ScreenshotFormat::RawRgba,
        }
    }

    /// Creates empty screenshot data with allocated buffer.
    #[must_use]
    pub fn with_capacity(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * BYTES_PER_PIXEL;
        Self {
            width,
            height,
            pixels: vec![0; size],
            format: ScreenshotFormat::RawRgba,
        }
    }

    /// Gets the expected buffer size.
    #[must_use]
    pub fn expected_size(&self) -> usize {
        (self.width as usize) * (self.height as usize) * BYTES_PER_PIXEL
    }

    /// Validates the data integrity.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.pixels.len() == self.expected_size() && self.width > 0 && self.height > 0
    }

    /// Gets a pixel at the given coordinates.
    #[must_use]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) as usize) * BYTES_PER_PIXEL;
        if idx + 3 < self.pixels.len() {
            Some([
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            ])
        } else {
            None
        }
    }

    /// Sets a pixel at the given coordinates.
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) as usize) * BYTES_PER_PIXEL;
        if idx + 3 < self.pixels.len() {
            self.pixels[idx] = rgba[0];
            self.pixels[idx + 1] = rgba[1];
            self.pixels[idx + 2] = rgba[2];
            self.pixels[idx + 3] = rgba[3];
        }
    }

    /// Flips the image vertically in place.
    pub fn flip_vertical(&mut self) {
        let row_size = (self.width as usize) * BYTES_PER_PIXEL;
        let half_height = self.height as usize / 2;

        for y in 0..half_height {
            let top_start = y * row_size;
            let bottom_start = (self.height as usize - 1 - y) * row_size;

            for x in 0..row_size {
                self.pixels.swap(top_start + x, bottom_start + x);
            }
        }
    }
}

/// Screenshot manager for async capture operations.
#[derive(Debug, Clone)]
pub struct ScreenshotManager {
    /// Current capture status.
    status: CaptureStatus,
    /// Active capture configuration.
    config: CaptureConfig,
    /// Pending capture request.
    request: Option<CaptureRequest>,
    /// Full resolution capture data.
    full_data: Option<ScreenshotData>,
    /// Thumbnail capture data.
    thumbnail_data: Option<ScreenshotData>,
    /// Frame delay counter for async readback.
    frame_delay: u32,
    /// Number of frames to wait for GPU readback.
    readback_delay_frames: u32,
}

impl Default for ScreenshotManager {
    fn default() -> Self {
        Self {
            status: CaptureStatus::Idle,
            config: CaptureConfig::default(),
            request: None,
            full_data: None,
            thumbnail_data: None,
            frame_delay: 0,
            readback_delay_frames: 2,
        }
    }
}

impl ScreenshotManager {
    /// Creates a new screenshot manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the readback delay in frames.
    pub fn set_readback_delay(&mut self, frames: u32) {
        self.readback_delay_frames = frames.max(1);
    }

    /// Initiates a screenshot capture.
    pub fn capture(&mut self, config: CaptureConfig, screen_width: u32, screen_height: u32) {
        if self.status.is_busy() {
            return;
        }

        self.config = config;
        self.status = CaptureStatus::Pending;
        self.frame_delay = 0;

        let request = if config.generate_thumbnail {
            CaptureRequest::thumbnail(screen_width, screen_height)
                .with_flip(config.flip_vertical)
                .with_ui(config.include_ui)
        } else {
            CaptureRequest::full_framebuffer(screen_width, screen_height)
                .with_flip(config.flip_vertical)
                .with_ui(config.include_ui)
        };

        self.request = Some(request);
    }

    /// Captures a thumbnail screenshot.
    pub fn capture_thumbnail(&mut self, screen_width: u32, screen_height: u32) {
        self.capture(CaptureConfig::thumbnail_only(), screen_width, screen_height);
    }

    /// Captures a full-resolution screenshot.
    pub fn capture_full(&mut self, screen_width: u32, screen_height: u32) {
        self.capture(
            CaptureConfig::full_resolution(),
            screen_width,
            screen_height,
        );
    }

    /// Updates the capture state (call once per frame).
    pub fn update(&mut self) {
        if self.status == CaptureStatus::Pending {
            self.frame_delay += 1;
            if self.frame_delay >= self.readback_delay_frames {
                self.status = CaptureStatus::Processing;
            }
        }
    }

    /// Receives the captured pixel data from GPU readback.
    pub fn receive_data(&mut self, pixels: Vec<u8>) {
        if self.status != CaptureStatus::Processing {
            return;
        }

        if let Some(request) = &self.request {
            let data = ScreenshotData::new(request.dst_width, request.dst_height, pixels);

            if data.is_valid() {
                if self.config.generate_thumbnail {
                    self.thumbnail_data = Some(data);
                } else {
                    self.full_data = Some(data);
                }
                self.status = CaptureStatus::Ready;
            } else {
                self.status = CaptureStatus::Failed;
            }
        } else {
            self.status = CaptureStatus::Failed;
        }
    }

    /// Marks the capture as failed.
    pub fn mark_failed(&mut self) {
        self.status = CaptureStatus::Failed;
        self.request = None;
    }

    /// Gets the current capture status.
    #[must_use]
    pub fn status(&self) -> CaptureStatus {
        self.status
    }

    /// Gets the pending capture request.
    #[must_use]
    pub fn pending_request(&self) -> Option<&CaptureRequest> {
        if self.status == CaptureStatus::Pending {
            self.request.as_ref()
        } else {
            None
        }
    }

    /// Takes the thumbnail data if ready.
    #[must_use]
    pub fn take_thumbnail(&mut self) -> Option<ScreenshotData> {
        if self.status == CaptureStatus::Ready {
            self.status = CaptureStatus::Idle;
            self.request = None;
            self.thumbnail_data.take()
        } else {
            None
        }
    }

    /// Takes the full-resolution data if ready.
    #[must_use]
    pub fn take_full(&mut self) -> Option<ScreenshotData> {
        if self.status == CaptureStatus::Ready {
            self.status = CaptureStatus::Idle;
            self.request = None;
            self.full_data.take()
        } else {
            None
        }
    }

    /// Resets the manager to idle state.
    pub fn reset(&mut self) {
        self.status = CaptureStatus::Idle;
        self.request = None;
        self.full_data = None;
        self.thumbnail_data = None;
        self.frame_delay = 0;
    }

    /// Checks if a capture is in progress.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.status.is_busy()
    }

    /// Checks if a capture is ready.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.status.is_ready()
    }
}

/// Downsamples image data using box filter.
#[must_use]
pub fn downsample_box(
    src: &[u8],
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> Vec<u8> {
    if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
        return Vec::new();
    }

    let expected_src_size = (src_width as usize) * (src_height as usize) * BYTES_PER_PIXEL;
    if src.len() < expected_src_size {
        return Vec::new();
    }

    let mut dst = vec![0u8; (dst_width as usize) * (dst_height as usize) * BYTES_PER_PIXEL];

    let x_ratio = src_width as f32 / dst_width as f32;
    let y_ratio = src_height as f32 / dst_height as f32;

    for dy in 0..dst_height {
        for dx in 0..dst_width {
            let src_x_start = (dx as f32 * x_ratio) as u32;
            let src_y_start = (dy as f32 * y_ratio) as u32;
            let src_x_end = ((dx + 1) as f32 * x_ratio).ceil() as u32;
            let src_y_end = ((dy + 1) as f32 * y_ratio).ceil() as u32;

            let src_x_end = src_x_end.min(src_width);
            let src_y_end = src_y_end.min(src_height);

            let mut r_sum: u32 = 0;
            let mut g_sum: u32 = 0;
            let mut b_sum: u32 = 0;
            let mut a_sum: u32 = 0;
            let mut count: u32 = 0;

            for sy in src_y_start..src_y_end {
                for sx in src_x_start..src_x_end {
                    let src_idx = ((sy * src_width + sx) as usize) * BYTES_PER_PIXEL;
                    r_sum += src[src_idx] as u32;
                    g_sum += src[src_idx + 1] as u32;
                    b_sum += src[src_idx + 2] as u32;
                    a_sum += src[src_idx + 3] as u32;
                    count += 1;
                }
            }

            if count > 0 {
                let dst_idx = ((dy * dst_width + dx) as usize) * BYTES_PER_PIXEL;
                dst[dst_idx] = (r_sum / count) as u8;
                dst[dst_idx + 1] = (g_sum / count) as u8;
                dst[dst_idx + 2] = (b_sum / count) as u8;
                dst[dst_idx + 3] = (a_sum / count) as u8;
            }
        }
    }

    dst
}

/// Creates a PNG-ready header for raw RGBA data.
/// Note: This creates a minimal header; actual PNG encoding requires a library.
#[must_use]
pub fn create_png_header(width: u32, height: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(33);

    // PNG signature
    header.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    // IHDR chunk
    header.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]); // Length
    header.extend_from_slice(b"IHDR");
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.push(8); // Bit depth
    header.push(6); // Color type (RGBA)
    header.push(0); // Compression
    header.push(0); // Filter
    header.push(0); // Interlace

    // CRC placeholder (would need actual CRC32 calculation)
    header.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_status_default() {
        let status = CaptureStatus::default();
        assert_eq!(status, CaptureStatus::Idle);
        assert!(!status.is_busy());
        assert!(!status.is_ready());
    }

    #[test]
    fn test_capture_status_busy() {
        assert!(CaptureStatus::Pending.is_busy());
        assert!(CaptureStatus::Processing.is_busy());
        assert!(!CaptureStatus::Ready.is_busy());
    }

    #[test]
    fn test_capture_status_ready() {
        assert!(CaptureStatus::Ready.is_ready());
        assert!(!CaptureStatus::Pending.is_ready());
    }

    #[test]
    fn test_capture_status_failed() {
        assert!(CaptureStatus::Failed.is_failed());
        assert!(!CaptureStatus::Ready.is_failed());
    }

    #[test]
    fn test_screenshot_quality_jpeg() {
        assert_eq!(ScreenshotQuality::Low.jpeg_quality(), 50);
        assert_eq!(ScreenshotQuality::Medium.jpeg_quality(), 75);
        assert_eq!(ScreenshotQuality::High.jpeg_quality(), 90);
        assert_eq!(ScreenshotQuality::Maximum.jpeg_quality(), 100);
    }

    #[test]
    fn test_screenshot_quality_png() {
        assert_eq!(ScreenshotQuality::Low.png_compression(), 9);
        assert_eq!(ScreenshotQuality::Medium.png_compression(), 6);
        assert_eq!(ScreenshotQuality::Maximum.png_compression(), 0);
    }

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert!(config.generate_thumbnail);
        assert_eq!(config.thumbnail_width, THUMBNAIL_WIDTH);
        assert_eq!(config.thumbnail_height, THUMBNAIL_HEIGHT);
    }

    #[test]
    fn test_capture_config_thumbnail_only() {
        let config = CaptureConfig::thumbnail_only();
        assert!(config.generate_thumbnail);
    }

    #[test]
    fn test_capture_config_full_resolution() {
        let config = CaptureConfig::full_resolution();
        assert!(!config.generate_thumbnail);
    }

    #[test]
    fn test_capture_config_buffer_sizes() {
        let config = CaptureConfig::default();
        assert_eq!(
            config.thumbnail_buffer_size(),
            (THUMBNAIL_WIDTH as usize) * (THUMBNAIL_HEIGHT as usize) * BYTES_PER_PIXEL
        );
        assert_eq!(
            config.full_buffer_size(1920, 1080),
            1920 * 1080 * BYTES_PER_PIXEL
        );
    }

    #[test]
    fn test_capture_request_default() {
        let request = CaptureRequest::default();
        assert_eq!(request.src_x, 0);
        assert_eq!(request.src_y, 0);
        assert_eq!(request.dst_width, THUMBNAIL_WIDTH);
        assert_eq!(request.dst_height, THUMBNAIL_HEIGHT);
    }

    #[test]
    fn test_capture_request_full_framebuffer() {
        let request = CaptureRequest::full_framebuffer(1920, 1080);
        assert_eq!(request.src_width, 1920);
        assert_eq!(request.dst_width, 1920);
    }

    #[test]
    fn test_capture_request_thumbnail() {
        let request = CaptureRequest::thumbnail(1920, 1080);
        assert_eq!(request.src_width, 1920);
        assert_eq!(request.dst_width, THUMBNAIL_WIDTH);
        assert_eq!(request.dst_height, THUMBNAIL_HEIGHT);
    }

    #[test]
    fn test_capture_request_region() {
        let request = CaptureRequest::region(100, 100, 200, 200);
        assert_eq!(request.src_x, 100);
        assert_eq!(request.src_y, 100);
        assert_eq!(request.src_width, 200);
    }

    #[test]
    fn test_capture_request_flags() {
        let request = CaptureRequest::default().with_flip(true).with_ui(true);
        assert!(request.is_flipped());
        assert!(request.includes_ui());

        let request2 = request.with_flip(false);
        assert!(!request2.is_flipped());
        assert!(request2.includes_ui());
    }

    #[test]
    fn test_capture_request_output_size() {
        let request = CaptureRequest::thumbnail(1920, 1080);
        assert_eq!(
            request.output_size(),
            (THUMBNAIL_WIDTH as usize) * (THUMBNAIL_HEIGHT as usize) * BYTES_PER_PIXEL
        );
    }

    #[test]
    fn test_capture_request_valid() {
        let valid = CaptureRequest::thumbnail(1920, 1080);
        assert!(valid.is_valid());

        let invalid = CaptureRequest {
            src_width: 0,
            ..Default::default()
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_screenshot_data_new() {
        let pixels = vec![255u8; 16 * BYTES_PER_PIXEL];
        let data = ScreenshotData::new(4, 4, pixels);
        assert_eq!(data.width, 4);
        assert_eq!(data.height, 4);
        assert!(data.is_valid());
    }

    #[test]
    fn test_screenshot_data_with_capacity() {
        let data = ScreenshotData::with_capacity(4, 4);
        assert_eq!(data.pixels.len(), 16 * BYTES_PER_PIXEL);
        assert!(data.is_valid());
    }

    #[test]
    fn test_screenshot_data_get_set_pixel() {
        let mut data = ScreenshotData::with_capacity(4, 4);
        data.set_pixel(1, 1, [255, 128, 64, 255]);
        let pixel = data.get_pixel(1, 1);
        assert_eq!(pixel, Some([255, 128, 64, 255]));
    }

    #[test]
    fn test_screenshot_data_get_pixel_out_of_bounds() {
        let data = ScreenshotData::with_capacity(4, 4);
        assert_eq!(data.get_pixel(10, 10), None);
    }

    #[test]
    fn test_screenshot_data_flip_vertical() {
        let mut data = ScreenshotData::with_capacity(2, 2);
        data.set_pixel(0, 0, [1, 0, 0, 255]); // Top-left
        data.set_pixel(0, 1, [2, 0, 0, 255]); // Bottom-left
        data.flip_vertical();
        assert_eq!(data.get_pixel(0, 0), Some([2, 0, 0, 255]));
        assert_eq!(data.get_pixel(0, 1), Some([1, 0, 0, 255]));
    }

    #[test]
    fn test_screenshot_manager_default() {
        let manager = ScreenshotManager::new();
        assert_eq!(manager.status(), CaptureStatus::Idle);
        assert!(!manager.is_busy());
    }

    #[test]
    fn test_screenshot_manager_capture() {
        let mut manager = ScreenshotManager::new();
        manager.capture_thumbnail(1920, 1080);
        assert_eq!(manager.status(), CaptureStatus::Pending);
        assert!(manager.is_busy());
    }

    #[test]
    fn test_screenshot_manager_pending_request() {
        let mut manager = ScreenshotManager::new();
        manager.capture_thumbnail(1920, 1080);
        assert!(manager.pending_request().is_some());
    }

    #[test]
    fn test_screenshot_manager_update() {
        let mut manager = ScreenshotManager::new();
        manager.set_readback_delay(2);
        manager.capture_thumbnail(1920, 1080);
        manager.update();
        assert_eq!(manager.status(), CaptureStatus::Pending);
        manager.update();
        assert_eq!(manager.status(), CaptureStatus::Processing);
    }

    #[test]
    fn test_screenshot_manager_receive_data() {
        let mut manager = ScreenshotManager::new();
        manager.set_readback_delay(1);
        manager.capture_thumbnail(1920, 1080);
        manager.update();

        let pixels =
            vec![0u8; (THUMBNAIL_WIDTH as usize) * (THUMBNAIL_HEIGHT as usize) * BYTES_PER_PIXEL];
        manager.receive_data(pixels);
        assert_eq!(manager.status(), CaptureStatus::Ready);
    }

    #[test]
    fn test_screenshot_manager_take_thumbnail() {
        let mut manager = ScreenshotManager::new();
        manager.set_readback_delay(1);
        manager.capture_thumbnail(1920, 1080);
        manager.update();

        let pixels =
            vec![0u8; (THUMBNAIL_WIDTH as usize) * (THUMBNAIL_HEIGHT as usize) * BYTES_PER_PIXEL];
        manager.receive_data(pixels);

        let data = manager.take_thumbnail();
        assert!(data.is_some());
        assert_eq!(manager.status(), CaptureStatus::Idle);
    }

    #[test]
    fn test_screenshot_manager_reset() {
        let mut manager = ScreenshotManager::new();
        manager.capture_thumbnail(1920, 1080);
        manager.reset();
        assert_eq!(manager.status(), CaptureStatus::Idle);
    }

    #[test]
    fn test_screenshot_manager_mark_failed() {
        let mut manager = ScreenshotManager::new();
        manager.capture_thumbnail(1920, 1080);
        manager.mark_failed();
        assert_eq!(manager.status(), CaptureStatus::Failed);
    }

    #[test]
    fn test_downsample_box() {
        let src = vec![255u8; 4 * 4 * BYTES_PER_PIXEL];
        let dst = downsample_box(&src, 4, 4, 2, 2);
        assert_eq!(dst.len(), 2 * 2 * BYTES_PER_PIXEL);
        // All pixels should be 255 since source is uniform
        assert!(dst.iter().all(|&x| x == 255));
    }

    #[test]
    fn test_downsample_box_empty() {
        let dst = downsample_box(&[], 0, 0, 0, 0);
        assert!(dst.is_empty());
    }

    #[test]
    fn test_downsample_box_insufficient_data() {
        let src = vec![255u8; 8]; // Too small
        let dst = downsample_box(&src, 4, 4, 2, 2);
        assert!(dst.is_empty());
    }

    #[test]
    fn test_create_png_header() {
        let header = create_png_header(256, 144);
        // Check PNG signature
        assert_eq!(&header[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        // Check IHDR marker
        assert_eq!(&header[12..16], b"IHDR");
    }

    #[test]
    fn test_capture_request_size() {
        assert_eq!(std::mem::size_of::<CaptureRequest>(), 32);
    }

    #[test]
    fn test_screenshot_manager_busy_rejection() {
        let mut manager = ScreenshotManager::new();
        manager.capture_thumbnail(1920, 1080);
        let status_before = manager.status();
        manager.capture_full(1920, 1080); // Should be ignored
        assert_eq!(manager.status(), status_before);
    }
}
