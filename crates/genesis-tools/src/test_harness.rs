//! Integration test harness for end-to-end testing.
//!
//! This module provides a headless test environment for:
//! - Simulating multiple frames with scripted inputs
//! - Asserting on world state after simulation
//! - Comparing against golden file snapshots
//! - Running reproducible test scenarios

use crate::replay::{Input, InputFrame, MouseInput};
use genesis_common::{ChunkCoord, EntityId, WorldCoord};
use genesis_kernel::Cell;
use genesis_world::Chunk;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Error types for test harness operations.
#[derive(Debug, Error)]
pub enum HarnessError {
    /// Scenario file not found
    #[error("Scenario not found: {0}")]
    ScenarioNotFound(String),
    /// Scenario parsing failed
    #[error("Failed to parse scenario: {0}")]
    ScenarioParseFailed(String),
    /// Assertion failed
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),
    /// Golden file not found
    #[error("Golden file not found: {0}")]
    GoldenNotFound(String),
    /// Golden file mismatch
    #[error("Golden file mismatch: {0}")]
    GoldenMismatch(String),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for harness operations.
pub type HarnessResult<T> = Result<T, HarnessError>;

/// World snapshot for golden file comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldSnapshot {
    /// Snapshot name/identifier
    pub name: String,
    /// Frame number when snapshot was taken
    pub frame: u64,
    /// World seed
    pub seed: u64,
    /// Chunk snapshots
    pub chunks: Vec<ChunkSnapshot>,
    /// Entity snapshots
    pub entities: Vec<EntitySnapshot>,
    /// Hash of the entire world state
    pub state_hash: u64,
}

impl WorldSnapshot {
    /// Creates a new empty snapshot.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            frame: 0,
            seed: 0,
            chunks: Vec::new(),
            entities: Vec::new(),
            state_hash: 0,
        }
    }

    /// Sets the frame number.
    #[must_use]
    pub const fn with_frame(mut self, frame: u64) -> Self {
        self.frame = frame;
        self
    }

    /// Sets the seed.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Calculates and sets the state hash.
    pub fn calculate_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.frame.hash(&mut hasher);
        self.seed.hash(&mut hasher);

        for chunk in &self.chunks {
            chunk.coord.x.hash(&mut hasher);
            chunk.coord.y.hash(&mut hasher);
            chunk.cell_hash.hash(&mut hasher);
        }

        for entity in &self.entities {
            entity.id.hash(&mut hasher);
            entity.position.x.hash(&mut hasher);
            entity.position.y.hash(&mut hasher);
        }

        self.state_hash = hasher.finish();
    }

    /// Compares two snapshots and returns differences.
    #[must_use]
    pub fn diff(&self, other: &Self) -> SnapshotDiff {
        let mut diff = SnapshotDiff::default();

        if self.frame != other.frame {
            diff.frame_mismatch = Some((self.frame, other.frame));
        }

        if self.seed != other.seed {
            diff.seed_mismatch = Some((self.seed, other.seed));
        }

        // Check chunk differences
        let self_chunks: HashMap<ChunkCoord, &ChunkSnapshot> =
            self.chunks.iter().map(|c| (c.coord, c)).collect();
        let other_chunks: HashMap<ChunkCoord, &ChunkSnapshot> =
            other.chunks.iter().map(|c| (c.coord, c)).collect();

        for (coord, chunk) in &self_chunks {
            if let Some(other_chunk) = other_chunks.get(coord) {
                if chunk.cell_hash != other_chunk.cell_hash {
                    diff.chunk_mismatches.push(*coord);
                }
            } else {
                diff.missing_chunks.push(*coord);
            }
        }

        for coord in other_chunks.keys() {
            if !self_chunks.contains_key(coord) {
                diff.extra_chunks.push(*coord);
            }
        }

        // Check entity differences
        let self_entities: HashMap<u64, &EntitySnapshot> =
            self.entities.iter().map(|e| (e.id, e)).collect();
        let other_entities: HashMap<u64, &EntitySnapshot> =
            other.entities.iter().map(|e| (e.id, e)).collect();

        for (id, entity) in &self_entities {
            if let Some(other_entity) = other_entities.get(id) {
                if entity != other_entity {
                    diff.entity_mismatches.push(*id);
                }
            } else {
                diff.missing_entities.push(*id);
            }
        }

        for id in other_entities.keys() {
            if !self_entities.contains_key(id) {
                diff.extra_entities.push(*id);
            }
        }

        diff
    }

    /// Saves snapshot to a file.
    pub fn save(&self, path: &Path) -> HarnessResult<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| HarnessError::SerializationError(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads snapshot from a file.
    pub fn load(path: &Path) -> HarnessResult<Self> {
        if !path.exists() {
            return Err(HarnessError::GoldenNotFound(path.display().to_string()));
        }
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(|e| HarnessError::SerializationError(e.to_string()))
    }
}

