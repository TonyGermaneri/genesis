//! Debug console for in-game commands using egui.
//!
//! This module provides:
//! - Command input with history (up/down arrows)
//! - Tab completion for commands
//! - Scrollable output log
//! - Toggle with keybind (default: `)
//! - Customizable command handlers

use egui::{
    Align, Color32, Context, FontFamily, FontId, Key, Modifiers, RichText, ScrollArea, TextEdit,
    Ui, Window,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Default maximum history entries.
pub const DEFAULT_HISTORY_SIZE: usize = 100;

/// Default maximum output lines.
pub const DEFAULT_OUTPUT_SIZE: usize = 1000;

/// Output message level for coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputLevel {
    /// Normal output
    #[default]
    Info,
    /// Success/confirmation message
    Success,
    /// Warning message
    Warning,
    /// Error message
    Error,
    /// System message
    System,
    /// User command echo
    Command,
}

impl OutputLevel {
    /// Returns the color for this output level.
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            OutputLevel::Info => Color32::WHITE,
            OutputLevel::Success => Color32::from_rgb(100, 255, 100),
            OutputLevel::Warning => Color32::YELLOW,
            OutputLevel::Error => Color32::from_rgb(255, 100, 100),
            OutputLevel::System => Color32::from_rgb(150, 150, 255),
            OutputLevel::Command => Color32::from_rgb(200, 200, 200),
        }
    }
}

/// A line of output in the console.
#[derive(Debug, Clone)]
pub struct OutputLine {
    /// The text content
    pub text: String,
    /// Output level for coloring
    pub level: OutputLevel,
    /// Timestamp (optional)
    pub timestamp: Option<String>,
}

impl OutputLine {
    /// Creates a new output line.
    #[must_use]
    pub fn new(text: impl Into<String>, level: OutputLevel) -> Self {
        Self {
            text: text.into(),
            level,
            timestamp: None,
        }
    }

    /// Creates with timestamp.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Creates an info line.
    #[must_use]
    pub fn info(text: impl Into<String>) -> Self {
        Self::new(text, OutputLevel::Info)
    }

    /// Creates a success line.
    #[must_use]
    pub fn success(text: impl Into<String>) -> Self {
        Self::new(text, OutputLevel::Success)
    }

    /// Creates a warning line.
    #[must_use]
    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(text, OutputLevel::Warning)
    }

    /// Creates an error line.
    #[must_use]
    pub fn error(text: impl Into<String>) -> Self {
        Self::new(text, OutputLevel::Error)
    }

    /// Creates a system line.
    #[must_use]
    pub fn system(text: impl Into<String>) -> Self {
        Self::new(text, OutputLevel::System)
    }

    /// Creates a command echo line.
    #[must_use]
    pub fn command(text: impl Into<String>) -> Self {
        Self::new(text, OutputLevel::Command)
    }
}

/// Command definition for registration.
#[derive(Debug, Clone)]
pub struct CommandDef {
    /// Command name (what user types)
    pub name: String,
    /// Brief description
    pub description: String,
    /// Usage syntax
    pub usage: String,
    /// Category for grouping
    pub category: String,
}

impl CommandDef {
    /// Creates a new command definition.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        usage: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            usage: usage.into(),
            category: "General".to_string(),
        }
    }

    /// Sets the category.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }
}

/// Result of executing a command.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Output lines to display
    pub output: Vec<OutputLine>,
    /// Whether the command was successful
    pub success: bool,
}

impl CommandResult {
    /// Creates a successful result with output.
    #[must_use]
    pub fn ok(output: Vec<OutputLine>) -> Self {
        Self {
            output,
            success: true,
        }
    }

    /// Creates a successful result with a single message.
    #[must_use]
    pub fn ok_msg(msg: impl Into<String>) -> Self {
        Self {
            output: vec![OutputLine::success(msg)],
            success: true,
        }
    }

