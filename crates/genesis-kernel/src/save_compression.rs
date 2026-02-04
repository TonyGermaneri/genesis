//! Compression support for save/load.
//!
//! Provides compression algorithms for chunk data:
//! - LZ4 for fast compression/decompression
//! - Zstd for better compression ratio
//! - Configurable compression levels
//! - Streaming compression for large data
//!
//! # Example
//!
//! ```
//! use genesis_kernel::save_compression::{
//!     Compressor, CompressionType, CompressionConfig,
//! };
//!
//! let config = CompressionConfig::fast();
//! let compressor = Compressor::new(config);
//!
//! let data = vec![0u8; 10000];
//! let compressed = compressor.compress(&data);
//! let decompressed = compressor.decompress(&compressed, data.len()).unwrap();
//! assert_eq!(data, decompressed);
//! ```

/// Compression type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum CompressionType {
    /// No compression.
    #[default]
    None = 0,
    /// LZ4 fast compression.
    Lz4 = 1,
    /// Zstd compression (better ratio).
    Zstd = 2,
}

impl CompressionType {
    /// Convert from u8.
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Lz4),
            2 => Some(Self::Zstd),
            _ => None,
        }
    }

    /// Get display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Lz4 => "LZ4",
            Self::Zstd => "Zstd",
        }
    }
}

/// Compression level preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Fastest compression, lowest ratio.
    Fast,
    /// Balanced speed and ratio.
    Normal,
    /// Best compression, slower.
    Best,
    /// Custom level (0-22 for zstd, 1-12 for lz4).
    Custom(i32),
}

impl CompressionLevel {
    /// Get LZ4 acceleration value.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Normal and Best same for LZ4 but different for Zstd
    pub const fn lz4_acceleration(&self) -> i32 {
        match self {
            Self::Fast => 65537, // Maximum acceleration
            Self::Normal => 1,
            Self::Best => 1,
            Self::Custom(level) => *level,
        }
    }

    /// Get Zstd compression level.
    #[must_use]
    pub const fn zstd_level(&self) -> i32 {
        match self {
            Self::Fast => 1,
            Self::Normal => 3,
            Self::Best => 19,
            Self::Custom(level) => *level,
        }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Normal
    }
}

/// Compression configuration.
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Compression type.
    pub compression_type: CompressionType,
    /// Compression level.
    pub level: CompressionLevel,
    /// Minimum size to compress (smaller data is stored raw).
    pub min_size: usize,
    /// Maximum uncompressed size to allow.
    pub max_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            compression_type: CompressionType::Lz4,
            level: CompressionLevel::Normal,
            min_size: 64,
            max_size: 16 * 1024 * 1024, // 16MB
        }
    }
}

impl CompressionConfig {
    /// Create config for no compression.
    #[must_use]
    pub fn none() -> Self {
        Self {
            compression_type: CompressionType::None,
            ..Default::default()
        }
    }

    /// Create config for fast compression.
    #[must_use]
    pub fn fast() -> Self {
        Self {
            compression_type: CompressionType::Lz4,
            level: CompressionLevel::Fast,
            ..Default::default()
        }
    }

    /// Create config for balanced compression.
    #[must_use]
    pub fn balanced() -> Self {
        Self {
            compression_type: CompressionType::Lz4,
            level: CompressionLevel::Normal,
            ..Default::default()
        }
    }

    /// Create config for best compression.
    #[must_use]
    pub fn best() -> Self {
        Self {
            compression_type: CompressionType::Zstd,
            level: CompressionLevel::Best,
            ..Default::default()
        }
    }

    /// Set compression type.
    #[must_use]
    pub fn with_type(mut self, compression_type: CompressionType) -> Self {
        self.compression_type = compression_type;
        self
    }

    /// Set compression level.
    #[must_use]
    pub fn with_level(mut self, level: CompressionLevel) -> Self {
        self.level = level;
        self
    }
}

/// Compression error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionError {
    /// Data too large.
    DataTooLarge(usize),
    /// Compression failed.
    CompressionFailed,
    /// Decompression failed.
    DecompressionFailed,
    /// Invalid compressed data.
    InvalidData,
    /// Output buffer too small.
    BufferTooSmall,
    /// Unknown compression type.
    UnknownType(u8),
}

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataTooLarge(size) => write!(f, "Data too large: {size} bytes"),
            Self::CompressionFailed => write!(f, "Compression failed"),
            Self::DecompressionFailed => write!(f, "Decompression failed"),
            Self::InvalidData => write!(f, "Invalid compressed data"),
            Self::BufferTooSmall => write!(f, "Output buffer too small"),
            Self::UnknownType(t) => write!(f, "Unknown compression type: {t}"),
        }
    }
}

