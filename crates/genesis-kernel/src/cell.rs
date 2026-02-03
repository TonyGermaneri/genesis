//! Pixel-cell data structures.

use bytemuck::{Pod, Zeroable};

/// A single simulated pixel-cell.
///
/// Each pixel in the world is a cell with its own state.
/// The cell format is designed for GPU efficiency:
/// - 8 bytes total (cache-line friendly)
/// - Material ID determines physical properties
/// - Flags for dynamic state
/// - Temperature for thermal simulation
/// - Velocity for fluid/particle simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Cell {
    /// Material type ID (0 = air/void)
    pub material: u16,
    /// Cell flags (see CellFlags)
    pub flags: u8,
    /// Temperature (0-255 mapped to temperature range)
    pub temperature: u8,
    /// X velocity component (signed, -128 to 127)
    pub velocity_x: i8,
    /// Y velocity component (signed, -128 to 127)
    pub velocity_y: i8,
    /// Additional data (material-specific)
    pub data: u16,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            material: 0,
            flags: 0,
            temperature: 20, // Room temperature
            velocity_x: 0,
            velocity_y: 0,
            data: 0,
        }
    }
}

impl Cell {
    /// Creates a new cell with the given material.
    #[must_use]
    pub const fn new(material: u16) -> Self {
        Self {
            material,
            flags: 0,
            temperature: 20,
            velocity_x: 0,
            velocity_y: 0,
            data: 0,
        }
    }

    /// Creates an air/void cell.
    #[must_use]
    pub const fn air() -> Self {
        Self::new(0)
    }

    /// Checks if this cell is empty (air/void).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.material == 0
    }

    /// Checks if this cell is solid.
    #[must_use]
    pub const fn is_solid(&self) -> bool {
        self.flags & CellFlags::SOLID.bits() != 0
    }
}

/// Cell flag bits.
pub struct CellFlags;

impl CellFlags {
    /// Cell is solid (blocks movement)
    pub const SOLID: CellFlags = CellFlags;
    /// Cell is liquid (flows)
    pub const LIQUID: CellFlags = CellFlags;
    /// Cell is on fire
    pub const BURNING: CellFlags = CellFlags;
    /// Cell is electrified
    pub const ELECTRIC: CellFlags = CellFlags;
    /// Cell was updated this frame
    pub const UPDATED: CellFlags = CellFlags;

    /// Returns the bit value for this flag.
    #[must_use]
    pub const fn bits(&self) -> u8 {
        1
    }
}

/// Material properties lookup table entry.
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct MaterialProperties {
    /// Density (affects falling/floating)
    pub density: u16,
    /// Friction coefficient
    pub friction: u8,
    /// Flammability (0 = fireproof)
    pub flammability: u8,
    /// Conductivity (thermal/electric)
    pub conductivity: u8,
    /// Hardness (for breaking/mining)
    pub hardness: u8,
    /// Flags for material behavior
    pub flags: u8,
    /// Reserved for alignment
    reserved: u8,
}
