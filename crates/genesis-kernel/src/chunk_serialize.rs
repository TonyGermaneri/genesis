//! Chunk serialization for save/load.
//!
//! Provides efficient binary serialization of chunk data:
//! - Version header for format migration
//! - Sparse encoding for chunks with lots of air
//! - RLE compression for homogeneous regions
//! - CRC32 checksum for corruption detection
//!
//! # Example
//!
//! ```
//! use genesis_kernel::chunk_serialize::{
//!     ChunkSerializer, ChunkHeader, SerializedChunk,
//! };
//! use genesis_kernel::cell::Cell;
//!
//! // Serialize a chunk
//! let cells = vec![Cell::default(); 256 * 256];
//! let serialized = ChunkSerializer::serialize(&cells, (0, 0), 256);
//!
//! // Deserialize - use to_bytes() to get full serialized form
//! let bytes = serialized.to_bytes();
//! let (restored, header) = ChunkSerializer::deserialize(&bytes).unwrap();
//! assert_eq!(cells.len(), restored.len());
//! ```

use crate::cell::Cell;
use bytemuck::{Pod, Zeroable};

/// Current serialization format version.
pub const CHUNK_FORMAT_VERSION: u32 = 1;

/// Magic bytes for chunk file identification.
pub const CHUNK_MAGIC: [u8; 4] = *b"GNCK"; // GeNesis ChunK

/// Maximum uncompressed chunk size (256x256 * 8 bytes = 512KB).
pub const MAX_CHUNK_SIZE: usize = 256 * 256 * 8;

/// Header for serialized chunk data.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ChunkHeader {
    /// Magic bytes for identification.
    pub magic: [u8; 4],
    /// Format version.
    pub version: u32,
    /// Timestamp (seconds since epoch) - placed early for alignment.
    pub timestamp: u64,
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Y coordinate.
    pub chunk_y: i32,
    /// Chunk size (width = height).
    pub chunk_size: u32,
    /// Encoding type (0=raw, 1=sparse, 2=rle).
    pub encoding: u32,
    /// Uncompressed data size.
    pub uncompressed_size: u32,
    /// Compressed data size.
    pub compressed_size: u32,
    /// CRC32 checksum of uncompressed data.
    pub checksum: u32,
    /// Reserved for future use.
    pub reserved: [u32; 5],
}

impl ChunkHeader {
    /// Header size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a new header.
    #[must_use]
    pub fn new(chunk_x: i32, chunk_y: i32, chunk_size: u32) -> Self {
        Self {
            magic: CHUNK_MAGIC,
            version: CHUNK_FORMAT_VERSION,
            chunk_x,
            chunk_y,
            chunk_size,
            encoding: 0,
            uncompressed_size: 0,
            compressed_size: 0,
            checksum: 0,
            timestamp: 0,
            reserved: [0; 5],
        }
    }

    /// Validate the header.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.magic == CHUNK_MAGIC && self.version <= CHUNK_FORMAT_VERSION
    }
}

/// Encoding type for chunk data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ChunkEncoding {
    /// Raw cell data (no compression).
    Raw = 0,
    /// Sparse encoding (only non-air cells).
    Sparse = 1,
    /// Run-length encoding.
    Rle = 2,
}

impl ChunkEncoding {
    /// Convert from u32.
    #[must_use]
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Raw),
            1 => Some(Self::Sparse),
            2 => Some(Self::Rle),
            _ => None,
        }
    }
}

/// A serialized chunk with header and data.
#[derive(Debug, Clone)]
pub struct SerializedChunk {
    /// Chunk header.
    pub header: ChunkHeader,
    /// Serialized data (may be compressed).
    pub data: Vec<u8>,
}

impl SerializedChunk {
    /// Get total size in bytes.
    #[must_use]
    pub fn total_size(&self) -> usize {
        ChunkHeader::SIZE + self.data.len()
    }

    /// Convert to bytes for writing.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.total_size());
        bytes.extend_from_slice(bytemuck::bytes_of(&self.header));
        bytes.extend_from_slice(&self.data);
        bytes
    }

    /// Parse from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SerializeError> {
        if bytes.len() < ChunkHeader::SIZE {
            return Err(SerializeError::InvalidHeader);
        }

        let header: ChunkHeader =
            *bytemuck::from_bytes(&bytes[..ChunkHeader::SIZE]);

        if !header.is_valid() {
            return Err(SerializeError::InvalidMagic);
        }

        let data = bytes[ChunkHeader::SIZE..].to_vec();

        Ok(Self { header, data })
    }
}

