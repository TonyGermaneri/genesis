# PROMPT — Infra Agent — Iteration 4

> **Branch**: `infra-agent`
> **Focus**: Asset pipeline, localization, crash reporting, analytics

## Your Mission

Complete the following tasks. Work through them sequentially. After each task, run the validation loop. Commit after each task passes.

---

## Tasks

### I-12: Asset Pipeline (P0)
**File**: `crates/genesis-tools/src/assets.rs` and `scripts/build_assets.sh`

Implement asset loading and processing:

```rust
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

pub struct AssetManager {
    base_path: PathBuf,
    cache: HashMap<AssetId, CachedAsset>,
    manifest: AssetManifest,
}

#[derive(Serialize, Deserialize)]
pub struct AssetManifest {
    pub version: u32,
    pub assets: HashMap<String, AssetEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct AssetEntry {
    pub path: String,
    pub asset_type: AssetType,
    pub hash: String,
    pub size: u64,
    pub compressed: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum AssetType {
    Texture,
    Sound,
    Music,
    Font,
    Shader,
    Data,
    Localization,
}

pub struct CachedAsset {
    pub data: Vec<u8>,
    pub asset_type: AssetType,
    pub loaded_at: std::time::Instant,
}

impl AssetManager {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self, AssetError>;

    pub fn load(&mut self, id: &str) -> Result<&CachedAsset, AssetError>;
    pub fn load_async(&mut self, id: &str) -> AssetHandle;
    pub fn unload(&mut self, id: &str);

    pub fn preload_group(&mut self, group: &str);
    pub fn get_memory_usage(&self) -> usize;
    pub fn clear_cache(&mut self);
}

pub enum AssetError {
    NotFound(String),
    IoError(std::io::Error),
    DecompressionError,
    ManifestCorrupt,
}
```

Also create `scripts/build_assets.sh`:
```bash
#!/bin/bash
# Build and compress game assets

set -e

ASSET_DIR="${1:-assets}"
OUTPUT_DIR="${2:-target/assets}"

echo "Building assets from $ASSET_DIR to $OUTPUT_DIR"

# Create manifest, compress textures, etc.
```

Requirements:
- Manifest-based asset tracking
- LZ4 compression for large assets
- Async loading with handles
- Memory budget enforcement
- Hot reload in debug mode

### I-13: Localization System (P0)
**File**: `crates/genesis-tools/src/localization.rs`

Implement multi-language support:

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub struct Localization {
    current_locale: String,
    strings: HashMap<String, HashMap<String, String>>, // locale -> key -> value
    fallback_locale: String,
}

#[derive(Serialize, Deserialize)]
pub struct LocaleFile {
    pub locale: String,
    pub name: String,
    pub strings: HashMap<String, String>,
}

impl Localization {
    pub fn new(default_locale: &str) -> Self;

    pub fn load_locale(&mut self, path: impl AsRef<Path>) -> Result<(), LocaleError>;
    pub fn set_locale(&mut self, locale: &str) -> Result<(), LocaleError>;
    pub fn get_locale(&self) -> &str;
    pub fn available_locales(&self) -> Vec<&str>;

    pub fn get(&self, key: &str) -> &str;
    pub fn get_formatted(&self, key: &str, args: &[(&str, &str)]) -> String;
    pub fn get_plural(&self, key: &str, count: u32) -> String;
}

// Macro for compile-time key checking (optional)
#[macro_export]
macro_rules! t {
    ($loc:expr, $key:literal) => {
        $loc.get($key)
    };
    ($loc:expr, $key:literal, $($arg:tt)*) => {
        $loc.get_formatted($key, &[$($arg)*])
    };
}
```

Create locale files structure:
```
assets/locales/
├── en.json
├── es.json
├── fr.json
├── de.json
├── ja.json
└── zh.json
```

Requirements:
- JSON locale files
- Fallback to default locale
- Format string interpolation
- Plural forms support
- Runtime language switching

### I-14: Crash Reporting (P1)
**File**: `crates/genesis-engine/src/crash_report.rs`

Implement crash capture and reporting:

```rust
use std::panic;
use backtrace::Backtrace;

pub struct CrashReporter {
    report_dir: PathBuf,
    app_version: String,
    enabled: bool,
}

#[derive(Serialize)]
pub struct CrashReport {
    pub timestamp: String,
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub panic_message: String,
    pub backtrace: String,
    pub system_info: SystemInfo,
    pub recent_logs: Vec<String>,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub os_version: String,
    pub cpu: String,
    pub ram_mb: u64,
    pub gpu: Option<String>,
}

impl CrashReporter {
    pub fn new(report_dir: impl AsRef<Path>, app_version: &str) -> Self;

    pub fn install_panic_hook(&self);

    fn capture_crash(&self, panic_info: &panic::PanicInfo) -> CrashReport;
    fn write_report(&self, report: &CrashReport) -> Result<PathBuf, std::io::Error>;

    pub fn get_pending_reports(&self) -> Vec<PathBuf>;
    pub fn submit_report(&self, path: &Path) -> Result<(), ReportError>;
    pub fn delete_report(&self, path: &Path);
}

pub enum ReportError {
    IoError(std::io::Error),
    NetworkError(String),
    ServerError(u16),
}
```

Requirements:
- Custom panic hook
- Backtrace capture
- System info collection
- Report file persistence
- Optional upload (disabled by default)

### I-15: Telemetry & Analytics (P2)
**File**: `crates/genesis-engine/src/analytics.rs`

Implement opt-in gameplay analytics:

```rust
use serde::Serialize;

pub struct Analytics {
    enabled: bool,
    session_id: String,
    events: Vec<AnalyticsEvent>,
    flush_interval: std::time::Duration,
}

#[derive(Serialize)]
pub struct AnalyticsEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

pub struct AnalyticsConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub flush_interval_secs: u64,
    pub batch_size: usize,
}

impl Analytics {
    pub fn new(config: AnalyticsConfig) -> Self;

    pub fn track(&mut self, event_type: &str, properties: HashMap<String, serde_json::Value>);

    // Pre-defined events
    pub fn track_session_start(&mut self);
    pub fn track_session_end(&mut self, play_time_secs: u64);
    pub fn track_level_complete(&mut self, level: &str, time_secs: u64);
    pub fn track_death(&mut self, cause: &str, location: (f32, f32));
    pub fn track_achievement(&mut self, achievement: &str);

    pub fn flush(&mut self);

    pub fn set_enabled(&mut self, enabled: bool);
    pub fn is_enabled(&self) -> bool;
}
```

Requirements:
- Disabled by default (opt-in)
- Local event buffering
- Batch submission
- No PII collection
- Session tracking

---

## Validation Loop

After each task:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --workspace
```

If ANY step fails, FIX IT before committing.

---

## Commit Convention

```
[infra] feat: I-12 asset pipeline
[infra] feat: I-13 localization system
[infra] feat: I-14 crash reporting
[infra] feat: I-15 telemetry and analytics
```

---

## Dependencies

Add to `crates/genesis-engine/Cargo.toml`:
```toml
backtrace = "0.3"
lz4 = "1.24"
```

---

## Integration Notes

- I-12 assets used by all crates for loading resources
- I-13 localization integrated with all UI
- I-14 crash reporter installed at startup
- I-15 analytics tracks gameplay events
- Create sample locale files in `assets/locales/`
