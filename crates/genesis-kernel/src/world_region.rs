//! World region file management.
//!
//! Provides region-based chunk storage:
//! - 32x32 chunks per region file
//! - Chunk offset table for fast lookup
//! - Memory-mapped file access option
//! - Automatic region creation/loading
//!
//! # Example
//!
//! ```no_run
//! use genesis_kernel::world_region::{
//!     RegionManager, RegionCoord, RegionHeader,
//! };
//!
//! // Create region manager
//! let mut manager = RegionManager::new("./world");
//!
//! // Write chunk data
//! manager.write_chunk((5, 10), &[0u8; 1024]).unwrap();
//!
//! // Read chunk data
//! if let Some(data) = manager.read_chunk((5, 10)).unwrap() {
//!     println!("Loaded {} bytes", data.len());
//! }
//! ```

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use bytemuck::{Pod, Zeroable};

/// Number of chunks per region (32x32).
pub const CHUNKS_PER_REGION: u32 = 32;

/// Total chunks in a region.
pub const TOTAL_CHUNKS_PER_REGION: usize = (CHUNKS_PER_REGION * CHUNKS_PER_REGION) as usize;

/// Magic bytes for region file identification.
pub const REGION_MAGIC: [u8; 4] = *b"GNRG"; // GeNesis ReGion

/// Current region format version.
pub const REGION_FORMAT_VERSION: u32 = 1;

/// Sector size for chunk alignment (4KB).
pub const SECTOR_SIZE: u32 = 4096;

/// Maximum chunk size in sectors (256 * 4KB = 1MB).
pub const MAX_CHUNK_SECTORS: u32 = 256;

/// Region file header.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RegionHeader {
    /// Magic bytes for identification.
    pub magic: [u8; 4],
    /// Format version.
    pub version: u32,
    /// Region X coordinate.
    pub region_x: i32,
    /// Region Y coordinate.
    pub region_y: i32,
    /// Timestamp of last modification.
    pub timestamp: u64,
    /// Number of chunks stored.
    pub chunk_count: u32,
    /// Reserved for future use.
    pub reserved: [u32; 7],
}

impl RegionHeader {
    /// Header size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a new region header.
    #[must_use]
    pub fn new(region_x: i32, region_y: i32) -> Self {
        Self {
            magic: REGION_MAGIC,
            version: REGION_FORMAT_VERSION,
            region_x,
            region_y,
            timestamp: 0,
            chunk_count: 0,
            reserved: [0; 7],
        }
    }

    /// Validate the header.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.magic == REGION_MAGIC && self.version <= REGION_FORMAT_VERSION
    }
}

/// Chunk location entry in the offset table.
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct ChunkLocation {
    /// Sector offset from start of file.
    pub sector_offset: u32,
    /// Number of sectors used.
    pub sector_count: u8,
    /// Compression type (0=none, 1=lz4, 2=zstd).
    pub compression: u8,
    /// Reserved for alignment.
    pub reserved: [u8; 2],
}

impl ChunkLocation {
    /// Size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Check if this location is empty (no chunk stored).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sector_offset == 0 && self.sector_count == 0
    }

    /// Get byte offset in file.
    #[must_use]
    pub fn byte_offset(&self) -> u64 {
        u64::from(self.sector_offset) * u64::from(SECTOR_SIZE)
    }

    /// Get byte size.
    #[must_use]
    pub fn byte_size(&self) -> usize {
        self.sector_count as usize * SECTOR_SIZE as usize
    }
}

/// Chunk timestamp entry.
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct ChunkTimestamp {
    /// Seconds since epoch.
    pub timestamp: u32,
}

impl ChunkTimestamp {
    /// Size in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

/// Region coordinate (different from chunk coordinate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionCoord {
    /// Region X.
    pub x: i32,
    /// Region Y.
    pub y: i32,
}

