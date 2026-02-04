//! Crash Reporting System
//!
//! Captures panics with backtraces and system info for debugging.

use serde::Serialize;
use std::fs;
use std::io::Write;
use std::panic::{self, PanicHookInfo};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use tracing::{error, info, warn};

/// Global flag to prevent recursive panics
static HANDLING_PANIC: AtomicBool = AtomicBool::new(false);

/// Errors that can occur during crash reporting
#[derive(Debug, Error)]
pub enum ReportError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Network error during upload
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Server returned error
    #[error("Server error: HTTP {0}")]
    ServerError(u16),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// System information at time of crash
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    /// Operating system version
    pub os_version: String,
    /// CPU information
    pub cpu: String,
    /// RAM in megabytes
    pub ram_mb: u64,
    /// GPU information (if available)
    pub gpu: Option<String>,
    /// Architecture (x86_64, aarch64, etc.)
    pub arch: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::collect()
    }
}

impl SystemInfo {
    /// Collect system information
    pub fn collect() -> Self {
        Self {
            os_version: Self::get_os_version(),
            cpu: Self::get_cpu_info(),
            ram_mb: Self::get_ram_mb(),
            gpu: None, // Would require wgpu/graphics context
            arch: std::env::consts::ARCH.to_string(),
        }
    }

    fn get_os_version() -> String {
        format!("{} {}", std::env::consts::OS, std::env::consts::FAMILY)
    }

    fn get_cpu_info() -> String {
        // Basic CPU info - would need sysinfo crate for detailed info
        format!("{} CPU", std::env::consts::ARCH)
    }

    fn get_ram_mb() -> u64 {
        // Would need sysinfo crate for actual RAM
        0
    }
}

/// A crash report containing all relevant debugging information
#[derive(Debug, Clone, Serialize)]
pub struct CrashReport {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Application version
    pub app_version: String,
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// The panic message
    pub panic_message: String,
    /// Location where panic occurred
    pub panic_location: Option<String>,
    /// Stack backtrace
    pub backtrace: String,
    /// System information
    pub system_info: SystemInfo,
    /// Recent log entries
    pub recent_logs: Vec<String>,
    /// Build information
    pub build_info: BuildInfo,
}

/// Build-time information
#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    /// Rust version used to compile
    pub rustc_version: String,
    /// Build target triple
    pub target: String,
    /// Debug or release build
    pub profile: String,
    /// Git commit hash (if available)
    pub git_hash: Option<String>,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            rustc_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            target: std::env::consts::ARCH.to_string(),
            profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
            .to_string(),
            git_hash: option_env!("GIT_HASH").map(String::from),
        }
    }
}