    /// Creates an error result.
    #[must_use]
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            output: vec![OutputLine::error(msg)],
            success: false,
        }
    }

    /// Creates a result with multiple output lines.
    #[must_use]
    pub fn with_lines(lines: Vec<OutputLine>) -> Self {
        let success = !lines.iter().any(|l| l.level == OutputLevel::Error);
        Self {
            output: lines,
            success,
        }
    }
}

/// Actions from the debug console.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleAction {
    /// Command was submitted
    CommandSubmitted(String),
    /// Console was opened
    Opened,
    /// Console was closed
    Closed,
    /// Console was toggled
    Toggled,
}

/// Configuration for the debug console.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    /// Toggle key (default: backtick/grave)
    pub toggle_key: String,
    /// Maximum command history entries
    pub max_history: usize,
    /// Maximum output lines
    pub max_output: usize,
    /// Console height as fraction of screen
    pub height_fraction: f32,
    /// Background color
    pub background_color: [u8; 4],
    /// Input background color
    pub input_bg_color: [u8; 4],
    /// Font size
    pub font_size: f32,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Command prefix (e.g., ">")
    pub command_prefix: String,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            toggle_key: "`".to_string(),
            max_history: DEFAULT_HISTORY_SIZE,
            max_output: DEFAULT_OUTPUT_SIZE,
            height_fraction: 0.4,
            background_color: [20, 20, 30, 240],
            input_bg_color: [30, 30, 40, 255],
            font_size: 14.0,
            show_timestamps: false,
            command_prefix: "> ".to_string(),
        }
    }
}

/// Debug console UI.
#[derive(Debug)]
pub struct DebugConsole {
    /// Whether the console is open
    pub is_open: bool,
    /// Configuration
    pub config: ConsoleConfig,
    /// Current input text
    input: String,
    /// Command history
    history: VecDeque<String>,
    /// Current position in history (-1 = current input)
    history_index: Option<usize>,
    /// Saved input when browsing history
    saved_input: String,
    /// Output lines
    output: VecDeque<OutputLine>,
    /// Registered commands
    commands: Vec<CommandDef>,
    /// Autocomplete suggestions
    suggestions: Vec<String>,
    /// Current suggestion index
    suggestion_index: usize,
    /// Whether we need to scroll to bottom
    scroll_to_bottom: bool,
    /// Whether input should be focused
    focus_input: bool,
}

impl Default for DebugConsole {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugConsole {
    /// Creates a new debug console.
    #[must_use]
    pub fn new() -> Self {
        let mut console = Self {
            is_open: false,
            config: ConsoleConfig::default(),
            input: String::new(),
            history: VecDeque::new(),
            history_index: None,
            saved_input: String::new(),
            output: VecDeque::new(),
            commands: Vec::new(),
            suggestions: Vec::new(),
            suggestion_index: 0,
            scroll_to_bottom: false,
            focus_input: false,
        };

        // Register built-in commands
        console.register_builtin_commands();
        console.print_system("Debug console initialized. Type 'help' for commands.");

        console
    }

    /// Creates with custom configuration.
    #[must_use]
    pub fn with_config(config: ConsoleConfig) -> Self {
        let mut console = Self {
            is_open: false,
            config,
            input: String::new(),
            history: VecDeque::new(),
            history_index: None,
            saved_input: String::new(),
            output: VecDeque::new(),
            commands: Vec::new(),
            suggestions: Vec::new(),
            suggestion_index: 0,
            scroll_to_bottom: false,
            focus_input: false,
        };

        console.register_builtin_commands();
        console.print_system("Debug console initialized. Type 'help' for commands.");

        console
    }

    /// Registers built-in commands.
    fn register_builtin_commands(&mut self) {
        self.register_command(CommandDef::new(
            "help",
            "Show available commands",
            "help [command]",
        ));
        self.register_command(CommandDef::new(
            "clear",
            "Clear the console output",
            "clear",
        ));
        self.register_command(CommandDef::new(
            "history",
            "Show command history",
            "history",
        ));
        self.register_command(
            CommandDef::new("echo", "Print a message", "echo <message>").with_category("Utility"),
        );
    }