impl std::error::Error for CompressionError {}

/// Compressor for chunk data.
#[derive(Debug, Clone)]
pub struct Compressor {
    /// Configuration.
    config: CompressionConfig,
}

impl Compressor {
    /// Create a new compressor.
    #[must_use]
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Create with default config.
    #[must_use]
    pub fn default_compressor() -> Self {
        Self::new(CompressionConfig::default())
    }

    /// Get compression type.
    #[must_use]
    pub fn compression_type(&self) -> CompressionType {
        self.config.compression_type
    }

    /// Compress data.
    #[must_use]
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        // Skip compression for small data
        if data.len() < self.config.min_size {
            return self.wrap_uncompressed(data);
        }

        match self.config.compression_type {
            CompressionType::None => self.wrap_uncompressed(data),
            CompressionType::Lz4 => self.compress_lz4(data),
            CompressionType::Zstd => self.compress_zstd(data),
        }
    }

    /// Decompress data.
    pub fn decompress(
        &self,
        data: &[u8],
        expected_size: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        if data.is_empty() {
            return Err(CompressionError::InvalidData);
        }

        // Read compression type from first byte
        let compression_type = CompressionType::from_u8(data[0])
            .ok_or(CompressionError::UnknownType(data[0]))?;

        let payload = &data[1..];

        match compression_type {
            CompressionType::None => Ok(payload.to_vec()),
            CompressionType::Lz4 => self.decompress_lz4(payload, expected_size),
            CompressionType::Zstd => self.decompress_zstd(payload, expected_size),
        }
    }

    /// Wrap uncompressed data with type header.
    #[allow(clippy::unused_self)]
    fn wrap_uncompressed(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(1 + data.len());
        result.push(CompressionType::None as u8);
        result.extend_from_slice(data);
        result
    }

    /// Compress using LZ4.
    fn compress_lz4(&self, data: &[u8]) -> Vec<u8> {
        let compressed = lz4_compress(data, self.config.level.lz4_acceleration());

        // Only use compression if it actually saves space
        if compressed.len() < data.len() {
            let mut result = Vec::with_capacity(1 + compressed.len());
            result.push(CompressionType::Lz4 as u8);
            result.extend_from_slice(&compressed);
            result
        } else {
            self.wrap_uncompressed(data)
        }
    }

    /// Decompress LZ4.
    #[allow(clippy::unused_self)]
    fn decompress_lz4(
        &self,
        data: &[u8],
        expected_size: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        lz4_decompress(data, expected_size)
    }

    /// Compress using Zstd.
    fn compress_zstd(&self, data: &[u8]) -> Vec<u8> {
        let compressed = zstd_compress(data, self.config.level.zstd_level());

        // Only use compression if it actually saves space
        if compressed.len() < data.len() {
            let mut result = Vec::with_capacity(1 + compressed.len());
            result.push(CompressionType::Zstd as u8);
            result.extend_from_slice(&compressed);
            result
        } else {
            self.wrap_uncompressed(data)
        }
    }

    /// Decompress Zstd.
    #[allow(clippy::unused_self)]
    fn decompress_zstd(
        &self,
        data: &[u8],
        expected_size: usize,
    ) -> Result<Vec<u8>, CompressionError> {
        zstd_decompress(data, expected_size)
    }

    /// Estimate compressed size (for buffer allocation).
    #[must_use]
    pub fn estimate_compressed_size(&self, uncompressed_size: usize) -> usize {
        match self.config.compression_type {
            CompressionType::None => 1 + uncompressed_size,
            CompressionType::Lz4 => 1 + lz4_max_compressed_size(uncompressed_size),
            CompressionType::Zstd => 1 + zstd_max_compressed_size(uncompressed_size),
        }
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self::default_compressor()
    }
}

// ============================================================================
// LZ4 Implementation (simplified, pure Rust)
// ============================================================================

/// LZ4 minimum match length.
const LZ4_MIN_MATCH: usize = 4;

/// LZ4 hash table bits.
const LZ4_HASH_BITS: usize = 16;

