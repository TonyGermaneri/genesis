//! Automated screenshot testing for visual regression.
//!
//! This module provides:
//! - Rendering to image buffers
//! - Golden screenshot comparison
//! - Pixel difference reporting
//! - Visual diff generation

use crate::test_harness::{HarnessError, HarnessResult, TestHarness};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// RGBA pixel data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    /// Red channel (0-255)
    pub r: u8,
    /// Green channel (0-255)
    pub g: u8,
    /// Blue channel (0-255)
    pub b: u8,
    /// Alpha channel (0-255)
    pub a: u8,
}

impl Pixel {
    /// Creates a new pixel.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a fully opaque pixel.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Black pixel.
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// White pixel.
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// Red pixel (for diff visualization).
    pub const RED: Self = Self::rgb(255, 0, 0);
    /// Green pixel (for diff visualization).
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    /// Transparent pixel.
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    /// Calculates the difference from another pixel (0.0 - 1.0).
    #[must_use]
    pub fn difference(&self, other: &Self) -> f32 {
        let dr = (i32::from(self.r) - i32::from(other.r)).abs();
        let dg = (i32::from(self.g) - i32::from(other.g)).abs();
        let db = (i32::from(self.b) - i32::from(other.b)).abs();
        let da = (i32::from(self.a) - i32::from(other.a)).abs();

        // Normalized difference (max possible diff = 255*4 = 1020)
        (dr + dg + db + da) as f32 / 1020.0
    }

    /// Checks if this pixel is "similar" to another within a tolerance.
    #[must_use]
    pub fn is_similar(&self, other: &Self, tolerance: f32) -> bool {
        self.difference(other) <= tolerance
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Self::BLACK
    }
}

/// An image buffer for screenshot operations.
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Pixel data (row-major, RGBA)
    pixels: Vec<Pixel>,
}

impl ImageBuffer {
    /// Creates a new image buffer filled with a color.
    #[must_use]
    pub fn new(width: u32, height: u32, fill: Pixel) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            pixels: vec![fill; size],
        }
    }

    /// Creates a new black image buffer.
    #[must_use]
    pub fn new_black(width: u32, height: u32) -> Self {
        Self::new(width, height, Pixel::BLACK)
    }

    /// Returns the width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Gets a pixel at (x, y).
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> Option<&Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        self.pixels.get(idx)
    }

    /// Gets a mutable pixel at (x, y).
    #[must_use]
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        self.pixels.get_mut(idx)
    }

    /// Sets a pixel at (x, y).
    pub fn set(&mut self, x: u32, y: u32, pixel: Pixel) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.pixels[idx] = pixel;
        }
    }

    /// Returns the raw pixel data.
    #[must_use]
    pub fn pixels(&self) -> &[Pixel] {
        &self.pixels
    }

    /// Returns the raw pixel data as bytes (RGBA).
    #[must_use]
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 4);
        for pixel in &self.pixels {
            bytes.push(pixel.r);
            bytes.push(pixel.g);
            bytes.push(pixel.b);
            bytes.push(pixel.a);
        }
        bytes
    }

    /// Creates an image buffer from RGBA bytes.
    #[must_use]
    pub fn from_rgba_bytes(width: u32, height: u32, bytes: &[u8]) -> Option<Self> {
        let expected_len = (width * height * 4) as usize;
        if bytes.len() != expected_len {
            return None;
        }

        let pixels: Vec<Pixel> = bytes
            .chunks(4)
            .map(|chunk| Pixel::new(chunk[0], chunk[1], chunk[2], chunk[3]))
            .collect();

        Some(Self {
            width,
            height,
            pixels,
        })
    }

    /// Fills a rectangle with a color.
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Pixel) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x + dx, y + dy, color);
            }
        }
    }

    /// Draws a simple cell visualization (for testing).
    pub fn draw_cell(&mut self, x: u32, y: u32, size: u32, material_color: Pixel) {
        self.fill_rect(x, y, size, size, material_color);
    }
}

/// Result of comparing two images.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageComparison {
    /// Width of both images
    pub width: u32,
    /// Height of both images
    pub height: u32,
    /// Total number of pixels
    pub total_pixels: u64,
    /// Number of different pixels
    pub different_pixels: u64,
    /// Percentage of pixels that differ (0.0 - 1.0)
    pub diff_percentage: f64,
    /// Maximum pixel difference found
    pub max_difference: f32,
    /// Average pixel difference
    pub avg_difference: f32,
    /// Whether the comparison passed the threshold
    pub passed: bool,
}