/// Sparse cell entry for sparse encoding.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SparseCell {
    /// Cell index in chunk (0 to chunk_size^2 - 1).
    pub index: u32,
    /// Cell data.
    pub cell: Cell,
    /// Padding for alignment.
    padding: u32,
}

impl SparseCell {
    /// Size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

/// RLE run entry.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RleRun {
    /// Number of consecutive cells with this value.
    pub count: u32,
    /// Cell value.
    pub cell: Cell,
}

impl RleRun {
    /// Size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

/// Serialization error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializeError {
    /// Invalid header magic.
    InvalidMagic,
    /// Invalid header data.
    InvalidHeader,
    /// Version not supported.
    UnsupportedVersion(u32),
    /// Invalid encoding type.
    InvalidEncoding(u32),
    /// Data corruption detected.
    ChecksumMismatch,
    /// Data too short.
    InsufficientData,
    /// Decompression failed.
    DecompressionFailed,
    /// Invalid sparse data.
    InvalidSparseData,
    /// Invalid RLE data.
    InvalidRleData,
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "Invalid chunk magic bytes"),
            Self::InvalidHeader => write!(f, "Invalid chunk header"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported version: {v}"),
            Self::InvalidEncoding(e) => write!(f, "Invalid encoding: {e}"),
            Self::ChecksumMismatch => write!(f, "Checksum mismatch - data corrupted"),
            Self::InsufficientData => write!(f, "Insufficient data"),
            Self::DecompressionFailed => write!(f, "Decompression failed"),
            Self::InvalidSparseData => write!(f, "Invalid sparse data"),
            Self::InvalidRleData => write!(f, "Invalid RLE data"),
        }
    }
}

impl std::error::Error for SerializeError {}

/// CRC32 checksum calculator.
pub struct Crc32;

impl Crc32 {
    /// CRC32 polynomial (IEEE 802.3).
    const POLYNOMIAL: u32 = 0xEDB8_8320;

    /// Calculate CRC32 checksum.
    #[must_use]
    pub fn calculate(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFF_u32;

        for &byte in data {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ Self::POLYNOMIAL;
                } else {
                    crc >>= 1;
                }
            }
        }

        !crc
    }

    /// Verify checksum.
    #[must_use]
    pub fn verify(data: &[u8], expected: u32) -> bool {
        Self::calculate(data) == expected
    }
}

/// Chunk serializer.
pub struct ChunkSerializer;

impl ChunkSerializer {
    /// Serialize cells to binary format.
    ///
    /// Automatically chooses the best encoding based on data.
    #[must_use]
    pub fn serialize(cells: &[Cell], position: (i32, i32), chunk_size: u32) -> SerializedChunk {
        let expected_cells = (chunk_size * chunk_size) as usize;
        assert_eq!(cells.len(), expected_cells, "Cell count mismatch");

        // Count non-air cells to decide encoding
        let non_air_count = cells.iter().filter(|c| !c.is_empty()).count();
        let air_ratio = 1.0 - (non_air_count as f32 / cells.len() as f32);

        // Choose encoding based on data characteristics
        let encoding = if air_ratio > 0.9 {
            // More than 90% air - use sparse encoding
            ChunkEncoding::Sparse
        } else if Self::estimate_rle_ratio(cells) < 0.5 {
            // RLE would be at least 50% smaller
            ChunkEncoding::Rle
        } else {
            ChunkEncoding::Raw
        };

        let (data, uncompressed_size) = match encoding {
            ChunkEncoding::Raw => Self::encode_raw(cells),
            ChunkEncoding::Sparse => Self::encode_sparse(cells),
            ChunkEncoding::Rle => Self::encode_rle(cells),
        };

        let checksum = Crc32::calculate(&data);

        let mut header = ChunkHeader::new(position.0, position.1, chunk_size);
        header.encoding = encoding as u32;
        header.uncompressed_size = uncompressed_size as u32;
        header.compressed_size = data.len() as u32;
        header.checksum = checksum;

        SerializedChunk { header, data }
    }

