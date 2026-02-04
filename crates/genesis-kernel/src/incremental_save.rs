//! Incremental save system.
//!
//! Provides efficient saving of modified chunks:
//! - Dirty chunk tracking
//! - Delta encoding for changes
//! - Background save queue
//! - Periodic auto-save
//!
//! # Example
//!
//! ```
//! use genesis_kernel::incremental_save::{
//!     IncrementalSaver, SaveConfig, SavePriority,
//! };
//!
//! let config = SaveConfig::default();
//! let mut saver = IncrementalSaver::new(config);
//!
//! // Mark chunks as dirty
//! saver.mark_dirty((0, 0), SavePriority::Normal);
//! saver.mark_dirty((1, 0), SavePriority::High);
//!
//! // Process save queue
//! while let Some(chunk) = saver.next_chunk_to_save() {
//!     println!("Saving chunk {:?}", chunk);
//!     saver.mark_saved(chunk);
//! }
//! ```

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::{Duration, Instant};

/// Save priority for chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SavePriority {
    /// Low priority (background saves).
    Low = 0,
    /// Normal priority (regular dirty chunks).
    Normal = 1,
    /// High priority (player-modified chunks).
    High = 2,
    /// Critical priority (must save immediately).
    Critical = 3,
}

impl SavePriority {
    /// Convert from u8.
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Low),
            1 => Some(Self::Normal),
            2 => Some(Self::High),
            3 => Some(Self::Critical),
            _ => None,
        }
    }
}

impl Default for SavePriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl PartialOrd for SavePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SavePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

/// Save entry for the priority queue.
#[derive(Debug, Clone, Eq, PartialEq)]
struct SaveEntry {
    /// Chunk position.
    position: (i32, i32),
    /// Save priority.
    priority: SavePriority,
    /// Time when marked dirty.
    dirty_time: Instant,
}

impl Ord for SaveEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older dirty time
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.dirty_time.cmp(&self.dirty_time),
            other => other,
        }
    }
}

impl PartialOrd for SaveEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Save configuration.
#[derive(Debug, Clone)]
pub struct SaveConfig {
    /// Maximum chunks to save per tick.
    pub max_chunks_per_tick: usize,
    /// Auto-save interval.
    pub auto_save_interval: Duration,
    /// Maximum dirty chunks before forced save.
    pub max_dirty_chunks: usize,
    /// Enable delta encoding.
    pub use_delta_encoding: bool,
    /// Minimum time between saves for same chunk.
    pub chunk_save_cooldown: Duration,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            max_chunks_per_tick: 4,
            auto_save_interval: Duration::from_secs(60),
            max_dirty_chunks: 1024,
            use_delta_encoding: true,
            chunk_save_cooldown: Duration::from_secs(5),
        }
    }
}

impl SaveConfig {
    /// Create config for aggressive saving.
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            max_chunks_per_tick: 16,
            auto_save_interval: Duration::from_secs(30),
            max_dirty_chunks: 256,
            use_delta_encoding: true,
            chunk_save_cooldown: Duration::from_secs(2),
        }
    }

    /// Create config for minimal saving.
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            max_chunks_per_tick: 1,
            auto_save_interval: Duration::from_secs(300),
            max_dirty_chunks: 4096,
            use_delta_encoding: false,
            chunk_save_cooldown: Duration::from_secs(30),
        }
    }
}

/// Delta operation for incremental changes.
#[derive(Debug, Clone)]
pub enum DeltaOp {
    /// Set a single cell.
    SetCell {
        /// Local index in chunk.
        index: u32,
        /// New cell data.
        data: [u8; 8],
    },
    /// Set a range of cells to same value.
    FillRange {
        /// Start index.
        start: u32,
        /// Number of cells.
        count: u32,
        /// Cell data.
        data: [u8; 8],
    },
    /// Copy from previous version.
    CopyRange {
        /// Destination start.
        dst_start: u32,
        /// Source start.
        src_start: u32,
        /// Number of cells.
        count: u32,
    },
}