/// Differences between two snapshots.
#[derive(Debug, Clone, Default)]
pub struct SnapshotDiff {
    /// Frame number mismatch (expected, actual)
    pub frame_mismatch: Option<(u64, u64)>,
    /// Seed mismatch (expected, actual)
    pub seed_mismatch: Option<(u64, u64)>,
    /// Chunks with different content
    pub chunk_mismatches: Vec<ChunkCoord>,
    /// Chunks in expected but not actual
    pub missing_chunks: Vec<ChunkCoord>,
    /// Chunks in actual but not expected
    pub extra_chunks: Vec<ChunkCoord>,
    /// Entities with different state
    pub entity_mismatches: Vec<u64>,
    /// Entities in expected but not actual
    pub missing_entities: Vec<u64>,
    /// Entities in actual but not expected
    pub extra_entities: Vec<u64>,
}

impl SnapshotDiff {
    /// Returns whether there are any differences.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frame_mismatch.is_none()
            && self.seed_mismatch.is_none()
            && self.chunk_mismatches.is_empty()
            && self.missing_chunks.is_empty()
            && self.extra_chunks.is_empty()
            && self.entity_mismatches.is_empty()
            && self.missing_entities.is_empty()
            && self.extra_entities.is_empty()
    }

    /// Formats the diff as a human-readable string.
    #[must_use]
    pub fn to_report(&self) -> String {
        use std::fmt::Write;

        let mut report = String::new();

        if let Some((expected, actual)) = self.frame_mismatch {
            let _ = writeln!(report, "Frame mismatch: expected {expected}, got {actual}");
        }

        if let Some((expected, actual)) = self.seed_mismatch {
            let _ = writeln!(report, "Seed mismatch: expected {expected}, got {actual}");
        }

        if !self.chunk_mismatches.is_empty() {
            let _ = writeln!(
                report,
                "Chunk content mismatches: {:?}",
                self.chunk_mismatches
            );
        }

        if !self.missing_chunks.is_empty() {
            let _ = writeln!(report, "Missing chunks: {:?}", self.missing_chunks);
        }

        if !self.extra_chunks.is_empty() {
            let _ = writeln!(report, "Extra chunks: {:?}", self.extra_chunks);
        }

        if !self.entity_mismatches.is_empty() {
            let _ = writeln!(
                report,
                "Entity state mismatches: {:?}",
                self.entity_mismatches
            );
        }

        if !self.missing_entities.is_empty() {
            let _ = writeln!(report, "Missing entities: {:?}", self.missing_entities);
        }

        if !self.extra_entities.is_empty() {
            let _ = writeln!(report, "Extra entities: {:?}", self.extra_entities);
        }

        if report.is_empty() {
            report.push_str("No differences found");
        }

        report
    }
}

/// Snapshot of a chunk's state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkSnapshot {
    /// Chunk coordinate
    pub coord: ChunkCoord,
    /// Hash of cell data
    pub cell_hash: u64,
    /// Sample cells for debugging (optional)
    pub sample_cells: Vec<CellSample>,
}

