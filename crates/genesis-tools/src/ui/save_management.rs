//! Save Management UI
//!
//! Dialogs and tools for managing save files including delete confirmation,
//! copy to new slot, export to file, and import from file.

use egui::{Color32, Ui};
use serde::{Deserialize, Serialize};

/// Unique identifier for save management operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SaveManagementId(pub u64);

impl SaveManagementId {
    /// Create a new ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Generate a new random ID
    pub fn generate() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        Self(timestamp)
    }
}

/// Type of dialog currently shown
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DialogType {
    /// No dialog open
    #[default]
    None,
    /// Delete confirmation
    DeleteConfirm,
    /// Copy to slot selection
    CopyToSlot,
    /// Export options
    Export,
    /// Import options
    Import,
    /// Overwrite confirmation
    OverwriteConfirm,
    /// Error display
    Error,
}

impl DialogType {
    /// Get dialog title
    pub fn title(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::DeleteConfirm => "Delete Save",
            Self::CopyToSlot => "Copy Save",
            Self::Export => "Export Save",
            Self::Import => "Import Save",
            Self::OverwriteConfirm => "Overwrite Save",
            Self::Error => "Error",
        }
    }

    /// Check if this is a confirmation dialog
    pub fn is_confirmation(&self) -> bool {
        matches!(self, Self::DeleteConfirm | Self::OverwriteConfirm)
    }
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Native binary format (smallest)
    #[default]
    Binary,
    /// JSON format (readable)
    Json,
    /// Compressed binary
    Compressed,
}

impl ExportFormat {
    /// Get all formats
    pub fn all() -> &'static [Self] {
        &[Self::Binary, Self::Json, Self::Compressed]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Binary => "Binary (.sav)",
            Self::Json => "JSON (.json)",
            Self::Compressed => "Compressed (.sav.gz)",
        }
    }

    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Binary => "sav",
            Self::Json => "json",
            Self::Compressed => "sav.gz",
        }
    }

    /// Get estimated size multiplier relative to binary
    pub fn size_multiplier(&self) -> f32 {
        match self {
            Self::Binary => 1.0,
            Self::Json => 2.5,
            Self::Compressed => 0.3,
        }
    }
}

/// Import validation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportValidation {
    /// Save is valid and compatible
    Valid,
    /// Save is from a newer version
    NewerVersion {
        /// Required version
        required: u32,
        /// Current version
        current: u32,
    },
    /// Save is from an older version (may need migration)
    OlderVersion {
        /// Save file version
        save_version: u32,
        /// Current version
        current: u32,
    },
    /// Save file is corrupted
    Corrupted {
        /// Reason for corruption
        reason: String,
    },
    /// Save is incompatible
    Incompatible {
        /// Reason for incompatibility
        reason: String,
    },
}

impl ImportValidation {
    /// Check if import can proceed
    pub fn can_import(&self) -> bool {
        matches!(self, Self::Valid | Self::OlderVersion { .. })
    }

    /// Get warning message if any
    pub fn warning(&self) -> Option<String> {
        match self {
            Self::Valid => None,
            Self::NewerVersion { required, current } => {
                Some(format!(
                    "This save requires version {required} or newer. Current version: {current}"
                ))
            }
            Self::OlderVersion {
                save_version,
                current,
            } => Some(format!(
                "This save is from version {save_version}. It will be upgraded to version {current}."
            )),
            Self::Corrupted { reason } => Some(format!("Save file is corrupted: {reason}")),
            Self::Incompatible { reason } => Some(format!("Save is incompatible: {reason}")),
        }
    }

    /// Get display color
    pub fn color(&self) -> Color32 {
        match self {
            Self::Valid => Color32::from_rgb(100, 200, 100),
            Self::OlderVersion { .. } => Color32::from_rgb(200, 200, 100),
            Self::NewerVersion { .. } | Self::Corrupted { .. } | Self::Incompatible { .. } => {
                Color32::from_rgb(200, 100, 100)
            },
        }
    }
}

/// Brief save info for selection dialogs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlotBrief {
    /// Slot index
    pub slot: u32,
    /// Whether slot is empty
    pub empty: bool,
    /// Player name (if occupied)
    pub player_name: Option<String>,
    /// Player level (if occupied)
    pub level: Option<u32>,
    /// Formatted playtime
    pub playtime: Option<String>,
}

impl SaveSlotBrief {
    /// Create empty slot brief
    pub fn empty(slot: u32) -> Self {
        Self {
            slot,
            empty: true,
            player_name: None,
            level: None,
            playtime: None,
        }
    }

