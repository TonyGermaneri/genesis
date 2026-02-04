//! Equipment stats UI components.
//!
//! Provides equipment stats functionality including:
//! - Weapon damage display
//! - Armor values display
//! - Stat comparison between items
//! - DPS calculation

use egui::{Color32, Ui};
use serde::{Deserialize, Serialize};

/// Equipment slot types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    /// Main hand weapon.
    MainHand,
    /// Off hand (shield/weapon).
    OffHand,
    /// Head armor.
    Head,
    /// Chest armor.
    Chest,
    /// Legs armor.
    Legs,
    /// Feet armor.
    Feet,
    /// Hands armor.
    Hands,
    /// Ring slot 1.
    Ring1,
    /// Ring slot 2.
    Ring2,
    /// Necklace.
    Necklace,
    /// Back (cape/cloak).
    Back,
}

impl EquipmentSlot {
    /// Get all equipment slots.
    pub fn all() -> &'static [EquipmentSlot] {
        &[
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Head,
            EquipmentSlot::Chest,
            EquipmentSlot::Legs,
            EquipmentSlot::Feet,
            EquipmentSlot::Hands,
            EquipmentSlot::Ring1,
            EquipmentSlot::Ring2,
            EquipmentSlot::Necklace,
            EquipmentSlot::Back,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            EquipmentSlot::MainHand => "Main Hand",
            EquipmentSlot::OffHand => "Off Hand",
            EquipmentSlot::Head => "Head",
            EquipmentSlot::Chest => "Chest",
            EquipmentSlot::Legs => "Legs",
            EquipmentSlot::Feet => "Feet",
            EquipmentSlot::Hands => "Hands",
            EquipmentSlot::Ring1 => "Ring 1",
            EquipmentSlot::Ring2 => "Ring 2",
            EquipmentSlot::Necklace => "Necklace",
            EquipmentSlot::Back => "Back",
        }
    }

    /// Get icon.
    pub fn icon(&self) -> &'static str {
        match self {
            EquipmentSlot::MainHand => "⚔",
            EquipmentSlot::OffHand => "🛡",
            EquipmentSlot::Head => "🎩",
            EquipmentSlot::Chest => "👕",
            EquipmentSlot::Legs => "👖",
            EquipmentSlot::Feet => "👟",
            EquipmentSlot::Hands => "🧤",
            EquipmentSlot::Ring1 | EquipmentSlot::Ring2 => "💍",
            EquipmentSlot::Necklace => "📿",
            EquipmentSlot::Back => "🧥",
        }
    }

    /// Check if this is an armor slot.
    pub fn is_armor(&self) -> bool {
        matches!(
            self,
            EquipmentSlot::Head
                | EquipmentSlot::Chest
                | EquipmentSlot::Legs
                | EquipmentSlot::Feet
                | EquipmentSlot::Hands
        )
    }

    /// Check if this is an accessory slot.
    pub fn is_accessory(&self) -> bool {
        matches!(
            self,
            EquipmentSlot::Ring1
                | EquipmentSlot::Ring2
                | EquipmentSlot::Necklace
                | EquipmentSlot::Back
        )
    }
}

/// Damage types for weapons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WeaponDamageType {
    /// Physical damage.
    #[default]
    Physical,
    /// Fire elemental damage.
    Fire,
    /// Ice elemental damage.
    Ice,
    /// Lightning elemental damage.
    Lightning,
    /// Poison damage.
    Poison,
    /// Arcane/magic damage.
    Arcane,
    /// Holy damage.
    Holy,
    /// Shadow damage.
    Shadow,
}

impl WeaponDamageType {
    /// Get all damage types.
    pub fn all() -> &'static [WeaponDamageType] {
        &[
            WeaponDamageType::Physical,
            WeaponDamageType::Fire,
            WeaponDamageType::Ice,
            WeaponDamageType::Lightning,
            WeaponDamageType::Poison,
            WeaponDamageType::Arcane,
            WeaponDamageType::Holy,
            WeaponDamageType::Shadow,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            WeaponDamageType::Physical => "Physical",
            WeaponDamageType::Fire => "Fire",
            WeaponDamageType::Ice => "Ice",
            WeaponDamageType::Lightning => "Lightning",
            WeaponDamageType::Poison => "Poison",
            WeaponDamageType::Arcane => "Arcane",
            WeaponDamageType::Holy => "Holy",
            WeaponDamageType::Shadow => "Shadow",
        }
    }

    /// Get color.
    pub fn color(&self) -> Color32 {
        match self {
            WeaponDamageType::Physical => Color32::from_gray(200),
            WeaponDamageType::Fire => Color32::from_rgb(255, 150, 50),
            WeaponDamageType::Ice => Color32::from_rgb(150, 200, 255),
            WeaponDamageType::Lightning => Color32::from_rgb(255, 255, 100),
            WeaponDamageType::Poison => Color32::from_rgb(100, 200, 100),
            WeaponDamageType::Arcane => Color32::from_rgb(200, 100, 255),
            WeaponDamageType::Holy => Color32::from_rgb(255, 255, 200),
            WeaponDamageType::Shadow => Color32::from_rgb(100, 50, 150),
        }
    }
}