impl DeltaOp {
    /// Estimate size of this operation in bytes.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Different expressions document different semantics
    pub fn size_estimate(&self) -> usize {
        match self {
            Self::SetCell { .. } => 1 + 4 + 8,       // op + index + data
            Self::FillRange { .. } => 1 + 4 + 4 + 8, // op + start + count + data
            Self::CopyRange { .. } => 1 + 4 + 4 + 4, // op + dst + src + count
        }
    }
}

/// Delta patch for chunk changes.
#[derive(Debug, Clone, Default)]
pub struct ChunkDelta {
    /// Chunk position.
    pub position: (i32, i32),
    /// Version this delta applies to.
    pub base_version: u64,
    /// Delta operations.
    pub operations: Vec<DeltaOp>,
}

impl ChunkDelta {
    /// Create a new delta.
    #[must_use]
    pub fn new(position: (i32, i32), base_version: u64) -> Self {
        Self {
            position,
            base_version,
            operations: Vec::new(),
        }
    }

    /// Add a set cell operation.
    pub fn set_cell(&mut self, index: u32, data: [u8; 8]) {
        self.operations.push(DeltaOp::SetCell { index, data });
    }

    /// Add a fill range operation.
    pub fn fill_range(&mut self, start: u32, count: u32, data: [u8; 8]) {
        self.operations
            .push(DeltaOp::FillRange { start, count, data });
    }

    /// Check if delta is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Get number of operations.
    #[must_use]
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Estimate serialized size.
    #[must_use]
    pub fn size_estimate(&self) -> usize {
        let header = 8 + 8 + 4; // position + version + op count
        let ops: usize = self.operations.iter().map(DeltaOp::size_estimate).sum();
        header + ops
    }
}

/// Chunk state for dirty tracking.
#[derive(Debug, Clone)]
struct ChunkState {
    /// Current version.
    version: u64,
    /// Last saved version.
    saved_version: u64,
    /// Time of last modification.
    last_modified: Instant,
    /// Time of last save.
    last_saved: Option<Instant>,
    /// Accumulated delta.
    delta: ChunkDelta,
}

impl ChunkState {
    fn new(position: (i32, i32)) -> Self {
        Self {
            version: 0,
            saved_version: 0,
            last_modified: Instant::now(),
            last_saved: None,
            delta: ChunkDelta::new(position, 0),
        }
    }

    fn is_dirty(&self) -> bool {
        self.version > self.saved_version
    }
}

/// Incremental save manager.
#[derive(Debug)]
pub struct IncrementalSaver {
    /// Configuration.
    config: SaveConfig,
    /// Priority queue of chunks to save.
    save_queue: BinaryHeap<SaveEntry>,
    /// Set of chunks in queue (for dedup).
    queued_chunks: HashSet<(i32, i32)>,
    /// Chunk states.
    chunk_states: HashMap<(i32, i32), ChunkState>,
    /// Last auto-save time.
    last_auto_save: Instant,
    /// Statistics.
    stats: SaveStats,
}

impl IncrementalSaver {
    /// Create a new incremental saver.
    #[must_use]
    pub fn new(config: SaveConfig) -> Self {
        Self {
            config,
            save_queue: BinaryHeap::new(),
            queued_chunks: HashSet::new(),
            chunk_states: HashMap::new(),
            last_auto_save: Instant::now(),
            stats: SaveStats::default(),
        }
    }

    /// Mark a chunk as dirty.
    pub fn mark_dirty(&mut self, position: (i32, i32), priority: SavePriority) {
        // Update chunk state
        let state = self
            .chunk_states
            .entry(position)
            .or_insert_with(|| ChunkState::new(position));

        state.version += 1;
        state.last_modified = Instant::now();

        // Add to queue if not already queued
        if !self.queued_chunks.contains(&position) {
            self.save_queue.push(SaveEntry {
                position,
                priority,
                dirty_time: Instant::now(),
            });
            self.queued_chunks.insert(position);
            self.stats.chunks_marked_dirty += 1;
        }
    }

    /// Record a cell change for delta encoding.
    pub fn record_cell_change(&mut self, position: (i32, i32), cell_index: u32, data: [u8; 8]) {
        if !self.config.use_delta_encoding {
            return;
        }

        let state = self
            .chunk_states
            .entry(position)
            .or_insert_with(|| ChunkState::new(position));

        state.delta.set_cell(cell_index, data);
    }