impl RegionCoord {
    /// Create from chunk coordinates.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn from_chunk(chunk_x: i32, chunk_y: i32) -> Self {
        Self {
            x: chunk_x.div_euclid(CHUNKS_PER_REGION as i32),
            y: chunk_y.div_euclid(CHUNKS_PER_REGION as i32),
        }
    }

    /// Get local chunk index within this region.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn local_chunk_index(chunk_x: i32, chunk_y: i32) -> usize {
        let local_x = chunk_x.rem_euclid(CHUNKS_PER_REGION as i32) as usize;
        let local_y = chunk_y.rem_euclid(CHUNKS_PER_REGION as i32) as usize;
        local_y * CHUNKS_PER_REGION as usize + local_x
    }

    /// Get region filename.
    #[must_use]
    pub fn filename(&self) -> String {
        format!("r.{}.{}.gnr", self.x, self.y)
    }
}

/// Region file error.
#[derive(Debug)]
pub enum RegionError {
    /// IO error.
    Io(io::Error),
    /// Invalid region file magic.
    InvalidMagic,
    /// Invalid region file header.
    InvalidHeader,
    /// Unsupported version.
    UnsupportedVersion(u32),
    /// Chunk too large.
    ChunkTooLarge(usize),
    /// No space in region file.
    NoSpace,
    /// Chunk not found.
    ChunkNotFound,
}

impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::InvalidMagic => write!(f, "Invalid region file magic"),
            Self::InvalidHeader => write!(f, "Invalid region file header"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported region version: {v}"),
            Self::ChunkTooLarge(size) => write!(f, "Chunk too large: {size} bytes"),
            Self::NoSpace => write!(f, "No space in region file"),
            Self::ChunkNotFound => write!(f, "Chunk not found"),
        }
    }
}

impl std::error::Error for RegionError {}

impl From<io::Error> for RegionError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Open region file handle.
#[derive(Debug)]
#[allow(dead_code)] // path retained for debugging
pub struct RegionFile {
    /// File handle.
    file: File,
    /// Region header.
    header: RegionHeader,
    /// Chunk locations.
    locations: [ChunkLocation; TOTAL_CHUNKS_PER_REGION],
    /// Chunk timestamps.
    timestamps: [ChunkTimestamp; TOTAL_CHUNKS_PER_REGION],
    /// Path to file.
    path: PathBuf,
    /// Whether file has been modified.
    dirty: bool,
}

impl RegionFile {
    /// Size of the offset table in bytes.
    const OFFSET_TABLE_SIZE: usize = TOTAL_CHUNKS_PER_REGION * ChunkLocation::SIZE;

    /// Size of the timestamp table in bytes.
    const TIMESTAMP_TABLE_SIZE: usize = TOTAL_CHUNKS_PER_REGION * ChunkTimestamp::SIZE;

    /// Start of chunk data (after header and tables).
    const DATA_START: u64 = (RegionHeader::SIZE
        + Self::OFFSET_TABLE_SIZE
        + Self::TIMESTAMP_TABLE_SIZE) as u64;

    /// Create or open a region file.
    pub fn open(path: &Path, coord: RegionCoord) -> Result<Self, RegionError> {
        let exists = path.exists();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        if exists {
            Self::load_existing(file, path.to_path_buf())
        } else {
            Self::create_new(file, path.to_path_buf(), coord)
        }
    }