/// Weapon stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponStats {
    /// Minimum damage.
    pub min_damage: f32,
    /// Maximum damage.
    pub max_damage: f32,
    /// Attack speed (attacks per second).
    pub attack_speed: f32,
    /// Critical hit chance (0.0 - 1.0).
    pub crit_chance: f32,
    /// Critical hit multiplier.
    pub crit_multiplier: f32,
    /// Damage type.
    pub damage_type: WeaponDamageType,
    /// Armor penetration.
    pub armor_pen: f32,
    /// Range (for ranged weapons).
    pub range: Option<f32>,
}

impl Default for WeaponStats {
    fn default() -> Self {
        Self {
            min_damage: 10.0,
            max_damage: 20.0,
            attack_speed: 1.0,
            crit_chance: 0.05,
            crit_multiplier: 1.5,
            damage_type: WeaponDamageType::Physical,
            armor_pen: 0.0,
            range: None,
        }
    }
}

impl WeaponStats {
    /// Create new weapon stats.
    pub fn new(min_damage: f32, max_damage: f32, attack_speed: f32) -> Self {
        Self {
            min_damage,
            max_damage,
            attack_speed,
            ..Default::default()
        }
    }

    /// Calculate average damage per hit.
    pub fn average_damage(&self) -> f32 {
        (self.min_damage + self.max_damage) / 2.0
    }

    /// Calculate DPS (damage per second).
    pub fn dps(&self) -> f32 {
        let avg = self.average_damage();
        let crit_bonus = self.crit_chance * (self.crit_multiplier - 1.0);
        avg * (1.0 + crit_bonus) * self.attack_speed
    }

    /// Calculate DPS with crit only.
    pub fn dps_with_crit(&self) -> f32 {
        let avg = self.average_damage();
        avg * self.crit_multiplier * self.attack_speed
    }

    /// Get damage range as string.
    pub fn damage_range_text(&self) -> String {
        format!("{:.0} - {:.0}", self.min_damage, self.max_damage)
    }
}

/// Armor stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorStats {
    /// Physical armor value.
    pub armor: f32,
    /// Magic resistance.
    pub magic_resist: f32,
    /// Fire resistance.
    pub fire_resist: f32,
    /// Ice resistance.
    pub ice_resist: f32,
    /// Lightning resistance.
    pub lightning_resist: f32,
    /// Poison resistance.
    pub poison_resist: f32,
}

impl Default for ArmorStats {
    fn default() -> Self {
        Self {
            armor: 0.0,
            magic_resist: 0.0,
            fire_resist: 0.0,
            ice_resist: 0.0,
            lightning_resist: 0.0,
            poison_resist: 0.0,
        }
    }
}

impl ArmorStats {
    /// Create new armor stats.
    pub fn new(armor: f32, magic_resist: f32) -> Self {
        Self {
            armor,
            magic_resist,
            ..Default::default()
        }
    }

    /// Calculate damage reduction percentage for physical.
    pub fn physical_reduction(&self) -> f32 {
        // Diminishing returns formula
        self.armor / (self.armor + 100.0)
    }

    /// Calculate damage reduction percentage for magic.
    pub fn magic_reduction(&self) -> f32 {
        self.magic_resist / (self.magic_resist + 100.0)
    }

    /// Get total resistance to a damage type.
    pub fn resistance_for(&self, damage_type: WeaponDamageType) -> f32 {
        match damage_type {
            WeaponDamageType::Physical => self.physical_reduction(),
            WeaponDamageType::Fire => self.fire_resist / (self.fire_resist + 100.0),
            WeaponDamageType::Ice => self.ice_resist / (self.ice_resist + 100.0),
            WeaponDamageType::Lightning => self.lightning_resist / (self.lightning_resist + 100.0),
            WeaponDamageType::Poison => self.poison_resist / (self.poison_resist + 100.0),
            WeaponDamageType::Arcane | WeaponDamageType::Holy | WeaponDamageType::Shadow => {
                self.magic_reduction()
            },
        }
    }

    /// Combine with another armor stats (for total).
    pub fn combine(&self, other: &ArmorStats) -> ArmorStats {
        ArmorStats {
            armor: self.armor + other.armor,
            magic_resist: self.magic_resist + other.magic_resist,
            fire_resist: self.fire_resist + other.fire_resist,
            ice_resist: self.ice_resist + other.ice_resist,
            lightning_resist: self.lightning_resist + other.lightning_resist,
            poison_resist: self.poison_resist + other.poison_resist,
        }
    }
}

