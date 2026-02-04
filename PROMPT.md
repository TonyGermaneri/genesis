# Iteration 13: Infra Agent - Save System Integration

## Objective
Coordinate save/load operations, auto-save, versioning, and cloud prep.

## Tasks

### 1. Save File Manager (save_manager.rs)
- SaveManager: orchestrate all save operations
- Save slot directory structure
- Atomic save operations (temp file + rename)
- Error handling and recovery

### 2. Auto-save System (autosave.rs)
- Configurable auto-save interval
- Trigger on key events (area transition)
- Pause during combat/cutscenes
- Rotating auto-save slots

### 3. Save File Versioning (save_version.rs)
- Save format version number
- Migration functions between versions
- Backward compatibility where possible
- Version mismatch warnings

### 4. Cloud Save Preparation (cloud_storage.rs)
- StorageBackend trait abstraction
- LocalStorage implementation
- Sync status tracking
- Conflict resolution hooks

### 5. Update Engine Integration
- Wire save/load to pause menu
- Integrate with game state
- Handle save during gameplay