/// Crash reporter that captures and persists crash information
pub struct CrashReporter {
    report_dir: PathBuf,
    app_version: String,
    enabled: bool,
    upload_endpoint: Option<String>,
    log_buffer: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl CrashReporter {
    /// Create a new crash reporter
    ///
    /// # Arguments
    /// * `report_dir` - Directory to store crash reports
    /// * `app_version` - Application version string
    pub fn new(report_dir: impl AsRef<Path>, app_version: &str) -> Self {
        let report_dir = report_dir.as_ref().to_path_buf();

        // Create report directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&report_dir) {
            warn!("Failed to create crash report directory: {}", e);
        }

        Self {
            report_dir,
            app_version: app_version.to_string(),
            enabled: true,
            upload_endpoint: None,
            log_buffer: std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(100))),
        }
    }

    /// Enable or disable crash reporting
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if crash reporting is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set the upload endpoint for crash reports
    pub fn set_upload_endpoint(&mut self, endpoint: Option<String>) {
        self.upload_endpoint = endpoint;
    }

    /// Add a log entry to the buffer (for inclusion in crash reports)
    pub fn log(&self, message: &str) {
        if let Ok(mut buffer) = self.log_buffer.lock() {
            if buffer.len() >= 100 {
                buffer.remove(0);
            }
            buffer.push(message.to_string());
        }
    }

    /// Install the panic hook
    ///
    /// This should be called early in application startup.
    pub fn install_panic_hook(self) -> std::sync::Arc<Self> {
        let reporter = std::sync::Arc::new(self);
        let reporter_clone = reporter.clone();

        panic::set_hook(Box::new(move |panic_info| {
            // Prevent recursive panics
            if HANDLING_PANIC.swap(true, Ordering::SeqCst) {
                eprintln!("Recursive panic detected, aborting");
                std::process::abort();
            }

            if reporter_clone.enabled {
                match reporter_clone.handle_panic(panic_info) {
                    Ok(path) => {
                        eprintln!("\n╔══════════════════════════════════════════╗");
                        eprintln!("║           CRASH REPORT SAVED             ║");
                        eprintln!("╠══════════════════════════════════════════╣");
                        eprintln!("║ {}", path.display());
                        eprintln!("╚══════════════════════════════════════════╝\n");
                    },
                    Err(e) => {
                        eprintln!("Failed to save crash report: {e}");
                    },
                }
            }

            // Print the panic info
            eprintln!("\n{panic_info}");

            HANDLING_PANIC.store(false, Ordering::SeqCst);
        }));

        reporter
    }

    /// Handle a panic and generate a crash report
    fn handle_panic(&self, panic_info: &PanicHookInfo<'_>) -> Result<PathBuf, ReportError> {
        let report = self.capture_crash(panic_info);
        self.write_report(&report)
    }

    /// Capture crash information from a panic
    fn capture_crash(&self, panic_info: &PanicHookInfo<'_>) -> CrashReport {
        let panic_message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let panic_location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));

        // Capture backtrace
        let backtrace = std::backtrace::Backtrace::force_capture().to_string();

        // Get recent logs
        let recent_logs = self
            .log_buffer
            .lock()
            .map(|b| b.clone())
            .unwrap_or_default();

        CrashReport {
            timestamp: chrono_lite_timestamp(),
            app_version: self.app_version.clone(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            panic_message,
            panic_location,
            backtrace,
            system_info: SystemInfo::collect(),
            recent_logs,
            build_info: BuildInfo::default(),
        }
    }

    /// Write a crash report to disk
    fn write_report(&self, report: &CrashReport) -> Result<PathBuf, ReportError> {
        let filename = format!(
            "crash-{}.json",
            report.timestamp.replace([':', '-', 'T'], "")
        );
        let path = self.report_dir.join(filename);

        let json = serde_json::to_string_pretty(report)
            .map_err(|e| ReportError::SerializationError(e.to_string()))?;

        let mut file = fs::File::create(&path)?;
        file.write_all(json.as_bytes())?;

        info!("Crash report written to: {}", path.display());
        Ok(path)
    }

    /// Get list of pending crash reports
    pub fn get_pending_reports(&self) -> Vec<PathBuf> {
        fs::read_dir(&self.report_dir)
            .map(|entries| {
                entries
                    .filter_map(std::result::Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|e| e == "json"))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Submit a crash report to the server (if endpoint is configured)
    pub fn submit_report(&self, path: &Path) -> Result<(), ReportError> {
        let Some(ref _endpoint) = self.upload_endpoint else {
            return Err(ReportError::NetworkError(
                "No upload endpoint configured".to_string(),
            ));
        };

        // Read the report
        let _data = fs::read_to_string(path)?;

        // In a real implementation, this would POST to the endpoint
        // For now, just log that we would upload
        info!("Would upload crash report: {}", path.display());

        Ok(())
    }

    /// Delete a crash report
    pub fn delete_report(&self, path: &Path) {
        if let Err(e) = fs::remove_file(path) {
            error!("Failed to delete crash report: {}", e);
        }
    }

    /// Get the number of pending crash reports
    pub fn pending_count(&self) -> usize {
        self.get_pending_reports().len()
    }
}

/// Generate a simple ISO 8601-ish timestamp without external deps
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple timestamp format: seconds since epoch
    // In production, would use chrono for proper formatting
    format!("T{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_crash_reporter_creation() {
        let dir = TempDir::new().expect("create temp dir");
        let reporter = CrashReporter::new(dir.path(), "1.0.0");
        assert!(reporter.is_enabled());
        assert_eq!(reporter.pending_count(), 0);
    }

    #[test]
    fn test_system_info_collection() {
        let info = SystemInfo::collect();
        assert!(!info.os_version.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn test_build_info() {
        let info = BuildInfo::default();
        assert!(!info.target.is_empty());
    }

    #[test]
    fn test_log_buffer() {
        let dir = TempDir::new().expect("create temp dir");
        let reporter = CrashReporter::new(dir.path(), "1.0.0");

        reporter.log("Test message 1");
        reporter.log("Test message 2");

        let buffer = reporter.log_buffer.lock().expect("lock");
        assert_eq!(buffer.len(), 2);
    }
}
