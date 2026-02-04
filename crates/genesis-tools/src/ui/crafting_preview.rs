//! Crafting preview UI for showing output item information.
//!
//! Provides output preview functionality including:
//! - Item preview with stats display
//! - Material availability indicators
//! - Crafting time estimate
//! - Success/failure probability

use egui::{Color32, Ui, Vec2};
use serde::{Deserialize, Serialize};

use super::crafting_grid::ItemRarity;
use super::recipe_book::{Recipe, RecipeId, RecipeIngredient};

/// Statistics for a craftable item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemStats {
    /// Base damage (weapons).
    pub damage: Option<f32>,
    /// Defense value (armor).
    pub defense: Option<f32>,
    /// Durability.
    pub durability: Option<u32>,
    /// Movement speed modifier.
    pub speed_modifier: Option<f32>,
    /// Tool efficiency.
    pub efficiency: Option<f32>,
    /// Custom stats.
    pub custom: Vec<(String, String)>,
}

impl ItemStats {
    /// Create empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set damage stat.
    pub fn with_damage(mut self, damage: f32) -> Self {
        self.damage = Some(damage);
        self
    }

    /// Set defense stat.
    pub fn with_defense(mut self, defense: f32) -> Self {
        self.defense = Some(defense);
        self
    }

    /// Set durability stat.
    pub fn with_durability(mut self, durability: u32) -> Self {
        self.durability = Some(durability);
        self
    }

    /// Set speed modifier.
    pub fn with_speed_modifier(mut self, modifier: f32) -> Self {
        self.speed_modifier = Some(modifier);
        self
    }

    /// Set efficiency.
    pub fn with_efficiency(mut self, efficiency: f32) -> Self {
        self.efficiency = Some(efficiency);
        self
    }

    /// Add custom stat.
    pub fn with_custom(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.push((name.into(), value.into()));
        self
    }

    /// Check if any stats are defined.
    pub fn has_stats(&self) -> bool {
        self.damage.is_some()
            || self.defense.is_some()
            || self.durability.is_some()
            || self.speed_modifier.is_some()
            || self.efficiency.is_some()
            || !self.custom.is_empty()
    }
}

/// Crafting quality level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CraftQuality {
    /// Poor quality craft.
    Poor,
    /// Normal quality.
    #[default]
    Normal,
    /// Good quality.
    Good,
    /// Excellent quality.
    Excellent,
    /// Perfect/masterwork quality.
    Masterwork,
}

impl CraftQuality {
    /// Get all quality levels.
    pub fn all() -> &'static [CraftQuality] {
        &[
            CraftQuality::Poor,
            CraftQuality::Normal,
            CraftQuality::Good,
            CraftQuality::Excellent,
            CraftQuality::Masterwork,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            CraftQuality::Poor => "Poor",
            CraftQuality::Normal => "Normal",
            CraftQuality::Good => "Good",
            CraftQuality::Excellent => "Excellent",
            CraftQuality::Masterwork => "Masterwork",
        }
    }

    /// Get quality color.
    pub fn color(&self) -> Color32 {
        match self {
            CraftQuality::Poor => Color32::from_rgb(150, 100, 100),
            CraftQuality::Normal => Color32::from_rgb(200, 200, 200),
            CraftQuality::Good => Color32::from_rgb(100, 200, 100),
            CraftQuality::Excellent => Color32::from_rgb(100, 150, 255),
            CraftQuality::Masterwork => Color32::from_rgb(255, 200, 100),
        }
    }

    /// Get stat multiplier.
    pub fn stat_multiplier(&self) -> f32 {
        match self {
            CraftQuality::Poor => 0.75,
            CraftQuality::Normal => 1.0,
            CraftQuality::Good => 1.15,
            CraftQuality::Excellent => 1.30,
            CraftQuality::Masterwork => 1.50,
        }
    }
}

/// Crafting success probability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingProbability {
    /// Base success chance (0.0 - 1.0).
    pub base_success: f32,
    /// Chance for quality upgrade.
    pub quality_chance: f32,
    /// Current player skill modifier.
    pub skill_modifier: f32,
    /// Station quality modifier.
    pub station_modifier: f32,
}