    /// Registers a command.
    pub fn register_command(&mut self, cmd: CommandDef) {
        if !self.commands.iter().any(|c| c.name == cmd.name) {
            self.commands.push(cmd);
        }
    }

    /// Shows the console UI.
    pub fn show(&mut self, ctx: &Context) -> Vec<ConsoleAction> {
        let mut actions = Vec::new();

        // Handle toggle key
        if self.handle_toggle_key(ctx) {
            actions.push(ConsoleAction::Toggled);
            if self.is_open {
                actions.push(ConsoleAction::Opened);
            } else {
                actions.push(ConsoleAction::Closed);
            }
        }

        if !self.is_open {
            return actions;
        }

        let screen_rect = ctx.screen_rect();
        let console_height = screen_rect.height() * self.config.height_fraction;

        Window::new("Console")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO)
            .fixed_size(egui::Vec2::new(screen_rect.width(), console_height))
            .frame(egui::Frame::none().fill(Color32::from_rgba_unmultiplied(
                self.config.background_color[0],
                self.config.background_color[1],
                self.config.background_color[2],
                self.config.background_color[3],
            )))
            .show(ctx, |ui| {
                self.render_console(ui, &mut actions);
            });

        actions
    }

    /// Handles the toggle key.
    fn handle_toggle_key(&mut self, ctx: &Context) -> bool {
        ctx.input(|i| {
            // Check for backtick/grave key
            if i.key_pressed(Key::Backtick) && i.modifiers == Modifiers::NONE {
                self.toggle();
                return true;
            }
            // Also check Escape to close
            if self.is_open && i.key_pressed(Key::Escape) {
                self.close();
                return true;
            }
            false
        })
    }