    /// Load an existing region file.
    fn load_existing(mut file: File, path: PathBuf) -> Result<Self, RegionError> {
        // Read header
        let mut header_bytes = [0u8; RegionHeader::SIZE];
        file.read_exact(&mut header_bytes)?;
        let header: RegionHeader = *bytemuck::from_bytes(&header_bytes);

        if !header.is_valid() {
            return Err(RegionError::InvalidMagic);
        }

        // Read location table
        let mut location_bytes = vec![0u8; Self::OFFSET_TABLE_SIZE];
        file.read_exact(&mut location_bytes)?;
        let locations: [ChunkLocation; TOTAL_CHUNKS_PER_REGION] =
            *bytemuck::from_bytes(&location_bytes);

        // Read timestamp table
        let mut timestamp_bytes = vec![0u8; Self::TIMESTAMP_TABLE_SIZE];
        file.read_exact(&mut timestamp_bytes)?;
        let timestamps: [ChunkTimestamp; TOTAL_CHUNKS_PER_REGION] =
            *bytemuck::from_bytes(&timestamp_bytes);

        Ok(Self {
            file,
            header,
            locations,
            timestamps,
            path,
            dirty: false,
        })
    }

    /// Create a new region file.
    fn create_new(mut file: File, path: PathBuf, coord: RegionCoord) -> Result<Self, RegionError> {
        let header = RegionHeader::new(coord.x, coord.y);
        let locations = [ChunkLocation::default(); TOTAL_CHUNKS_PER_REGION];
        let timestamps = [ChunkTimestamp::default(); TOTAL_CHUNKS_PER_REGION];

        // Write header
        file.write_all(bytemuck::bytes_of(&header))?;

        // Write empty location table
        file.write_all(bytemuck::cast_slice(&locations))?;

        // Write empty timestamp table
        file.write_all(bytemuck::cast_slice(&timestamps))?;

        file.sync_all()?;

        Ok(Self {
            file,
            header,
            locations,
            timestamps,
            path,
            dirty: false,
        })
    }

    /// Read chunk data.
    pub fn read_chunk(&mut self, local_index: usize) -> Result<Option<Vec<u8>>, RegionError> {
        if local_index >= TOTAL_CHUNKS_PER_REGION {
            return Err(RegionError::ChunkNotFound);
        }

        let location = &self.locations[local_index];
        if location.is_empty() {
            return Ok(None);
        }

        // Seek to chunk data
        self.file.seek(SeekFrom::Start(location.byte_offset()))?;

        // Read chunk header (4 bytes length + data)
        let mut length_bytes = [0u8; 4];
        self.file.read_exact(&mut length_bytes)?;
        let length = u32::from_le_bytes(length_bytes) as usize;

        // Read chunk data
        let mut data = vec![0u8; length];
        self.file.read_exact(&mut data)?;

        Ok(Some(data))
    }

    /// Write chunk data.
    pub fn write_chunk(&mut self, local_index: usize, data: &[u8]) -> Result<(), RegionError> {
        if local_index >= TOTAL_CHUNKS_PER_REGION {
            return Err(RegionError::ChunkNotFound);
        }

        // Calculate sectors needed
        let total_size = 4 + data.len(); // 4 bytes for length prefix
        let sectors_needed = total_size.div_ceil(SECTOR_SIZE as usize) as u32;

        if sectors_needed > MAX_CHUNK_SECTORS {
            return Err(RegionError::ChunkTooLarge(data.len()));
        }

        // Find space for the chunk
        let sector_offset = self.find_free_space(local_index, sectors_needed);

        // Update location
        self.locations[local_index] = ChunkLocation {
            sector_offset,
            sector_count: sectors_needed as u8,
            compression: 0,
            reserved: [0; 2],
        };

        // Update timestamp
        self.timestamps[local_index] = ChunkTimestamp {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as u32)
                .unwrap_or(0),
        };

        // Seek to chunk location
        let byte_offset = u64::from(sector_offset) * u64::from(SECTOR_SIZE);
        self.file.seek(SeekFrom::Start(byte_offset))?;

        // Write length prefix
        self.file.write_all(&(data.len() as u32).to_le_bytes())?;

        // Write data
        self.file.write_all(data)?;

        // Pad to sector boundary
        let padding = (sectors_needed as usize * SECTOR_SIZE as usize) - total_size;
        if padding > 0 {
            self.file.write_all(&vec![0u8; padding])?;
        }

