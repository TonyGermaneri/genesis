//! Factions UI Panel
//!
//! Provides a full faction management interface:
//! - Browse and create factions
//! - Edit faction properties (name, description, joinable, join requirement)
//! - View and edit inter-faction relationships
//! - View and modify player reputation with each faction
//! - Visualize reputation standings with color-coded bars

use egui::{Color32, RichText, Ui};
use genesis_common::FactionId;
use genesis_gameplay::faction::{
    Faction, FactionRegistry, FactionRelation, ReputationStanding, ReputationTracker,
};

// ============================================================================
// Factions Panel
// ============================================================================

/// Factions UI panel for managing factions, relationships, and reputation.
pub struct FactionsPanel {
    /// Faction registry (all defined factions).
    pub registry: FactionRegistry,
    /// Player reputation tracker.
    pub reputation: ReputationTracker,
    /// Currently selected faction index.
    selected_faction: Option<FactionId>,
    /// Which sub-section is active.
    active_section: FactionSection,
    /// New faction name input buffer.
    new_faction_name: String,
    /// New faction description input buffer.
    new_faction_desc: String,
    /// Next faction ID counter.
    next_faction_id: u16,
    /// Status message.
    status_message: Option<(String, Color32)>,
}

/// Sub-sections within the factions panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FactionSection {
    /// Faction list and details
    #[default]
    Factions,
    /// Inter-faction relationships matrix
    Relationships,
    /// Player reputation overview
    Reputation,
}

impl FactionSection {
    fn all() -> &'static [Self] {
        &[Self::Factions, Self::Relationships, Self::Reputation]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Factions => "📋 Factions",
            Self::Relationships => "🔗 Relationships",
            Self::Reputation => "⭐ Reputation",
        }
    }
}

impl Default for FactionsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl FactionsPanel {
    /// Create a new factions panel with sample factions.
    pub fn new() -> Self {
        let mut registry = FactionRegistry::new();
        let reputation = ReputationTracker::new();

        // Add some default factions
        let factions = vec![
            Faction::new(FactionId::new(1), "Villagers")
                .with_description("Peaceful townsfolk who value community and trade.")
                .with_join_requirement(0)
                .with_joinable(true),
            Faction::new(FactionId::new(2), "Merchant Guild")
                .with_description("A powerful trading organization controlling commerce routes.")
                .with_join_requirement(20)
                .with_joinable(true),
            Faction::new(FactionId::new(3), "Forest Wardens")
                .with_description("Guardians of the wild, protecting nature from exploitation.")
                .with_join_requirement(10)
                .with_joinable(true),
            Faction::new(FactionId::new(4), "Bandits")
                .with_description("Outlaws who prey on travelers and settlements.")
                .with_join_requirement(-20)
                .with_joinable(false),
            Faction::new(FactionId::new(5), "Royal Guard")
                .with_description("Elite soldiers sworn to protect the crown.")
                .with_join_requirement(40)
                .with_joinable(true),
        ];

        for faction in factions {
            registry.register(faction);
        }

        // Set some default relationships
        registry.set_mutual_relation(
            FactionId::new(1),
            FactionId::new(2),
            FactionRelation::Friendly,
        );
        registry.set_mutual_relation(
            FactionId::new(1),
            FactionId::new(4),
            FactionRelation::Enemy,
        );
        registry.set_mutual_relation(
            FactionId::new(3),
            FactionId::new(4),
            FactionRelation::AtWar,
        );
        registry.set_mutual_relation(
            FactionId::new(5),
            FactionId::new(4),
            FactionRelation::AtWar,
        );
        registry.set_mutual_relation(
            FactionId::new(5),
            FactionId::new(1),
            FactionRelation::Friendly,
        );

        Self {
            registry,
            reputation,
            selected_faction: None,
            active_section: FactionSection::default(),
            new_faction_name: String::new(),
            new_faction_desc: String::new(),
            next_faction_id: 6,
            status_message: None,
        }
    }