impl ImageComparison {
    /// Creates a comparison indicating matched images.
    #[must_use]
    pub fn identical(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            total_pixels: u64::from(width) * u64::from(height),
            different_pixels: 0,
            diff_percentage: 0.0,
            max_difference: 0.0,
            avg_difference: 0.0,
            passed: true,
        }
    }

    /// Returns a human-readable report.
    #[must_use]
    pub fn to_report(&self) -> String {
        if self.passed {
            format!(
                "✓ Images match ({} pixels, {:.2}% diff)",
                self.total_pixels,
                self.diff_percentage * 100.0
            )
        } else {
            format!(
                "✗ Images differ: {} of {} pixels ({:.2}%), max diff: {:.3}, avg diff: {:.3}",
                self.different_pixels,
                self.total_pixels,
                self.diff_percentage * 100.0,
                self.max_difference,
                self.avg_difference
            )
        }
    }
}

/// Configuration for screenshot comparison.
#[derive(Debug, Clone)]
pub struct ScreenshotConfig {
    /// Threshold for pixel difference (0.0 - 1.0)
    pub pixel_threshold: f32,
    /// Maximum percentage of pixels allowed to differ
    pub max_diff_percentage: f64,
    /// Whether to generate diff images
    pub generate_diff_image: bool,
    /// Directory for golden files
    pub golden_dir: String,
    /// Directory for diff outputs
    pub diff_dir: String,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            pixel_threshold: 0.01,
            max_diff_percentage: 0.01,
            generate_diff_image: true,
            golden_dir: "tests/golden".to_string(),
            diff_dir: "tests/diff".to_string(),
        }
    }
}

/// Screenshot test runner.
#[derive(Debug)]
pub struct ScreenshotTest {
    /// Configuration
    pub config: ScreenshotConfig,
}