        self.dirty = true;
        self.header.chunk_count = self
            .locations
            .iter()
            .filter(|l| !l.is_empty())
            .count() as u32;

        Ok(())
    }

    /// Find free space in the region file.
    fn find_free_space(&self, local_index: usize, sectors_needed: u32) -> u32 {
        // First, check if we can reuse the current location
        let current = &self.locations[local_index];
        if !current.is_empty() && u32::from(current.sector_count) >= sectors_needed {
            return current.sector_offset;
        }

        // Find the highest used sector
        // Round up to ensure data starts after header and tables
        let mut max_sector = Self::DATA_START.div_ceil(u64::from(SECTOR_SIZE)) as u32;

        for loc in &self.locations {
            if !loc.is_empty() {
                let end = loc.sector_offset + u32::from(loc.sector_count);
                max_sector = max_sector.max(end);
            }
        }

        // Allocate at the end
        max_sector
    }

    /// Delete a chunk.
    pub fn delete_chunk(&mut self, local_index: usize) -> Result<(), RegionError> {
        if local_index >= TOTAL_CHUNKS_PER_REGION {
            return Err(RegionError::ChunkNotFound);
        }

        self.locations[local_index] = ChunkLocation::default();
        self.timestamps[local_index] = ChunkTimestamp::default();
        self.dirty = true;

        self.header.chunk_count = self
            .locations
            .iter()
            .filter(|l| !l.is_empty())
            .count() as u32;

        Ok(())
    }

    /// Check if chunk exists.
    #[must_use]
    pub fn has_chunk(&self, local_index: usize) -> bool {
        local_index < TOTAL_CHUNKS_PER_REGION && !self.locations[local_index].is_empty()
    }

    /// Get chunk timestamp.
    #[must_use]
    pub fn chunk_timestamp(&self, local_index: usize) -> Option<u32> {
        if local_index < TOTAL_CHUNKS_PER_REGION && !self.locations[local_index].is_empty() {
            Some(self.timestamps[local_index].timestamp)
        } else {
            None
        }
    }

    /// Flush changes to disk.
    pub fn flush(&mut self) -> Result<(), RegionError> {
        if !self.dirty {
            return Ok(());
        }

        // Update header
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(bytemuck::bytes_of(&self.header))?;

        // Update location table
        self.file
            .write_all(bytemuck::cast_slice(&self.locations))?;

        // Update timestamp table
        self.file
            .write_all(bytemuck::cast_slice(&self.timestamps))?;

        self.file.sync_all()?;
        self.dirty = false;

        Ok(())
    }

    /// Get region coordinate.
    #[must_use]
    pub fn coord(&self) -> RegionCoord {
        RegionCoord {
            x: self.header.region_x,
            y: self.header.region_y,
        }
    }

    /// Get number of stored chunks.
    #[must_use]
    pub fn chunk_count(&self) -> u32 {
        self.header.chunk_count
    }
}

