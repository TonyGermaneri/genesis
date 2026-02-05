//! Game automation system for E2E testing and scripted scenarios.
//!
//! This module provides a macro system for automating game interactions:
//! - Character movement
//! - Menu navigation  
//! - UI interactions
//! - Screenshot capture
//! - Parameter changes
//!
//! Macros can be loaded from JSON files or defined programmatically.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// A single automation action to perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationAction {
    /// Wait for a duration before the next action
    Wait {
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Move the player in a direction
    Move {
        /// X direction (-1 = left, 1 = right, 0 = none)
        dx: f32,
        /// Y direction (-1 = up, 1 = down, 0 = none)
        dy: f32,
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Set absolute player position
    SetPosition {
        /// X world coordinate
        x: f32,
        /// Y world coordinate
        y: f32,
    },

    /// Set camera zoom level
    SetZoom {
        /// Zoom level (1.0 = normal, 2.0 = 2x zoom, etc.)
        zoom: f32,
    },

    /// Move camera to position
    SetCameraPosition {
        /// X world coordinate
        x: f32,
        /// Y world coordinate
        y: f32,
    },

    /// Press a key (simulated input)
    PressKey {
        /// Key name (e.g., "escape", "f12", "e", "space")
        key: String,
    },

    /// Start a new game (from main menu)
    StartNewGame,

    /// Open the pause menu (press escape)
    OpenPauseMenu,

    /// Close all menus and resume gameplay
    ResumeGame,

    /// Open world tools panel
    OpenWorldTools,

    /// Set world generation seed
    SetSeed {
        /// New world seed
        seed: u64,
    },

    /// Regenerate the world with current parameters
    RegenerateWorld,

    /// Capture a screenshot
    Screenshot {
        /// Optional filename (defaults to timestamp-based name)
        filename: Option<String>,
        /// Optional prompt for AI analysis
        prompt: Option<String>,
    },

    /// Log a message
    Log {
        /// Message to log
        message: String,
    },

    /// Set a world tools parameter
    SetWorldParam {
        /// Parameter category (e.g., "noise", "biome", "weather")
        category: String,
        /// Parameter name
        name: String,
        /// Parameter value (as string, will be parsed)
        value: String,
    },

    /// Run a sub-macro by name
    RunMacro {
        /// Name of macro to run
        name: String,
    },

    /// Repeat a set of actions
    Repeat {
        /// Number of times to repeat
        count: u32,
        /// Actions to repeat
        actions: Vec<AutomationAction>,
    },

    /// Wait for condition (placeholder for future expansion)
    WaitForCondition {
        /// Condition type
        condition: String,
        /// Timeout in milliseconds
        timeout_ms: u64,
    },

    /// Quit the application
    Quit,
}

/// A named macro containing a sequence of actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationMacro {
    /// Macro name
    pub name: String,
    /// Description of what this macro does
    pub description: Option<String>,
    /// Sequence of actions
    pub actions: Vec<AutomationAction>,
}

/// Result of executing an action.
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Action completed successfully
    Completed,
    /// Action is still in progress
    InProgress,
    /// Action failed with error
    Failed(String),
    /// Request to capture screenshot
    CaptureScreenshot {
        filename: Option<String>,
        prompt: Option<String>,
    },
}

/// State of an in-progress action.
#[derive(Debug, Clone)]
pub struct ActiveAction {
    /// The action being executed
    pub action: AutomationAction,
    /// When the action started
    pub started_at: Instant,
    /// Expected completion time (for timed actions)
    pub ends_at: Option<Instant>,
}

/// The automation system that manages macro execution.
#[derive(Debug)]
pub struct AutomationSystem {
    /// Queue of pending actions
    action_queue: VecDeque<AutomationAction>,
    /// Currently executing action
    current_action: Option<ActiveAction>,
    /// Named macros that can be referenced
    macros: std::collections::HashMap<String, AutomationMacro>,
    /// Whether automation is enabled
    enabled: bool,
    /// Path to macro files directory
    macro_dir: PathBuf,
    /// Movement input override
    movement_override: Option<(f32, f32)>,
    /// Position teleport request
    position_teleport: Option<(f32, f32)>,
    /// Zoom change request
    zoom_request: Option<f32>,
    /// Camera position request
    camera_position_request: Option<(f32, f32)>,
    /// Key press requests
    key_presses: Vec<String>,
    /// Pending action requests (for app to process)
    pending_requests: Vec<AutomationRequest>,
    /// Screenshots captured this session
    screenshots_captured: Vec<PathBuf>,
}