    /// Mark a chunk as saved.
    pub fn mark_saved(&mut self, position: (i32, i32)) {
        if let Some(state) = self.chunk_states.get_mut(&position) {
            state.saved_version = state.version;
            state.last_saved = Some(Instant::now());
            state.delta = ChunkDelta::new(position, state.version);
        }

        self.queued_chunks.remove(&position);
        self.stats.chunks_saved += 1;
    }

    /// Get next chunk to save.
    #[must_use]
    pub fn next_chunk_to_save(&mut self) -> Option<(i32, i32)> {
        while let Some(entry) = self.save_queue.pop() {
            self.queued_chunks.remove(&entry.position);

            // Check if still dirty
            if let Some(state) = self.chunk_states.get(&entry.position) {
                if !state.is_dirty() {
                    continue;
                }

                // Check cooldown
                if let Some(last_saved) = state.last_saved {
                    if last_saved.elapsed() < self.config.chunk_save_cooldown
                        && entry.priority != SavePriority::Critical
                    {
                        // Re-queue with lower priority
                        self.save_queue.push(SaveEntry {
                            position: entry.position,
                            priority: SavePriority::Low,
                            dirty_time: entry.dirty_time,
                        });
                        self.queued_chunks.insert(entry.position);
                        continue;
                    }
                }

                return Some(entry.position);
            }
        }

        None
    }

    /// Get batch of chunks to save.
    pub fn get_save_batch(&mut self) -> Vec<(i32, i32)> {
        let mut batch = Vec::with_capacity(self.config.max_chunks_per_tick);

        for _ in 0..self.config.max_chunks_per_tick {
            if let Some(pos) = self.next_chunk_to_save() {
                batch.push(pos);
            } else {
                break;
            }
        }

        batch
    }

    /// Get delta for a chunk (if using delta encoding).
    #[must_use]
    pub fn get_delta(&self, position: (i32, i32)) -> Option<&ChunkDelta> {
        self.chunk_states.get(&position).map(|s| &s.delta)
    }

    /// Check if chunk is dirty.
    #[must_use]
    pub fn is_dirty(&self, position: (i32, i32)) -> bool {
        self.chunk_states
            .get(&position)
            .is_some_and(ChunkState::is_dirty)
    }

    /// Get number of dirty chunks.
    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.chunk_states.values().filter(|s| s.is_dirty()).count()
    }

    /// Check if auto-save is due.
    #[must_use]
    pub fn should_auto_save(&self) -> bool {
        self.last_auto_save.elapsed() >= self.config.auto_save_interval
    }

    /// Check if forced save is needed (too many dirty chunks).
    #[must_use]
    pub fn needs_forced_save(&self) -> bool {
        self.dirty_count() >= self.config.max_dirty_chunks
    }

    /// Record that auto-save was performed.
    pub fn record_auto_save(&mut self) {
        self.last_auto_save = Instant::now();
        self.stats.auto_saves += 1;
    }

    /// Get all dirty chunks (for full save).
    #[must_use]
    pub fn all_dirty_chunks(&self) -> Vec<(i32, i32)> {
        self.chunk_states
            .iter()
            .filter(|(_, s)| s.is_dirty())
            .map(|(&pos, _)| pos)
            .collect()
    }

    /// Clear all dirty state (after full save).
    pub fn clear_all_dirty(&mut self) {
        for state in self.chunk_states.values_mut() {
            state.saved_version = state.version;
            state.last_saved = Some(Instant::now());
            state.delta = ChunkDelta::new(state.delta.position, state.version);
        }

        self.save_queue.clear();
        self.queued_chunks.clear();
    }

    /// Remove chunk from tracking.
    pub fn remove_chunk(&mut self, position: (i32, i32)) {
        self.chunk_states.remove(&position);
        self.queued_chunks.remove(&position);
    }

    /// Get statistics.
    #[must_use]
    pub fn stats(&self) -> &SaveStats {
        &self.stats
    }

    /// Reset statistics.
    pub fn reset_stats(&mut self) {
        self.stats = SaveStats::default();
    }

    /// Get queue length.
    #[must_use]
    pub fn queue_length(&self) -> usize {
        self.save_queue.len()
    }
}