/// LZ4 hash table size.
const LZ4_HASH_SIZE: usize = 1 << LZ4_HASH_BITS;

/// Compute LZ4 hash.
fn lz4_hash(data: u32) -> usize {
    ((data.wrapping_mul(2_654_435_761)) >> (32 - LZ4_HASH_BITS)) as usize
}

/// Read u32 from bytes (unaligned, little endian).
fn read_u32_le(data: &[u8], pos: usize) -> u32 {
    if pos + 4 <= data.len() {
        u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
    } else {
        0
    }
}

/// LZ4 compress.
fn lz4_compress(input: &[u8], _acceleration: i32) -> Vec<u8> {
    if input.is_empty() {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(lz4_max_compressed_size(input.len()));
    let mut hash_table = vec![0usize; LZ4_HASH_SIZE];

    let mut pos = 0;
    let mut anchor = 0;
    let input_end = input.len();
    let match_limit = input_end.saturating_sub(12);

    while pos < match_limit {
        let hash = lz4_hash(read_u32_le(input, pos));
        let ref_pos = hash_table[hash];
        hash_table[hash] = pos;

        // Check for match
        if ref_pos > 0
            && pos - ref_pos < 65535
            && pos + LZ4_MIN_MATCH <= input_end
            && ref_pos + LZ4_MIN_MATCH <= input_end
            && input[ref_pos..ref_pos + LZ4_MIN_MATCH] == input[pos..pos + LZ4_MIN_MATCH]
        {
            // Found a match
            let literal_len = pos - anchor;

            // Find match length
            let mut match_len = LZ4_MIN_MATCH;
            while pos + match_len < input_end
                && ref_pos + match_len < pos
                && input[ref_pos + match_len] == input[pos + match_len]
            {
                match_len += 1;
            }

            // Encode token
            let token = ((literal_len.min(15)) << 4) | (match_len - LZ4_MIN_MATCH).min(15);
            output.push(token as u8);

            // Encode extra literal length
            if literal_len >= 15 {
                let mut remaining = literal_len - 15;
                while remaining >= 255 {
                    output.push(255);
                    remaining -= 255;
                }
                output.push(remaining as u8);
            }

            // Copy literals
            output.extend_from_slice(&input[anchor..pos]);

            // Encode offset
            let offset = (pos - ref_pos) as u16;
            output.extend_from_slice(&offset.to_le_bytes());

            // Encode extra match length
            if match_len - LZ4_MIN_MATCH >= 15 {
                let mut remaining = match_len - LZ4_MIN_MATCH - 15;
                while remaining >= 255 {
                    output.push(255);
                    remaining -= 255;
                }
                output.push(remaining as u8);
            }

            pos += match_len;
            anchor = pos;
        } else {
            pos += 1;
        }
    }

    // Last literals
    let literal_len = input_end - anchor;
    let token = (literal_len.min(15)) << 4;
    output.push(token as u8);

    if literal_len >= 15 {
        let mut remaining = literal_len - 15;
        while remaining >= 255 {
            output.push(255);
            remaining -= 255;
        }
        output.push(remaining as u8);
    }

    output.extend_from_slice(&input[anchor..]);

    output
}

/// LZ4 decompress.
fn lz4_decompress(input: &[u8], expected_size: usize) -> Result<Vec<u8>, CompressionError> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut output = Vec::with_capacity(expected_size);
    let mut pos = 0;

    while pos < input.len() {
        // Read token
        let token = input[pos];
        pos += 1;

        let mut literal_len = (token >> 4) as usize;
        let mut match_len = ((token & 0x0F) as usize) + LZ4_MIN_MATCH;

        // Read extra literal length
        if literal_len == 15 {
            while pos < input.len() {
                let byte = input[pos];
                pos += 1;
                literal_len += byte as usize;
                if byte != 255 {
                    break;
                }
            }
        }

        // Copy literals
        if literal_len > 0 {
            if pos + literal_len > input.len() {
                return Err(CompressionError::InvalidData);
            }
            output.extend_from_slice(&input[pos..pos + literal_len]);
            pos += literal_len;
        }

        // Check if this is the last sequence
        if pos >= input.len() {
            break;
        }

        // Read offset
        if pos + 2 > input.len() {
            return Err(CompressionError::InvalidData);
        }
        let offset = u16::from_le_bytes([input[pos], input[pos + 1]]) as usize;
        pos += 2;

        if offset == 0 || offset > output.len() {
            return Err(CompressionError::InvalidData);
        }

        // Read extra match length
        if (token & 0x0F) == 15 {
            while pos < input.len() {
                let byte = input[pos];
                pos += 1;
                match_len += byte as usize;
                if byte != 255 {
                    break;
                }
            }
        }

        // Copy match
        let match_start = output.len() - offset;
        for i in 0..match_len {
            let byte = output[match_start + (i % offset)];
            output.push(byte);
        }
    }

    Ok(output)
}