    /// Create occupied slot brief
    pub fn occupied(slot: u32, player_name: impl Into<String>, level: u32) -> Self {
        Self {
            slot,
            empty: false,
            player_name: Some(player_name.into()),
            level: Some(level),
            playtime: None,
        }
    }

    /// Set playtime
    pub fn with_playtime(mut self, playtime: impl Into<String>) -> Self {
        self.playtime = Some(playtime.into());
        self
    }

    /// Get display label
    pub fn display_label(&self) -> String {
        if self.empty {
            format!("Slot {} - Empty", self.slot + 1)
        } else {
            let name = self.player_name.as_deref().unwrap_or("Unknown");
            let level = self.level.unwrap_or(1);
            format!("Slot {} - {} (Lv.{})", self.slot + 1, name, level)
        }
    }
}

/// Actions generated by save management dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum SaveManagementAction {
    /// Confirm delete for a slot
    ConfirmDelete(u32),
    /// Cancel delete operation
    CancelDelete,
    /// Confirm copy from slot to slot
    ConfirmCopy {
        /// Source slot
        from: u32,
        /// Target slot
        to: u32,
    },
    /// Cancel copy operation
    CancelCopy,
    /// Export save to path
    ExportSave {
        /// Source slot
        slot: u32,
        /// Export path
        path: String,
        /// Export format
        format: ExportFormat,
    },
    /// Cancel export
    CancelExport,
    /// Import save from path to slot
    ImportSave {
        /// Import path
        path: String,
        /// Target slot
        slot: u32,
    },
    /// Cancel import
    CancelImport,
    /// Confirm overwrite
    ConfirmOverwrite {
        /// Slot to overwrite
        slot: u32,
    },
    /// Cancel overwrite
    CancelOverwrite,
    /// Dismiss error dialog
    DismissError,
    /// Close all dialogs
    CloseAll,
}

/// Configuration for save management UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveManagementConfig {
    /// Default export format
    pub default_export_format: ExportFormat,
    /// Whether to show advanced options
    pub show_advanced: bool,
    /// Whether to require confirmation for delete
    pub confirm_delete: bool,
    /// Whether to require confirmation for overwrite
    pub confirm_overwrite: bool,
    /// Dialog background color
    pub dialog_background: [u8; 4],
    /// Dialog border color
    pub dialog_border: [u8; 4],
    /// Warning color
    pub warning_color: [u8; 4],
    /// Error color
    pub error_color: [u8; 4],
}

impl Default for SaveManagementConfig {
    fn default() -> Self {
        Self {
            default_export_format: ExportFormat::Binary,
            show_advanced: false,
            confirm_delete: true,
            confirm_overwrite: true,
            dialog_background: [30, 30, 40, 245],
            dialog_border: [80, 80, 100, 255],
            warning_color: [255, 200, 100, 255],
            error_color: [255, 100, 100, 255],
        }
    }
}

/// Delete confirmation dialog state
#[derive(Debug, Clone)]
pub struct DeleteConfirmDialog {
    /// Slot to delete
    pub slot: u32,
    /// Player name in slot
    pub player_name: String,
    /// Whether user has typed "DELETE" to confirm
    pub confirmation_text: String,
    /// Whether hardcore delete (requires typing)
    pub require_typing: bool,
}

impl DeleteConfirmDialog {
    /// Create new delete confirmation
    pub fn new(slot: u32, player_name: impl Into<String>) -> Self {
        Self {
            slot,
            player_name: player_name.into(),
            confirmation_text: String::new(),
            require_typing: false,
        }
    }

    /// Require typing "DELETE" to confirm
    pub fn with_typing_required(mut self) -> Self {
        self.require_typing = true;
        self
    }

    /// Check if deletion is confirmed
    pub fn is_confirmed(&self) -> bool {
        if self.require_typing {
            self.confirmation_text.to_uppercase() == "DELETE"
        } else {
            true
        }
    }
}

/// Copy to slot dialog state
#[derive(Debug, Clone)]
pub struct CopyToSlotDialog {
    /// Source slot
    pub source_slot: u32,
    /// Available target slots
    pub available_slots: Vec<SaveSlotBrief>,
    /// Selected target slot
    pub selected_target: Option<u32>,
}

impl CopyToSlotDialog {
    /// Create new copy dialog
    pub fn new(source_slot: u32, available_slots: Vec<SaveSlotBrief>) -> Self {
        Self {
            source_slot,
            available_slots,
            selected_target: None,
        }
    }

    /// Select a target slot
    pub fn select_target(&mut self, slot: u32) {
        if self.available_slots.iter().any(|s| s.slot == slot) {
            self.selected_target = Some(slot);
        }
    }