impl Default for CraftingProbability {
    fn default() -> Self {
        Self {
            base_success: 1.0,
            quality_chance: 0.0,
            skill_modifier: 1.0,
            station_modifier: 1.0,
        }
    }
}

impl CraftingProbability {
    /// Create new probability.
    pub fn new(base_success: f32) -> Self {
        Self {
            base_success: base_success.clamp(0.0, 1.0),
            ..Self::default()
        }
    }

    /// Set quality upgrade chance.
    pub fn with_quality_chance(mut self, chance: f32) -> Self {
        self.quality_chance = chance.clamp(0.0, 1.0);
        self
    }

    /// Set skill modifier.
    pub fn with_skill_modifier(mut self, modifier: f32) -> Self {
        self.skill_modifier = modifier.max(0.0);
        self
    }

    /// Set station modifier.
    pub fn with_station_modifier(mut self, modifier: f32) -> Self {
        self.station_modifier = modifier.max(0.0);
        self
    }

    /// Calculate final success chance.
    pub fn final_success_chance(&self) -> f32 {
        (self.base_success * self.skill_modifier * self.station_modifier).clamp(0.0, 1.0)
    }

    /// Get color based on success chance.
    pub fn color(&self) -> Color32 {
        let chance = self.final_success_chance();
        if chance >= 0.9 {
            Color32::from_rgb(100, 200, 100)
        } else if chance >= 0.7 {
            Color32::from_rgb(200, 200, 100)
        } else if chance >= 0.5 {
            Color32::from_rgb(255, 150, 100)
        } else {
            Color32::from_rgb(200, 100, 100)
        }
    }
}

/// Preview data for a craftable item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingPreviewData {
    /// Recipe ID.
    pub recipe_id: RecipeId,
    /// Output item name.
    pub name: String,
    /// Output item description.
    pub description: String,
    /// Output quantity.
    pub quantity: u32,
    /// Item rarity.
    pub rarity: ItemRarity,
    /// Item stats.
    pub stats: ItemStats,
    /// Expected craft quality.
    pub expected_quality: CraftQuality,
    /// Crafting probability.
    pub probability: CraftingProbability,
    /// Required materials.
    pub materials: Vec<MaterialRequirement>,
    /// Craft time in seconds.
    pub craft_time: f32,
    /// Required station name.
    pub station: Option<String>,
}

impl CraftingPreviewData {
    /// Create preview data from recipe.
    pub fn from_recipe(recipe: &Recipe) -> Self {
        Self {
            recipe_id: recipe.id.clone(),
            name: recipe.output.name.clone(),
            description: recipe.output.description.clone(),
            quantity: recipe.output.quantity,
            rarity: recipe.output.rarity,
            stats: ItemStats::default(),
            expected_quality: CraftQuality::Normal,
            probability: CraftingProbability::default(),
            materials: recipe
                .ingredients
                .iter()
                .map(MaterialRequirement::from_ingredient)
                .collect(),
            craft_time: recipe.craft_time,
            station: recipe.station.clone(),
        }
    }

    /// Set item stats.
    pub fn with_stats(mut self, stats: ItemStats) -> Self {
        self.stats = stats;
        self
    }

    /// Set expected quality.
    pub fn with_quality(mut self, quality: CraftQuality) -> Self {
        self.expected_quality = quality;
        self
    }

    /// Set crafting probability.
    pub fn with_probability(mut self, probability: CraftingProbability) -> Self {
        self.probability = probability;
        self
    }

    /// Check if all materials are available.
    pub fn has_all_materials(&self) -> bool {
        self.materials.iter().all(MaterialRequirement::has_enough)
    }

    /// Get missing material count.
    pub fn missing_material_count(&self) -> usize {
        self.materials.iter().filter(|m| !m.has_enough()).count()
    }

    /// Get estimated craft time with modifiers.
    pub fn estimated_time(&self) -> f32 {
        self.craft_time
    }
}