    /// Render the factions panel.
    pub fn render(&mut self, ui: &mut Ui) {
        // Section selector
        ui.horizontal(|ui| {
            for section in FactionSection::all() {
                let is_active = self.active_section == *section;
                let text = if is_active {
                    RichText::new(section.label())
                        .color(Color32::WHITE)
                        .strong()
                } else {
                    RichText::new(section.label()).color(Color32::LIGHT_GRAY)
                };
                let btn = egui::Button::new(text)
                    .fill(if is_active {
                        Color32::from_rgb(50, 50, 90)
                    } else {
                        Color32::from_rgb(35, 35, 50)
                    })
                    .rounding(egui::Rounding::same(3.0));
                if ui.add(btn).clicked() {
                    self.active_section = *section;
                }
            }
        });

        ui.add_space(8.0);

        // Status message
        if let Some((msg, color)) = &self.status_message {
            ui.label(RichText::new(msg.as_str()).color(*color).size(12.0));
            ui.add_space(4.0);
        }

        match self.active_section {
            FactionSection::Factions => self.render_factions_list(ui),
            FactionSection::Relationships => self.render_relationships(ui),
            FactionSection::Reputation => self.render_reputation(ui),
        }
    }

    // ========================================================================
    // Factions List & Details
    // ========================================================================

    fn render_factions_list(&mut self, ui: &mut Ui) {
        // Collect faction data first to avoid borrow issues
        let faction_ids: Vec<FactionId> = self.registry.all().map(|f| f.id).collect();
        let faction_names: Vec<String> = faction_ids
            .iter()
            .filter_map(|id| self.registry.get(*id).map(|f| f.name.clone()))
            .collect();

        ui.horizontal(|ui| {
            // Left: faction list
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                ui.label(RichText::new("Factions").strong().size(16.0));
                ui.separator();

                for (id, name) in faction_ids.iter().zip(faction_names.iter()) {
                    let is_selected = self.selected_faction == Some(*id);
                    let standing = self.reputation.standing(*id);
                    let standing_color = standing_color(standing);

                    let text = if is_selected {
                        RichText::new(format!("▸ {}", name))
                            .color(Color32::WHITE)
                            .strong()
                    } else {
                        RichText::new(format!("  {}", name)).color(Color32::LIGHT_GRAY)
                    };

                    ui.horizontal(|ui| {
                        // Standing indicator dot
                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 4.0, standing_color);

                        if ui
                            .add(egui::Label::new(text).sense(egui::Sense::click()))
                            .clicked()
                        {
                            self.selected_faction = Some(*id);
                        }
                    });
                }

                ui.add_space(12.0);
                ui.separator();
                ui.label(RichText::new("Add Faction").strong());

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_faction_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Desc:");
                    ui.text_edit_singleline(&mut self.new_faction_desc);
                });