/// Character stat types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatType {
    /// Strength - affects physical damage.
    Strength,
    /// Dexterity - affects attack speed, crit.
    Dexterity,
    /// Intelligence - affects magic damage.
    Intelligence,
    /// Vitality - affects health.
    Vitality,
    /// Endurance - affects stamina.
    Endurance,
    /// Luck - affects crit, drops.
    Luck,
}

impl StatType {
    /// Get all stat types.
    pub fn all() -> &'static [StatType] {
        &[
            StatType::Strength,
            StatType::Dexterity,
            StatType::Intelligence,
            StatType::Vitality,
            StatType::Endurance,
            StatType::Luck,
        ]
    }

    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            StatType::Strength => "Strength",
            StatType::Dexterity => "Dexterity",
            StatType::Intelligence => "Intelligence",
            StatType::Vitality => "Vitality",
            StatType::Endurance => "Endurance",
            StatType::Luck => "Luck",
        }
    }

    /// Get short name.
    pub fn short_name(&self) -> &'static str {
        match self {
            StatType::Strength => "STR",
            StatType::Dexterity => "DEX",
            StatType::Intelligence => "INT",
            StatType::Vitality => "VIT",
            StatType::Endurance => "END",
            StatType::Luck => "LCK",
        }
    }

    /// Get color.
    pub fn color(&self) -> Color32 {
        match self {
            StatType::Strength => Color32::from_rgb(255, 100, 100),
            StatType::Dexterity => Color32::from_rgb(100, 255, 100),
            StatType::Intelligence => Color32::from_rgb(100, 150, 255),
            StatType::Vitality => Color32::from_rgb(255, 150, 100),
            StatType::Endurance => Color32::from_rgb(200, 200, 100),
            StatType::Luck => Color32::from_rgb(200, 150, 255),
        }
    }
}

/// A stat bonus from equipment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBonus {
    /// Stat type.
    pub stat: StatType,
    /// Bonus value.
    pub value: i32,
}

impl StatBonus {
    /// Create new stat bonus.
    pub fn new(stat: StatType, value: i32) -> Self {
        Self { stat, value }
    }

    /// Get formatted display string.
    pub fn display_text(&self) -> String {
        let sign = if self.value >= 0 { "+" } else { "" };
        format!("{sign}{} {}", self.value, self.stat.short_name())
    }
}

/// Equipment item rarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EquipmentRarity {
    /// Common (white).
    #[default]
    Common,
    /// Uncommon (green).
    Uncommon,
    /// Rare (blue).
    Rare,
    /// Epic (purple).
    Epic,
    /// Legendary (orange).
    Legendary,
    /// Mythic (red).
    Mythic,
}

impl EquipmentRarity {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            EquipmentRarity::Common => "Common",
            EquipmentRarity::Uncommon => "Uncommon",
            EquipmentRarity::Rare => "Rare",
            EquipmentRarity::Epic => "Epic",
            EquipmentRarity::Legendary => "Legendary",
            EquipmentRarity::Mythic => "Mythic",
        }
    }

    /// Get color.
    pub fn color(&self) -> Color32 {
        match self {
            EquipmentRarity::Common => Color32::from_gray(200),
            EquipmentRarity::Uncommon => Color32::from_rgb(100, 200, 100),
            EquipmentRarity::Rare => Color32::from_rgb(100, 150, 255),
            EquipmentRarity::Epic => Color32::from_rgb(200, 100, 255),
            EquipmentRarity::Legendary => Color32::from_rgb(255, 180, 50),
            EquipmentRarity::Mythic => Color32::from_rgb(255, 100, 100),
        }
    }
}

/// Equipment item data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentItem {
    /// Item ID.
    pub id: String,
    /// Item name.
    pub name: String,
    /// Item rarity.
    pub rarity: EquipmentRarity,
    /// Equipment slot.
    pub slot: EquipmentSlot,
    /// Item level.
    pub item_level: u32,
    /// Required player level.
    pub required_level: u32,
    /// Weapon stats (if weapon).
    pub weapon: Option<WeaponStats>,
    /// Armor stats.
    pub armor: ArmorStats,
    /// Stat bonuses.
    pub stat_bonuses: Vec<StatBonus>,
    /// Description.
    pub description: Option<String>,
}