impl Default for IncrementalSaver {
    fn default() -> Self {
        Self::new(SaveConfig::default())
    }
}

/// Save statistics.
#[derive(Debug, Clone, Default)]
pub struct SaveStats {
    /// Total chunks marked dirty.
    pub chunks_marked_dirty: u64,
    /// Total chunks saved.
    pub chunks_saved: u64,
    /// Number of auto-saves performed.
    pub auto_saves: u64,
    /// Number of forced saves.
    pub forced_saves: u64,
    /// Bytes saved using delta encoding.
    pub delta_bytes_saved: u64,
}

impl SaveStats {
    /// Get save efficiency (saved / dirty).
    #[must_use]
    pub fn efficiency(&self) -> f32 {
        if self.chunks_marked_dirty == 0 {
            1.0
        } else {
            self.chunks_saved as f32 / self.chunks_marked_dirty as f32
        }
    }
}

/// Save request for background thread.
#[derive(Debug, Clone)]
pub struct SaveRequest {
    /// Chunk position.
    pub position: (i32, i32),
    /// Priority.
    pub priority: SavePriority,
    /// Full chunk data (if not using delta).
    pub full_data: Option<Vec<u8>>,
    /// Delta data (if using delta encoding).
    pub delta: Option<ChunkDelta>,
}

impl SaveRequest {
    /// Create full save request.
    #[must_use]
    pub fn full(position: (i32, i32), priority: SavePriority, data: Vec<u8>) -> Self {
        Self {
            position,
            priority,
            full_data: Some(data),
            delta: None,
        }
    }

    /// Create delta save request.
    #[must_use]
    pub fn delta(position: (i32, i32), priority: SavePriority, delta: ChunkDelta) -> Self {
        Self {
            position,
            priority,
            full_data: None,
            delta: Some(delta),
        }
    }

    /// Check if this is a delta save.
    #[must_use]
    pub fn is_delta(&self) -> bool {
        self.delta.is_some()
    }
}

/// Save response from background thread.
#[derive(Debug, Clone)]
pub struct SaveResponse {
    /// Chunk position.
    pub position: (i32, i32),
    /// Whether save succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// Bytes written.
    pub bytes_written: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_priority() {
        assert!(SavePriority::Critical > SavePriority::High);
        assert!(SavePriority::High > SavePriority::Normal);
        assert!(SavePriority::Normal > SavePriority::Low);
    }

    #[test]
    fn test_save_priority_from_u8() {
        assert_eq!(SavePriority::from_u8(0), Some(SavePriority::Low));
        assert_eq!(SavePriority::from_u8(1), Some(SavePriority::Normal));
        assert_eq!(SavePriority::from_u8(2), Some(SavePriority::High));
        assert_eq!(SavePriority::from_u8(3), Some(SavePriority::Critical));
        assert_eq!(SavePriority::from_u8(99), None);
    }

    #[test]
    fn test_save_config_presets() {
        let default = SaveConfig::default();
        assert_eq!(default.max_chunks_per_tick, 4);

        let aggressive = SaveConfig::aggressive();
        assert_eq!(aggressive.max_chunks_per_tick, 16);

        let minimal = SaveConfig::minimal();
        assert_eq!(minimal.max_chunks_per_tick, 1);
    }

