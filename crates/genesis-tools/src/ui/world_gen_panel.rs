//! World Generation Panel
//!
//! UI panel for configuring cubiomes-based world generation.
//! Exposes all practical settings: MC version, seed, scale, flags,
//! and per-biome color/texture editing.

use egui::{Color32, RichText, Ui};

/// Actions produced by the World Generation panel.
#[derive(Debug, Clone)]
pub enum WorldGenAction {
    /// Regenerate the world with updated config.
    Regenerate {
        /// Minecraft version ID.
        mc_version: i32,
        /// World seed.
        seed: u64,
        /// Generator flags (e.g. LARGE_BIOMES).
        flags: u32,
        /// Biome generation scale (1, 4, 16, 64, 256).
        scale: i32,
        /// Y level for biome sampling.
        y_level: i32,
        /// Tile size in world units (smaller = higher resolution).
        tile_size: f32,
    },
    /// Update a biome's display color.
    SetBiomeColor {
        /// Biome ID.
        biome_id: i32,
        /// New RGB color.
        color: [u8; 3],
    },
    /// Update a biome's texture path.
    SetBiomeTexture {
        /// Biome ID.
        biome_id: i32,
        /// Path to texture file.
        path: String,
    },
    /// Reset all biome visuals to cubiomes defaults.
    ResetBiomeColors,
}

/// MC version entry for the dropdown.
#[derive(Debug, Clone)]
struct McVersionEntry {
    id: i32,
    label: String,
}

/// Biome entry for the list editor.
#[derive(Debug, Clone)]
pub struct BiomeUiEntry {
    /// Biome ID from cubiomes.
    pub id: i32,
    /// Display name of the biome.
    pub name: String,
    /// Current RGB color.
    pub color: [u8; 3],
    /// Optional texture path override.
    pub texture_path: Option<String>,
}

/// Sub-section currently shown in the biome editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BiomeSection {
    #[default]
    Overworld,
    Ocean,
    Mountain,
    Forest,
    Other,
}

impl BiomeSection {
    fn all() -> &'static [Self] {
        &[Self::Overworld, Self::Ocean, Self::Mountain, Self::Forest, Self::Other]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Overworld => "Common",
            Self::Ocean => "Ocean",
            Self::Mountain => "Mountain",
            Self::Forest => "Forest/Jungle",
            Self::Other => "Other",
        }
    }

    fn matches(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        match self {
            Self::Overworld => {
                matches!(lower.as_str(),
                    "plains" | "desert" | "swamp" | "river" | "beach" | "savanna"
                    | "badlands" | "mushroom_fields" | "mushroom_field_shore"
                    | "meadow" | "cherry_grove" | "pale_garden" | "mangrove_swamp"
                    | "snowy_tundra" | "snowy_beach" | "stone_shore"
                    | "sunflower_plains" | "flower_forest" | "ice_spikes"
                    | "desert_lakes" | "desert_hills" | "swamp_hills"
                ) || lower.contains("plains") || lower.contains("desert")
                    || lower.contains("swamp") || lower.contains("meadow")
                    || lower.contains("beach") || lower.contains("savanna")
                    || lower.contains("badlands")
            }
            Self::Ocean => lower.contains("ocean") || lower.contains("river")
                || lower.contains("frozen_river"),
            Self::Mountain => lower.contains("mountain") || lower.contains("peak")
                || lower.contains("slope") || lower.contains("grove")
                || lower.contains("stony") || lower.contains("gravelly"),
            Self::Forest => lower.contains("forest") || lower.contains("jungle")
                || lower.contains("taiga") || lower.contains("birch")
                || lower.contains("dark_forest") || lower.contains("bamboo")
                || lower.contains("spruce"),
            Self::Other => true, // Catch-all
        }
    }
}