impl EquipmentItem {
    /// Create a weapon.
    pub fn weapon(
        id: impl Into<String>,
        name: impl Into<String>,
        weapon_stats: WeaponStats,
        rarity: EquipmentRarity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            rarity,
            slot: EquipmentSlot::MainHand,
            item_level: 1,
            required_level: 1,
            weapon: Some(weapon_stats),
            armor: ArmorStats::default(),
            stat_bonuses: Vec::new(),
            description: None,
        }
    }

    /// Create an armor piece.
    pub fn armor(
        id: impl Into<String>,
        name: impl Into<String>,
        slot: EquipmentSlot,
        armor_stats: ArmorStats,
        rarity: EquipmentRarity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            rarity,
            slot,
            item_level: 1,
            required_level: 1,
            weapon: None,
            armor: armor_stats,
            stat_bonuses: Vec::new(),
            description: None,
        }
    }

    /// Add a stat bonus.
    pub fn with_bonus(mut self, stat: StatType, value: i32) -> Self {
        self.stat_bonuses.push(StatBonus::new(stat, value));
        self
    }

    /// Set item level.
    pub fn with_item_level(mut self, level: u32) -> Self {
        self.item_level = level;
        self
    }

    /// Set required level.
    pub fn with_required_level(mut self, level: u32) -> Self {
        self.required_level = level;
        self
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Check if this is a weapon.
    pub fn is_weapon(&self) -> bool {
        self.weapon.is_some()
    }

    /// Get DPS if weapon.
    pub fn dps(&self) -> Option<f32> {
        self.weapon.as_ref().map(WeaponStats::dps)
    }

    /// Get total stat bonus for a type.
    pub fn total_stat(&self, stat: StatType) -> i32 {
        self.stat_bonuses
            .iter()
            .filter(|b| b.stat == stat)
            .map(|b| b.value)
            .sum()
    }
}

/// Stat comparison result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareResult {
    /// New item is better.
    Better,
    /// New item is worse.
    Worse,
    /// Items are equal.
    Equal,
    /// Cannot compare (different types).
    Incomparable,
}

impl CompareResult {
    /// Get color for display.
    pub fn color(&self) -> Color32 {
        match self {
            CompareResult::Better => Color32::from_rgb(100, 200, 100),
            CompareResult::Worse => Color32::from_rgb(200, 100, 100),
            CompareResult::Equal => Color32::from_gray(150),
            CompareResult::Incomparable => Color32::from_gray(100),
        }
    }

    /// Get arrow symbol.
    pub fn arrow(&self) -> &'static str {
        match self {
            CompareResult::Better => "▲",
            CompareResult::Worse => "▼",
            CompareResult::Equal => "=",
            CompareResult::Incomparable => "?",
        }
    }
}

/// Comparison between two values.
#[derive(Debug, Clone)]
pub struct StatComparison {
    /// Stat name.
    pub name: String,
    /// Current value.
    pub current: f32,
    /// New value.
    pub new_value: f32,
    /// Comparison result.
    pub result: CompareResult,
}

impl StatComparison {
    /// Create a new comparison.
    pub fn new(name: impl Into<String>, current: f32, new_value: f32) -> Self {
        let result = if (new_value - current).abs() < 0.01 {
            CompareResult::Equal
        } else if new_value > current {
            CompareResult::Better
        } else {
            CompareResult::Worse
        };

        Self {
            name: name.into(),
            current,
            new_value,
            result,
        }
    }

    /// Create with custom result direction (for armor where lower might be worse).
    pub fn new_higher_is_better(name: impl Into<String>, current: f32, new_value: f32) -> Self {
        Self::new(name, current, new_value)
    }

    /// Get difference.
    pub fn difference(&self) -> f32 {
        self.new_value - self.current
    }

    /// Get formatted difference string.
    pub fn difference_text(&self) -> String {
        let diff = self.difference();
        if diff.abs() < 0.01 {
            "0".to_string()
        } else if diff > 0.0 {
            format!("+{diff:.1}")
        } else {
            format!("{diff:.1}")
        }
    }

    /// Get arrow symbol for the comparison result.
    pub fn arrow(&self) -> &'static str {
        self.result.arrow()
    }
}

/// Equipment comparison between current and new item.
#[derive(Debug, Clone)]
pub struct EquipmentComparison {
    /// Current item (may be None).
    pub current: Option<EquipmentItem>,
    /// New item.
    pub new_item: EquipmentItem,
    /// DPS comparison.
    pub dps: Option<StatComparison>,
    /// Armor comparison.
    pub armor: Option<StatComparison>,
    /// Stat comparisons.
    pub stats: Vec<StatComparison>,
}