impl ChunkSnapshot {
    /// Creates a snapshot from a chunk.
    #[must_use]
    pub fn from_chunk(chunk: &Chunk) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let size = chunk.size();

        // Hash all cells
        for y in 0..size {
            for x in 0..size {
                if let Some(cell) = chunk.get_cell(x, y) {
                    cell.material.hash(&mut hasher);
                    cell.flags.hash(&mut hasher);
                    cell.temperature.hash(&mut hasher);
                }
            }
        }

        Self {
            coord: chunk.coord(),
            cell_hash: hasher.finish(),
            sample_cells: Vec::new(),
        }
    }

    /// Adds sample cells at corners and center.
    #[must_use]
    pub fn with_samples(mut self, chunk: &Chunk) -> Self {
        let size = chunk.size();
        let positions = [
            (0, 0),
            (size - 1, 0),
            (0, size - 1),
            (size - 1, size - 1),
            (size / 2, size / 2),
        ];

        for (x, y) in positions {
            if let Some(cell) = chunk.get_cell(x, y) {
                self.sample_cells.push(CellSample {
                    x,
                    y,
                    material: cell.material,
                    flags: cell.flags,
                    temperature: cell.temperature,
                });
            }
        }

        self
    }
}

/// Sample of a cell for debugging.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CellSample {
    /// Local X coordinate
    pub x: u32,
    /// Local Y coordinate
    pub y: u32,
    /// Material ID
    pub material: u16,
    /// Cell flags
    pub flags: u8,
    /// Temperature
    pub temperature: u8,
}

/// Snapshot of an entity's state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EntitySnapshot {
    /// Entity ID (as u64 for serialization)
    pub id: u64,
    /// Entity type name
    pub entity_type: String,
    /// World position
    pub position: WorldCoord,
    /// Whether entity is active
    pub active: bool,
    /// Health (if applicable)
    pub health: Option<u32>,
}

/// Test scenario definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    /// Scenario name
    pub name: String,
    /// Description
    pub description: String,
    /// World seed
    pub seed: u64,
    /// Chunk size for test world
    pub chunk_size: u32,
    /// Initial chunks to load
    pub initial_chunks: Vec<ChunkCoord>,
    /// Scripted input sequence
    pub inputs: Vec<ScriptedInput>,
    /// Assertions to run after simulation
    pub assertions: Vec<TestAssertion>,
    /// Golden file to compare against (if any)
    pub golden_file: Option<String>,
}

impl TestScenario {
    /// Creates a new test scenario.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            seed: 42,
            chunk_size: 64,
            initial_chunks: vec![ChunkCoord::new(0, 0)],
            inputs: Vec::new(),
            assertions: Vec::new(),
            golden_file: None,
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Sets the seed.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Adds a scripted input.
    #[must_use]
    pub fn with_input(mut self, input: ScriptedInput) -> Self {
        self.inputs.push(input);
        self
    }

    /// Adds an assertion.
    #[must_use]
    pub fn with_assertion(mut self, assertion: TestAssertion) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Sets the golden file.
    #[must_use]
    pub fn with_golden(mut self, path: impl Into<String>) -> Self {
        self.golden_file = Some(path.into());
        self
    }

    /// Loads a scenario from a file.
    pub fn load(path: &Path) -> HarnessResult<Self> {
        if !path.exists() {
            return Err(HarnessError::ScenarioNotFound(path.display().to_string()));
        }
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(|e| HarnessError::ScenarioParseFailed(e.to_string()))
    }