    /// Check if a target is selected
    pub fn has_selection(&self) -> bool {
        self.selected_target.is_some()
    }

    /// Check if selected slot would overwrite existing save
    pub fn would_overwrite(&self) -> bool {
        self.selected_target.is_some_and(|slot| {
            self.available_slots
                .iter()
                .any(|s| s.slot == slot && !s.empty)
        })
    }
}

/// Export dialog state
#[derive(Debug, Clone)]
pub struct ExportDialog {
    /// Slot to export
    pub slot: u32,
    /// Player name
    pub player_name: String,
    /// Selected export format
    pub format: ExportFormat,
    /// Export path/filename
    pub filename: String,
    /// Include screenshots
    pub include_screenshots: bool,
    /// Estimated file size
    pub estimated_size: Option<u64>,
}

impl ExportDialog {
    /// Create new export dialog
    pub fn new(slot: u32, player_name: impl Into<String>) -> Self {
        let name = player_name.into();
        let filename = format!("{}_slot{}", name.replace(' ', "_"), slot + 1);
        Self {
            slot,
            player_name: name,
            format: ExportFormat::Binary,
            filename,
            include_screenshots: true,
            estimated_size: None,
        }
    }

    /// Set format
    pub fn set_format(&mut self, format: ExportFormat) {
        self.format = format;
    }

    /// Get full filename with extension
    pub fn full_filename(&self) -> String {
        format!("{}.{}", self.filename, self.format.extension())
    }

    /// Set estimated size based on format
    pub fn set_base_size(&mut self, base_size: u64) {
        let multiplier = self.format.size_multiplier();
        self.estimated_size = Some((base_size as f32 * multiplier) as u64);
    }

    /// Format estimated size for display
    pub fn format_size(&self) -> String {
        match self.estimated_size {
            Some(bytes) => {
                if bytes >= 1_000_000 {
                    format!("{:.1} MB", bytes as f64 / 1_000_000.0)
                } else if bytes >= 1_000 {
                    format!("{:.1} KB", bytes as f64 / 1_000.0)
                } else {
                    format!("{bytes} bytes")
                }
            }
            None => String::from("Unknown"),
        }
    }
}

/// Import dialog state
#[derive(Debug, Clone)]
pub struct ImportDialog {
    /// Selected file path
    pub path: Option<String>,
    /// Detected format
    pub detected_format: Option<ExportFormat>,
    /// Validation result
    pub validation: Option<ImportValidation>,
    /// Available slots for import
    pub available_slots: Vec<SaveSlotBrief>,
    /// Selected target slot
    pub selected_slot: Option<u32>,
    /// Preview data from file
    pub preview_player_name: Option<String>,
    /// Preview level
    pub preview_level: Option<u32>,
    /// Preview playtime
    pub preview_playtime: Option<String>,
}

impl ImportDialog {
    /// Create new import dialog
    pub fn new(available_slots: Vec<SaveSlotBrief>) -> Self {
        Self {
            path: None,
            detected_format: None,
            validation: None,
            available_slots,
            selected_slot: None,
            preview_player_name: None,
            preview_level: None,
            preview_playtime: None,
        }
    }

    /// Set file path
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = Some(path.into());
    }

    /// Set validation result
    pub fn set_validation(&mut self, validation: ImportValidation) {
        self.validation = Some(validation);
    }

    /// Set preview data
    pub fn set_preview(
        &mut self,
        player_name: impl Into<String>,
        level: u32,
        playtime: impl Into<String>,
    ) {
        self.preview_player_name = Some(player_name.into());
        self.preview_level = Some(level);
        self.preview_playtime = Some(playtime.into());
    }

    /// Select target slot
    pub fn select_slot(&mut self, slot: u32) {
        if self.available_slots.iter().any(|s| s.slot == slot) {
            self.selected_slot = Some(slot);
        }
    }

    /// Check if can proceed with import
    pub fn can_import(&self) -> bool {
        self.path.is_some()
            && self.selected_slot.is_some()
            && self.validation.as_ref().is_some_and(ImportValidation::can_import)
    }

    /// Check if would overwrite
    pub fn would_overwrite(&self) -> bool {
        self.selected_slot.is_some_and(|slot| {
            self.available_slots
                .iter()
                .any(|s| s.slot == slot && !s.empty)
        })
    }
}

/// Error dialog state
#[derive(Debug, Clone)]
pub struct ErrorDialog {
    /// Error title
    pub title: String,
    /// Error message
    pub message: String,
    /// Error details (expandable)
    pub details: Option<String>,
    /// Whether details are shown
    pub show_details: bool,
}

