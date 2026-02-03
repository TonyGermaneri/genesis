//! GPU validation harness.
//!
//! This module provides validation infrastructure for GPU compute operations:
//! - Runtime validation checks for cell state
//! - wgpu validation layer integration (debug builds only)
//! - Error callback handling and logging

use tracing::{error, info, warn};

/// Returns wgpu instance flags with validation enabled for debug builds.
///
/// In debug mode, this enables:
/// - `VALIDATION`: GPU API validation to catch errors early
/// - `DEBUG`: Additional debug info
///
/// In release mode, validation is disabled for performance.
#[must_use]
pub fn gpu_instance_flags() -> wgpu::InstanceFlags {
    if cfg!(debug_assertions) {
        info!("GPU validation layer enabled (debug build)");
        wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::DEBUG
    } else {
        info!("GPU validation layer disabled (release build)");
        wgpu::InstanceFlags::empty()
    }
}

/// Creates a wgpu instance with appropriate validation settings.
///
/// In debug builds, validation is enabled to catch GPU errors early.
/// In release builds, validation is disabled for maximum performance.
#[must_use]
pub fn create_validated_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: gpu_instance_flags(),
        ..Default::default()
    })
}

/// Handles wgpu device errors by logging them.
///
/// This callback is invoked when the GPU device encounters an error.
/// Use this with `device.on_uncaptured_error()`.
pub fn handle_device_error(error: &wgpu::Error) {
    error!("GPU device error: {error}");
}

/// GPU validation result.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation messages
    pub messages: Vec<String>,
    /// Frame number when validated
    pub frame: u64,
}

impl ValidationResult {
    /// Creates a passing result.
    #[must_use]
    pub fn pass(frame: u64) -> Self {
        Self {
            passed: true,
            messages: vec![],
            frame,
        }
    }

    /// Creates a failing result.
    #[must_use]
    pub fn fail(frame: u64, message: impl Into<String>) -> Self {
        Self {
            passed: false,
            messages: vec![message.into()],
            frame,
        }
    }
}

/// GPU validation harness for debugging and testing.
pub struct ValidationHarness {
    /// Whether validation is enabled
    enabled: bool,
    /// Collected validation results
    results: Vec<ValidationResult>,
    /// Current frame counter
    frame: u64,
}

impl Default for ValidationHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationHarness {
    /// Creates a new validation harness.
    #[must_use]
    pub fn new() -> Self {
        Self {
            enabled: cfg!(debug_assertions),
            results: Vec::new(),
            frame: 0,
        }
    }

    /// Enables or disables validation.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("GPU validation enabled");
        } else {
            warn!("GPU validation disabled");
        }
    }

    /// Returns whether validation is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Records the start of a new frame.
    pub fn begin_frame(&mut self) {
        self.frame += 1;
    }

    /// Validates a condition, recording the result.
    pub fn validate(&mut self, condition: bool, message: &str) -> bool {
        if !self.enabled {
            return condition;
        }

        if condition {
            self.results.push(ValidationResult::pass(self.frame));
        } else {
            error!("GPU validation failed: {message}");
            self.results
                .push(ValidationResult::fail(self.frame, message));
        }

        condition
    }

    /// Returns all validation results.
    #[must_use]
    pub fn results(&self) -> &[ValidationResult] {
        &self.results
    }

    /// Clears all validation results.
    pub fn clear(&mut self) {
        self.results.clear();
    }

    /// Returns the number of failures.
    #[must_use]
    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_pass() {
        let mut harness = ValidationHarness::new();
        harness.set_enabled(true);
        harness.begin_frame();

        assert!(harness.validate(true, "should pass"));
        assert_eq!(harness.failure_count(), 0);
    }

    #[test]
    fn test_validation_fail() {
        let mut harness = ValidationHarness::new();
        harness.set_enabled(true);
        harness.begin_frame();

        assert!(!harness.validate(false, "should fail"));
        assert_eq!(harness.failure_count(), 1);
    }
}