    /// Saves a scenario to a file.
    pub fn save(&self, path: &Path) -> HarnessResult<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| HarnessError::SerializationError(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Scripted input for a specific frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptedInput {
    /// Frame to apply input
    pub frame: u64,
    /// Input actions
    pub inputs: Vec<Input>,
    /// Mouse input (if any)
    pub mouse: Option<MouseInput>,
}

impl ScriptedInput {
    /// Creates a new scripted input.
    #[must_use]
    pub fn new(frame: u64) -> Self {
        Self {
            frame,
            inputs: Vec::new(),
            mouse: None,
        }
    }

    /// Adds an input action.
    #[must_use]
    pub fn with_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    /// Sets mouse input.
    #[must_use]
    pub const fn with_mouse(mut self, mouse: MouseInput) -> Self {
        self.mouse = Some(mouse);
        self
    }

    /// Converts to an InputFrame.
    #[must_use]
    pub fn to_input_frame(&self) -> InputFrame {
        InputFrame {
            frame: self.frame,
            inputs: self.inputs.clone(),
            mouse: self.mouse,
            delta_time_us: 16667, // ~60fps
        }
    }
}

/// Test assertion types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestAssertion {
    /// Assert a cell at position has expected values
    CellEquals {
        /// Chunk coordinate
        chunk: ChunkCoord,
        /// Local X coordinate
        x: u32,
        /// Local Y coordinate
        y: u32,
        /// Expected material
        material: u16,
    },
    /// Assert a cell has specific flags set
    CellHasFlags {
        /// Chunk coordinate
        chunk: ChunkCoord,
        /// Local X coordinate
        x: u32,
        /// Local Y coordinate
        y: u32,
        /// Expected flags
        flags: u8,
    },
    /// Assert entity exists
    EntityExists {
        /// Entity ID
        id: u64,
    },
    /// Assert entity at position
    EntityAtPosition {
        /// Entity ID
        id: u64,
        /// Expected position
        position: WorldCoord,
    },
    /// Assert chunk is loaded
    ChunkLoaded {
        /// Chunk coordinate
        coord: ChunkCoord,
    },
    /// Assert minimum entity count
    MinEntityCount {
        /// Minimum count
        count: usize,
    },
    /// Assert state hash matches
    StateHashEquals {
        /// Expected hash
        hash: u64,
    },
}

/// Configuration for the test harness.
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    /// Chunk size for test world
    pub chunk_size: u32,
    /// Whether to capture snapshots
    pub capture_snapshots: bool,
    /// Snapshot interval (frames)
    pub snapshot_interval: u32,
    /// Whether to run in verbose mode
    pub verbose: bool,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64,
            capture_snapshots: true,
            snapshot_interval: 100,
            verbose: false,
        }
    }
}

/// Test harness for integration testing.
#[derive(Debug)]
pub struct TestHarness {
    /// Configuration
    config: HarnessConfig,
    /// World seed
    seed: u64,
    /// Current frame
    frame: u64,
    /// Loaded chunks
    chunks: HashMap<ChunkCoord, Chunk>,
    /// Entity snapshots (simplified for testing)
    entities: HashMap<u64, EntitySnapshot>,
    /// Captured snapshots
    snapshots: Vec<WorldSnapshot>,
    /// Input queue
    input_queue: Vec<ScriptedInput>,
    /// Current input index
    input_index: usize,
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl TestHarness {
    /// Creates a new headless test harness.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(HarnessConfig::default())
    }

    /// Creates a new harness with custom config.
    #[must_use]
    pub fn with_config(config: HarnessConfig) -> Self {
        Self {
            config,
            seed: 0,
            frame: 0,
            chunks: HashMap::new(),
            entities: HashMap::new(),
            snapshots: Vec::new(),
            input_queue: Vec::new(),
            input_index: 0,
        }
    }

    /// Creates a new headless harness (alias for new).
    #[must_use]
    pub fn new_headless() -> Self {
        Self::new()
    }

    /// Sets the world seed.
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    /// Returns the current seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the current frame.
    #[must_use]
    pub const fn frame(&self) -> u64 {
        self.frame
    }

    /// Loads a test scenario.
    pub fn load_scenario(&mut self, path: &Path) -> HarnessResult<TestScenario> {
        let scenario = TestScenario::load(path)?;
        self.seed = scenario.seed;

        // Load initial chunks
        for coord in &scenario.initial_chunks {
            self.load_chunk(*coord);
        }

        // Queue inputs
        self.input_queue.clone_from(&scenario.inputs);
        self.input_index = 0;

        Ok(scenario)
    }