                if ui.button("➕ Create Faction").clicked() && !self.new_faction_name.is_empty() {
                    let id = FactionId::new(self.next_faction_id);
                    self.next_faction_id += 1;
                    let faction = Faction::new(id, self.new_faction_name.clone())
                        .with_description(self.new_faction_desc.clone());
                    self.registry.register(faction);
                    self.new_faction_name.clear();
                    self.new_faction_desc.clear();
                    self.selected_faction = Some(id);
                    self.status_message =
                        Some(("Faction created!".into(), Color32::from_rgb(100, 255, 100)));
                }
            });

            ui.separator();

            // Right: selected faction details
            ui.vertical(|ui| {
                if let Some(faction_id) = self.selected_faction {
                    if let Some(faction) = self.registry.get(faction_id) {
                        let name = faction.name.clone();
                        let description = faction.description.clone();
                        let joinable = faction.joinable;
                        let join_req = faction.join_requirement;

                        ui.label(
                            RichText::new(&name)
                                .strong()
                                .size(18.0)
                                .color(Color32::WHITE),
                        );
                        ui.add_space(4.0);

                        ui.label(
                            RichText::new(&description)
                                .color(Color32::from_gray(180))
                                .italics(),
                        );
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label("Joinable:");
                            ui.label(if joinable {
                                RichText::new("Yes").color(Color32::from_rgb(100, 255, 100))
                            } else {
                                RichText::new("No").color(Color32::from_rgb(255, 100, 100))
                            });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Join Requirement:");
                            ui.label(
                                RichText::new(format!("{} reputation", join_req))
                                    .color(Color32::from_rgb(200, 200, 100)),
                            );
                        });

                        ui.add_space(8.0);

                        // Player standing with this faction
                        let rep_value = self.reputation.value(faction_id);
                        let standing = self.reputation.standing(faction_id);
                        let is_member = self.reputation.is_member(faction_id);

                        ui.label(RichText::new("Player Standing").strong());
                        ui.horizontal(|ui| {
                            ui.label("Reputation:");
                            ui.label(
                                RichText::new(format!("{}", rep_value))
                                    .color(standing_color(standing))
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(format!("({})", standing_label(standing)))
                                    .color(standing_color(standing)),
                            );
                        });

                        // Reputation bar
                        render_reputation_bar(ui, rep_value);

                        ui.horizontal(|ui| {
                            ui.label("Member:");
                            ui.label(if is_member {
                                RichText::new("✔ Yes")
                                    .color(Color32::from_rgb(100, 255, 100))
                                    .strong()
                            } else {
                                RichText::new("✘ No").color(Color32::from_gray(150))
                            });
                        });

                        ui.add_space(8.0);

                        // Reputation adjustment
                        ui.label(RichText::new("Adjust Reputation").strong());
                        ui.horizontal(|ui| {
                            if ui.button("-10").clicked() {
                                self.reputation.modify(faction_id, -10);
                            }
                            if ui.button("-5").clicked() {
                                self.reputation.modify(faction_id, -5);
                            }
                            if ui.button("-1").clicked() {
                                self.reputation.modify(faction_id, -1);
                            }
                            if ui.button("+1").clicked() {
                                self.reputation.modify(faction_id, 1);
                            }
                            if ui.button("+5").clicked() {
                                self.reputation.modify(faction_id, 5);
                            }
                            if ui.button("+10").clicked() {
                                self.reputation.modify(faction_id, 10);
                            }
                        });

                        // Relations with other factions
                        ui.add_space(12.0);
                        ui.label(RichText::new("Relations").strong());
                        ui.separator();

                        let other_factions: Vec<(FactionId, String, FactionRelation)> = faction_ids
                            .iter()
                            .filter(|id| **id != faction_id)
                            .filter_map(|id| {
                                self.registry.get(*id).map(|f| {
                                    let rel = self
                                        .registry
                                        .get(faction_id)
                                        .map(|selected| selected.relation(*id))
                                        .unwrap_or_default();
                                    (*id, f.name.clone(), rel)
                                })
                            })
                            .collect();

                        for (other_id, other_name, relation) in &other_factions {
                            ui.horizontal(|ui| {
                                ui.label(&*other_name);
                                ui.label(
                                    RichText::new(relation_label(*relation))
                                        .color(relation_color(*relation)),
                                );

                                // Relation selector
                                let relations = [
                                    FactionRelation::Allied,
                                    FactionRelation::Friendly,
                                    FactionRelation::Neutral,
                                    FactionRelation::Enemy,
                                    FactionRelation::AtWar,
                                ];
                                for rel in &relations {
                                    let label = relation_short(*rel);
                                    let is_current = *relation == *rel;
                                    let btn_text = if is_current {
                                        RichText::new(label)
                                            .color(relation_color(*rel))
                                            .strong()
                                            .size(11.0)
                                    } else {
                                        RichText::new(label)
                                            .color(Color32::from_gray(100))
                                            .size(11.0)
                                    };
                                    if ui.small_button(btn_text).clicked() {
                                        self.registry.set_mutual_relation(
                                            faction_id, *other_id, *rel,
                                        );
                                    }
                                }
                            });
                        }
                    } else {
                        self.selected_faction = None;
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("Select a faction to view details")
                                .color(Color32::from_gray(120))
                                .italics(),
                        );
                    });
                }
            });
        });
    }

    // ========================================================================
    // Relationships Matrix
    // ========================================================================

    fn render_relationships(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Faction Relationships Matrix").strong().size(16.0));
        ui.add_space(4.0);
        ui.label(
            RichText::new("Shows how each faction views the others")
                .color(Color32::from_gray(150))
                .size(12.0),
        );
        ui.add_space(8.0);

        // Collect faction data
        let factions: Vec<(FactionId, String)> = self
            .registry
            .all()
            .map(|f| (f.id, f.name.clone()))
            .collect();

        if factions.is_empty() {
            ui.label(
                RichText::new("No factions defined")
                    .color(Color32::from_gray(120))
                    .italics(),
            );
            return;
        }

        // Legend
        ui.horizontal(|ui| {
            ui.label(RichText::new("Legend:").size(11.0));
            for rel in &[
                FactionRelation::Allied,
                FactionRelation::Friendly,
                FactionRelation::Neutral,
                FactionRelation::Enemy,
                FactionRelation::AtWar,
            ] {
                ui.label(
                    RichText::new(format!("■ {}", relation_label(*rel)))
                        .color(relation_color(*rel))
                        .size(11.0),
                );
            }
        });
        ui.add_space(4.0);

        // Matrix grid
        egui::Grid::new("faction_relationship_matrix")
            .striped(true)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                // Header row
                ui.label(RichText::new("").size(12.0));
                for (_, name) in &factions {
                    ui.label(
                        RichText::new(truncate_name(name, 10))
                            .strong()
                            .size(11.0)
                            .color(Color32::from_gray(200)),
                    );
                }
                ui.end_row();

                // Data rows
                for (row_id, row_name) in &factions {
                    ui.label(
                        RichText::new(truncate_name(row_name, 12))
                            .strong()
                            .size(11.0)
                            .color(Color32::from_gray(200)),
                    );
                    for (col_id, _) in &factions {
                        if row_id == col_id {
                            ui.label(
                                RichText::new("—")
                                    .color(Color32::from_gray(80))
                                    .size(11.0),
                            );
                        } else {
                            let relation = self
                                .registry
                                .get(*row_id)
                                .map(|f| f.relation(*col_id))
                                .unwrap_or_default();
                            ui.label(
                                RichText::new(relation_short(relation))
                                    .color(relation_color(relation))
                                    .strong()
                                    .size(11.0),
                            );
                        }
                    }
                    ui.end_row();
                }
            });
    }

    // ========================================================================
    // Player Reputation Overview
    // ========================================================================

    fn render_reputation(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Player Reputation").strong().size(16.0));
        ui.add_space(4.0);
        ui.label(
            RichText::new("Your standing with each faction")
                .color(Color32::from_gray(150))
                .size(12.0),
        );
        ui.add_space(8.0);

        let factions: Vec<(FactionId, String, bool)> = self
            .registry
            .all()
            .map(|f| (f.id, f.name.clone(), f.joinable))
            .collect();

        if factions.is_empty() {
            ui.label(
                RichText::new("No factions defined")
                    .color(Color32::from_gray(120))
                    .italics(),
            );
            return;
        }

        for (id, name, joinable) in &factions {
            let value = self.reputation.value(*id);
            let standing = self.reputation.standing(*id);
            let is_member = self.reputation.is_member(*id);

            ui.horizontal(|ui| {
                // Faction name
                ui.label(
                    RichText::new(format!("{:<16}", name))
                        .strong()
                        .color(Color32::WHITE),
                );

                // Standing label
                ui.label(
                    RichText::new(format!("{:<10}", standing_label(standing)))
                        .color(standing_color(standing)),
                );

                // Reputation value
                ui.label(
                    RichText::new(format!("{:+}", value))
                        .color(standing_color(standing))
                        .strong(),
                );

                // Membership badge
                if is_member {
                    ui.label(RichText::new("★ Member").color(Color32::from_rgb(255, 215, 0)));
                }

                // Joinable indicator
                if *joinable && !is_member {
                    let join_req = self
                        .registry
                        .get(*id)
                        .map(|f| f.join_requirement)
                        .unwrap_or(0);
                    if value >= join_req {
                        ui.label(
                            RichText::new("(can join)")
                                .color(Color32::from_rgb(100, 200, 100))
                                .size(11.0),
                        );
                    }
                }
            });

            // Reputation bar
            render_reputation_bar(ui, value);
            ui.add_space(4.0);
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn standing_color(standing: ReputationStanding) -> Color32 {
    match standing {
        ReputationStanding::Hostile => Color32::from_rgb(255, 60, 60),
        ReputationStanding::Unfriendly => Color32::from_rgb(255, 150, 80),
        ReputationStanding::Neutral => Color32::from_rgb(180, 180, 180),
        ReputationStanding::Friendly => Color32::from_rgb(100, 200, 100),
        ReputationStanding::Allied => Color32::from_rgb(80, 160, 255),
    }
}

fn standing_label(standing: ReputationStanding) -> &'static str {
    match standing {
        ReputationStanding::Hostile => "Hostile",
        ReputationStanding::Unfriendly => "Unfriendly",
        ReputationStanding::Neutral => "Neutral",
        ReputationStanding::Friendly => "Friendly",
        ReputationStanding::Allied => "Allied",
    }
}

fn relation_color(relation: FactionRelation) -> Color32 {
    match relation {
        FactionRelation::Allied => Color32::from_rgb(80, 160, 255),
        FactionRelation::Friendly => Color32::from_rgb(100, 200, 100),
        FactionRelation::Neutral => Color32::from_rgb(180, 180, 180),
        FactionRelation::Enemy => Color32::from_rgb(255, 150, 80),
        FactionRelation::AtWar => Color32::from_rgb(255, 60, 60),
    }
}

fn relation_label(relation: FactionRelation) -> &'static str {
    match relation {
        FactionRelation::Allied => "Allied",
        FactionRelation::Friendly => "Friendly",
        FactionRelation::Neutral => "Neutral",
        FactionRelation::Enemy => "Enemy",
        FactionRelation::AtWar => "At War",
    }
}