/// Requests that need to be handled by the app.
#[derive(Debug, Clone)]
pub enum AutomationRequest {
    /// Start a new game
    StartNewGame,
    /// Open pause menu
    OpenPauseMenu,
    /// Resume game (close menus)
    ResumeGame,
    /// Open world tools
    OpenWorldTools,
    /// Set seed value
    SetSeed(u64),
    /// Regenerate world
    RegenerateWorld,
    /// Capture screenshot
    CaptureScreenshot {
        filename: Option<String>,
        prompt: Option<String>,
    },
    /// Set world parameter
    SetWorldParam {
        category: String,
        name: String,
        value: String,
    },
    /// Quit the application
    Quit,
}

impl Default for AutomationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AutomationSystem {
    /// Creates a new automation system.
    pub fn new() -> Self {
        Self {
            action_queue: VecDeque::new(),
            current_action: None,
            macros: std::collections::HashMap::new(),
            enabled: false,
            macro_dir: PathBuf::from("macros"),
            movement_override: None,
            position_teleport: None,
            zoom_request: None,
            camera_position_request: None,
            key_presses: Vec::new(),
            pending_requests: Vec::new(),
            screenshots_captured: Vec::new(),
        }
    }

    /// Enables the automation system.
    pub fn enable(&mut self) {
        self.enabled = true;
        info!("Automation system enabled");
    }