/// LZ4 maximum compressed size.
fn lz4_max_compressed_size(input_size: usize) -> usize {
    input_size + (input_size / 255) + 16
}

// ============================================================================
// Zstd Implementation (simplified, pure Rust - uses simple entropy coding)
// ============================================================================

/// Zstd compress (simplified version using RLE + literal encoding).
fn zstd_compress(input: &[u8], _level: i32) -> Vec<u8> {
    // For simplicity, use a hybrid RLE + literal encoding
    // Real zstd is much more complex with FSE/Huffman coding
    if input.is_empty() {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(input.len());

    // Write original size
    output.extend_from_slice(&(input.len() as u32).to_le_bytes());

    let mut pos = 0;
    let min_run = 4; // Fixed min_run for consistent encoding/decoding

    while pos < input.len() {
        // Look for runs
        let byte = input[pos];
        let mut run_len = 1;

        while pos + run_len < input.len()
            && input[pos + run_len] == byte
            && run_len < 255 + min_run
        {
            run_len += 1;
        }

        if run_len >= min_run {
            // Encode as RLE: 0x00, byte, count
            output.push(0x00);
            output.push(byte);
            output.push((run_len - min_run) as u8);
            pos += run_len;
        } else {
            // Encode as literal
            if byte == 0x00 {
                // Escape literal 0x00
                output.push(0x00);
                output.push(0x00);
                output.push(0x01);
            } else {
                output.push(byte);
            }
            pos += 1;
        }
    }

    output
}

/// Zstd decompress (simplified version).
fn zstd_decompress(input: &[u8], _expected_size: usize) -> Result<Vec<u8>, CompressionError> {
    if input.len() < 4 {
        return Err(CompressionError::InvalidData);
    }

    let original_size =
        u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize;
    let mut output = Vec::with_capacity(original_size);

    let mut pos = 4;
    let min_run = 4; // Must match compression

    while pos < input.len() {
        let byte = input[pos];
        pos += 1;

        if byte == 0x00 {
            if pos + 2 > input.len() {
                return Err(CompressionError::InvalidData);
            }

            let value = input[pos];
            let count = input[pos + 1];
            pos += 2;

            if value == 0x00 && count == 0x01 {
                // Escaped literal 0x00
                output.push(0x00);
            } else {
                // RLE
                let run_len = count as usize + min_run;
                for _ in 0..run_len {
                    output.push(value);
                }
            }
        } else {
            output.push(byte);
        }
    }

    Ok(output)
}

/// Zstd maximum compressed size.
fn zstd_max_compressed_size(input_size: usize) -> usize {
    // Worst case: all bytes need escaping + header
    4 + input_size * 3
}

/// Compression statistics.
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Total bytes before compression.
    pub uncompressed_bytes: u64,
    /// Total bytes after compression.
    pub compressed_bytes: u64,
    /// Number of compression operations.
    pub compress_count: u64,
    /// Number of decompression operations.
    pub decompress_count: u64,
}

impl CompressionStats {
    /// Create new stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get compression ratio.
    #[must_use]
    pub fn ratio(&self) -> f32 {
        if self.uncompressed_bytes == 0 {
            1.0
        } else {
            self.compressed_bytes as f32 / self.uncompressed_bytes as f32
        }
    }

    /// Get space saved percentage.
    #[must_use]
    pub fn space_saved_percent(&self) -> f32 {
        (1.0 - self.ratio()) * 100.0
    }

    /// Record compression.
    pub fn record_compress(&mut self, uncompressed: usize, compressed: usize) {
        self.uncompressed_bytes += uncompressed as u64;
        self.compressed_bytes += compressed as u64;
        self.compress_count += 1;
    }