    /// Loads a chunk at the given coordinate.
    pub fn load_chunk(&mut self, coord: ChunkCoord) {
        let chunk = Chunk::new(coord, self.config.chunk_size);
        self.chunks.insert(coord, chunk);
    }

    /// Gets a chunk at the given coordinate.
    #[must_use]
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    /// Gets a mutable chunk at the given coordinate.
    #[must_use]
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    /// Spawns an entity with the given snapshot.
    pub fn spawn_entity(&mut self, entity: EntitySnapshot) {
        self.entities.insert(entity.id, entity);
    }

    /// Gets an entity by ID.
    #[must_use]
    pub fn get_entity(&self, id: u64) -> Option<&EntitySnapshot> {
        self.entities.get(&id)
    }

    /// Simulates N frames with the queued inputs.
    pub fn simulate(&mut self, frames: u32, inputs: &[Input]) {
        for _ in 0..frames {
            // Get current frame's scripted input
            let scripted_input = self
                .input_queue
                .iter()
                .find(|si| si.frame == self.frame)
                .cloned();

            // Combine with provided inputs
            let mut frame_inputs = inputs.to_vec();
            if let Some(si) = &scripted_input {
                frame_inputs.extend(si.inputs.iter().copied());
            }

            // Process inputs (simplified simulation)
            self.process_inputs(&frame_inputs);

            // Capture snapshot if configured
            if self.config.capture_snapshots
                && self.frame > 0
                && self.frame % self.config.snapshot_interval as u64 == 0
            {
                self.snapshots.push(self.snapshot());
            }

            self.frame += 1;
        }
    }

    /// Processes inputs for the current frame.
    fn process_inputs(&mut self, inputs: &[Input]) {
        // Simplified: actual implementation would update world state
        // based on inputs. For testing, we mainly verify the framework.
        // Mark chunks dirty if any movement inputs are received.
        if inputs.iter().any(|i| {
            matches!(
                i,
                Input::MoveLeft | Input::MoveRight | Input::MoveUp | Input::MoveDown
            )
        }) {
            for chunk in self.chunks.values_mut() {
                chunk.mark_dirty();
            }
        }
    }

    /// Asserts a cell has expected values.
    pub fn assert_cell(
        &self,
        coord: ChunkCoord,
        x: u32,
        y: u32,
        expected: Cell,
    ) -> HarnessResult<()> {
        let chunk = self
            .chunks
            .get(&coord)
            .ok_or_else(|| HarnessError::AssertionFailed(format!("Chunk {coord:?} not loaded")))?;

        let cell = chunk.get_cell(x, y).ok_or_else(|| {
            HarnessError::AssertionFailed(format!("Cell ({x}, {y}) out of bounds"))
        })?;

        if cell.material != expected.material {
            return Err(HarnessError::AssertionFailed(format!(
                "Cell ({x}, {y}) material: expected {}, got {}",
                expected.material, cell.material
            )));
        }

        if cell.flags != expected.flags {
            return Err(HarnessError::AssertionFailed(format!(
                "Cell ({x}, {y}) flags: expected {:#x}, got {:#x}",
                expected.flags, cell.flags
            )));
        }

        Ok(())
    }

    /// Asserts an entity exists.
    pub fn assert_entity_exists(&self, id: EntityId) -> HarnessResult<()> {
        let id_val = id.raw();
        if !self.entities.contains_key(&id_val) {
            return Err(HarnessError::AssertionFailed(format!(
                "Entity {id_val} not found"
            )));
        }
        Ok(())
    }