impl EquipmentComparison {
    /// Create a new comparison.
    pub fn new(current: &Option<EquipmentItem>, new_item: &EquipmentItem) -> Self {
        let mut comparison = Self {
            current: current.clone(),
            new_item: new_item.clone(),
            dps: None,
            armor: None,
            stats: Vec::new(),
        };

        // Compare DPS if both are weapons
        if let Some(new_weapon) = &new_item.weapon {
            let current_dps = current
                .as_ref()
                .and_then(|c| c.weapon.as_ref())
                .map_or(0.0, WeaponStats::dps);
            comparison.dps = Some(StatComparison::new("DPS", current_dps, new_weapon.dps()));
        }

        // Compare armor
        let current_armor = current.as_ref().map_or(0.0, |c| c.armor.armor);
        if new_item.armor.armor > 0.0 || current_armor > 0.0 {
            comparison.armor = Some(StatComparison::new(
                "Armor",
                current_armor,
                new_item.armor.armor,
            ));
        }

        // Compare stats
        for stat_type in StatType::all() {
            let current_stat = current.as_ref().map_or(0, |c| c.total_stat(*stat_type));
            let new_stat = new_item.total_stat(*stat_type);
            if current_stat != 0 || new_stat != 0 {
                comparison.stats.push(StatComparison::new(
                    stat_type.display_name(),
                    current_stat as f32,
                    new_stat as f32,
                ));
            }
        }

        comparison
    }

    /// Check if new item is overall better.
    pub fn is_upgrade(&self) -> bool {
        let dps_better = self
            .dps
            .as_ref()
            .is_some_and(|d| d.result == CompareResult::Better);
        let armor_better = self
            .armor
            .as_ref()
            .is_some_and(|a| a.result == CompareResult::Better);
        let stats_better = self
            .stats
            .iter()
            .filter(|s| s.result == CompareResult::Better)
            .count();
        let stats_worse = self
            .stats
            .iter()
            .filter(|s| s.result == CompareResult::Worse)
            .count();

        dps_better || armor_better || stats_better > stats_worse
    }
}

/// Equipment stats configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentStatsConfig {
    /// Show DPS calculation.
    pub show_dps: bool,
    /// Show stat comparison.
    pub show_comparison: bool,
    /// Show all resistances.
    pub show_all_resistances: bool,
    /// Highlight upgrades.
    pub highlight_upgrades: bool,
}

impl Default for EquipmentStatsConfig {
    fn default() -> Self {
        Self {
            show_dps: true,
            show_comparison: true,
            show_all_resistances: false,
            highlight_upgrades: true,
        }
    }
}

/// Equipment stats panel widget.
#[derive(Debug)]
pub struct EquipmentStatsPanel {
    /// Configuration.
    pub config: EquipmentStatsConfig,
    /// Currently selected item.
    pub selected_item: Option<EquipmentItem>,
    /// Comparison item (hovering over).
    pub comparison_item: Option<EquipmentItem>,
    /// Player level (for requirement checks).
    pub player_level: u32,
    /// Whether panel is visible.
    pub visible: bool,
}

impl Default for EquipmentStatsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EquipmentStatsPanel {
    /// Create new equipment stats panel.
    pub fn new() -> Self {
        Self {
            config: EquipmentStatsConfig::default(),
            selected_item: None,
            comparison_item: None,
            player_level: 1,
            visible: true,
        }
    }