/// World Generation panel state.
pub struct WorldGenPanel {
    // === Config fields ===
    /// Current MC version index into the version list.
    mc_version_idx: usize,
    /// Available MC versions.
    mc_versions: Vec<McVersionEntry>,
    /// Seed as editable string.
    seed_str: String,
    /// Parsed seed value.
    seed: u64,
    /// Large biomes flag.
    large_biomes: bool,
    /// Force ocean variants flag.
    force_ocean_variants: bool,
    /// Generation scale (1, 4, 16, 64, 256).
    scale: i32,
    /// Y level for sampling.
    y_level: i32,
    /// Tile size in world units (smaller = higher visual resolution).
    tile_size: f32,

    // === Biome editor ===
    /// All biome entries for editing.
    biome_entries: Vec<BiomeUiEntry>,
    /// Filter text for biome search.
    biome_filter: String,
    /// Active biome section filter.
    biome_section: BiomeSection,
    /// Texture path editing buffer (biome_id → path string).
    texture_edit_buffers: std::collections::HashMap<i32, String>,

    // === Actions ===
    /// Pending actions.
    actions: Vec<WorldGenAction>,
}

impl Default for WorldGenPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldGenPanel {
    /// Create a new panel with default settings.
    pub fn new() -> Self {
        // Build MC version list
        let mc_versions: Vec<McVersionEntry> = cubiomes_sys::all_mc_versions()
            .into_iter()
            .map(|(id, name)| McVersionEntry {
                id,
                label: name,
            })
            .collect();

        // Default to latest version (last in list)
        let mc_version_idx = mc_versions.len().saturating_sub(1);

        Self {
            mc_version_idx,
            mc_versions,
            seed_str: "0".to_string(),
            seed: 0,
            large_biomes: false,
            force_ocean_variants: false,
            scale: 1,
            y_level: 64,
            tile_size: 1.0,
            biome_entries: Vec::new(),
            biome_filter: String::new(),
            biome_section: BiomeSection::default(),
            texture_edit_buffers: std::collections::HashMap::new(),
            actions: Vec::new(),
        }
    }

    /// Set biome entries from the engine's biome texture map.
    pub fn set_biome_entries(&mut self, entries: Vec<BiomeUiEntry>) {
        self.biome_entries = entries;
    }

    /// Update config from engine state (call when panel is opened or world regenerated).
    pub fn sync_config(&mut self, mc_version: i32, seed: u64, flags: u32, scale: i32, y_level: i32, tile_size: f32) {
        // Find MC version index
        if let Some(idx) = self.mc_versions.iter().position(|v| v.id == mc_version) {
            self.mc_version_idx = idx;
        }
        self.seed = seed;
        self.seed_str = seed.to_string();
        self.large_biomes = (flags & cubiomes_sys::LARGE_BIOMES) != 0;
        self.force_ocean_variants = (flags & cubiomes_sys::FORCE_OCEAN_VARIANTS) != 0;
        self.scale = scale;
        self.y_level = y_level;
        self.tile_size = tile_size;
    }

    /// Drain all pending actions.
    pub fn drain_actions(&mut self) -> Vec<WorldGenAction> {
        std::mem::take(&mut self.actions)
    }

    /// Build current flags from UI state.
    fn current_flags(&self) -> u32 {
        let mut flags = 0u32;
        if self.large_biomes { flags |= cubiomes_sys::LARGE_BIOMES; }
        if self.force_ocean_variants { flags |= cubiomes_sys::FORCE_OCEAN_VARIANTS; }
        flags
    }

    /// Get current MC version ID.
    fn current_mc_version(&self) -> i32 {
        self.mc_versions.get(self.mc_version_idx)
            .map(|v| v.id)
            .unwrap_or(cubiomes_sys::MC_1_21)
    }