impl Default for ScreenshotTest {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenshotTest {
    /// Creates a new screenshot test runner.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ScreenshotConfig::default(),
        }
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: ScreenshotConfig) -> Self {
        Self { config }
    }

    /// Compares two images.
    #[must_use]
    pub fn compare_images(&self, actual: &ImageBuffer, expected: &ImageBuffer) -> ImageComparison {
        // Check dimensions
        if actual.width() != expected.width() || actual.height() != expected.height() {
            return ImageComparison {
                width: actual.width(),
                height: actual.height(),
                total_pixels: u64::from(actual.width()) * u64::from(actual.height()),
                different_pixels: u64::from(actual.width()) * u64::from(actual.height()),
                diff_percentage: 1.0,
                max_difference: 1.0,
                avg_difference: 1.0,
                passed: false,
            };
        }

        let total = actual.pixels().len();
        let mut different = 0u64;
        let mut max_diff = 0.0f32;
        let mut total_diff = 0.0f64;

        for (a, e) in actual.pixels().iter().zip(expected.pixels().iter()) {
            let diff = a.difference(e);
            if diff > self.config.pixel_threshold {
                different += 1;
                max_diff = max_diff.max(diff);
            }
            total_diff += f64::from(diff);
        }

        let diff_percentage = different as f64 / total as f64;
        let avg_diff = (total_diff / total as f64) as f32;
        let passed = diff_percentage <= self.config.max_diff_percentage;

        ImageComparison {
            width: actual.width(),
            height: actual.height(),
            total_pixels: total as u64,
            different_pixels: different,
            diff_percentage,
            max_difference: max_diff,
            avg_difference: avg_diff,
            passed,
        }
    }

    /// Generates a diff image highlighting differences.
    #[must_use]
    pub fn generate_diff(&self, actual: &ImageBuffer, expected: &ImageBuffer) -> ImageBuffer {
        let width = actual.width().max(expected.width());
        let height = actual.height().max(expected.height());
        let mut diff = ImageBuffer::new(width, height, Pixel::BLACK);

        for y in 0..height {
            for x in 0..width {
                let actual_pixel = actual.get(x, y).copied().unwrap_or(Pixel::TRANSPARENT);
                let expected_pixel = expected.get(x, y).copied().unwrap_or(Pixel::TRANSPARENT);

                let pixel_diff = actual_pixel.difference(&expected_pixel);
                if pixel_diff > self.config.pixel_threshold {
                    // Red for different pixels, intensity based on difference
                    let intensity = (pixel_diff * 255.0) as u8;
                    diff.set(x, y, Pixel::rgb(255, intensity, intensity));
                } else {
                    // Green tint for matching pixels
                    diff.set(x, y, Pixel::rgb(0, 64, 0));
                }
            }
        }

        diff
    }

    /// Loads a golden image from file (simplified - just metadata).
    pub fn load_golden(&self, name: &str) -> HarnessResult<GoldenImage> {
        let path = Path::new(&self.config.golden_dir).join(format!("{name}.golden.json"));
        if !path.exists() {
            return Err(HarnessError::GoldenNotFound(path.display().to_string()));
        }

        let json = std::fs::read_to_string(&path)?;
        serde_json::from_str(&json).map_err(|e| HarnessError::SerializationError(e.to_string()))
    }

    /// Saves a golden image (metadata).
    pub fn save_golden(&self, name: &str, image: &ImageBuffer) -> HarnessResult<()> {
        let dir = Path::new(&self.config.golden_dir);
        std::fs::create_dir_all(dir)?;

        let golden = GoldenImage {
            name: name.to_string(),
            width: image.width(),
            height: image.height(),
            checksum: calculate_checksum(image),
        };

        let path = dir.join(format!("{name}.golden.json"));
        let json = serde_json::to_string_pretty(&golden)
            .map_err(|e| HarnessError::SerializationError(e.to_string()))?;
        std::fs::write(path, json)?;

        // Also save raw image data
        let data_path = dir.join(format!("{name}.golden.raw"));
        std::fs::write(data_path, image.to_rgba_bytes())?;

        Ok(())
    }

    /// Runs a screenshot test.
    pub fn run_test(&self, name: &str, actual: &ImageBuffer) -> HarnessResult<ImageComparison> {
        // Try to load golden
        match self.load_golden(name) {
            Ok(golden) => {
                // Load the raw image data
                let data_path =
                    Path::new(&self.config.golden_dir).join(format!("{name}.golden.raw"));
                let bytes = std::fs::read(&data_path)?;
                let expected = ImageBuffer::from_rgba_bytes(golden.width, golden.height, &bytes)
                    .ok_or_else(|| {
                        HarnessError::SerializationError("Invalid golden image data".to_string())
                    })?;

                let comparison = self.compare_images(actual, &expected);

                // Generate diff if needed and test failed
                if !comparison.passed && self.config.generate_diff_image {
                    let diff_dir = Path::new(&self.config.diff_dir);
                    std::fs::create_dir_all(diff_dir)?;

                    let diff_image = self.generate_diff(actual, &expected);
                    let diff_path = diff_dir.join(format!("{name}.diff.raw"));
                    std::fs::write(diff_path, diff_image.to_rgba_bytes())?;
                }

                if comparison.passed {
                    Ok(comparison)
                } else {
                    Err(HarnessError::GoldenMismatch(comparison.to_report()))
                }
            },
            Err(HarnessError::GoldenNotFound(_)) => {
                // No golden exists - save current as golden
                tracing::warn!("Golden not found for '{name}', saving current as golden");
                self.save_golden(name, actual)?;
                Ok(ImageComparison::identical(actual.width(), actual.height()))
            },
            Err(e) => Err(e),
        }
    }
}

/// Golden image metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenImage {
    /// Test name
    pub name: String,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Checksum of pixel data
    pub checksum: u64,
}

/// Calculates a checksum for an image.
fn calculate_checksum(image: &ImageBuffer) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    image.width().hash(&mut hasher);
    image.height().hash(&mut hasher);
    for pixel in image.pixels() {
        pixel.r.hash(&mut hasher);
        pixel.g.hash(&mut hasher);
        pixel.b.hash(&mut hasher);
        pixel.a.hash(&mut hasher);
    }
    hasher.finish()
}

/// Renders a simple chunk visualization for testing.
pub fn render_chunk_preview(
    harness: &TestHarness,
    chunk_coord: genesis_common::ChunkCoord,
    cell_size: u32,
) -> Option<ImageBuffer> {
    let chunk = harness.get_chunk(chunk_coord)?;
    let chunk_size = chunk.size();
    let image_size = chunk_size * cell_size;

    let mut image = ImageBuffer::new_black(image_size, image_size);

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            if let Some(cell) = chunk.get_cell(x, y) {
                // Simple material-to-color mapping
                let color = material_to_color(cell.material);
                let px = x * cell_size;
                let py = y * cell_size;
                image.draw_cell(px, py, cell_size, color);
            }
        }
    }

    Some(image)
}