    /// Create with config.
    pub fn with_config(config: EquipmentStatsConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Set selected item.
    pub fn select_item(&mut self, item: EquipmentItem) {
        self.selected_item = Some(item);
    }

    /// Clear selection.
    pub fn clear_selection(&mut self) {
        self.selected_item = None;
    }

    /// Set comparison item (for hover).
    pub fn set_comparison(&mut self, item: EquipmentItem) {
        self.comparison_item = Some(item);
    }

    /// Clear comparison.
    pub fn clear_comparison(&mut self) {
        self.comparison_item = None;
    }

    /// Get current comparison.
    pub fn get_comparison(&self) -> Option<EquipmentComparison> {
        self.comparison_item
            .as_ref()
            .map(|new_item| EquipmentComparison::new(&self.selected_item, new_item))
    }

    /// Show the stats panel.
    pub fn show(&mut self, ui: &mut Ui) {
        if !self.visible {
            return;
        }

        ui.vertical(|ui| {
            if let Some(item) = &self.selected_item {
                self.show_item_stats(ui, item);
            } else {
                ui.label("No item selected");
            }

            if self.config.show_comparison {
                if let Some(comparison) = self.get_comparison() {
                    ui.separator();
                    self.show_comparison(ui, &comparison);
                }
            }
        });
    }

    /// Show item stats.
    fn show_item_stats(&self, ui: &mut Ui, item: &EquipmentItem) {
        // Item name with rarity color
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&item.name)
                    .color(item.rarity.color())
                    .strong(),
            );
        });

        // Item level and type
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "iLvl {} {}",
                    item.item_level,
                    item.slot.display_name()
                ))
                .small()
                .color(Color32::from_gray(150)),
            );
        });

        ui.separator();

        // Weapon stats
        if let Some(weapon) = &item.weapon {
            ui.label(format!("Damage: {}", weapon.damage_range_text()));
            ui.label(format!("Attack Speed: {:.2}/s", weapon.attack_speed));

            if self.config.show_dps {
                ui.label(
                    egui::RichText::new(format!("DPS: {:.1}", weapon.dps()))
                        .color(Color32::from_rgb(255, 200, 100)),
                );
            }

            if weapon.crit_chance > 0.0 {
                ui.label(format!(
                    "Critical: {:.1}% (x{:.1})",
                    weapon.crit_chance * 100.0,
                    weapon.crit_multiplier
                ));
            }

            ui.label(
                egui::RichText::new(weapon.damage_type.display_name())
                    .color(weapon.damage_type.color()),
            );

            ui.separator();
        }

        // Armor stats
        if item.armor.armor > 0.0 {
            ui.label(format!(
                "Armor: {:.0} ({:.1}% reduction)",
                item.armor.armor,
                item.armor.physical_reduction() * 100.0
            ));
        }

        if item.armor.magic_resist > 0.0 {
            ui.label(format!(
                "Magic Resist: {:.0} ({:.1}%)",
                item.armor.magic_resist,
                item.armor.magic_reduction() * 100.0
            ));
        }

        if self.config.show_all_resistances {
            if item.armor.fire_resist > 0.0 {
                ui.label(
                    egui::RichText::new(format!("Fire Resist: {:.0}", item.armor.fire_resist))
                        .color(WeaponDamageType::Fire.color()),
                );
            }
            if item.armor.ice_resist > 0.0 {
                ui.label(
                    egui::RichText::new(format!("Ice Resist: {:.0}", item.armor.ice_resist))
                        .color(WeaponDamageType::Ice.color()),
                );
            }
            if item.armor.lightning_resist > 0.0 {
                ui.label(
                    egui::RichText::new(format!(
                        "Lightning Resist: {:.0}",
                        item.armor.lightning_resist
                    ))
                    .color(WeaponDamageType::Lightning.color()),
                );
            }
            if item.armor.poison_resist > 0.0 {
                ui.label(
                    egui::RichText::new(format!("Poison Resist: {:.0}", item.armor.poison_resist))
                        .color(WeaponDamageType::Poison.color()),
                );
            }
        }

        // Stat bonuses
        if !item.stat_bonuses.is_empty() {
            ui.separator();
            for bonus in &item.stat_bonuses {
                let color = if bonus.value > 0 {
                    Color32::from_rgb(100, 200, 100)
                } else {
                    Color32::from_rgb(200, 100, 100)
                };
                ui.label(egui::RichText::new(bonus.display_text()).color(color));
            }
        }

        // Required level
        if item.required_level > 1 {
            let level_color = if self.player_level >= item.required_level {
                Color32::from_gray(150)
            } else {
                Color32::from_rgb(200, 100, 100)
            };
            ui.label(
                egui::RichText::new(format!("Requires Level {}", item.required_level))
                    .color(level_color),
            );
        }

        // Description
        if let Some(desc) = &item.description {
            ui.separator();
            ui.label(
                egui::RichText::new(desc)
                    .italics()
                    .color(Color32::from_rgb(255, 200, 100)),
            );
        }
    }

    /// Show comparison between items.
    fn show_comparison(&self, ui: &mut Ui, comparison: &EquipmentComparison) {
        ui.label(
            egui::RichText::new("Comparison")
                .strong()
                .color(Color32::from_gray(200)),
        );

        // DPS comparison
        if let Some(dps) = &comparison.dps {
            ui.horizontal(|ui| {
                ui.label("DPS:");
                ui.label(egui::RichText::new(dps.arrow()).color(dps.result.color()));
                ui.label(egui::RichText::new(dps.difference_text()).color(dps.result.color()));
            });
        }

        // Armor comparison
        if let Some(armor) = &comparison.armor {
            ui.horizontal(|ui| {
                ui.label("Armor:");
                ui.label(egui::RichText::new(armor.arrow()).color(armor.result.color()));
                ui.label(egui::RichText::new(armor.difference_text()).color(armor.result.color()));
            });
        }

        // Stat comparisons
        for stat in &comparison.stats {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", stat.name));
                ui.label(egui::RichText::new(stat.arrow()).color(stat.result.color()));
                ui.label(egui::RichText::new(stat.difference_text()).color(stat.result.color()));
            });
        }

        // Overall assessment
        if self.config.highlight_upgrades {
            let upgrade_text = if comparison.is_upgrade() {
                egui::RichText::new("↑ UPGRADE")
                    .strong()
                    .color(Color32::from_rgb(100, 200, 100))
            } else {
                egui::RichText::new("↓ DOWNGRADE")
                    .strong()
                    .color(Color32::from_rgb(200, 100, 100))
            };
            ui.label(upgrade_text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equipment_slot_all() {
        assert_eq!(EquipmentSlot::all().len(), 11);
    }

    #[test]
    fn test_equipment_slot_display_name() {
        assert_eq!(EquipmentSlot::MainHand.display_name(), "Main Hand");
        assert_eq!(EquipmentSlot::Chest.display_name(), "Chest");
    }

    #[test]
    fn test_equipment_slot_is_armor() {
        assert!(EquipmentSlot::Chest.is_armor());
        assert!(EquipmentSlot::Head.is_armor());
        assert!(!EquipmentSlot::MainHand.is_armor());
        assert!(!EquipmentSlot::Ring1.is_armor());
    }

    #[test]
    fn test_equipment_slot_is_accessory() {
        assert!(EquipmentSlot::Ring1.is_accessory());
        assert!(EquipmentSlot::Necklace.is_accessory());
        assert!(!EquipmentSlot::Chest.is_accessory());
    }

    #[test]
    fn test_weapon_damage_type_all() {
        assert_eq!(WeaponDamageType::all().len(), 8);
    }

    #[test]
    fn test_weapon_stats_default() {
        let stats = WeaponStats::default();
        assert_eq!(stats.min_damage, 10.0);
        assert_eq!(stats.max_damage, 20.0);
    }

    #[test]
    fn test_weapon_stats_average_damage() {
        let stats = WeaponStats::new(10.0, 30.0, 1.0);
        assert_eq!(stats.average_damage(), 20.0);
    }

    #[test]
    fn test_weapon_stats_dps() {
        let stats = WeaponStats {
            min_damage: 10.0,
            max_damage: 10.0,
            attack_speed: 2.0,
            crit_chance: 0.0,
            crit_multiplier: 1.5,
            ..Default::default()
        };
        assert_eq!(stats.dps(), 20.0);
    }

    #[test]
    fn test_weapon_stats_dps_with_crit() {
        let stats = WeaponStats {
            min_damage: 10.0,
            max_damage: 10.0,
            attack_speed: 1.0,
            crit_chance: 0.1,
            crit_multiplier: 2.0,
            ..Default::default()
        };
        // DPS = 10 * (1 + 0.1 * 1.0) * 1.0 = 11
        assert!((stats.dps() - 11.0).abs() < 0.01);
    }

    #[test]
    fn test_armor_stats_physical_reduction() {
        let stats = ArmorStats::new(100.0, 0.0);
        assert_eq!(stats.physical_reduction(), 0.5);
    }

    #[test]
    fn test_armor_stats_combine() {
        let a = ArmorStats::new(50.0, 25.0);
        let b = ArmorStats::new(30.0, 15.0);
        let combined = a.combine(&b);
        assert_eq!(combined.armor, 80.0);
        assert_eq!(combined.magic_resist, 40.0);
    }

    #[test]
    fn test_stat_type_all() {
        assert_eq!(StatType::all().len(), 6);
    }

    #[test]
    fn test_stat_bonus_display_text() {
        let bonus = StatBonus::new(StatType::Strength, 10);
        assert_eq!(bonus.display_text(), "+10 STR");

        let penalty = StatBonus::new(StatType::Dexterity, -5);
        assert_eq!(penalty.display_text(), "-5 DEX");
    }

    #[test]
    fn test_item_rarity_color() {
        assert_ne!(EquipmentRarity::Common.color(), EquipmentRarity::Legendary.color());
    }

    #[test]
    fn test_equipment_item_weapon() {
        let weapon = EquipmentItem::weapon(
            "sword_1",
            "Iron Sword",
            WeaponStats::default(),
            EquipmentRarity::Common,
        );
        assert!(weapon.is_weapon());
        assert!(weapon.dps().is_some());
    }

    #[test]
    fn test_equipment_item_armor() {
        let armor = EquipmentItem::armor(
            "chest_1",
            "Leather Chest",
            EquipmentSlot::Chest,
            ArmorStats::new(50.0, 25.0),
            EquipmentRarity::Uncommon,
        );
        assert!(!armor.is_weapon());
        assert!(armor.dps().is_none());
    }

    #[test]
    fn test_equipment_item_with_bonus() {
        let item =
            EquipmentItem::weapon("sword", "Sword", WeaponStats::default(), EquipmentRarity::Rare)
                .with_bonus(StatType::Strength, 10)
                .with_bonus(StatType::Strength, 5);
        assert_eq!(item.total_stat(StatType::Strength), 15);
    }

    #[test]
    fn test_stat_comparison() {
        let better = StatComparison::new("DPS", 10.0, 20.0);
        assert_eq!(better.result, CompareResult::Better);
        assert_eq!(better.difference(), 10.0);

        let worse = StatComparison::new("Armor", 50.0, 30.0);
        assert_eq!(worse.result, CompareResult::Worse);

        let equal = StatComparison::new("Speed", 1.0, 1.0);
        assert_eq!(equal.result, CompareResult::Equal);
    }

    #[test]
    fn test_equipment_comparison() {
        let current = EquipmentItem::weapon(
            "sword_1",
            "Iron Sword",
            WeaponStats::new(10.0, 20.0, 1.0),
            EquipmentRarity::Common,
        );

        let new_item = EquipmentItem::weapon(
            "sword_2",
            "Steel Sword",
            WeaponStats::new(15.0, 25.0, 1.0),
            EquipmentRarity::Uncommon,
        );

        let comparison = EquipmentComparison::new(&Some(current), &new_item);
        assert!(comparison.dps.is_some());
        assert!(comparison.is_upgrade());
    }

    #[test]
    fn test_equipment_comparison_no_current() {
        let new_item = EquipmentItem::armor(
            "chest_1",
            "Leather Chest",
            EquipmentSlot::Chest,
            ArmorStats::new(50.0, 25.0),
            EquipmentRarity::Common,
        );

        let comparison = EquipmentComparison::new(&None, &new_item);
        assert!(comparison.is_upgrade());
    }

    #[test]
    fn test_equipment_stats_panel() {
        let panel = EquipmentStatsPanel::new();
        assert!(panel.visible);
        assert!(panel.selected_item.is_none());
    }

    #[test]
    fn test_equipment_stats_panel_select() {
        let mut panel = EquipmentStatsPanel::new();
        let item =
            EquipmentItem::weapon("sword", "Sword", WeaponStats::default(), EquipmentRarity::Common);
        panel.select_item(item);
        assert!(panel.selected_item.is_some());
        panel.clear_selection();
        assert!(panel.selected_item.is_none());
    }

    #[test]
    fn test_equipment_stats_panel_comparison() {
        let mut panel = EquipmentStatsPanel::new();

        let current = EquipmentItem::weapon(
            "sword_1",
            "Sword 1",
            WeaponStats::default(),
            EquipmentRarity::Common,
        );
        panel.select_item(current);

        let new_item = EquipmentItem::weapon(
            "sword_2",
            "Sword 2",
            WeaponStats::new(20.0, 30.0, 1.0),
            EquipmentRarity::Rare,
        );
        panel.set_comparison(new_item);

        let comparison = panel.get_comparison();
        assert!(comparison.is_some());
    }

    #[test]
    fn test_weapon_stats_serialization() {
        let stats = WeaponStats::new(10.0, 20.0, 1.5);
        let json = serde_json::to_string(&stats).unwrap();
        let loaded: WeaponStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats.min_damage, loaded.min_damage);
        assert_eq!(stats.max_damage, loaded.max_damage);
    }

    #[test]
    fn test_armor_stats_serialization() {
        let stats = ArmorStats::new(100.0, 50.0);
        let json = serde_json::to_string(&stats).unwrap();
        let loaded: ArmorStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats.armor, loaded.armor);
    }

    #[test]
    fn test_equipment_item_serialization() {
        let item =
            EquipmentItem::weapon("sword", "Sword", WeaponStats::default(), EquipmentRarity::Epic)
                .with_bonus(StatType::Strength, 10)
                .with_description("A fine sword");

        let json = serde_json::to_string(&item).unwrap();
        let loaded: EquipmentItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item.name, loaded.name);
        assert_eq!(item.rarity, loaded.rarity);
        assert_eq!(item.stat_bonuses.len(), loaded.stat_bonuses.len());
    }

    #[test]
    fn test_equipment_stats_config_serialization() {
        let config = EquipmentStatsConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: EquipmentStatsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.show_dps, loaded.show_dps);
    }

    #[test]
    fn test_compare_result_arrow() {
        assert_eq!(CompareResult::Better.arrow(), "▲");
        assert_eq!(CompareResult::Worse.arrow(), "▼");
        assert_eq!(CompareResult::Equal.arrow(), "=");
    }

    #[test]
    fn test_armor_stats_resistance_for() {
        let stats = ArmorStats {
            armor: 100.0,
            magic_resist: 100.0,
            fire_resist: 50.0,
            ..Default::default()
        };

        assert_eq!(stats.resistance_for(WeaponDamageType::Physical), 0.5);
        // fire_resist / (fire_resist + 100) = 50 / 150 = 0.333...
        assert!((stats.resistance_for(WeaponDamageType::Fire) - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_weapon_damage_range_text() {
        let stats = WeaponStats::new(15.0, 25.0, 1.0);
        assert_eq!(stats.damage_range_text(), "15 - 25");
    }
}
