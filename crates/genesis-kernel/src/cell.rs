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
        self.flags & CellFlags::SOLID != 0
    }

    /// Checks if this cell is liquid.
    #[must_use]
    pub const fn is_liquid(&self) -> bool {
        self.flags & CellFlags::LIQUID != 0
    }

    /// Checks if this cell is burning.
    #[must_use]
    pub const fn is_burning(&self) -> bool {
        self.flags & CellFlags::BURNING != 0
    }

    /// Returns the cell with a flag set.
    #[must_use]
    pub const fn with_flag(mut self, flag: u8) -> Self {
        self.flags |= flag;
        self
    }

    /// Returns the cell with temperature set.
    #[must_use]
    pub const fn with_temperature(mut self, temp: u8) -> Self {
        self.temperature = temp;
        self
    }

    /// Returns the cell with velocity set.
    #[must_use]
    pub const fn with_velocity(mut self, vx: i8, vy: i8) -> Self {
        self.velocity_x = vx;
        self.velocity_y = vy;
        self
    }

    /// Returns the cell with biome ID set (stored in low byte of data field).
    /// The shader reads this from `(velocity_data >> 16) & 0xFF`.
    #[must_use]
    pub const fn with_biome(mut self, biome_id: u8) -> Self {
        // biome_id goes in the low byte of data, elevation in high byte
        self.data = (self.data & 0xFF00) | (biome_id as u16);
        self
    }

    /// Returns the cell with elevation set (stored in high byte of data field).
    /// The shader reads this from `(velocity_data >> 24) & 0xFF`.
    #[must_use]
    pub const fn with_elevation(mut self, elevation: u8) -> Self {
        self.data = (self.data & 0x00FF) | ((elevation as u16) << 8);
        self
    }

    /// Returns the cell with both biome and elevation set.
    #[must_use]
    pub const fn with_biome_elevation(mut self, biome_id: u8, elevation: u8) -> Self {
        self.data = (biome_id as u16) | ((elevation as u16) << 8);
        self
    }
}

/// Cell flag bits.
pub struct CellFlags;

impl CellFlags {
    /// Cell is solid (blocks movement) - bit 0
    pub const SOLID: u8 = 1 << 0;
    /// Cell is liquid (flows) - bit 1
    pub const LIQUID: u8 = 1 << 1;
    /// Cell is on fire - bit 2
    pub const BURNING: u8 = 1 << 2;
    /// Cell is electrified - bit 3
    pub const ELECTRIC: u8 = 1 << 3;
    /// Cell was updated this frame - bit 4
    pub const UPDATED: u8 = 1 << 4;
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
    /// Reserved for alignment (must be public for bytemuck)
    pub reserved: u8,
}