    #[test]
    fn test_chunk_delta() {
        let mut delta = ChunkDelta::new((0, 0), 1);
        assert!(delta.is_empty());

        delta.set_cell(0, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(!delta.is_empty());
        assert_eq!(delta.operation_count(), 1);
    }

    #[test]
    fn test_incremental_saver_mark_dirty() {
        let mut saver = IncrementalSaver::default();

        saver.mark_dirty((0, 0), SavePriority::Normal);
        assert!(saver.is_dirty((0, 0)));
        assert_eq!(saver.dirty_count(), 1);
    }

    #[test]
    fn test_incremental_saver_mark_saved() {
        let mut saver = IncrementalSaver::default();

        saver.mark_dirty((0, 0), SavePriority::Normal);
        saver.mark_saved((0, 0));

        assert!(!saver.is_dirty((0, 0)));
        assert_eq!(saver.dirty_count(), 0);
    }

    #[test]
    fn test_incremental_saver_priority_order() {
        let mut saver = IncrementalSaver::default();

        saver.mark_dirty((0, 0), SavePriority::Low);
        saver.mark_dirty((1, 0), SavePriority::High);
        saver.mark_dirty((2, 0), SavePriority::Normal);

        // High priority should come first
        let first = saver.next_chunk_to_save();
        assert_eq!(first, Some((1, 0)));
    }

    #[test]
    fn test_incremental_saver_get_save_batch() {
        let mut saver = IncrementalSaver::new(SaveConfig {
            max_chunks_per_tick: 2,
            chunk_save_cooldown: Duration::ZERO, // Disable cooldown for test
            ..Default::default()
        });

        saver.mark_dirty((0, 0), SavePriority::Normal);
        saver.mark_dirty((1, 0), SavePriority::Normal);
        saver.mark_dirty((2, 0), SavePriority::Normal);

        let batch = saver.get_save_batch();
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_incremental_saver_all_dirty_chunks() {
        let mut saver = IncrementalSaver::default();

        saver.mark_dirty((0, 0), SavePriority::Normal);
        saver.mark_dirty((1, 1), SavePriority::Normal);
        saver.mark_dirty((2, 2), SavePriority::Normal);

        let dirty = saver.all_dirty_chunks();
        assert_eq!(dirty.len(), 3);
    }

    #[test]
    fn test_incremental_saver_clear_all_dirty() {
        let mut saver = IncrementalSaver::default();

        saver.mark_dirty((0, 0), SavePriority::Normal);
        saver.mark_dirty((1, 0), SavePriority::Normal);

        saver.clear_all_dirty();

        assert_eq!(saver.dirty_count(), 0);
        assert_eq!(saver.queue_length(), 0);
    }

    #[test]
    fn test_save_stats() {
        let mut saver = IncrementalSaver::default();

        saver.mark_dirty((0, 0), SavePriority::Normal);
        saver.mark_saved((0, 0));

        let stats = saver.stats();
        assert_eq!(stats.chunks_marked_dirty, 1);
        assert_eq!(stats.chunks_saved, 1);
    }

    #[test]
    fn test_save_request_full() {
        let req = SaveRequest::full((0, 0), SavePriority::Normal, vec![1, 2, 3]);
        assert!(!req.is_delta());
        assert!(req.full_data.is_some());
    }

    #[test]
    fn test_save_request_delta() {
        let delta = ChunkDelta::new((0, 0), 1);
        let req = SaveRequest::delta((0, 0), SavePriority::Normal, delta);
        assert!(req.is_delta());
        assert!(req.delta.is_some());
    }

    #[test]
    fn test_delta_op_size_estimate() {
        let op = DeltaOp::SetCell {
            index: 0,
            data: [0; 8],
        };
        assert_eq!(op.size_estimate(), 13);

        let op = DeltaOp::FillRange {
            start: 0,
            count: 10,
            data: [0; 8],
        };
        assert_eq!(op.size_estimate(), 17);
    }

    #[test]
    fn test_record_cell_change() {
        let mut saver = IncrementalSaver::default();

        saver.mark_dirty((0, 0), SavePriority::Normal);
        saver.record_cell_change((0, 0), 100, [1, 2, 3, 4, 5, 6, 7, 8]);

        let delta = saver.get_delta((0, 0));
        assert!(delta.is_some());
        assert_eq!(delta.unwrap().operation_count(), 1);
    }

    #[test]
    fn test_remove_chunk() {
        let mut saver = IncrementalSaver::default();

        saver.mark_dirty((0, 0), SavePriority::Normal);
        saver.remove_chunk((0, 0));

        assert!(!saver.is_dirty((0, 0)));
    }

    #[test]
    fn test_needs_forced_save() {
        let mut saver = IncrementalSaver::new(SaveConfig {
            max_dirty_chunks: 3,
            ..Default::default()
        });

        saver.mark_dirty((0, 0), SavePriority::Normal);
        saver.mark_dirty((1, 0), SavePriority::Normal);
        assert!(!saver.needs_forced_save());

        saver.mark_dirty((2, 0), SavePriority::Normal);
        assert!(saver.needs_forced_save());
    }
}