    /// Deserialize cells from binary format.
    pub fn deserialize(bytes: &[u8]) -> Result<(Vec<Cell>, ChunkHeader), SerializeError> {
        let serialized = SerializedChunk::from_bytes(bytes)?;
        let header = serialized.header;

        // Verify checksum
        if !Crc32::verify(&serialized.data, header.checksum) {
            return Err(SerializeError::ChecksumMismatch);
        }

        let encoding = ChunkEncoding::from_u32(header.encoding)
            .ok_or(SerializeError::InvalidEncoding(header.encoding))?;

        let cell_count = (header.chunk_size * header.chunk_size) as usize;

        let cells = match encoding {
            ChunkEncoding::Raw => Self::decode_raw(&serialized.data, cell_count)?,
            ChunkEncoding::Sparse => Self::decode_sparse(&serialized.data, cell_count)?,
            ChunkEncoding::Rle => Self::decode_rle(&serialized.data, cell_count)?,
        };

        Ok((cells, header))
    }

    /// Estimate RLE compression ratio.
    fn estimate_rle_ratio(cells: &[Cell]) -> f32 {
        if cells.is_empty() {
            return 1.0;
        }

        let mut run_count = 1;
        let mut prev = cells[0];

        for &cell in &cells[1..] {
            if cell != prev {
                run_count += 1;
                prev = cell;
            }
        }

        (run_count * RleRun::SIZE) as f32 / std::mem::size_of_val(cells) as f32
    }

    /// Encode cells as raw bytes.
    fn encode_raw(cells: &[Cell]) -> (Vec<u8>, usize) {
        let bytes = bytemuck::cast_slice::<Cell, u8>(cells).to_vec();
        let size = bytes.len();
        (bytes, size)
    }

    /// Decode raw bytes to cells.
    fn decode_raw(data: &[u8], expected_count: usize) -> Result<Vec<Cell>, SerializeError> {
        let expected_size = expected_count * std::mem::size_of::<Cell>();
        if data.len() < expected_size {
            return Err(SerializeError::InsufficientData);
        }

        let cells: &[Cell] = bytemuck::cast_slice(&data[..expected_size]);
        Ok(cells.to_vec())
    }

    /// Encode cells using sparse format.
    fn encode_sparse(cells: &[Cell]) -> (Vec<u8>, usize) {
        let non_air: Vec<SparseCell> = cells
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_empty())
            .map(|(i, &cell)| SparseCell {
                index: i as u32,
                cell,
                padding: 0,
            })
            .collect();

        // Header: count of sparse entries
        let mut data = Vec::with_capacity(4 + non_air.len() * SparseCell::SIZE);
        data.extend_from_slice(&(non_air.len() as u32).to_le_bytes());
        data.extend_from_slice(bytemuck::cast_slice(&non_air));

        let uncompressed_size = std::mem::size_of_val(cells);
        (data, uncompressed_size)
    }

    /// Decode sparse format to cells.
    fn decode_sparse(data: &[u8], cell_count: usize) -> Result<Vec<Cell>, SerializeError> {
        if data.len() < 4 {
            return Err(SerializeError::InsufficientData);
        }

        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let expected_size = 4 + count * SparseCell::SIZE;

        if data.len() < expected_size {
            return Err(SerializeError::InsufficientData);
        }

        let mut cells = vec![Cell::default(); cell_count];
        let sparse_data = &data[4..expected_size];
        let sparse_cells: &[SparseCell] = bytemuck::cast_slice(sparse_data);

        for entry in sparse_cells {
            let idx = entry.index as usize;
            if idx >= cell_count {
                return Err(SerializeError::InvalidSparseData);
            }
            cells[idx] = entry.cell;
        }

        Ok(cells)
    }

    /// Encode cells using RLE.
    fn encode_rle(cells: &[Cell]) -> (Vec<u8>, usize) {
        let mut runs = Vec::new();

        if cells.is_empty() {
            let data = Vec::new();
            return (data, 0);
        }

        let mut current_cell = cells[0];
        let mut current_count = 1u32;

        for &cell in &cells[1..] {
            if cell == current_cell && current_count < u32::MAX {
                current_count += 1;
            } else {
                runs.push(RleRun {
                    count: current_count,
                    cell: current_cell,
                });
                current_cell = cell;
                current_count = 1;
            }
        }

        // Push last run
        runs.push(RleRun {
            count: current_count,
            cell: current_cell,
        });

        // Header: count of runs
        let mut data = Vec::with_capacity(4 + runs.len() * RleRun::SIZE);
        data.extend_from_slice(&(runs.len() as u32).to_le_bytes());
        data.extend_from_slice(bytemuck::cast_slice(&runs));

        let uncompressed_size = std::mem::size_of_val(cells);
        (data, uncompressed_size)
    }

    /// Decode RLE format to cells.
    fn decode_rle(data: &[u8], cell_count: usize) -> Result<Vec<Cell>, SerializeError> {
        if data.len() < 4 {
            return Err(SerializeError::InsufficientData);
        }

        let run_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let expected_size = 4 + run_count * RleRun::SIZE;

        if data.len() < expected_size {
            return Err(SerializeError::InsufficientData);
        }

        let run_data = &data[4..expected_size];
        let runs: &[RleRun] = bytemuck::cast_slice(run_data);

        let mut cells = Vec::with_capacity(cell_count);

        for run in runs {
            for _ in 0..run.count {
                if cells.len() >= cell_count {
                    return Err(SerializeError::InvalidRleData);
                }
                cells.push(run.cell);
            }
        }

        if cells.len() != cell_count {
            return Err(SerializeError::InvalidRleData);
        }

        Ok(cells)
    }

    /// Serialize only the header for quick metadata access.
    #[must_use]
    pub fn serialize_header_only(position: (i32, i32), chunk_size: u32) -> Vec<u8> {
        let header = ChunkHeader::new(position.0, position.1, chunk_size);
        bytemuck::bytes_of(&header).to_vec()
    }

    /// Read header from bytes without parsing full chunk.
    pub fn read_header(bytes: &[u8]) -> Result<ChunkHeader, SerializeError> {
        if bytes.len() < ChunkHeader::SIZE {
            return Err(SerializeError::InvalidHeader);
        }

        let header: ChunkHeader = *bytemuck::from_bytes(&bytes[..ChunkHeader::SIZE]);

        if !header.is_valid() {
            return Err(SerializeError::InvalidMagic);
        }

        Ok(header)
    }
}