/// Material requirement with availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRequirement {
    /// Material name.
    pub name: String,
    /// Required quantity.
    pub required: u32,
    /// Available quantity.
    pub available: u32,
}

impl MaterialRequirement {
    /// Create from recipe ingredient.
    pub fn from_ingredient(ingredient: &RecipeIngredient) -> Self {
        Self {
            name: ingredient.name.clone(),
            required: ingredient.quantity,
            available: ingredient.available,
        }
    }

    /// Check if enough material is available.
    pub fn has_enough(&self) -> bool {
        self.available >= self.required
    }

    /// Get missing quantity.
    pub fn missing(&self) -> u32 {
        self.required.saturating_sub(self.available)
    }
}

/// Actions returned by the crafting preview.
#[derive(Debug, Clone, PartialEq)]
pub enum CraftingPreviewAction {
    /// Start crafting.
    Craft,
    /// Craft maximum possible quantity.
    CraftMax,
    /// Close preview.
    Close,
    /// View recipe details.
    ViewRecipe(RecipeId),
}

/// Configuration for crafting preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingPreviewConfig {
    /// Show item stats.
    pub show_stats: bool,
    /// Show probability info.
    pub show_probability: bool,
    /// Show materials list.
    pub show_materials: bool,
    /// Show time estimate.
    pub show_time: bool,
    /// Compact mode.
    pub compact: bool,
    /// Preview panel width.
    pub width: f32,
}

impl Default for CraftingPreviewConfig {
    fn default() -> Self {
        Self {
            show_stats: true,
            show_probability: true,
            show_materials: true,
            show_time: true,
            compact: false,
            width: 280.0,
        }
    }
}

/// Crafting preview widget.
#[derive(Debug)]
pub struct CraftingPreview {
    /// Current preview data.
    preview: Option<CraftingPreviewData>,
    /// Configuration.
    pub config: CraftingPreviewConfig,
    /// Craft quantity selector.
    pub craft_quantity: u32,
    /// Whether the preview is visible.
    pub visible: bool,
    /// Pending actions.
    pending_actions: Vec<CraftingPreviewAction>,
}

impl Default for CraftingPreview {
    fn default() -> Self {
        Self::new()
    }
}

impl CraftingPreview {
    /// Create a new crafting preview.
    pub fn new() -> Self {
        Self {
            preview: None,
            config: CraftingPreviewConfig::default(),
            craft_quantity: 1,
            visible: true,
            pending_actions: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: CraftingPreviewConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set preview data.
    pub fn set_preview(&mut self, data: CraftingPreviewData) {
        self.preview = Some(data);
        self.craft_quantity = 1;
    }

    /// Clear preview.
    pub fn clear_preview(&mut self) {
        self.preview = None;
    }

    /// Get current preview data.
    pub fn preview_data(&self) -> Option<&CraftingPreviewData> {
        self.preview.as_ref()
    }

    /// Check if preview is showing something.
    pub fn has_preview(&self) -> bool {
        self.preview.is_some()
    }

    /// Get maximum craftable quantity.
    pub fn max_craftable(&self) -> u32 {
        if let Some(preview) = &self.preview {
            if preview.materials.is_empty() {
                return 99;
            }
            preview
                .materials
                .iter()
                .map(|m| {
                    if m.required > 0 {
                        m.available / m.required
                    } else {
                        99
                    }
                })
                .min()
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Render the crafting preview and return actions.
    pub fn show(&mut self, ui: &mut Ui) -> Vec<CraftingPreviewAction> {
        self.pending_actions.clear();

        if !self.visible {
            return Vec::new();
        }

        let Some(preview) = &self.preview else {
            ui.weak("Select a recipe to preview");
            return Vec::new();
        };

        // Clone data for use in UI
        let name = preview.name.clone();
        let description = preview.description.clone();
        let quantity = preview.quantity;
        let rarity = preview.rarity;
        let expected_quality = preview.expected_quality;
        let craft_time = preview.craft_time;
        let station = preview.station.clone();
        let probability = preview.probability.clone();
        let materials = preview.materials.clone();
        let stats = preview.stats.clone();
        let has_all_materials = preview.has_all_materials();
        let recipe_id = preview.recipe_id.clone();

        let max_craft = self.max_craftable();

        ui.vertical(|ui| {
            // Header
            egui::Frame::none()
                .fill(rarity.color().linear_multiply(0.2))
                .inner_margin(8.0)
                .rounding(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&name)
                                .color(rarity.color())
                                .size(16.0)
                                .strong(),
                        );
                        if quantity > 1 {
                            ui.label(format!("x{quantity}"));
                        }
                    });

                    if !description.is_empty() {
                        ui.label(egui::RichText::new(&description).weak().small());
                    }

                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", rarity.icon(), rarity.display_name()));
                        ui.separator();
                        ui.colored_label(expected_quality.color(), expected_quality.display_name());
                    });
                });