/// Maps a material ID to a color (simplified).
fn material_to_color(material: u16) -> Pixel {
    match material {
        0 => Pixel::rgb(30, 30, 30),    // Air - dark gray
        1 => Pixel::rgb(139, 90, 43),   // Dirt - brown
        2 => Pixel::rgb(128, 128, 128), // Stone - gray
        3 => Pixel::rgb(34, 139, 34),   // Grass - green
        4 => Pixel::rgb(30, 144, 255),  // Water - blue
        5 => Pixel::rgb(238, 214, 175), // Sand - tan
        6 => Pixel::rgb(255, 69, 0),    // Lava - orange-red
        _ => Pixel::rgb(255, 0, 255),   // Unknown - magenta
    }
}

/// Convenience function for screenshot testing.
pub fn screenshot_test(name: &str, harness: &TestHarness) -> HarnessResult<ImageComparison> {
    let chunk_coord = genesis_common::ChunkCoord::new(0, 0);
    let image = render_chunk_preview(harness, chunk_coord, 1)
        .ok_or_else(|| HarnessError::AssertionFailed("No chunk at (0,0) to render".to_string()))?;

    let tester = ScreenshotTest::new();
    tester.run_test(name, &image)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_creation() {
        let p = Pixel::rgb(255, 128, 64);
        assert_eq!(p.r, 255);
        assert_eq!(p.g, 128);
        assert_eq!(p.b, 64);
        assert_eq!(p.a, 255);
    }

    #[test]
    fn test_pixel_difference() {
        let white = Pixel::WHITE;
        let black = Pixel::BLACK;

        // Same pixel has 0 difference
        assert!((white.difference(&white) - 0.0).abs() < 0.001);

        // White vs black should have significant difference
        let diff = white.difference(&black);
        assert!(diff > 0.5);
    }

    #[test]
    fn test_pixel_similarity() {
        let p1 = Pixel::rgb(100, 100, 100);
        let p2 = Pixel::rgb(102, 100, 100);

        // Small difference should be similar with reasonable tolerance
        assert!(p1.is_similar(&p2, 0.01));

        // Large difference should not be similar
        assert!(!p1.is_similar(&Pixel::WHITE, 0.1));
    }

    #[test]
    fn test_image_buffer_creation() {
        let img = ImageBuffer::new_black(10, 20);
        assert_eq!(img.width(), 10);
        assert_eq!(img.height(), 20);
    }

    #[test]
    fn test_image_buffer_get_set() {
        let mut img = ImageBuffer::new_black(10, 10);

        img.set(5, 5, Pixel::WHITE);
        assert_eq!(img.get(5, 5), Some(&Pixel::WHITE));
        assert_eq!(img.get(0, 0), Some(&Pixel::BLACK));
    }

    #[test]
    fn test_image_buffer_bounds() {
        let img = ImageBuffer::new_black(10, 10);

        assert!(img.get(9, 9).is_some());
        assert!(img.get(10, 10).is_none());
        assert!(img.get(100, 100).is_none());
    }

    #[test]
    fn test_image_buffer_rgba_roundtrip() {
        let mut img = ImageBuffer::new(4, 4, Pixel::rgb(100, 150, 200));
        img.set(0, 0, Pixel::RED);
        img.set(3, 3, Pixel::GREEN);

        let bytes = img.to_rgba_bytes();
        let restored = ImageBuffer::from_rgba_bytes(4, 4, &bytes);

        assert!(restored.is_some());
        let restored = restored.expect("failed to restore");
        assert_eq!(restored.get(0, 0), Some(&Pixel::RED));
        assert_eq!(restored.get(3, 3), Some(&Pixel::GREEN));
    }

    #[test]
    fn test_image_comparison_identical() {
        let img1 = ImageBuffer::new(10, 10, Pixel::rgb(100, 100, 100));
        let img2 = ImageBuffer::new(10, 10, Pixel::rgb(100, 100, 100));

        let tester = ScreenshotTest::new();
        let result = tester.compare_images(&img1, &img2);

        assert!(result.passed);
        assert_eq!(result.different_pixels, 0);
    }

    #[test]
    fn test_image_comparison_different() {
        let img1 = ImageBuffer::new(10, 10, Pixel::WHITE);
        let img2 = ImageBuffer::new(10, 10, Pixel::BLACK);

        let tester = ScreenshotTest::new();
        let result = tester.compare_images(&img1, &img2);

        assert!(!result.passed);
        assert_eq!(result.different_pixels, 100);
    }

    #[test]
    fn test_image_comparison_size_mismatch() {
        let img1 = ImageBuffer::new(10, 10, Pixel::WHITE);
        let img2 = ImageBuffer::new(20, 20, Pixel::WHITE);

        let tester = ScreenshotTest::new();
        let result = tester.compare_images(&img1, &img2);

        assert!(!result.passed);
    }

    #[test]
    fn test_generate_diff() {
        let img1 = ImageBuffer::new(10, 10, Pixel::WHITE);
        let mut img2 = ImageBuffer::new(10, 10, Pixel::WHITE);
        img2.set(5, 5, Pixel::BLACK);

        let tester = ScreenshotTest::new();
        let diff = tester.generate_diff(&img1, &img2);

        // Diff should highlight the changed pixel
        let diff_pixel = diff.get(5, 5);
        assert!(diff_pixel.is_some());
        // Changed pixels are shown in red tones
        assert!(diff_pixel.map(|p| p.r).unwrap_or(0) > 0);
    }

    #[test]
    fn test_screenshot_config_defaults() {
        let config = ScreenshotConfig::default();
        assert!(config.pixel_threshold > 0.0);
        assert!(config.max_diff_percentage > 0.0);
        assert!(config.generate_diff_image);
    }

    #[test]
    fn test_image_comparison_report() {
        let comparison = ImageComparison::identical(100, 100);
        let report = comparison.to_report();
        assert!(report.contains("✓"));

        let failed_comparison = ImageComparison {
            width: 100,
            height: 100,
            total_pixels: 10000,
            different_pixels: 500,
            diff_percentage: 0.05,
            max_difference: 0.5,
            avg_difference: 0.1,
            passed: false,
        };
        let report = failed_comparison.to_report();
        assert!(report.contains("✗"));
        assert!(report.contains("500"));
    }

    #[test]
    fn test_material_to_color() {
        // Air should be dark
        let air = material_to_color(0);
        assert!(air.r < 50 && air.g < 50 && air.b < 50);

        // Water should be blue
        let water = material_to_color(4);
        assert!(water.b > water.r);
        assert!(water.b > water.g);

        // Unknown materials should be magenta
        let unknown = material_to_color(999);
        assert_eq!(unknown.r, 255);
        assert_eq!(unknown.b, 255);
    }

    #[test]
    fn test_fill_rect() {
        let mut img = ImageBuffer::new_black(20, 20);
        img.fill_rect(5, 5, 10, 10, Pixel::WHITE);

        // Inside rectangle should be white
        assert_eq!(img.get(5, 5), Some(&Pixel::WHITE));
        assert_eq!(img.get(14, 14), Some(&Pixel::WHITE));

        // Outside should be black
        assert_eq!(img.get(0, 0), Some(&Pixel::BLACK));
        assert_eq!(img.get(19, 19), Some(&Pixel::BLACK));
    }

    #[test]
    fn test_checksum() {
        let img1 = ImageBuffer::new(10, 10, Pixel::WHITE);
        let img2 = ImageBuffer::new(10, 10, Pixel::WHITE);
        let img3 = ImageBuffer::new(10, 10, Pixel::BLACK);

        let sum1 = calculate_checksum(&img1);
        let sum2 = calculate_checksum(&img2);
        let sum3 = calculate_checksum(&img3);

        // Identical images should have same checksum
        assert_eq!(sum1, sum2);

        // Different images should have different checksums
        assert_ne!(sum1, sum3);
    }

    #[test]
    fn test_render_chunk_preview() {
        use genesis_common::ChunkCoord;

        let mut harness = TestHarness::new_headless();
        harness.load_chunk(ChunkCoord::new(0, 0));

        let image = render_chunk_preview(&harness, ChunkCoord::new(0, 0), 1);
        assert!(image.is_some());

        let image = image.expect("should have image");
        assert_eq!(image.width(), 64); // Default chunk size
        assert_eq!(image.height(), 64);
    }
}