    /// Render the World Generation panel.
    pub fn render(&mut self, ui: &mut Ui) {
        ui.heading(RichText::new("🌍 World Generation").color(Color32::WHITE).size(16.0));
        ui.add_space(4.0);

        // === Generation Settings ===
        egui::CollapsingHeader::new(RichText::new("⚙ Generation Settings").color(Color32::LIGHT_GRAY).size(14.0))
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(4.0);

                // MC Version dropdown
                ui.horizontal(|ui| {
                    ui.label(RichText::new("MC Version:").color(Color32::GRAY));
                    let current_label = self.mc_versions.get(self.mc_version_idx)
                        .map(|v| v.label.as_str())
                        .unwrap_or("Unknown");
                    egui::ComboBox::from_id_salt("mc_version")
                        .selected_text(current_label)
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            for (idx, version) in self.mc_versions.iter().enumerate() {
                                ui.selectable_value(&mut self.mc_version_idx, idx, &version.label);
                            }
                        });
                });

                ui.add_space(2.0);

                // Seed input
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Seed:").color(Color32::GRAY));
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.seed_str)
                            .desired_width(200.0)
                            .hint_text("Enter seed (number)")
                    );
                    if response.lost_focus() || response.changed() {
                        self.seed = self.seed_str.parse::<u64>().unwrap_or(0);
                    }
                    if ui.button("🎲 Random").clicked() {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        self.seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(42);
                        self.seed_str = self.seed.to_string();
                    }
                });

                ui.add_space(2.0);

                // Scale selector
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Scale:").color(Color32::GRAY));
                    for &s in &[1, 4, 16, 64, 256] {
                        let label = format!("1:{}", s);
                        if ui.selectable_label(self.scale == s, &label).clicked() {
                            self.scale = s;
                        }
                    }
                });

                ui.add_space(2.0);

                // Resolution (tile size) — smaller values = higher visual detail
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Tile Size:").color(Color32::GRAY));
                    ui.add(egui::Slider::new(&mut self.tile_size, 0.5..=16.0)
                        .logarithmic(true)
                        .text("px")
                        .max_decimals(1));
                });

                ui.add_space(2.0);

                // Y level
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Y Level:").color(Color32::GRAY));
                    ui.add(egui::Slider::new(&mut self.y_level, -64..=320).text("blocks"));
                });

                ui.add_space(2.0);

                // Flags
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.large_biomes, RichText::new("Large Biomes").color(Color32::LIGHT_GRAY));
                    ui.checkbox(&mut self.force_ocean_variants, RichText::new("Force Ocean Variants").color(Color32::LIGHT_GRAY));
                });

                ui.add_space(8.0);

                // Regenerate button
                let regen_btn = ui.add_sized(
                    [200.0, 32.0],
                    egui::Button::new(
                        RichText::new("🔄 Regenerate World")
                            .color(Color32::WHITE)
                            .strong()
                            .size(14.0)
                    ).fill(Color32::from_rgb(40, 100, 60)),
                );
                if regen_btn.clicked() {
                    self.actions.push(WorldGenAction::Regenerate {
                        mc_version: self.current_mc_version(),
                        seed: self.seed,
                        flags: self.current_flags(),
                        scale: self.scale,
                        y_level: self.y_level,
                        tile_size: self.tile_size,
                    });
                }
            });

        ui.add_space(8.0);

        // === Biome Color/Texture Editor ===
        egui::CollapsingHeader::new(RichText::new("🎨 Biome Visuals").color(Color32::LIGHT_GRAY).size(14.0))
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(4.0);

                // Section filter tabs
                ui.horizontal(|ui| {
                    for section in BiomeSection::all() {
                        let is_active = self.biome_section == *section;
                        let label = section.label();
                        let btn = if is_active {
                            egui::Button::new(RichText::new(label).color(Color32::WHITE).strong().size(12.0))
                                .fill(Color32::from_rgb(60, 60, 100))
                        } else {
                            egui::Button::new(RichText::new(label).color(Color32::LIGHT_GRAY).size(12.0))
                                .fill(Color32::from_rgb(35, 35, 50))
                        };
                        if ui.add(btn).clicked() {
                            self.biome_section = *section;
                        }
                    }
                });

                ui.add_space(4.0);

                // Search filter
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").color(Color32::GRAY));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.biome_filter)
                            .desired_width(200.0)
                            .hint_text("Filter biomes...")
                    );
                    if ui.button("✕").clicked() {
                        self.biome_filter.clear();
                    }
                });

                ui.add_space(4.0);

                // Reset button
                if ui.button(RichText::new("↺ Reset All Colors").color(Color32::LIGHT_GRAY).size(12.0)).clicked() {
                    self.actions.push(WorldGenAction::ResetBiomeColors);
                }

                ui.add_space(4.0);

                // Biome list (scrollable)
                let filter_lower = self.biome_filter.to_lowercase();
                let section = self.biome_section;

                // Collect matching entries
                let matching: Vec<usize> = self.biome_entries.iter().enumerate()
                    .filter(|(_, e)| {
                        // Section filter
                        let section_match = if section == BiomeSection::Other {
                            // "Other" shows everything not matched by specific sections
                            !BiomeSection::Ocean.matches(&e.name)
                                && !BiomeSection::Mountain.matches(&e.name)
                                && !BiomeSection::Forest.matches(&e.name)
                                && !BiomeSection::Overworld.matches(&e.name)
                        } else {
                            section.matches(&e.name)
                        };
                        // Text filter
                        let text_match = filter_lower.is_empty()
                            || e.name.to_lowercase().contains(&filter_lower)
                            || e.id.to_string().contains(&filter_lower);
                        section_match && text_match
                    })
                    .map(|(i, _)| i)
                    .collect();

                ui.label(RichText::new(format!("{} biomes", matching.len())).color(Color32::GRAY).size(11.0));

                egui::ScrollArea::vertical()
                    .max_height(350.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for idx in matching {
                            // Copy data out to avoid borrow conflicts
                            let biome_id = self.biome_entries[idx].id;
                            let biome_name = self.biome_entries[idx].name.clone();
                            let mut color = self.biome_entries[idx].color;
                            let texture_path = self.biome_entries[idx].texture_path.clone();

                            ui.horizontal(|ui| {
                                // Color preview swatch
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(20.0, 20.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    2.0,
                                    Color32::from_rgb(color[0], color[1], color[2]),
                                );

                                // Biome name and ID
                                ui.label(RichText::new(format!("{} ({})", biome_name, biome_id))
                                    .color(Color32::LIGHT_GRAY)
                                    .size(12.0));

                                // Color picker
                                let mut egui_color = [
                                    color[0] as f32 / 255.0,
                                    color[1] as f32 / 255.0,
                                    color[2] as f32 / 255.0,
                                ];
                                if ui.color_edit_button_rgb(&mut egui_color).changed() {
                                    color = [
                                        (egui_color[0] * 255.0) as u8,
                                        (egui_color[1] * 255.0) as u8,
                                        (egui_color[2] * 255.0) as u8,
                                    ];
                                    self.biome_entries[idx].color = color;
                                    self.actions.push(WorldGenAction::SetBiomeColor {
                                        biome_id,
                                        color,
                                    });
                                }

                                // Texture path input
                                let buf = self.texture_edit_buffers
                                    .entry(biome_id)
                                    .or_insert_with(|| {
                                        texture_path.unwrap_or_default()
                                    });
                                let tex_response = ui.add(
                                    egui::TextEdit::singleline(buf)
                                        .desired_width(150.0)
                                        .hint_text("texture path")
                                        .font(egui::TextStyle::Small)
                                );
                                if tex_response.lost_focus() && !buf.is_empty() {
                                    self.actions.push(WorldGenAction::SetBiomeTexture {
                                        biome_id,
                                        path: buf.clone(),
                                    });
                                }
                            });

                            ui.add_space(1.0);
                        }
                    });
            });
    }
}