/// Statistics about chunk serialization.
#[derive(Debug, Clone, Default)]
pub struct SerializeStats {
    /// Number of chunks serialized.
    pub chunks_serialized: u64,
    /// Number of chunks deserialized.
    pub chunks_deserialized: u64,
    /// Total bytes written.
    pub bytes_written: u64,
    /// Total bytes read.
    pub bytes_read: u64,
    /// Total uncompressed size.
    pub uncompressed_total: u64,
    /// Compression ratio (compressed/uncompressed).
    pub compression_ratio: f32,
    /// Number of checksum failures.
    pub checksum_failures: u64,
}

impl SerializeStats {
    /// Create new stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a serialization.
    pub fn record_serialize(&mut self, serialized: &SerializedChunk) {
        self.chunks_serialized += 1;
        self.bytes_written += serialized.data.len() as u64;
        self.uncompressed_total += u64::from(serialized.header.uncompressed_size);
        self.update_ratio();
    }

    /// Record a deserialization.
    pub fn record_deserialize(&mut self, data_len: usize) {
        self.chunks_deserialized += 1;
        self.bytes_read += data_len as u64;
    }

    /// Record a checksum failure.
    pub fn record_checksum_failure(&mut self) {
        self.checksum_failures += 1;
    }

    /// Update compression ratio.
    fn update_ratio(&mut self) {
        if self.uncompressed_total > 0 {
            self.compression_ratio = self.bytes_written as f32 / self.uncompressed_total as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_header() {
        let header = ChunkHeader::new(5, -3, 256);
        assert_eq!(header.magic, CHUNK_MAGIC);
        assert_eq!(header.version, CHUNK_FORMAT_VERSION);
        assert_eq!(header.chunk_x, 5);
        assert_eq!(header.chunk_y, -3);
        assert!(header.is_valid());
    }

    #[test]
    fn test_header_size() {
        assert_eq!(ChunkHeader::SIZE, 64);
    }

    #[test]
    fn test_crc32() {
        let data = b"Hello, World!";
        let crc = Crc32::calculate(data);
        assert!(Crc32::verify(data, crc));
        assert!(!Crc32::verify(data, crc + 1));
    }

    #[test]
    fn test_serialize_raw() {
        let cells = vec![Cell::new(1); 64 * 64];
        let serialized = ChunkSerializer::serialize(&cells, (0, 0), 64);

        let (restored, header) = ChunkSerializer::deserialize(&serialized.to_bytes()).unwrap();
        assert_eq!(restored.len(), cells.len());
        assert_eq!(header.chunk_x, 0);
        assert_eq!(header.chunk_y, 0);
    }

    #[test]
    fn test_serialize_sparse() {
        // Mostly air with a few non-air cells
        let mut cells = vec![Cell::default(); 64 * 64];
        cells[0] = Cell::new(1);
        cells[100] = Cell::new(2);
        cells[500] = Cell::new(3);

        let serialized = ChunkSerializer::serialize(&cells, (1, 2), 64);
        assert_eq!(serialized.header.encoding, ChunkEncoding::Sparse as u32);

        let (restored, _) = ChunkSerializer::deserialize(&serialized.to_bytes()).unwrap();
        assert_eq!(restored[0], Cell::new(1));
        assert_eq!(restored[100], Cell::new(2));
        assert_eq!(restored[500], Cell::new(3));
        assert!(restored[1].is_empty());
    }

    #[test]
    fn test_serialize_rle() {
        // Alternating pattern that compresses well with RLE
        let mut cells = vec![Cell::default(); 64 * 64];
        // Fill with runs of same material
        for i in 0..32 {
            let material = (i % 4) as u16;
            let start = i * 128;
            let end = start + 128;
            for j in start..end {
                cells[j] = Cell::new(material);
            }
        }

        let serialized = ChunkSerializer::serialize(&cells, (0, 0), 64);

        let (restored, _) = ChunkSerializer::deserialize(&serialized.to_bytes()).unwrap();
        assert_eq!(restored.len(), cells.len());
        for (i, (original, restored)) in cells.iter().zip(restored.iter()).enumerate() {
            assert_eq!(original, restored, "Mismatch at index {i}");
        }
    }

    #[test]
    fn test_checksum_failure() {
        let cells = vec![Cell::default(); 64 * 64];
        let mut serialized = ChunkSerializer::serialize(&cells, (0, 0), 64);

        // Corrupt the data
        if !serialized.data.is_empty() {
            serialized.data[0] ^= 0xFF;
        }

        // Re-pack with corrupted data
        let mut bytes = bytemuck::bytes_of(&serialized.header).to_vec();
        bytes.extend_from_slice(&serialized.data);

        let result = ChunkSerializer::deserialize(&bytes);
        assert!(matches!(result, Err(SerializeError::ChecksumMismatch)));
    }

    #[test]
    fn test_encoding_from_u32() {
        assert_eq!(ChunkEncoding::from_u32(0), Some(ChunkEncoding::Raw));
        assert_eq!(ChunkEncoding::from_u32(1), Some(ChunkEncoding::Sparse));
        assert_eq!(ChunkEncoding::from_u32(2), Some(ChunkEncoding::Rle));
        assert_eq!(ChunkEncoding::from_u32(99), None);
    }

    #[test]
    fn test_serialize_error_display() {
        let err = SerializeError::ChecksumMismatch;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_serialized_chunk_total_size() {
        let cells = vec![Cell::default(); 64 * 64];
        let serialized = ChunkSerializer::serialize(&cells, (0, 0), 64);
        assert_eq!(
            serialized.total_size(),
            ChunkHeader::SIZE + serialized.data.len()
        );
    }

    #[test]
    fn test_read_header() {
        let cells = vec![Cell::default(); 64 * 64];
        let serialized = ChunkSerializer::serialize(&cells, (5, 10), 64);
        let bytes = serialized.to_bytes();

        let header = ChunkSerializer::read_header(&bytes).unwrap();
        assert_eq!(header.chunk_x, 5);
        assert_eq!(header.chunk_y, 10);
        assert_eq!(header.chunk_size, 64);
    }

    #[test]
    fn test_serialize_stats() {
        let mut stats = SerializeStats::new();
        let cells = vec![Cell::default(); 64 * 64];
        let serialized = ChunkSerializer::serialize(&cells, (0, 0), 64);

        stats.record_serialize(&serialized);
        assert_eq!(stats.chunks_serialized, 1);
        assert!(stats.bytes_written > 0);

        stats.record_deserialize(serialized.data.len());
        assert_eq!(stats.chunks_deserialized, 1);

        stats.record_checksum_failure();
        assert_eq!(stats.checksum_failures, 1);
    }

    #[test]
    fn test_sparse_cell_size() {
        assert_eq!(SparseCell::SIZE, 16);
    }

    #[test]
    fn test_rle_run_size() {
        assert_eq!(RleRun::SIZE, 12);
    }

    #[test]
    fn test_empty_chunk() {
        let cells = vec![Cell::default(); 64 * 64];
        let serialized = ChunkSerializer::serialize(&cells, (0, 0), 64);

        // All air should use sparse encoding
        assert_eq!(serialized.header.encoding, ChunkEncoding::Sparse as u32);

        let (restored, _) = ChunkSerializer::deserialize(&serialized.to_bytes()).unwrap();
        assert!(restored.iter().all(|c| c.is_empty()));
    }
}