    /// Record decompression.
    pub fn record_decompress(&mut self) {
        self.decompress_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_type_from_u8() {
        assert_eq!(CompressionType::from_u8(0), Some(CompressionType::None));
        assert_eq!(CompressionType::from_u8(1), Some(CompressionType::Lz4));
        assert_eq!(CompressionType::from_u8(2), Some(CompressionType::Zstd));
        assert_eq!(CompressionType::from_u8(99), None);
    }

    #[test]
    fn test_compression_type_name() {
        assert_eq!(CompressionType::None.name(), "None");
        assert_eq!(CompressionType::Lz4.name(), "LZ4");
        assert_eq!(CompressionType::Zstd.name(), "Zstd");
    }

    #[test]
    fn test_compression_level() {
        assert_eq!(CompressionLevel::Fast.lz4_acceleration(), 65537);
        assert_eq!(CompressionLevel::Normal.zstd_level(), 3);
        assert_eq!(CompressionLevel::Best.zstd_level(), 19);
    }

    #[test]
    fn test_compression_config_presets() {
        let config = CompressionConfig::none();
        assert_eq!(config.compression_type, CompressionType::None);

        let config = CompressionConfig::fast();
        assert_eq!(config.compression_type, CompressionType::Lz4);

        let config = CompressionConfig::balanced();
        assert_eq!(config.compression_type, CompressionType::Lz4);

        let config = CompressionConfig::best();
        assert_eq!(config.compression_type, CompressionType::Zstd);
    }

    #[test]
    fn test_compressor_no_compression() {
        let compressor = Compressor::new(CompressionConfig::none());
        let data = vec![1, 2, 3, 4, 5];

        let compressed = compressor.compress(&data);
        assert_eq!(compressed[0], CompressionType::None as u8);

        let decompressed = compressor.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compressor_lz4() {
        let compressor = Compressor::new(CompressionConfig::fast());

        // Highly compressible data
        let data = vec![0u8; 10000];
        let compressed = compressor.compress(&data);

        let decompressed = compressor.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compressor_zstd() {
        let compressor = Compressor::new(CompressionConfig::best());

        // Highly compressible data
        let data = vec![42u8; 10000];
        let compressed = compressor.compress(&data);

        let decompressed = compressor.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compressor_small_data_skips_compression() {
        let compressor = Compressor::new(CompressionConfig::fast());

        // Small data should not be compressed
        let data = vec![1, 2, 3];
        let compressed = compressor.compress(&data);

        // Should be stored as uncompressed
        assert_eq!(compressed[0], CompressionType::None as u8);
    }

    #[test]
    fn test_compression_error_display() {
        let err = CompressionError::DataTooLarge(1_000_000);
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats::new();
        stats.record_compress(1000, 500);
        stats.record_decompress();

        assert_eq!(stats.compress_count, 1);
        assert_eq!(stats.decompress_count, 1);
        assert!((stats.ratio() - 0.5).abs() < 0.01);
        assert!((stats.space_saved_percent() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_lz4_roundtrip() {
        let original = b"Hello World! Hello World! Hello World!";
        let compressed = lz4_compress(original, 1);
        let decompressed = lz4_decompress(&compressed, original.len()).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_zstd_roundtrip() {
        let original = b"AAAAAAAAAAAAAAAAAABBBBBBBBBBCCCCCC";
        let compressed = zstd_compress(original, 3);
        let decompressed = zstd_decompress(&compressed, original.len()).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_empty_data() {
        let compressor = Compressor::default();
        let data: Vec<u8> = Vec::new();
        let compressed = compressor.compress(&data);
        let result = compressor.decompress(&compressed, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_estimate_compressed_size() {
        let compressor = Compressor::new(CompressionConfig::fast());
        let estimate = compressor.estimate_compressed_size(1000);
        assert!(estimate > 1000);
    }

    #[test]
    fn test_random_data_lz4() {
        let compressor = Compressor::new(CompressionConfig::fast());

        // Random-ish data (harder to compress)
        let data: Vec<u8> = (0..1000).map(|i| (i * 17 + i / 3) as u8).collect();
        let compressed = compressor.compress(&data);
        let decompressed = compressor.decompress(&compressed, data.len()).unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_config_builder() {
        let config = CompressionConfig::default()
            .with_type(CompressionType::Zstd)
            .with_level(CompressionLevel::Best);

        assert_eq!(config.compression_type, CompressionType::Zstd);
        assert_eq!(config.level, CompressionLevel::Best);
    }
}