    /// Disables the automation system.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.clear();
        info!("Automation system disabled");
    }

    /// Returns whether automation is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Clears all pending actions.
    pub fn clear(&mut self) {
        self.action_queue.clear();
        self.current_action = None;
        self.movement_override = None;
        self.position_teleport = None;
        self.zoom_request = None;
        self.camera_position_request = None;
        self.key_presses.clear();
        self.pending_requests.clear();
    }

    /// Loads a macro from a JSON file.
    pub fn load_macro_file(&mut self, path: &std::path::Path) -> Result<String, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read macro file: {}", e))?;
        
        let macro_def: AutomationMacro = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse macro JSON: {}", e))?;
        
        let name = macro_def.name.clone();
        info!("Loaded macro '{}' with {} actions", name, macro_def.actions.len());
        self.macros.insert(name.clone(), macro_def);
        Ok(name)
    }

    /// Loads all macros from the macro directory.
    pub fn load_macros_from_dir(&mut self) -> Vec<String> {
        let mut loaded = Vec::new();
        
        if !self.macro_dir.exists() {
            debug!("Macro directory does not exist: {:?}", self.macro_dir);
            return loaded;
        }

        if let Ok(entries) = std::fs::read_dir(&self.macro_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    match self.load_macro_file(&path) {
                        Ok(name) => loaded.push(name),
                        Err(e) => warn!("Failed to load {:?}: {}", path, e),
                    }
                }
            }
        }

        loaded
    }

    /// Registers a macro programmatically.
    pub fn register_macro(&mut self, macro_def: AutomationMacro) {
        let name = macro_def.name.clone();
        self.macros.insert(name, macro_def);
    }

    /// Queues a macro for execution by name.
    pub fn run_macro(&mut self, name: &str) -> Result<(), String> {
        let macro_def = self.macros.get(name)
            .ok_or_else(|| format!("Macro '{}' not found", name))?
            .clone();
        
        info!("Running macro '{}': {:?}", name, macro_def.description);
        
        for action in macro_def.actions {
            self.action_queue.push_back(action);
        }
        
        Ok(())
    }

    /// Queues a single action for execution.
    pub fn queue_action(&mut self, action: AutomationAction) {
        self.action_queue.push_back(action);
    }

    /// Queues multiple actions for execution.
    pub fn queue_actions(&mut self, actions: Vec<AutomationAction>) {
        for action in actions {
            self.action_queue.push_back(action);
        }
    }

    /// Updates the automation system. Call this each frame.
    /// Returns any pending requests that need to be handled by the app.
    pub fn update(&mut self, _dt: f32) -> Vec<AutomationRequest> {
        if !self.enabled {
            return Vec::new();
        }

        // Clear per-frame state
        self.movement_override = None;
        self.position_teleport = None;
        self.key_presses.clear();

        // Check if current action is complete
        if let Some(ref active) = self.current_action {
            if let Some(ends_at) = active.ends_at {
                if Instant::now() >= ends_at {
                    debug!("Action completed: {:?}", active.action);
                    self.current_action = None;
                } else {
                    // Action still in progress - apply effects
                    self.apply_active_action_effects(&active.action.clone());
                    // Return any pending requests accumulated so far
                    return std::mem::take(&mut self.pending_requests);
                }
            }
        }

        // Get next action if none is active
        if self.current_action.is_none() {
            if let Some(action) = self.action_queue.pop_front() {
                self.start_action(action);
            }
        }

        // Return requests AFTER starting action (so immediate actions get processed same frame)
        std::mem::take(&mut self.pending_requests)
    }

    /// Starts executing an action.
    fn start_action(&mut self, action: AutomationAction) {
        debug!("Starting action: {:?}", action);
        
        let now = Instant::now();
        let ends_at = match &action {
            AutomationAction::Wait { duration_ms } => {
                Some(now + Duration::from_millis(*duration_ms))
            }
            AutomationAction::Move { duration_ms, .. } => {
                Some(now + Duration::from_millis(*duration_ms))
            }
            _ => None,
        };

        // Handle immediate actions
        match &action {
            AutomationAction::SetPosition { x, y } => {
                self.position_teleport = Some((*x, *y));
            }
            AutomationAction::SetZoom { zoom } => {
                self.zoom_request = Some(*zoom);
            }
            AutomationAction::SetCameraPosition { x, y } => {
                self.camera_position_request = Some((*x, *y));
            }
            AutomationAction::PressKey { key } => {
                self.key_presses.push(key.clone());
            }
            AutomationAction::StartNewGame => {
                self.pending_requests.push(AutomationRequest::StartNewGame);
            }
            AutomationAction::OpenPauseMenu => {
                self.pending_requests.push(AutomationRequest::OpenPauseMenu);
            }
            AutomationAction::ResumeGame => {
                self.pending_requests.push(AutomationRequest::ResumeGame);
            }
            AutomationAction::OpenWorldTools => {
                self.pending_requests.push(AutomationRequest::OpenWorldTools);
            }
            AutomationAction::SetSeed { seed } => {
                self.pending_requests.push(AutomationRequest::SetSeed(*seed));
            }
            AutomationAction::RegenerateWorld => {
                self.pending_requests.push(AutomationRequest::RegenerateWorld);
            }
            AutomationAction::Screenshot { filename, prompt } => {
                self.pending_requests.push(AutomationRequest::CaptureScreenshot {
                    filename: filename.clone(),
                    prompt: prompt.clone(),
                });
            }
            AutomationAction::Log { message } => {
                info!("[AUTOMATION] {}", message);
            }
            AutomationAction::SetWorldParam { category, name, value } => {
                self.pending_requests.push(AutomationRequest::SetWorldParam {
                    category: category.clone(),
                    name: name.clone(),
                    value: value.clone(),
                });
            }
            AutomationAction::RunMacro { name } => {
                if let Err(e) = self.run_macro(name) {
                    warn!("Failed to run macro '{}': {}", name, e);
                }
            }
            AutomationAction::Repeat { count, actions } => {
                // Expand the repeat into individual actions
                for _ in 0..*count {
                    for action in actions.iter().cloned() {
                        self.action_queue.push_front(action);
                    }
                }
            }
            AutomationAction::Move { dx, dy, .. } => {
                self.movement_override = Some((*dx, *dy));
            }
            AutomationAction::Wait { .. } => {
                // Just wait, no immediate effect
            }
            AutomationAction::WaitForCondition { .. } => {
                // TODO: Implement condition checking
                warn!("WaitForCondition not yet implemented");
            }
            AutomationAction::Quit => {
                self.pending_requests.push(AutomationRequest::Quit);
            }
        }

        // Store as current action if it has duration
        if ends_at.is_some() {
            self.current_action = Some(ActiveAction {
                action,
                started_at: now,
                ends_at,
            });
        }
    }

    /// Apply effects of an active (in-progress) action.
    fn apply_active_action_effects(&mut self, action: &AutomationAction) {
        match action {
            AutomationAction::Move { dx, dy, .. } => {
                self.movement_override = Some((*dx, *dy));
            }
            _ => {}
        }
    }

    /// Returns movement override if active.
    pub fn movement_override(&self) -> Option<(f32, f32)> {
        self.movement_override
    }

    /// Returns position teleport request if any.
    pub fn take_position_teleport(&mut self) -> Option<(f32, f32)> {
        self.position_teleport.take()
    }

    /// Returns zoom request if any.
    pub fn take_zoom_request(&mut self) -> Option<f32> {
        self.zoom_request.take()
    }

    /// Returns camera position request if any.
    pub fn take_camera_position_request(&mut self) -> Option<(f32, f32)> {
        self.camera_position_request.take()
    }

    /// Returns key press requests.
    pub fn key_presses(&self) -> &[String] {
        &self.key_presses
    }

    /// Returns whether the automation queue is empty.
    pub fn is_idle(&self) -> bool {
        self.action_queue.is_empty() && self.current_action.is_none()
    }

    /// Returns the number of pending actions.
    pub fn pending_action_count(&self) -> usize {
        self.action_queue.len() + if self.current_action.is_some() { 1 } else { 0 }
    }

    /// Records a captured screenshot path.
    pub fn record_screenshot(&mut self, path: PathBuf) {
        self.screenshots_captured.push(path);
    }

    /// Returns screenshots captured during this session.
    pub fn screenshots_captured(&self) -> &[PathBuf] {
        &self.screenshots_captured
    }

    /// Creates a simple test macro for biome testing.
    pub fn create_biome_test_macro() -> AutomationMacro {
        AutomationMacro {
            name: "biome_test".to_string(),
            description: Some("Test biome generation by moving around and capturing screenshots".to_string()),
            actions: vec![
                AutomationAction::Log { message: "Starting biome test macro".to_string() },
                AutomationAction::StartNewGame,
                AutomationAction::Wait { duration_ms: 2000 },
                AutomationAction::SetZoom { zoom: 2.0 },
                AutomationAction::Wait { duration_ms: 500 },
                AutomationAction::Screenshot { 
                    filename: Some("biome_test_start.png".to_string()),
                    prompt: Some("Describe the initial terrain biomes visible.".to_string()),
                },
                AutomationAction::Move { dx: 1.0, dy: 0.0, duration_ms: 3000 },
                AutomationAction::Screenshot {
                    filename: Some("biome_test_east.png".to_string()),
                    prompt: Some("Describe the terrain after moving east.".to_string()),
                },
                AutomationAction::Move { dx: 0.0, dy: 1.0, duration_ms: 3000 },
                AutomationAction::Screenshot {
                    filename: Some("biome_test_south.png".to_string()),
                    prompt: Some("Describe the terrain after moving south.".to_string()),
                },
                AutomationAction::Move { dx: -1.0, dy: 0.0, duration_ms: 3000 },
                AutomationAction::Move { dx: 0.0, dy: -1.0, duration_ms: 3000 },
                AutomationAction::Screenshot {
                    filename: Some("biome_test_return.png".to_string()),
                    prompt: Some("Final terrain view after returning to start.".to_string()),
                },
                AutomationAction::Log { message: "Biome test macro complete".to_string() },
            ],
        }
    }

    /// Creates a world regeneration test macro.
    pub fn create_regen_test_macro() -> AutomationMacro {
        AutomationMacro {
            name: "regen_test".to_string(),
            description: Some("Test world regeneration with different seeds".to_string()),
            actions: vec![
                AutomationAction::Log { message: "Starting regeneration test".to_string() },
                AutomationAction::StartNewGame,
                AutomationAction::Wait { duration_ms: 2000 },
                AutomationAction::Screenshot {
                    filename: Some("regen_seed_default.png".to_string()),
                    prompt: Some("Describe the terrain with default seed.".to_string()),
                },
                AutomationAction::OpenPauseMenu,
                AutomationAction::Wait { duration_ms: 500 },
                AutomationAction::OpenWorldTools,
                AutomationAction::Wait { duration_ms: 500 },
                AutomationAction::SetSeed { seed: 12345 },
                AutomationAction::RegenerateWorld,
                AutomationAction::Wait { duration_ms: 1000 },
                AutomationAction::ResumeGame,
                AutomationAction::Wait { duration_ms: 500 },
                AutomationAction::Screenshot {
                    filename: Some("regen_seed_12345.png".to_string()),
                    prompt: Some("Describe the terrain with seed 12345.".to_string()),
                },
                AutomationAction::OpenPauseMenu,
                AutomationAction::Wait { duration_ms: 500 },
                AutomationAction::OpenWorldTools,
                AutomationAction::Wait { duration_ms: 500 },
                AutomationAction::SetSeed { seed: 99999 },
                AutomationAction::RegenerateWorld,
                AutomationAction::Wait { duration_ms: 1000 },
                AutomationAction::ResumeGame,
                AutomationAction::Wait { duration_ms: 500 },
                AutomationAction::Screenshot {
                    filename: Some("regen_seed_99999.png".to_string()),
                    prompt: Some("Describe the terrain with seed 99999.".to_string()),
                },
                AutomationAction::Log { message: "Regeneration test complete".to_string() },
            ],
        }
    }
}