    /// Renders the console content.
    fn render_console(&mut self, ui: &mut Ui, actions: &mut Vec<ConsoleAction>) {
        let font_id = FontId::new(self.config.font_size, FontFamily::Monospace);

        // Output area
        let available_height = ui.available_height() - 30.0; // Reserve space for input
        ScrollArea::vertical()
            .id_salt("console_output")
            .max_height(available_height)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &self.output {
                    self.render_output_line(ui, line, &font_id);
                }

                if self.scroll_to_bottom {
                    ui.scroll_to_cursor(Some(Align::BOTTOM));
                    self.scroll_to_bottom = false;
                }
            });

        ui.separator();

        // Input area
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&self.config.command_prefix)
                    .font(font_id.clone())
                    .color(Color32::GREEN),
            );

            let response = ui.add(
                TextEdit::singleline(&mut self.input)
                    .font(font_id.clone())
                    .desired_width(ui.available_width() - 10.0)
                    .frame(false),
            );

            // Focus input when console opens
            if self.focus_input {
                response.request_focus();
                self.focus_input = false;
            }

            // Handle input keys
            if response.has_focus() {
                self.handle_input_keys(ui.ctx(), actions);
            }

            // Show suggestions
            if !self.suggestions.is_empty() {
                ui.label(
                    RichText::new(format!(" [{}]", self.suggestions[self.suggestion_index]))
                        .font(font_id)
                        .color(Color32::GRAY),
                );
            }
        });
    }

    /// Renders a single output line.
    fn render_output_line(&self, ui: &mut Ui, line: &OutputLine, font_id: &FontId) {
        ui.horizontal(|ui| {
            if self.config.show_timestamps {
                if let Some(ts) = &line.timestamp {
                    ui.label(
                        RichText::new(format!("[{ts}] "))
                            .font(font_id.clone())
                            .color(Color32::GRAY),
                    );
                }
            }

            ui.label(
                RichText::new(&line.text)
                    .font(font_id.clone())
                    .color(line.level.color()),
            );
        });
    }

    /// Handles keyboard input for the console.
    fn handle_input_keys(&mut self, ctx: &Context, actions: &mut Vec<ConsoleAction>) {
        ctx.input(|i| {
            // Enter: submit command
            if i.key_pressed(Key::Enter) && !self.input.is_empty() {
                let cmd = self.input.clone();
                self.submit_command(&cmd, actions);
            }

            // Up: previous history
            if i.key_pressed(Key::ArrowUp) {
                self.history_up();
            }

            // Down: next history
            if i.key_pressed(Key::ArrowDown) {
                self.history_down();
            }

            // Tab: autocomplete
            if i.key_pressed(Key::Tab) {
                self.autocomplete();
            }
        });

        // Update suggestions as user types
        self.update_suggestions();
    }

    /// Submits a command for execution.
    fn submit_command(&mut self, cmd: &str, actions: &mut Vec<ConsoleAction>) {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return;
        }

        // Echo command
        self.print_command(trimmed);

        // Add to history
        self.add_to_history(trimmed.to_string());

        // Clear input
        self.input.clear();
        self.history_index = None;
        self.suggestions.clear();

        // Add action
        actions.push(ConsoleAction::CommandSubmitted(trimmed.to_string()));

        // Handle built-in commands
        if let Some(result) = self.handle_builtin(trimmed) {
            for line in result.output {
                self.output.push_back(line);
            }
            self.trim_output();
        }

        self.scroll_to_bottom = true;
    }

    /// Handles built-in commands.
    fn handle_builtin(&mut self, cmd: &str) -> Option<CommandResult> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts.first()?;
        let args = &parts[1..];

        match *command {
            "help" => Some(self.cmd_help(args)),
            "clear" => {
                self.output.clear();
                Some(CommandResult::ok(vec![]))
            },
            "history" => Some(self.cmd_history()),
            "echo" => Some(CommandResult::ok_msg(args.join(" "))),
            _ => None, // Not a built-in command
        }
    }

    /// Help command implementation.
    fn cmd_help(&self, args: &[&str]) -> CommandResult {
        if let Some(cmd_name) = args.first() {
            // Help for specific command
            if let Some(cmd) = self.commands.iter().find(|c| c.name == *cmd_name) {
                CommandResult::ok(vec![
                    OutputLine::info(format!("{}: {}", cmd.name, cmd.description)),
                    OutputLine::info(format!("Usage: {}", cmd.usage)),
                    OutputLine::info(format!("Category: {}", cmd.category)),
                ])
            } else {
                CommandResult::err(format!("Unknown command: {cmd_name}"))
            }
        } else {
            // List all commands
            let mut lines = vec![OutputLine::info("Available commands:")];
            for cmd in &self.commands {
                lines.push(OutputLine::info(format!(
                    "  {} - {}",
                    cmd.name, cmd.description
                )));
            }
            lines.push(OutputLine::info(""));
            lines.push(OutputLine::info("Type 'help <command>' for more info."));
            CommandResult::ok(lines)
        }
    }

    /// History command implementation.
    fn cmd_history(&self) -> CommandResult {
        if self.history.is_empty() {
            return CommandResult::ok_msg("No command history.");
        }

        let mut lines = vec![OutputLine::info("Command history:")];
        for (i, cmd) in self.history.iter().enumerate() {
            lines.push(OutputLine::info(format!("  {}: {}", i + 1, cmd)));
        }
        CommandResult::ok(lines)
    }

    /// Adds a command to history.
    fn add_to_history(&mut self, cmd: String) {
        // Don't add duplicates of last command
        if self.history.front() == Some(&cmd) {
            return;
        }

        self.history.push_front(cmd);

        // Trim history
        while self.history.len() > self.config.max_history {
            self.history.pop_back();
        }
    }

    /// Navigates up in history.
    fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.saved_input = self.input.clone();
                self.history_index = Some(0);
                self.input = self.history[0].clone();
            },
            Some(idx) if idx + 1 < self.history.len() => {
                self.history_index = Some(idx + 1);
                self.input = self.history[idx + 1].clone();
            },
            _ => {},
        }
    }

    /// Navigates down in history.
    fn history_down(&mut self) {
        match self.history_index {
            Some(0) => {
                self.history_index = None;
                self.input = self.saved_input.clone();
            },
            Some(idx) => {
                self.history_index = Some(idx - 1);
                self.input = self.history[idx - 1].clone();
            },
            None => {},
        }
    }

    /// Updates autocomplete suggestions.
    fn update_suggestions(&mut self) {
        let input_lower = self.input.to_lowercase();
        if input_lower.is_empty() {
            self.suggestions.clear();
            return;
        }

        self.suggestions = self
            .commands
            .iter()
            .filter(|c| c.name.to_lowercase().starts_with(&input_lower))
            .map(|c| c.name.clone())
            .collect();

        self.suggestion_index = 0;
    }

    /// Applies autocomplete.
    fn autocomplete(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }

        self.input = self.suggestions[self.suggestion_index].clone();
        self.suggestion_index = (self.suggestion_index + 1) % self.suggestions.len();
    }

    /// Trims output to max size.
    fn trim_output(&mut self) {
        while self.output.len() > self.config.max_output {
            self.output.pop_front();
        }
    }

    /// Prints an info message.
    pub fn print_info(&mut self, msg: impl Into<String>) {
        self.output.push_back(OutputLine::info(msg));
        self.trim_output();
        self.scroll_to_bottom = true;
    }

    /// Prints a success message.
    pub fn print_success(&mut self, msg: impl Into<String>) {
        self.output.push_back(OutputLine::success(msg));
        self.trim_output();
        self.scroll_to_bottom = true;
    }

    /// Prints a warning message.
    pub fn print_warning(&mut self, msg: impl Into<String>) {
        self.output.push_back(OutputLine::warning(msg));
        self.trim_output();
        self.scroll_to_bottom = true;
    }

    /// Prints an error message.
    pub fn print_error(&mut self, msg: impl Into<String>) {
        self.output.push_back(OutputLine::error(msg));
        self.trim_output();
        self.scroll_to_bottom = true;
    }

    /// Prints a system message.
    pub fn print_system(&mut self, msg: impl Into<String>) {
        self.output.push_back(OutputLine::system(msg));
        self.trim_output();
        self.scroll_to_bottom = true;
    }

    /// Prints a command echo.
    fn print_command(&mut self, cmd: &str) {
        self.output
            .push_back(OutputLine::command(format!("> {cmd}")));
        self.trim_output();
    }

    /// Prints command result output.
    pub fn print_result(&mut self, result: CommandResult) {
        for line in result.output {
            self.output.push_back(line);
        }
        self.trim_output();
        self.scroll_to_bottom = true;
    }

    /// Opens the console.
    pub fn open(&mut self) {
        self.is_open = true;
        self.focus_input = true;
    }

    /// Closes the console.
    pub fn close(&mut self) {
        self.is_open = false;
        self.suggestions.clear();
    }

    /// Toggles the console.
    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Clears the output.
    pub fn clear(&mut self) {
        self.output.clear();
    }

    /// Returns the command history.
    #[must_use]
    pub fn get_history(&self) -> &VecDeque<String> {
        &self.history
    }

    /// Returns registered commands.
    #[must_use]
    pub fn get_commands(&self) -> &[CommandDef] {
        &self.commands
    }

    /// Returns the current input.
    #[must_use]
    pub fn get_input(&self) -> &str {
        &self.input
    }

    /// Returns the output lines.
    #[must_use]
    pub fn get_output(&self) -> &VecDeque<OutputLine> {
        &self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_level_colors() {
        assert_eq!(OutputLevel::Info.color(), Color32::WHITE);
        assert_eq!(OutputLevel::Error.color(), Color32::from_rgb(255, 100, 100));
        assert_eq!(
            OutputLevel::Success.color(),
            Color32::from_rgb(100, 255, 100)
        );
    }

    #[test]
    fn test_output_line_new() {
        let line = OutputLine::new("Test message", OutputLevel::Info);
        assert_eq!(line.text, "Test message");
        assert_eq!(line.level, OutputLevel::Info);
        assert!(line.timestamp.is_none());
    }

    #[test]
    fn test_output_line_with_timestamp() {
        let line = OutputLine::info("Test").with_timestamp("12:00:00");
        assert_eq!(line.timestamp, Some("12:00:00".to_string()));
    }

    #[test]
    fn test_output_line_factories() {
        assert_eq!(OutputLine::info("test").level, OutputLevel::Info);
        assert_eq!(OutputLine::success("test").level, OutputLevel::Success);
        assert_eq!(OutputLine::warning("test").level, OutputLevel::Warning);
        assert_eq!(OutputLine::error("test").level, OutputLevel::Error);
        assert_eq!(OutputLine::system("test").level, OutputLevel::System);
        assert_eq!(OutputLine::command("test").level, OutputLevel::Command);
    }

    #[test]
    fn test_command_def_new() {
        let cmd = CommandDef::new("test", "A test command", "test <arg>");
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.description, "A test command");
        assert_eq!(cmd.usage, "test <arg>");
        assert_eq!(cmd.category, "General");
    }

    #[test]
    fn test_command_def_with_category() {
        let cmd = CommandDef::new("test", "desc", "usage").with_category("Debug");
        assert_eq!(cmd.category, "Debug");
    }

    #[test]
    fn test_command_result_ok() {
        let result = CommandResult::ok(vec![OutputLine::info("test")]);
        assert!(result.success);
        assert_eq!(result.output.len(), 1);
    }

    #[test]
    fn test_command_result_ok_msg() {
        let result = CommandResult::ok_msg("Success!");
        assert!(result.success);
        assert_eq!(result.output[0].level, OutputLevel::Success);
    }

    #[test]
    fn test_command_result_err() {
        let result = CommandResult::err("Failed!");
        assert!(!result.success);
        assert_eq!(result.output[0].level, OutputLevel::Error);
    }

    #[test]
    fn test_command_result_with_lines() {
        let lines = vec![OutputLine::info("Info"), OutputLine::success("Success")];
        let result = CommandResult::with_lines(lines);
        assert!(result.success);

        let error_lines = vec![OutputLine::error("Error")];
        let result = CommandResult::with_lines(error_lines);
        assert!(!result.success);
    }

    #[test]
    fn test_console_action_equality() {
        assert_eq!(
            ConsoleAction::CommandSubmitted("test".to_string()),
            ConsoleAction::CommandSubmitted("test".to_string())
        );
        assert_eq!(ConsoleAction::Opened, ConsoleAction::Opened);
        assert_ne!(ConsoleAction::Opened, ConsoleAction::Closed);
    }

    #[test]
    fn test_console_config_defaults() {
        let config = ConsoleConfig::default();
        assert_eq!(config.toggle_key, "`");
        assert_eq!(config.max_history, DEFAULT_HISTORY_SIZE);
        assert_eq!(config.max_output, DEFAULT_OUTPUT_SIZE);
    }

    #[test]
    fn test_debug_console_new() {
        let console = DebugConsole::new();
        assert!(!console.is_open);
        assert!(console.input.is_empty());
        assert!(!console.commands.is_empty()); // Built-in commands
    }

    #[test]
    fn test_debug_console_register_command() {
        let mut console = DebugConsole::new();
        let initial_count = console.commands.len();

        console.register_command(CommandDef::new("custom", "Custom command", "custom"));
        assert_eq!(console.commands.len(), initial_count + 1);

        // Don't add duplicates
        console.register_command(CommandDef::new("custom", "Duplicate", "custom"));
        assert_eq!(console.commands.len(), initial_count + 1);
    }

    #[test]
    fn test_debug_console_toggle() {
        let mut console = DebugConsole::new();
        assert!(!console.is_open);

        console.toggle();
        assert!(console.is_open);

        console.toggle();
        assert!(!console.is_open);
    }

    #[test]
    fn test_debug_console_open_close() {
        let mut console = DebugConsole::new();

        console.open();
        assert!(console.is_open);
        assert!(console.focus_input);

        console.close();
        assert!(!console.is_open);
    }

    #[test]
    fn test_debug_console_print_methods() {
        let mut console = DebugConsole::new();
        let initial = console.output.len();

        console.print_info("Info message");
        console.print_success("Success message");
        console.print_warning("Warning message");
        console.print_error("Error message");
        console.print_system("System message");

        assert_eq!(console.output.len(), initial + 5);
    }

    #[test]
    fn test_debug_console_clear() {
        let mut console = DebugConsole::new();
        console.print_info("Test");
        assert!(!console.output.is_empty());

        console.clear();
        assert!(console.output.is_empty());
    }

    #[test]
    fn test_debug_console_history() {
        let mut console = DebugConsole::new();

        console.add_to_history("command1".to_string());
        console.add_to_history("command2".to_string());

        assert_eq!(console.history.len(), 2);
        assert_eq!(console.history[0], "command2");
        assert_eq!(console.history[1], "command1");

        // Don't add duplicate of last command
        console.add_to_history("command2".to_string());
        assert_eq!(console.history.len(), 2);
    }

    #[test]
    fn test_debug_console_history_navigation() {
        let mut console = DebugConsole::new();
        console.add_to_history("first".to_string());
        console.add_to_history("second".to_string());

        console.input = "current".to_string();
        console.history_up();
        assert_eq!(console.input, "second");

        console.history_up();
        assert_eq!(console.input, "first");

        console.history_down();
        assert_eq!(console.input, "second");

        console.history_down();
        assert_eq!(console.input, "current");
    }

    #[test]
    fn test_debug_console_suggestions() {
        let mut console = DebugConsole::new();
        console.input = "he".to_string();
        console.update_suggestions();

        assert!(!console.suggestions.is_empty());
        assert!(console.suggestions.contains(&"help".to_string()));
    }

    #[test]
    fn test_debug_console_autocomplete() {
        let mut console = DebugConsole::new();
        console.input = "he".to_string();
        console.update_suggestions();
        console.autocomplete();

        assert_eq!(console.input, "help");
    }

    #[test]
    fn test_debug_console_cmd_help() {
        let console = DebugConsole::new();

        let result = console.cmd_help(&[]);
        assert!(result.success);
        assert!(!result.output.is_empty());

        let result = console.cmd_help(&["help"]);
        assert!(result.success);

        let result = console.cmd_help(&["nonexistent"]);
        assert!(!result.success);
    }

    #[test]
    fn test_debug_console_cmd_history() {
        let mut console = DebugConsole::new();

        let result = console.cmd_history();
        assert!(result.success);
        assert_eq!(result.output.len(), 1); // "No command history."

        console.add_to_history("test".to_string());
        let result = console.cmd_history();
        assert!(result.success);
        assert!(result.output.len() > 1);
    }

    #[test]
    fn test_debug_console_print_result() {
        let mut console = DebugConsole::new();
        let initial = console.output.len();

        let result =
            CommandResult::ok(vec![OutputLine::info("Line 1"), OutputLine::info("Line 2")]);
        console.print_result(result);

        assert_eq!(console.output.len(), initial + 2);
    }

    #[test]
    fn test_debug_console_output_trim() {
        let mut console = DebugConsole::new();
        console.config.max_output = 5;

        for i in 0..10 {
            console.print_info(format!("Message {i}"));
        }

        assert!(console.output.len() <= 5);
    }

    #[test]
    fn test_debug_console_getters() {
        let console = DebugConsole::new();
        assert!(console.get_input().is_empty());
        assert!(!console.get_commands().is_empty());
        assert!(console.get_history().is_empty());
        assert!(!console.get_output().is_empty()); // Has init message
    }
}