impl Drop for RegionFile {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Region file manager.
#[derive(Debug)]
pub struct RegionManager {
    /// Base directory for region files.
    base_path: PathBuf,
    /// Open region files (cached).
    regions: HashMap<RegionCoord, RegionFile>,
    /// Maximum cached regions.
    max_cached: usize,
}

impl RegionManager {
    /// Create a new region manager.
    #[must_use]
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            regions: HashMap::new(),
            max_cached: 32,
        }
    }

    /// Set maximum cached regions.
    pub fn set_max_cached(&mut self, max: usize) {
        self.max_cached = max;
    }

    /// Get or open a region file.
    fn get_region(&mut self, coord: RegionCoord) -> Result<&mut RegionFile, RegionError> {
        // Evict old regions if needed
        if self.regions.len() >= self.max_cached && !self.regions.contains_key(&coord) {
            // Remove a random region (simple eviction)
            if let Some(&key) = self.regions.keys().next() {
                self.regions.remove(&key);
            }
        }

        // Open or get cached region using entry API
        match self.regions.entry(coord) {
            std::collections::hash_map::Entry::Occupied(e) => Ok(e.into_mut()),
            std::collections::hash_map::Entry::Vacant(e) => {
                fs::create_dir_all(&self.base_path)?;
                let path = self.base_path.join(coord.filename());
                let region = RegionFile::open(&path, coord)?;
                Ok(e.insert(region))
            }
        }
    }

    /// Read chunk data by chunk coordinates.
    pub fn read_chunk(&mut self, chunk_pos: (i32, i32)) -> Result<Option<Vec<u8>>, RegionError> {
        let coord = RegionCoord::from_chunk(chunk_pos.0, chunk_pos.1);
        let local_index = RegionCoord::local_chunk_index(chunk_pos.0, chunk_pos.1);
        let region = self.get_region(coord)?;
        region.read_chunk(local_index)
    }

    /// Write chunk data by chunk coordinates.
    pub fn write_chunk(&mut self, chunk_pos: (i32, i32), data: &[u8]) -> Result<(), RegionError> {
        let coord = RegionCoord::from_chunk(chunk_pos.0, chunk_pos.1);
        let local_index = RegionCoord::local_chunk_index(chunk_pos.0, chunk_pos.1);
        let region = self.get_region(coord)?;
        region.write_chunk(local_index, data)
    }

    /// Delete a chunk by coordinates.
    pub fn delete_chunk(&mut self, chunk_pos: (i32, i32)) -> Result<(), RegionError> {
        let coord = RegionCoord::from_chunk(chunk_pos.0, chunk_pos.1);
        let local_index = RegionCoord::local_chunk_index(chunk_pos.0, chunk_pos.1);
        let region = self.get_region(coord)?;
        region.delete_chunk(local_index)
    }

    /// Check if a chunk exists.
    pub fn has_chunk(&mut self, chunk_pos: (i32, i32)) -> Result<bool, RegionError> {
        let coord = RegionCoord::from_chunk(chunk_pos.0, chunk_pos.1);
        let local_index = RegionCoord::local_chunk_index(chunk_pos.0, chunk_pos.1);
        let region = self.get_region(coord)?;
        Ok(region.has_chunk(local_index))
    }

    /// Flush all cached regions.
    pub fn flush_all(&mut self) -> Result<(), RegionError> {
        for region in self.regions.values_mut() {
            region.flush()?;
        }
        Ok(())
    }

    /// Close all cached regions.
    pub fn close_all(&mut self) {
        self.regions.clear();
    }

    /// Get statistics.
    #[must_use]
    pub fn stats(&self) -> RegionStats {
        let mut total_chunks = 0;
        for region in self.regions.values() {
            total_chunks += region.chunk_count();
        }

        RegionStats {
            cached_regions: self.regions.len(),
            total_chunks,
        }
    }
}