/// Parse a macro from command line arguments.
/// Format: --macro "action1;action2;action3"
pub fn parse_cli_macro(args: &str) -> Result<Vec<AutomationAction>, String> {
    let mut actions = Vec::new();
    
    for part in args.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        
        let action = parse_action_string(part)?;
        actions.push(action);
    }
    
    Ok(actions)
}

/// Parse a single action from a string.
fn parse_action_string(s: &str) -> Result<AutomationAction, String> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty action string".to_string());
    }
    
    match parts[0].to_lowercase().as_str() {
        "wait" => {
            let ms = parts.get(1)
                .ok_or("wait requires duration_ms")?
                .parse::<u64>()
                .map_err(|e| format!("Invalid duration: {}", e))?;
            Ok(AutomationAction::Wait { duration_ms: ms })
        }
        "move" => {
            let dx = parts.get(1)
                .ok_or("move requires dx")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid dx: {}", e))?;
            let dy = parts.get(2)
                .ok_or("move requires dy")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid dy: {}", e))?;
            let ms = parts.get(3)
                .ok_or("move requires duration_ms")?
                .parse::<u64>()
                .map_err(|e| format!("Invalid duration: {}", e))?;
            Ok(AutomationAction::Move { dx, dy, duration_ms: ms })
        }
        "setpos" | "teleport" => {
            let x = parts.get(1)
                .ok_or("setpos requires x")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid x: {}", e))?;
            let y = parts.get(2)
                .ok_or("setpos requires y")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid y: {}", e))?;
            Ok(AutomationAction::SetPosition { x, y })
        }
        "zoom" => {
            let z = parts.get(1)
                .ok_or("zoom requires level")?
                .parse::<f32>()
                .map_err(|e| format!("Invalid zoom: {}", e))?;
            Ok(AutomationAction::SetZoom { zoom: z })
        }
        "screenshot" | "capture" => {
            let filename = parts.get(1).map(|s| s.to_string());
            Ok(AutomationAction::Screenshot { filename, prompt: None })
        }
        "newgame" | "start" => Ok(AutomationAction::StartNewGame),
        "pause" => Ok(AutomationAction::OpenPauseMenu),
        "resume" => Ok(AutomationAction::ResumeGame),
        "worldtools" => Ok(AutomationAction::OpenWorldTools),
        "seed" => {
            let seed = parts.get(1)
                .ok_or("seed requires value")?
                .parse::<u64>()
                .map_err(|e| format!("Invalid seed: {}", e))?;
            Ok(AutomationAction::SetSeed { seed })
        }
        "regen" | "regenerate" => Ok(AutomationAction::RegenerateWorld),
        "log" => {
            let msg = parts[1..].join(" ");
            Ok(AutomationAction::Log { message: msg })
        }
        "quit" | "exit" => Ok(AutomationAction::Quit),
        _ => Err(format!("Unknown action: {}", parts[0])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_action_string() {
        let action = parse_action_string("wait 1000").unwrap();
        assert!(matches!(action, AutomationAction::Wait { duration_ms: 1000 }));

        let action = parse_action_string("move 1 0 2000").unwrap();
        assert!(matches!(action, AutomationAction::Move { dx: _, dy: _, duration_ms: 2000 }));

        let action = parse_action_string("screenshot test.png").unwrap();
        assert!(matches!(action, AutomationAction::Screenshot { filename: Some(_), .. }));
    }

    #[test]
    fn test_parse_cli_macro() {
        let actions = parse_cli_macro("wait 500; screenshot; move 1 0 1000").unwrap();
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn test_automation_system() {
        let mut system = AutomationSystem::new();
        system.enable();
        
        system.queue_action(AutomationAction::Log { message: "test".to_string() });
        assert_eq!(system.pending_action_count(), 1);
        
        let requests = system.update(0.016);
        assert!(system.is_idle());
    }
}