    /// Takes a snapshot of the current world state.
    #[must_use]
    pub fn snapshot(&self) -> WorldSnapshot {
        let mut snapshot = WorldSnapshot::new(format!("frame_{}", self.frame))
            .with_frame(self.frame)
            .with_seed(self.seed);

        // Snapshot chunks
        for chunk in self.chunks.values() {
            snapshot
                .chunks
                .push(ChunkSnapshot::from_chunk(chunk).with_samples(chunk));
        }

        // Snapshot entities
        snapshot.entities = self.entities.values().cloned().collect();

        // Calculate hash
        snapshot.calculate_hash();

        snapshot
    }

    /// Runs assertions from a scenario.
    pub fn run_assertions(&self, assertions: &[TestAssertion]) -> Vec<HarnessResult<()>> {
        assertions
            .iter()
            .map(|assertion| self.run_assertion(assertion))
            .collect()
    }

    /// Runs a single assertion.
    pub fn run_assertion(&self, assertion: &TestAssertion) -> HarnessResult<()> {
        match assertion {
            TestAssertion::CellEquals {
                chunk,
                x,
                y,
                material,
            } => {
                let c = self.chunks.get(chunk).ok_or_else(|| {
                    HarnessError::AssertionFailed(format!("Chunk {chunk:?} not loaded"))
                })?;

                let cell = c.get_cell(*x, *y).ok_or_else(|| {
                    HarnessError::AssertionFailed(format!("Cell ({x}, {y}) out of bounds"))
                })?;

                if cell.material != *material {
                    return Err(HarnessError::AssertionFailed(format!(
                        "Cell ({x}, {y}) material: expected {material}, got {}",
                        cell.material
                    )));
                }
                Ok(())
            },

            TestAssertion::CellHasFlags { chunk, x, y, flags } => {
                let c = self.chunks.get(chunk).ok_or_else(|| {
                    HarnessError::AssertionFailed(format!("Chunk {chunk:?} not loaded"))
                })?;

                let cell = c.get_cell(*x, *y).ok_or_else(|| {
                    HarnessError::AssertionFailed(format!("Cell ({x}, {y}) out of bounds"))
                })?;

                if cell.flags & *flags != *flags {
                    return Err(HarnessError::AssertionFailed(format!(
                        "Cell ({x}, {y}) missing flags: expected {flags:#x}, got {:#x}",
                        cell.flags
                    )));
                }
                Ok(())
            },

            TestAssertion::EntityExists { id } => {
                if !self.entities.contains_key(id) {
                    return Err(HarnessError::AssertionFailed(format!(
                        "Entity {id} not found"
                    )));
                }
                Ok(())
            },

            TestAssertion::EntityAtPosition { id, position } => {
                let entity = self.entities.get(id).ok_or_else(|| {
                    HarnessError::AssertionFailed(format!("Entity {id} not found"))
                })?;

                if entity.position != *position {
                    return Err(HarnessError::AssertionFailed(format!(
                        "Entity {id} position: expected {:?}, got {:?}",
                        position, entity.position
                    )));
                }
                Ok(())
            },

            TestAssertion::ChunkLoaded { coord } => {
                if !self.chunks.contains_key(coord) {
                    return Err(HarnessError::AssertionFailed(format!(
                        "Chunk {coord:?} not loaded"
                    )));
                }
                Ok(())
            },

            TestAssertion::MinEntityCount { count } => {
                if self.entities.len() < *count {
                    return Err(HarnessError::AssertionFailed(format!(
                        "Entity count: expected at least {count}, got {}",
                        self.entities.len()
                    )));
                }
                Ok(())
            },

            TestAssertion::StateHashEquals { hash } => {
                let snapshot = self.snapshot();
                if snapshot.state_hash != *hash {
                    return Err(HarnessError::AssertionFailed(format!(
                        "State hash: expected {hash}, got {}",
                        snapshot.state_hash
                    )));
                }
                Ok(())
            },
        }
    }

    /// Compares current state against a golden file.
    pub fn compare_golden(&self, golden_path: &Path) -> HarnessResult<SnapshotDiff> {
        let golden = WorldSnapshot::load(golden_path)?;
        let current = self.snapshot();
        Ok(current.diff(&golden))
    }