/// Region manager statistics.
#[derive(Debug, Clone, Default)]
pub struct RegionStats {
    /// Number of cached regions.
    pub cached_regions: usize,
    /// Total chunks across all cached regions.
    pub total_chunks: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_dir() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("genesis_test_{}", std::process::id()));
        path
    }

    #[test]
    fn test_region_coord_from_chunk() {
        let coord = RegionCoord::from_chunk(0, 0);
        assert_eq!(coord.x, 0);
        assert_eq!(coord.y, 0);

        let coord = RegionCoord::from_chunk(31, 31);
        assert_eq!(coord.x, 0);
        assert_eq!(coord.y, 0);

        let coord = RegionCoord::from_chunk(32, 0);
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 0);

        let coord = RegionCoord::from_chunk(-1, -1);
        assert_eq!(coord.x, -1);
        assert_eq!(coord.y, -1);
    }

    #[test]
    fn test_local_chunk_index() {
        assert_eq!(RegionCoord::local_chunk_index(0, 0), 0);
        assert_eq!(RegionCoord::local_chunk_index(1, 0), 1);
        assert_eq!(RegionCoord::local_chunk_index(0, 1), 32);
        assert_eq!(RegionCoord::local_chunk_index(31, 31), 1023);
        assert_eq!(RegionCoord::local_chunk_index(32, 0), 0);
        assert_eq!(RegionCoord::local_chunk_index(-1, -1), 31 * 32 + 31);
    }

    #[test]
    fn test_region_filename() {
        let coord = RegionCoord { x: 5, y: -3 };
        assert_eq!(coord.filename(), "r.5.-3.gnr");
    }

    #[test]
    fn test_chunk_location() {
        let loc = ChunkLocation::default();
        assert!(loc.is_empty());

        let loc = ChunkLocation {
            sector_offset: 10,
            sector_count: 2,
            compression: 0,
            reserved: [0; 2],
        };
        assert!(!loc.is_empty());
        assert_eq!(loc.byte_offset(), 10 * 4096);
        assert_eq!(loc.byte_size(), 2 * 4096);
    }

    #[test]
    fn test_region_header() {
        let header = RegionHeader::new(5, -3);
        assert_eq!(header.magic, REGION_MAGIC);
        assert_eq!(header.version, REGION_FORMAT_VERSION);
        assert_eq!(header.region_x, 5);
        assert_eq!(header.region_y, -3);
        assert!(header.is_valid());
    }

    #[test]
    fn test_header_size() {
        // 4 (magic) + 4 (version) + 4 (region_x) + 4 (region_y) + 8 (timestamp) + 4 (chunk_count) + 28 (reserved) = 56
        assert_eq!(RegionHeader::SIZE, 56);
    }

    #[test]
    fn test_chunk_location_size() {
        assert_eq!(ChunkLocation::SIZE, 8);
    }

    #[test]
    fn test_region_error_display() {
        let err = RegionError::ChunkTooLarge(1_000_000);
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_region_manager_creation() {
        let manager = RegionManager::new("./test_regions");
        assert_eq!(manager.max_cached, 32);
    }

    #[test]
    fn test_region_file_write_read() {
        let dir = temp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("test.gnr");
        let coord = RegionCoord { x: 0, y: 0 };

        {
            let mut region = RegionFile::open(&path, coord).unwrap();

            // Write chunk data
            let data = vec![1, 2, 3, 4, 5];
            region.write_chunk(0, &data).unwrap();
            region.flush().unwrap();

            // Read it back
            let read_data = region.read_chunk(0).unwrap();
            assert_eq!(read_data, Some(data));
        }

        // Clean up
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_region_manager_write_read() {
        let dir = temp_dir();
        let _ = fs::remove_dir_all(&dir);

        {
            let mut manager = RegionManager::new(&dir);

            // Write chunks
            manager.write_chunk((0, 0), b"chunk_0_0").unwrap();
            manager.write_chunk((1, 0), b"chunk_1_0").unwrap();
            manager.write_chunk((32, 0), b"chunk_32_0").unwrap(); // Different region

            // Read chunks
            let data = manager.read_chunk((0, 0)).unwrap();
            assert_eq!(data, Some(b"chunk_0_0".to_vec()));

            let data = manager.read_chunk((1, 0)).unwrap();
            assert_eq!(data, Some(b"chunk_1_0".to_vec()));

            let data = manager.read_chunk((32, 0)).unwrap();
            assert_eq!(data, Some(b"chunk_32_0".to_vec()));

            // Non-existent chunk
            let data = manager.read_chunk((100, 100)).unwrap();
            assert!(data.is_none());

            manager.flush_all().unwrap();
        }

        // Clean up
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_region_stats() {
        let manager = RegionManager::new("./test");
        let stats = manager.stats();
        assert_eq!(stats.cached_regions, 0);
        assert_eq!(stats.total_chunks, 0);
    }
}