impl ErrorDialog {
    /// Create new error dialog
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            details: None,
            show_details: false,
        }
    }

    /// Add details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Toggle details visibility
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
}

/// Save management panel
#[derive(Debug)]
pub struct SaveManagement {
    /// Current dialog type
    dialog_type: DialogType,
    /// Delete confirmation state
    delete_dialog: Option<DeleteConfirmDialog>,
    /// Copy dialog state
    copy_dialog: Option<CopyToSlotDialog>,
    /// Export dialog state
    export_dialog: Option<ExportDialog>,
    /// Import dialog state
    import_dialog: Option<ImportDialog>,
    /// Error dialog state
    error_dialog: Option<ErrorDialog>,
    /// Pending actions
    actions: Vec<SaveManagementAction>,
    /// Configuration
    config: SaveManagementConfig,
}

impl SaveManagement {
    /// Create new save management panel
    pub fn new(config: SaveManagementConfig) -> Self {
        Self {
            dialog_type: DialogType::None,
            delete_dialog: None,
            copy_dialog: None,
            export_dialog: None,
            import_dialog: None,
            error_dialog: None,
            actions: Vec::new(),
            config,
        }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(SaveManagementConfig::default())
    }

    /// Get current dialog type
    pub fn dialog_type(&self) -> DialogType {
        self.dialog_type
    }

    /// Check if any dialog is open
    pub fn has_dialog(&self) -> bool {
        self.dialog_type != DialogType::None
    }