    /// Saves current state as a golden file.
    pub fn save_golden(&self, path: &Path) -> HarnessResult<()> {
        let snapshot = self.snapshot();
        snapshot.save(path)
    }
}

/// Test result for reporting.
#[derive(Debug)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Whether test passed
    pub passed: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl TestResult {
    /// Creates a passing result.
    #[must_use]
    pub fn pass(name: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            passed: true,
            error: None,
            duration_ms,
        }
    }

    /// Creates a failing result.
    #[must_use]
    pub fn fail(name: impl Into<String>, error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            passed: false,
            error: Some(error.into()),
            duration_ms,
        }
    }
}

/// Runs a test scenario and returns results.
pub fn run_scenario(scenario: &TestScenario) -> TestResult {
    use std::time::Instant;

    let start = Instant::now();
    let name = scenario.name.clone();

    let mut harness = TestHarness::new_headless();
    harness.set_seed(scenario.seed);

    // Load initial chunks
    for coord in &scenario.initial_chunks {
        harness.load_chunk(*coord);
    }

    // Queue and run inputs
    let max_frame = scenario.inputs.iter().map(|i| i.frame).max().unwrap_or(0);

    harness.input_queue.clone_from(&scenario.inputs);
    harness.simulate((max_frame + 1) as u32, &[]);

    // Run assertions
    for assertion in &scenario.assertions {
        if let Err(e) = harness.run_assertion(assertion) {
            return TestResult::fail(&name, e.to_string(), start.elapsed().as_millis() as u64);
        }
    }

    // Compare golden if specified
    if let Some(golden_path) = &scenario.golden_file {
        let path = Path::new(golden_path);
        match harness.compare_golden(path) {
            Ok(diff) if !diff.is_empty() => {
                return TestResult::fail(
                    &name,
                    format!("Golden mismatch:\n{}", diff.to_report()),
                    start.elapsed().as_millis() as u64,
                );
            },
            Err(HarnessError::GoldenNotFound(_)) => {
                // Golden doesn't exist - consider saving it
                tracing::warn!("Golden file not found: {golden_path}");
            },
            Err(e) => {
                return TestResult::fail(&name, e.to_string(), start.elapsed().as_millis() as u64);
            },
            Ok(_) => {},
        }
    }

    TestResult::pass(&name, start.elapsed().as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = TestHarness::new_headless();
        assert_eq!(harness.frame(), 0);
        assert_eq!(harness.seed(), 0);
    }

    #[test]
    fn test_harness_simulate() {
        let mut harness = TestHarness::new_headless();
        harness.set_seed(42);
        harness.load_chunk(ChunkCoord::new(0, 0));

        harness.simulate(10, &[]);
        assert_eq!(harness.frame(), 10);
    }

    #[test]
    fn test_harness_snapshot() {
        let mut harness = TestHarness::new_headless();
        harness.set_seed(123);
        harness.load_chunk(ChunkCoord::new(0, 0));

        let snapshot = harness.snapshot();
        assert_eq!(snapshot.seed, 123);
        assert_eq!(snapshot.chunks.len(), 1);
    }

    #[test]
    fn test_snapshot_diff_empty() {
        let snapshot1 = WorldSnapshot::new("test").with_seed(42);
        let snapshot2 = WorldSnapshot::new("test").with_seed(42);

        let diff = snapshot1.diff(&snapshot2);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_snapshot_diff_seed_mismatch() {
        let snapshot1 = WorldSnapshot::new("test").with_seed(42);
        let snapshot2 = WorldSnapshot::new("test").with_seed(99);

        let diff = snapshot1.diff(&snapshot2);
        assert!(!diff.is_empty());
        assert_eq!(diff.seed_mismatch, Some((42, 99)));
    }

    #[test]
    fn test_assertion_chunk_loaded() {
        let mut harness = TestHarness::new_headless();
        harness.load_chunk(ChunkCoord::new(1, 2));

        let assertion = TestAssertion::ChunkLoaded {
            coord: ChunkCoord::new(1, 2),
        };
        assert!(harness.run_assertion(&assertion).is_ok());

        let bad_assertion = TestAssertion::ChunkLoaded {
            coord: ChunkCoord::new(99, 99),
        };
        assert!(harness.run_assertion(&bad_assertion).is_err());
    }

    #[test]
    fn test_assertion_entity_exists() {
        let mut harness = TestHarness::new_headless();
        harness.spawn_entity(EntitySnapshot {
            id: 42,
            entity_type: "Player".into(),
            position: WorldCoord::new(0, 0),
            active: true,
            health: Some(100),
        });

        let assertion = TestAssertion::EntityExists { id: 42 };
        assert!(harness.run_assertion(&assertion).is_ok());

        let bad_assertion = TestAssertion::EntityExists { id: 999 };
        assert!(harness.run_assertion(&bad_assertion).is_err());
    }

    #[test]
    fn test_assertion_min_entity_count() {
        let mut harness = TestHarness::new_headless();

        for i in 0..5 {
            harness.spawn_entity(EntitySnapshot {
                id: i,
                entity_type: "NPC".into(),
                position: WorldCoord::new(i as i64, 0),
                active: true,
                health: None,
            });
        }

        let assertion = TestAssertion::MinEntityCount { count: 5 };
        assert!(harness.run_assertion(&assertion).is_ok());

        let bad_assertion = TestAssertion::MinEntityCount { count: 10 };
        assert!(harness.run_assertion(&bad_assertion).is_err());
    }

    #[test]
    fn test_test_scenario_creation() {
        let scenario = TestScenario::new("basic_test")
            .with_description("A basic test scenario")
            .with_seed(42)
            .with_input(ScriptedInput::new(0).with_input(Input::MoveRight))
            .with_assertion(TestAssertion::ChunkLoaded {
                coord: ChunkCoord::new(0, 0),
            });

        assert_eq!(scenario.name, "basic_test");
        assert_eq!(scenario.seed, 42);
        assert_eq!(scenario.inputs.len(), 1);
        assert_eq!(scenario.assertions.len(), 1);
    }

    #[test]
    fn test_scripted_input() {
        let input = ScriptedInput::new(5)
            .with_input(Input::Jump)
            .with_input(Input::MoveLeft);

        assert_eq!(input.frame, 5);
        assert_eq!(input.inputs.len(), 2);

        let frame = input.to_input_frame();
        assert_eq!(frame.frame, 5);
        assert_eq!(frame.inputs.len(), 2);
    }

    #[test]
    fn test_run_scenario() {
        let scenario = TestScenario::new("simple_test")
            .with_seed(42)
            .with_assertion(TestAssertion::ChunkLoaded {
                coord: ChunkCoord::new(0, 0),
            });

        let result = run_scenario(&scenario);
        assert!(result.passed);
    }

    #[test]
    fn test_chunk_snapshot() {
        let chunk = Chunk::new(ChunkCoord::new(1, 2), 64);
        let snapshot = ChunkSnapshot::from_chunk(&chunk).with_samples(&chunk);

        assert_eq!(snapshot.coord, ChunkCoord::new(1, 2));
        assert_eq!(snapshot.sample_cells.len(), 5); // corners + center
    }

    #[test]
    fn test_snapshot_diff_report() {
        let mut diff = SnapshotDiff::default();
        diff.seed_mismatch = Some((42, 99));
        diff.missing_entities.push(123);

        let report = diff.to_report();
        assert!(report.contains("Seed mismatch"));
        assert!(report.contains("Missing entities"));
    }

    #[test]
    fn test_test_result() {
        let pass = TestResult::pass("test1", 100);
        assert!(pass.passed);
        assert!(pass.error.is_none());

        let fail = TestResult::fail("test2", "Something went wrong", 50);
        assert!(!fail.passed);
        assert!(fail.error.is_some());
    }
}