fn relation_short(relation: FactionRelation) -> &'static str {
    match relation {
        FactionRelation::Allied => "A",
        FactionRelation::Friendly => "F",
        FactionRelation::Neutral => "N",
        FactionRelation::Enemy => "E",
        FactionRelation::AtWar => "W",
    }
}

fn truncate_name(name: &str, max: usize) -> String {
    if name.len() > max {
        format!("{}…", &name[..max - 1])
    } else {
        name.to_string()
    }
}

fn render_reputation_bar(ui: &mut Ui, value: i32) {
    let bar_width = 200.0;
    let bar_height = 12.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());
    let painter = ui.painter();

    // Background
    painter.rect_filled(rect, 2.0, Color32::from_gray(30));

    // Center line (neutral)
    let center_x = rect.min.x + bar_width / 2.0;
    painter.line_segment(
        [
            egui::pos2(center_x, rect.min.y),
            egui::pos2(center_x, rect.max.y),
        ],
        egui::Stroke::new(1.0, Color32::from_gray(80)),
    );

    // Reputation fill
    let normalized = value as f32 / 100.0; // -1.0 to 1.0
    let fill_color = if value >= 0 {
        Color32::from_rgb(80, 180, 100)
    } else {
        Color32::from_rgb(220, 80, 80)
    };

    let fill_width = (normalized.abs() * bar_width / 2.0).min(bar_width / 2.0);
    let fill_rect = if value >= 0 {
        egui::Rect::from_min_size(
            egui::pos2(center_x, rect.min.y),
            egui::vec2(fill_width, bar_height),
        )
    } else {
        egui::Rect::from_min_size(
            egui::pos2(center_x - fill_width, rect.min.y),
            egui::vec2(fill_width, bar_height),
        )
    };
    painter.rect_filled(fill_rect, 0.0, fill_color);

    // Border
    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, Color32::from_gray(60)));
}