    /// Get configuration
    pub fn config(&self) -> &SaveManagementConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: SaveManagementConfig) {
        self.config = config;
    }

    /// Open delete confirmation dialog
    pub fn open_delete(&mut self, slot: u32, player_name: impl Into<String>) {
        self.delete_dialog = Some(DeleteConfirmDialog::new(slot, player_name));
        self.dialog_type = DialogType::DeleteConfirm;
    }

    /// Open copy to slot dialog
    pub fn open_copy(&mut self, source_slot: u32, available_slots: Vec<SaveSlotBrief>) {
        self.copy_dialog = Some(CopyToSlotDialog::new(source_slot, available_slots));
        self.dialog_type = DialogType::CopyToSlot;
    }

    /// Open export dialog
    pub fn open_export(&mut self, slot: u32, player_name: impl Into<String>) {
        self.export_dialog = Some(ExportDialog::new(slot, player_name));
        self.dialog_type = DialogType::Export;
    }

    /// Open import dialog
    pub fn open_import(&mut self, available_slots: Vec<SaveSlotBrief>) {
        self.import_dialog = Some(ImportDialog::new(available_slots));
        self.dialog_type = DialogType::Import;
    }

    /// Show error dialog
    pub fn show_error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.error_dialog = Some(ErrorDialog::new(title, message));
        self.dialog_type = DialogType::Error;
    }

    /// Show error with details
    pub fn show_error_with_details(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) {
        self.error_dialog = Some(ErrorDialog::new(title, message).with_details(details));
        self.dialog_type = DialogType::Error;
    }

    /// Close current dialog
    pub fn close_dialog(&mut self) {
        self.dialog_type = DialogType::None;
        self.delete_dialog = None;
        self.copy_dialog = None;
        self.export_dialog = None;
        self.import_dialog = None;
        self.error_dialog = None;
    }

    /// Get delete dialog (if open)
    pub fn delete_dialog(&self) -> Option<&DeleteConfirmDialog> {
        self.delete_dialog.as_ref()
    }

    /// Get delete dialog mutably
    pub fn delete_dialog_mut(&mut self) -> Option<&mut DeleteConfirmDialog> {
        self.delete_dialog.as_mut()
    }

    /// Get copy dialog (if open)
    pub fn copy_dialog(&self) -> Option<&CopyToSlotDialog> {
        self.copy_dialog.as_ref()
    }

    /// Get copy dialog mutably
    pub fn copy_dialog_mut(&mut self) -> Option<&mut CopyToSlotDialog> {
        self.copy_dialog.as_mut()
    }

    /// Get export dialog (if open)
    pub fn export_dialog(&self) -> Option<&ExportDialog> {
        self.export_dialog.as_ref()
    }

    /// Get export dialog mutably
    pub fn export_dialog_mut(&mut self) -> Option<&mut ExportDialog> {
        self.export_dialog.as_mut()
    }

    /// Get import dialog (if open)
    pub fn import_dialog(&self) -> Option<&ImportDialog> {
        self.import_dialog.as_ref()
    }

    /// Get import dialog mutably
    pub fn import_dialog_mut(&mut self) -> Option<&mut ImportDialog> {
        self.import_dialog.as_mut()
    }

    /// Drain pending actions
    pub fn drain_actions(&mut self) -> Vec<SaveManagementAction> {
        std::mem::take(&mut self.actions)
    }

    /// Render the current dialog
    pub fn render(&mut self, ui: &mut Ui) {
        if self.dialog_type == DialogType::None {
            return;
        }

        let bg = Color32::from_rgba_unmultiplied(
            self.config.dialog_background[0],
            self.config.dialog_background[1],
            self.config.dialog_background[2],
            self.config.dialog_background[3],
        );
        let border = Color32::from_rgba_unmultiplied(
            self.config.dialog_border[0],
            self.config.dialog_border[1],
            self.config.dialog_border[2],
            self.config.dialog_border[3],
        );

        egui::Window::new(self.dialog_type.title())
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                egui::Frame::none()
                    .fill(bg)
                    .stroke(egui::Stroke::new(1.0, border))
                    .inner_margin(16.0)
                    .rounding(8.0),
            )
            .show(ui.ctx(), |ui| match self.dialog_type {
                DialogType::DeleteConfirm => self.render_delete_dialog(ui),
                DialogType::CopyToSlot => self.render_copy_dialog(ui),
                DialogType::Export => self.render_export_dialog(ui),
                DialogType::Import => self.render_import_dialog(ui),
                DialogType::Error => self.render_error_dialog(ui),
                _ => {},
            });
    }

    fn render_delete_dialog(&mut self, ui: &mut Ui) {
        let Some(dialog) = &self.delete_dialog else {
            return;
        };

        ui.label(format!(
            "Are you sure you want to delete the save for \"{}\"?",
            dialog.player_name
        ));
        ui.add_space(8.0);

        let warning_color = Color32::from_rgba_unmultiplied(
            self.config.warning_color[0],
            self.config.warning_color[1],
            self.config.warning_color[2],
            self.config.warning_color[3],
        );
        ui.label(egui::RichText::new("⚠ This action cannot be undone!").color(warning_color));

        let slot = dialog.slot;
        let can_confirm = dialog.is_confirmed();

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(can_confirm, |ui| {
                if ui
                    .button(egui::RichText::new("Delete").color(Color32::from_rgb(255, 100, 100)))
                    .clicked()
                {
                    self.actions.push(SaveManagementAction::ConfirmDelete(slot));
                    self.close_dialog();
                }
            });

            if ui.button("Cancel").clicked() {
                self.actions.push(SaveManagementAction::CancelDelete);
                self.close_dialog();
            }
        });
    }

    fn render_copy_dialog(&mut self, ui: &mut Ui) {
        let Some(dialog) = self.copy_dialog.clone() else {
            return;
        };

        ui.label(format!(
            "Copy save from Slot {} to:",
            dialog.source_slot + 1
        ));
        ui.add_space(8.0);

        // Slot selection
        let mut selected = dialog.selected_target;
        for slot in &dialog.available_slots {
            let label = slot.display_label();
            let is_source = slot.slot == dialog.source_slot;

            ui.add_enabled_ui(!is_source, |ui| {
                if ui
                    .selectable_label(selected == Some(slot.slot), &label)
                    .clicked()
                {
                    selected = Some(slot.slot);
                }
            });
        }

        if let Some(copy_dialog) = &mut self.copy_dialog {
            copy_dialog.selected_target = selected;
        }

        // Warning if overwriting
        if let Some(target) = selected {
            if dialog
                .available_slots
                .iter()
                .any(|s| s.slot == target && !s.empty)
            {
                ui.add_space(8.0);
                let warning_color = Color32::from_rgba_unmultiplied(
                    self.config.warning_color[0],
                    self.config.warning_color[1],
                    self.config.warning_color[2],
                    self.config.warning_color[3],
                );
                ui.label(
                    egui::RichText::new("⚠ This will overwrite the existing save!")
                        .color(warning_color),
                );
            }
        }

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(selected.is_some(), |ui| {
                if ui.button("Copy").clicked() {
                    if let Some(to) = selected {
                        self.actions.push(SaveManagementAction::ConfirmCopy {
                            from: dialog.source_slot,
                            to,
                        });
                        self.close_dialog();
                    }
                }
            });

            if ui.button("Cancel").clicked() {
                self.actions.push(SaveManagementAction::CancelCopy);
                self.close_dialog();
            }
        });
    }

    fn render_export_dialog(&mut self, ui: &mut Ui) {
        let Some(dialog) = self.export_dialog.clone() else {
            return;
        };

        ui.label(format!("Export \"{}\" save:", dialog.player_name));
        ui.add_space(8.0);

        // Filename input
        let mut filename = dialog.filename.clone();
        ui.horizontal(|ui| {
            ui.label("Filename:");
            ui.text_edit_singleline(&mut filename);
        });
        if let Some(export_dialog) = &mut self.export_dialog {
            export_dialog.filename = filename;
        }

        // Format selection
        ui.add_space(8.0);
        ui.label("Format:");
        let mut format = dialog.format;
        for fmt in ExportFormat::all() {
            if ui
                .selectable_label(format == *fmt, fmt.display_name())
                .clicked()
            {
                format = *fmt;
            }
        }
        if let Some(export_dialog) = &mut self.export_dialog {
            export_dialog.format = format;
        }

        // Size estimate
        ui.add_space(8.0);
        ui.label(format!("Estimated size: {}", dialog.format_size()));

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui.button("Export").clicked() {
                self.actions.push(SaveManagementAction::ExportSave {
                    slot: dialog.slot,
                    path: dialog.full_filename(),
                    format: dialog.format,
                });
                self.close_dialog();
            }

            if ui.button("Cancel").clicked() {
                self.actions.push(SaveManagementAction::CancelExport);
                self.close_dialog();
            }
        });
    }

    fn render_import_dialog(&mut self, ui: &mut Ui) {
        let Some(dialog) = self.import_dialog.clone() else {
            return;
        };

        ui.label("Import save file:");
        ui.add_space(8.0);

        // File path
        if let Some(path) = &dialog.path {
            ui.label(format!("File: {path}"));
        } else {
            ui.label("No file selected");
        }

        // Validation status
        if let Some(validation) = &dialog.validation {
            ui.add_space(8.0);
            if let Some(warning) = validation.warning() {
                ui.label(egui::RichText::new(warning).color(validation.color()));
            } else {
                ui.label(egui::RichText::new("✓ Save file is valid").color(validation.color()));
            }
        }

        // Preview
        if dialog.preview_player_name.is_some() {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Preview:").strong());
            if let Some(name) = &dialog.preview_player_name {
                ui.label(format!("Player: {name}"));
            }
            if let Some(level) = dialog.preview_level {
                ui.label(format!("Level: {level}"));
            }
            if let Some(playtime) = &dialog.preview_playtime {
                ui.label(format!("Playtime: {playtime}"));
            }
        }

        // Slot selection
        ui.add_space(8.0);
        ui.label("Import to slot:");
        let mut selected = dialog.selected_slot;
        for slot in &dialog.available_slots {
            if ui
                .selectable_label(selected == Some(slot.slot), slot.display_label())
                .clicked()
            {
                selected = Some(slot.slot);
            }
        }
        if let Some(import_dialog) = &mut self.import_dialog {
            import_dialog.selected_slot = selected;
        }

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            let can_import = dialog.can_import();
            ui.add_enabled_ui(can_import, |ui| {
                if ui.button("Import").clicked() {
                    if let (Some(path), Some(slot)) = (&dialog.path, dialog.selected_slot) {
                        self.actions.push(SaveManagementAction::ImportSave {
                            path: path.clone(),
                            slot,
                        });
                        self.close_dialog();
                    }
                }
            });

            if ui.button("Cancel").clicked() {
                self.actions.push(SaveManagementAction::CancelImport);
                self.close_dialog();
            }
        });
    }

    fn render_error_dialog(&mut self, ui: &mut Ui) {
        let Some(dialog) = self.error_dialog.clone() else {
            return;
        };

        let error_color = Color32::from_rgba_unmultiplied(
            self.config.error_color[0],
            self.config.error_color[1],
            self.config.error_color[2],
            self.config.error_color[3],
        );

        ui.label(egui::RichText::new(&dialog.message).color(error_color));

        if dialog.details.is_some() {
            ui.add_space(8.0);
            let show_details = dialog.show_details;
            if ui.selectable_label(show_details, "Show details").clicked() {
                if let Some(error_dialog) = &mut self.error_dialog {
                    error_dialog.toggle_details();
                }
            }

            if show_details {
                if let Some(details) = &dialog.details {
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(details)
                                    .monospace()
                                    .small()
                                    .color(Color32::from_gray(180)),
                            );
                        });
                }
            }
        }

        ui.add_space(16.0);
        if ui.button("OK").clicked() {
            self.actions.push(SaveManagementAction::DismissError);
            self.close_dialog();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_management_id() {
        let id = SaveManagementId::new(12345);
        assert_eq!(id.0, 12345);

        let generated = SaveManagementId::generate();
        assert!(generated.0 > 0);
    }

    #[test]
    fn test_dialog_type_title() {
        assert_eq!(DialogType::DeleteConfirm.title(), "Delete Save");
        assert_eq!(DialogType::CopyToSlot.title(), "Copy Save");
        assert_eq!(DialogType::Export.title(), "Export Save");
        assert_eq!(DialogType::Import.title(), "Import Save");
    }

    #[test]
    fn test_dialog_type_is_confirmation() {
        assert!(DialogType::DeleteConfirm.is_confirmation());
        assert!(DialogType::OverwriteConfirm.is_confirmation());
        assert!(!DialogType::Export.is_confirmation());
        assert!(!DialogType::Import.is_confirmation());
    }

    #[test]
    fn test_export_format_all() {
        let formats = ExportFormat::all();
        assert_eq!(formats.len(), 3);
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Binary.extension(), "sav");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Compressed.extension(), "sav.gz");
    }

    #[test]
    fn test_export_format_size_multiplier() {
        assert_eq!(ExportFormat::Binary.size_multiplier(), 1.0);
        assert!(ExportFormat::Json.size_multiplier() > 1.0);
        assert!(ExportFormat::Compressed.size_multiplier() < 1.0);
    }

    #[test]
    fn test_import_validation_can_import() {
        assert!(ImportValidation::Valid.can_import());
        assert!(ImportValidation::OlderVersion {
            save_version: 1,
            current: 2
        }
        .can_import());
        assert!(!ImportValidation::NewerVersion {
            required: 3,
            current: 2
        }
        .can_import());
        assert!(!ImportValidation::Corrupted {
            reason: "test".to_string()
        }
        .can_import());
    }

    #[test]
    fn test_import_validation_warning() {
        assert!(ImportValidation::Valid.warning().is_none());
        assert!(ImportValidation::OlderVersion {
            save_version: 1,
            current: 2
        }
        .warning()
        .is_some());
    }

    #[test]
    fn test_save_slot_brief_empty() {
        let slot = SaveSlotBrief::empty(0);
        assert!(slot.empty);
        assert!(slot.player_name.is_none());
        assert!(slot.display_label().contains("Empty"));
    }

    #[test]
    fn test_save_slot_brief_occupied() {
        let slot = SaveSlotBrief::occupied(0, "Hero", 25);
        assert!(!slot.empty);
        assert_eq!(slot.player_name, Some("Hero".to_string()));
        assert!(slot.display_label().contains("Hero"));
        assert!(slot.display_label().contains("25"));
    }

    #[test]
    fn test_delete_confirm_dialog() {
        let dialog = DeleteConfirmDialog::new(0, "Hero");
        assert_eq!(dialog.slot, 0);
        assert_eq!(dialog.player_name, "Hero");
        assert!(dialog.is_confirmed()); // No typing required

        let strict = DeleteConfirmDialog::new(0, "Hero").with_typing_required();
        assert!(!strict.is_confirmed()); // Need to type DELETE
    }

    #[test]
    fn test_copy_to_slot_dialog() {
        let slots = vec![
            SaveSlotBrief::empty(0),
            SaveSlotBrief::occupied(1, "Hero", 10),
            SaveSlotBrief::empty(2),
        ];
        let mut dialog = CopyToSlotDialog::new(1, slots);

        assert!(!dialog.has_selection());

        dialog.select_target(2);
        assert!(dialog.has_selection());
        assert_eq!(dialog.selected_target, Some(2));
    }

    #[test]
    fn test_copy_to_slot_would_overwrite() {
        let slots = vec![
            SaveSlotBrief::occupied(0, "Hero1", 10),
            SaveSlotBrief::empty(1),
        ];
        let mut dialog = CopyToSlotDialog::new(0, slots);

        dialog.select_target(0);
        assert!(dialog.would_overwrite());

        dialog.select_target(1);
        assert!(!dialog.would_overwrite());
    }

    #[test]
    fn test_export_dialog() {
        let dialog = ExportDialog::new(0, "Hero");
        assert_eq!(dialog.slot, 0);
        assert_eq!(dialog.player_name, "Hero");
        assert!(dialog.full_filename().contains("Hero"));
        assert!(dialog.full_filename().ends_with(".sav"));
    }

    #[test]
    fn test_export_dialog_format() {
        let mut dialog = ExportDialog::new(0, "Hero");
        dialog.set_format(ExportFormat::Json);
        assert!(dialog.full_filename().ends_with(".json"));
    }

    #[test]
    fn test_export_dialog_format_size() {
        let mut dialog = ExportDialog::new(0, "Hero");
        dialog.set_base_size(1000);
        assert!(dialog.estimated_size.is_some());

        let size_str = dialog.format_size();
        assert!(!size_str.is_empty());
    }

    #[test]
    fn test_import_dialog() {
        let slots = vec![SaveSlotBrief::empty(0)];
        let dialog = ImportDialog::new(slots);
        assert!(dialog.path.is_none());
        assert!(!dialog.can_import());
    }

    #[test]
    fn test_import_dialog_can_import() {
        let slots = vec![SaveSlotBrief::empty(0)];
        let mut dialog = ImportDialog::new(slots);

        dialog.set_path("/test/save.sav");
        dialog.set_validation(ImportValidation::Valid);
        dialog.select_slot(0);

        assert!(dialog.can_import());
    }

    #[test]
    fn test_error_dialog() {
        let dialog = ErrorDialog::new("Error", "Something went wrong");
        assert_eq!(dialog.title, "Error");
        assert_eq!(dialog.message, "Something went wrong");
        assert!(!dialog.show_details);
    }

    #[test]
    fn test_error_dialog_with_details() {
        let mut dialog = ErrorDialog::new("Error", "Failed").with_details("Stack trace...");
        assert!(dialog.details.is_some());

        dialog.toggle_details();
        assert!(dialog.show_details);

        dialog.toggle_details();
        assert!(!dialog.show_details);
    }

    #[test]
    fn test_save_management_new() {
        let mgmt = SaveManagement::with_defaults();
        assert_eq!(mgmt.dialog_type(), DialogType::None);
        assert!(!mgmt.has_dialog());
    }

    #[test]
    fn test_save_management_open_delete() {
        let mut mgmt = SaveManagement::with_defaults();
        mgmt.open_delete(0, "Hero");

        assert_eq!(mgmt.dialog_type(), DialogType::DeleteConfirm);
        assert!(mgmt.has_dialog());
        assert!(mgmt.delete_dialog().is_some());
    }

    #[test]
    fn test_save_management_open_copy() {
        let mut mgmt = SaveManagement::with_defaults();
        let slots = vec![SaveSlotBrief::empty(0), SaveSlotBrief::empty(1)];
        mgmt.open_copy(0, slots);

        assert_eq!(mgmt.dialog_type(), DialogType::CopyToSlot);
        assert!(mgmt.copy_dialog().is_some());
    }

    #[test]
    fn test_save_management_open_export() {
        let mut mgmt = SaveManagement::with_defaults();
        mgmt.open_export(0, "Hero");

        assert_eq!(mgmt.dialog_type(), DialogType::Export);
        assert!(mgmt.export_dialog().is_some());
    }

    #[test]
    fn test_save_management_open_import() {
        let mut mgmt = SaveManagement::with_defaults();
        let slots = vec![SaveSlotBrief::empty(0)];
        mgmt.open_import(slots);

        assert_eq!(mgmt.dialog_type(), DialogType::Import);
        assert!(mgmt.import_dialog().is_some());
    }

    #[test]
    fn test_save_management_show_error() {
        let mut mgmt = SaveManagement::with_defaults();
        mgmt.show_error("Test Error", "Something failed");

        assert_eq!(mgmt.dialog_type(), DialogType::Error);
        assert!(mgmt.error_dialog.is_some());
    }

    #[test]
    fn test_save_management_close_dialog() {
        let mut mgmt = SaveManagement::with_defaults();
        mgmt.open_delete(0, "Hero");

        mgmt.close_dialog();

        assert_eq!(mgmt.dialog_type(), DialogType::None);
        assert!(mgmt.delete_dialog().is_none());
    }

    #[test]
    fn test_save_management_config_defaults() {
        let config = SaveManagementConfig::default();
        assert!(config.confirm_delete);
        assert!(config.confirm_overwrite);
        assert_eq!(config.default_export_format, ExportFormat::Binary);
    }

    #[test]
    fn test_export_format_serialization() {
        let format = ExportFormat::Json;
        let json = serde_json::to_string(&format).unwrap();
        let parsed: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, format);
    }

    #[test]
    fn test_import_validation_serialization() {
        let valid = ImportValidation::Valid;
        let json = serde_json::to_string(&valid).unwrap();
        let parsed: ImportValidation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, valid);
    }

    #[test]
    fn test_save_management_config_serialization() {
        let config = SaveManagementConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: SaveManagementConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.confirm_delete, config.confirm_delete);
        assert_eq!(parsed.default_export_format, config.default_export_format);
    }

    #[test]
    fn test_save_slot_brief_serialization() {
        let slot = SaveSlotBrief::occupied(0, "Hero", 25).with_playtime("10:00:00");
        let json = serde_json::to_string(&slot).unwrap();
        let parsed: SaveSlotBrief = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.player_name, slot.player_name);
        assert_eq!(parsed.level, slot.level);
    }
}