            ui.add_space(4.0);

            // Stats section
            if self.config.show_stats && stats.has_stats() {
                self.show_stats_section(ui, &stats);
                ui.add_space(4.0);
            }

            // Probability section
            if self.config.show_probability {
                self.show_probability_section(ui, &probability);
                ui.add_space(4.0);
            }

            // Materials section
            if self.config.show_materials && !materials.is_empty() {
                self.show_materials_section(ui, &materials);
                ui.add_space(4.0);
            }

            // Time section
            if self.config.show_time {
                self.show_time_section(ui, craft_time, station.as_deref());
                ui.add_space(4.0);
            }

            // Craft controls
            ui.separator();
            self.show_craft_controls(ui, has_all_materials, max_craft, &recipe_id);
        });

        std::mem::take(&mut self.pending_actions)
    }

    /// Show stats section.
    fn show_stats_section(&self, ui: &mut Ui, stats: &ItemStats) {
        let _ = self; // Mark self as used for future extensibility
        ui.collapsing("📊 Stats", |ui| {
            if let Some(damage) = stats.damage {
                ui.horizontal(|ui| {
                    ui.label("⚔ Damage:");
                    ui.strong(format!("{damage:.0}"));
                });
            }
            if let Some(defense) = stats.defense {
                ui.horizontal(|ui| {
                    ui.label("🛡 Defense:");
                    ui.strong(format!("{defense:.0}"));
                });
            }
            if let Some(durability) = stats.durability {
                ui.horizontal(|ui| {
                    ui.label("💪 Durability:");
                    ui.strong(format!("{durability}"));
                });
            }
            if let Some(speed) = stats.speed_modifier {
                let sign = if speed >= 0.0 { "+" } else { "" };
                ui.horizontal(|ui| {
                    ui.label("🏃 Speed:");
                    ui.strong(format!("{sign}{:.0}%", speed * 100.0));
                });
            }
            if let Some(efficiency) = stats.efficiency {
                ui.horizontal(|ui| {
                    ui.label("⚡ Efficiency:");
                    ui.strong(format!("{efficiency:.0}%"));
                });
            }
            for (name, value) in &stats.custom {
                ui.horizontal(|ui| {
                    ui.label(format!("{name}:"));
                    ui.strong(value);
                });
            }
        });
    }

    /// Show probability section.
    fn show_probability_section(&self, ui: &mut Ui, probability: &CraftingProbability) {
        let _ = self; // Mark self as used for future extensibility
        let success = probability.final_success_chance();

        ui.horizontal(|ui| {
            ui.label("Success:");
            let bar_width = 100.0;
            let bar_rect = ui.allocate_space(Vec2::new(bar_width, 16.0)).1;

            ui.painter()
                .rect_filled(bar_rect, 2.0, Color32::from_gray(40));

            let filled_rect = egui::Rect::from_min_size(
                bar_rect.min,
                Vec2::new(bar_rect.width() * success, bar_rect.height()),
            );
            ui.painter()
                .rect_filled(filled_rect, 2.0, probability.color());

            ui.colored_label(probability.color(), format!("{:.0}%", success * 100.0));
        });

        if probability.quality_chance > 0.0 {
            ui.small(format!(
                "✨ {:.0}% quality bonus chance",
                probability.quality_chance * 100.0
            ));
        }
    }

    /// Show materials section.
    fn show_materials_section(&self, ui: &mut Ui, materials: &[MaterialRequirement]) {
        let _ = self; // Mark self as used for future extensibility
        ui.collapsing("📦 Materials", |ui| {
            for mat in materials {
                let has_enough = mat.has_enough();
                let color = if has_enough {
                    Color32::from_rgb(150, 200, 150)
                } else {
                    Color32::from_rgb(200, 100, 100)
                };

                ui.horizontal(|ui| {
                    let icon = if has_enough { "✓" } else { "✗" };
                    ui.colored_label(color, icon);
                    ui.label(&mat.name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(color, format!("{}/{}", mat.available, mat.required));
                    });
                });
            }
        });
    }

    /// Show time section.
    fn show_time_section(&self, ui: &mut Ui, craft_time: f32, station: Option<&str>) {
        let _ = self; // Mark self as used for future extensibility
        ui.horizontal(|ui| {
            ui.label("⏱ Time:");
            if craft_time >= 60.0 {
                let minutes = (craft_time / 60.0).floor();
                let seconds = craft_time % 60.0;
                ui.strong(format!("{minutes:.0}m {seconds:.0}s"));
            } else {
                ui.strong(format!("{craft_time:.1}s"));
            }

            if let Some(station) = station {
                ui.separator();
                ui.weak(format!("@ {station}"));
            }
        });
    }

    /// Show craft controls.
    fn show_craft_controls(
        &mut self,
        ui: &mut Ui,
        has_all_materials: bool,
        max_craft: u32,
        recipe_id: &RecipeId,
    ) {
        ui.horizontal(|ui| {
            ui.label("Quantity:");

            if ui.small_button("-").clicked() && self.craft_quantity > 1 {
                self.craft_quantity -= 1;
            }

            let mut qty = i32::try_from(self.craft_quantity).unwrap_or(i32::MAX);
            let max_qty = i32::try_from(max_craft.max(1)).unwrap_or(i32::MAX);
            ui.add(egui::DragValue::new(&mut qty).range(1..=max_qty).speed(0.1));
            self.craft_quantity = u32::try_from(qty.max(1)).unwrap_or(1).min(max_craft.max(1));

            if ui.small_button("+").clicked() && self.craft_quantity < max_craft.max(1) {
                self.craft_quantity += 1;
            }

            ui.weak(format!("(max: {max_craft})"));
        });

        ui.horizontal(|ui| {
            let can_craft = has_all_materials && max_craft > 0;

            if ui
                .add_enabled(can_craft, egui::Button::new("🔨 Craft"))
                .clicked()
            {
                self.pending_actions.push(CraftingPreviewAction::Craft);
            }

            if ui
                .add_enabled(can_craft && max_craft > 1, egui::Button::new("Craft Max"))
                .clicked()
            {
                self.craft_quantity = max_craft;
                self.pending_actions.push(CraftingPreviewAction::CraftMax);
            }

            if ui.button("📖").on_hover_text("View recipe").clicked() {
                self.pending_actions
                    .push(CraftingPreviewAction::ViewRecipe(recipe_id.clone()));
            }
        });

        if !has_all_materials {
            ui.colored_label(Color32::from_rgb(200, 150, 100), "⚠ Missing materials");
        }
    }

    /// Drain pending actions.
    pub fn drain_actions(&mut self) -> Vec<CraftingPreviewAction> {
        std::mem::take(&mut self.pending_actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_stats_new() {
        let stats = ItemStats::new();
        assert!(stats.damage.is_none());
        assert!(!stats.has_stats());
    }

    #[test]
    fn test_item_stats_builder() {
        let stats = ItemStats::new()
            .with_damage(50.0)
            .with_defense(10.0)
            .with_durability(100)
            .with_speed_modifier(0.1)
            .with_efficiency(150.0)
            .with_custom("Fire Damage", "25");

        assert_eq!(stats.damage, Some(50.0));
        assert_eq!(stats.defense, Some(10.0));
        assert_eq!(stats.durability, Some(100));
        assert_eq!(stats.speed_modifier, Some(0.1));
        assert_eq!(stats.efficiency, Some(150.0));
        assert_eq!(stats.custom.len(), 1);
        assert!(stats.has_stats());
    }

    #[test]
    fn test_craft_quality() {
        assert_eq!(CraftQuality::all().len(), 5);
        assert_eq!(CraftQuality::Normal.display_name(), "Normal");
        assert_eq!(CraftQuality::Normal.stat_multiplier(), 1.0);
        assert!(
            CraftQuality::Masterwork.stat_multiplier() > CraftQuality::Normal.stat_multiplier()
        );
    }

    #[test]
    fn test_crafting_probability_new() {
        let prob = CraftingProbability::new(0.8);
        assert_eq!(prob.base_success, 0.8);
        assert_eq!(prob.final_success_chance(), 0.8);
    }

    #[test]
    fn test_crafting_probability_clamp() {
        let prob = CraftingProbability::new(1.5);
        assert_eq!(prob.base_success, 1.0);

        let prob2 = CraftingProbability::new(-0.5);
        assert_eq!(prob2.base_success, 0.0);
    }

    #[test]
    fn test_crafting_probability_modifiers() {
        let prob = CraftingProbability::new(0.8)
            .with_skill_modifier(1.2)
            .with_station_modifier(1.1);

        let chance = prob.final_success_chance();
        // 0.8 * 1.2 * 1.1 = 1.056, clamped to 1.0
        assert_eq!(chance, 1.0);
    }

    #[test]
    fn test_material_requirement() {
        let mat = MaterialRequirement {
            name: "Iron".into(),
            required: 5,
            available: 3,
        };

        assert!(!mat.has_enough());
        assert_eq!(mat.missing(), 2);

        let mat2 = MaterialRequirement {
            name: "Wood".into(),
            required: 2,
            available: 10,
        };

        assert!(mat2.has_enough());
        assert_eq!(mat2.missing(), 0);
    }

    #[test]
    fn test_crafting_preview_data_has_all_materials() {
        let data = CraftingPreviewData {
            recipe_id: RecipeId::new("test"),
            name: "Test".into(),
            description: String::new(),
            quantity: 1,
            rarity: ItemRarity::Common,
            stats: ItemStats::default(),
            expected_quality: CraftQuality::Normal,
            probability: CraftingProbability::default(),
            materials: vec![
                MaterialRequirement {
                    name: "A".into(),
                    required: 2,
                    available: 5,
                },
                MaterialRequirement {
                    name: "B".into(),
                    required: 3,
                    available: 3,
                },
            ],
            craft_time: 1.0,
            station: None,
        };

        assert!(data.has_all_materials());
        assert_eq!(data.missing_material_count(), 0);
    }

    #[test]
    fn test_crafting_preview_data_missing_materials() {
        let data = CraftingPreviewData {
            recipe_id: RecipeId::new("test"),
            name: "Test".into(),
            description: String::new(),
            quantity: 1,
            rarity: ItemRarity::Common,
            stats: ItemStats::default(),
            expected_quality: CraftQuality::Normal,
            probability: CraftingProbability::default(),
            materials: vec![
                MaterialRequirement {
                    name: "A".into(),
                    required: 5,
                    available: 2,
                },
                MaterialRequirement {
                    name: "B".into(),
                    required: 3,
                    available: 10,
                },
            ],
            craft_time: 1.0,
            station: None,
        };

        assert!(!data.has_all_materials());
        assert_eq!(data.missing_material_count(), 1);
    }

    #[test]
    fn test_crafting_preview_new() {
        let preview = CraftingPreview::new();
        assert!(!preview.has_preview());
        assert!(preview.visible);
        assert_eq!(preview.craft_quantity, 1);
    }

    #[test]
    fn test_crafting_preview_set_preview() {
        let mut preview = CraftingPreview::new();
        let data = CraftingPreviewData {
            recipe_id: RecipeId::new("test"),
            name: "Test".into(),
            description: String::new(),
            quantity: 1,
            rarity: ItemRarity::Common,
            stats: ItemStats::default(),
            expected_quality: CraftQuality::Normal,
            probability: CraftingProbability::default(),
            materials: Vec::new(),
            craft_time: 1.0,
            station: None,
        };

        preview.set_preview(data);
        assert!(preview.has_preview());

        preview.clear_preview();
        assert!(!preview.has_preview());
    }

    #[test]
    fn test_crafting_preview_max_craftable() {
        let mut preview = CraftingPreview::new();

        // No preview
        assert_eq!(preview.max_craftable(), 0);

        // With materials
        let data = CraftingPreviewData {
            recipe_id: RecipeId::new("test"),
            name: "Test".into(),
            description: String::new(),
            quantity: 1,
            rarity: ItemRarity::Common,
            stats: ItemStats::default(),
            expected_quality: CraftQuality::Normal,
            probability: CraftingProbability::default(),
            materials: vec![
                MaterialRequirement {
                    name: "A".into(),
                    required: 2,
                    available: 10,
                },
                MaterialRequirement {
                    name: "B".into(),
                    required: 3,
                    available: 9,
                },
            ],
            craft_time: 1.0,
            station: None,
        };

        preview.set_preview(data);
        // min(10/2=5, 9/3=3) = 3
        assert_eq!(preview.max_craftable(), 3);
    }

    #[test]
    fn test_crafting_preview_config_defaults() {
        let config = CraftingPreviewConfig::default();
        assert!(config.show_stats);
        assert!(config.show_probability);
        assert!(config.show_materials);
        assert!(config.show_time);
        assert!(!config.compact);
    }

    #[test]
    fn test_crafting_preview_action_equality() {
        let action1 = CraftingPreviewAction::Craft;
        let action2 = CraftingPreviewAction::Craft;
        assert_eq!(action1, action2);

        let action3 = CraftingPreviewAction::ViewRecipe(RecipeId::new("test"));
        let action4 = CraftingPreviewAction::ViewRecipe(RecipeId::new("test"));
        assert_eq!(action3, action4);
    }

    #[test]
    fn test_item_stats_serialization() {
        let stats = ItemStats::new().with_damage(50.0).with_durability(100);

        let json = serde_json::to_string(&stats).unwrap();
        let loaded: ItemStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.damage, loaded.damage);
        assert_eq!(stats.durability, loaded.durability);
    }

    #[test]
    fn test_craft_quality_serialization() {
        for quality in CraftQuality::all() {
            let json = serde_json::to_string(quality).unwrap();
            let loaded: CraftQuality = serde_json::from_str(&json).unwrap();
            assert_eq!(*quality, loaded);
        }
    }

    #[test]
    fn test_crafting_probability_serialization() {
        let prob = CraftingProbability::new(0.85)
            .with_quality_chance(0.2)
            .with_skill_modifier(1.1);

        let json = serde_json::to_string(&prob).unwrap();
        let loaded: CraftingProbability = serde_json::from_str(&json).unwrap();

        assert_eq!(prob.base_success, loaded.base_success);
        assert_eq!(prob.quality_chance, loaded.quality_chance);
    }

    #[test]
    fn test_crafting_preview_config_serialization() {
        let config = CraftingPreviewConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: CraftingPreviewConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.width, loaded.width);
    }

    #[test]
    fn test_crafting_preview_drain_actions() {
        let mut preview = CraftingPreview::new();
        preview.pending_actions.push(CraftingPreviewAction::Craft);

        let actions = preview.drain_actions();
        assert_eq!(actions.len(), 1);

        let actions2 = preview.drain_actions();
        assert!(actions2.is_empty());
    }

    #[test]
    fn test_probability_color_high() {
        let prob = CraftingProbability::new(0.95);
        // High success should be green
        let color = prob.color();
        assert_eq!(color, Color32::from_rgb(100, 200, 100));
    }

    #[test]
    fn test_probability_color_low() {
        let prob = CraftingProbability::new(0.3);
        // Low success should be red
        let color = prob.color();
        assert_eq!(color, Color32::from_rgb(200, 100, 100));
    }
}
