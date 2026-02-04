//! Game progress tracking.
//!
//! This module provides comprehensive game progress saving:
//! - Discovered regions and chunks
//! - Achievements and milestones
//! - Statistics (kills, crafts, distances, etc.)
//! - Playtime tracking

use genesis_common::{ChunkCoord, EntityId, ItemTypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// G-56: Map Discovery Save
// ============================================================================

/// Region discovery state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionDiscoverySave {
    /// Region ID.
    pub region_id: String,
    /// Time discovered.
    pub discovered_at: f64,
    /// Percentage explored (0.0-1.0).
    pub explored: f32,
    /// Points of interest found.
    pub pois_found: HashSet<String>,
    /// Total POIs in region.
    pub total_pois: u32,
    /// Whether region is fully explored.
    pub complete: bool,
}

impl RegionDiscoverySave {
    /// Create new region discovery.
    #[must_use]
    pub fn new(region_id: impl Into<String>, discovered_at: f64) -> Self {
        Self {
            region_id: region_id.into(),
            discovered_at,
            explored: 0.0,
            pois_found: HashSet::new(),
            total_pois: 0,
            complete: false,
        }
    }

    /// Set total POIs.
    #[must_use]
    pub fn with_total_pois(mut self, count: u32) -> Self {
        self.total_pois = count;
        self
    }

    /// Mark POI as found.
    pub fn find_poi(&mut self, poi_id: impl Into<String>) {
        self.pois_found.insert(poi_id.into());
        self.update_exploration();
    }

    /// Update exploration percentage.
    fn update_exploration(&mut self) {
        if self.total_pois > 0 {
            self.explored = self.pois_found.len() as f32 / self.total_pois as f32;
            self.complete = self.explored >= 1.0;
        }
    }

    /// Get exploration percentage.
    #[must_use]
    pub fn exploration_percent(&self) -> f32 {
        self.explored * 100.0
    }
}

/// Chunk discovery data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkDiscoverySave {
    /// Chunk X coordinate.
    pub x: i32,
    /// Chunk Y coordinate.
    pub y: i32,
    /// Time discovered.
    pub discovered_at: f64,
    /// Whether fully revealed.
    pub revealed: bool,
}

impl ChunkDiscoverySave {
    /// Create new chunk discovery.
    #[must_use]
    pub fn new(coord: ChunkCoord, discovered_at: f64) -> Self {
        Self {
            x: coord.x,
            y: coord.y,
            discovered_at,
            revealed: false,
        }
    }

    /// Get chunk coordinate.
    #[must_use]
    pub fn coord(&self) -> ChunkCoord {
        ChunkCoord::new(self.x, self.y)
    }

    /// Mark as fully revealed.
    pub fn reveal(&mut self) {
        self.revealed = true;
    }
}

/// Map discovery save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDiscoverySave {
    /// Discovered regions.
    pub regions: HashMap<String, RegionDiscoverySave>,
    /// Discovered chunks.
    pub chunks: HashSet<(i32, i32)>,
    /// Revealed chunks (fog of war cleared).
    pub revealed_chunks: HashSet<(i32, i32)>,
    /// Discovered fast travel points.
    pub fast_travel_points: HashSet<String>,
    /// Map markers placed by player.
    pub markers: Vec<MapMarkerSave>,
}

/// Player-placed map marker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapMarkerSave {
    /// Marker ID.
    pub id: String,
    /// Position (x, y).
    pub position: (f32, f32),
    /// Marker label.
    pub label: String,
    /// Marker icon type.
    pub icon: String,
    /// Marker color.
    pub color: u32,
}

impl MapMarkerSave {
    /// Create new marker.
    #[must_use]
    pub fn new(id: impl Into<String>, x: f32, y: f32, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            position: (x, y),
            label: label.into(),
            icon: "default".to_string(),
            color: 0x00FF_FFFF,
        }
    }

    /// Set icon.
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Set color.
    #[must_use]
    pub fn with_color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }
}

impl MapDiscoverySave {
    /// Create new map discovery save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Discover a region.
    pub fn discover_region(&mut self, region: RegionDiscoverySave) {
        self.regions.insert(region.region_id.clone(), region);
    }

    /// Get region discovery.
    #[must_use]
    pub fn get_region(&self, region_id: &str) -> Option<&RegionDiscoverySave> {
        self.regions.get(region_id)
    }

    /// Get mutable region discovery.
    pub fn get_region_mut(&mut self, region_id: &str) -> Option<&mut RegionDiscoverySave> {
        self.regions.get_mut(region_id)
    }

    /// Discover a chunk.
    pub fn discover_chunk(&mut self, coord: ChunkCoord) {
        self.chunks.insert((coord.x, coord.y));
    }

    /// Reveal a chunk.
    pub fn reveal_chunk(&mut self, coord: ChunkCoord) {
        self.chunks.insert((coord.x, coord.y));
        self.revealed_chunks.insert((coord.x, coord.y));
    }

    /// Check if chunk is discovered.
    #[must_use]
    pub fn is_chunk_discovered(&self, coord: ChunkCoord) -> bool {
        self.chunks.contains(&(coord.x, coord.y))
    }

    /// Check if chunk is revealed.
    #[must_use]
    pub fn is_chunk_revealed(&self, coord: ChunkCoord) -> bool {
        self.revealed_chunks.contains(&(coord.x, coord.y))
    }

    /// Discover fast travel point.
    pub fn discover_fast_travel(&mut self, point_id: impl Into<String>) {
        self.fast_travel_points.insert(point_id.into());
    }

    /// Add map marker.
    pub fn add_marker(&mut self, marker: MapMarkerSave) {
        self.markers.push(marker);
    }

    /// Remove marker by ID.
    pub fn remove_marker(&mut self, marker_id: &str) {
        self.markers.retain(|m| m.id != marker_id);
    }

    /// Count discovered regions.
    #[must_use]
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Count discovered chunks.
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get overall exploration percentage.
    #[must_use]
    pub fn overall_exploration(&self) -> f32 {
        if self.regions.is_empty() {
            return 0.0;
        }

        let total: f32 = self.regions.values().map(|r| r.explored).sum();
        total / self.regions.len() as f32 * 100.0
    }
}

// ============================================================================
// G-56: Achievement Save
// ============================================================================

/// Achievement tier/rarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementTier {
    /// Common achievement.
    Bronze,
    /// Uncommon achievement.
    Silver,
    /// Rare achievement.
    Gold,
    /// Very rare achievement.
    Platinum,
    /// Secret achievement.
    Secret,
}

impl Default for AchievementTier {
    fn default() -> Self {
        Self::Bronze
    }
}

impl AchievementTier {
    /// Get point value.
    #[must_use]
    pub fn points(self) -> u32 {
        match self {
            Self::Bronze => 10,
            Self::Silver => 25,
            Self::Gold => 50,
            Self::Platinum => 100,
            Self::Secret => 75,
        }
    }
}

/// Achievement progress data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AchievementProgressSave {
    /// Achievement ID.
    pub achievement_id: String,
    /// Current progress.
    pub progress: u32,
    /// Required for completion.
    pub required: u32,
    /// Whether unlocked.
    pub unlocked: bool,
    /// Time unlocked.
    pub unlocked_at: Option<f64>,
    /// Achievement tier.
    pub tier: AchievementTier,
    /// Whether hidden until unlocked.
    pub hidden: bool,
}

impl AchievementProgressSave {
    /// Create new achievement progress.
    #[must_use]
    pub fn new(achievement_id: impl Into<String>, required: u32) -> Self {
        Self {
            achievement_id: achievement_id.into(),
            progress: 0,
            required,
            unlocked: false,
            unlocked_at: None,
            tier: AchievementTier::Bronze,
            hidden: false,
        }
    }

    /// Set tier.
    #[must_use]
    pub fn with_tier(mut self, tier: AchievementTier) -> Self {
        self.tier = tier;
        self
    }

    /// Set hidden.
    #[must_use]
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Add progress.
    pub fn add_progress(&mut self, amount: u32, current_time: f64) -> bool {
        if self.unlocked {
            return false;
        }

        self.progress = self.progress.saturating_add(amount).min(self.required);

        if self.progress >= self.required {
            self.unlocked = true;
            self.unlocked_at = Some(current_time);
            return true;
        }

        false
    }

    /// Get progress percentage.
    #[must_use]
    pub fn progress_percent(&self) -> f32 {
        if self.required == 0 {
            return if self.unlocked { 100.0 } else { 0.0 };
        }
        self.progress as f32 / self.required as f32 * 100.0
    }
}

/// Achievements save data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AchievementsSave {
    /// All achievement progress.
    pub achievements: HashMap<String, AchievementProgressSave>,
    /// Total achievement points.
    pub total_points: u32,
    /// Achievement showcase (displayed achievements).
    pub showcase: Vec<String>,
}

impl AchievementsSave {
    /// Create new achievements save.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add achievement to track.
    pub fn add_achievement(&mut self, achievement: AchievementProgressSave) {
        self.achievements
            .insert(achievement.achievement_id.clone(), achievement);
    }

    /// Get achievement progress.
    #[must_use]
    pub fn get(&self, achievement_id: &str) -> Option<&AchievementProgressSave> {
        self.achievements.get(achievement_id)
    }

    /// Update achievement progress.
    pub fn update_progress(
        &mut self,
        achievement_id: &str,
        amount: u32,
        current_time: f64,
    ) -> bool {
        if let Some(achievement) = self.achievements.get_mut(achievement_id) {
            let just_unlocked = achievement.add_progress(amount, current_time);
            if just_unlocked {
                self.total_points += achievement.tier.points();
            }
            just_unlocked
        } else {
            false
        }
    }

    /// Count unlocked achievements.
    #[must_use]
    pub fn unlocked_count(&self) -> usize {
        self.achievements.values().filter(|a| a.unlocked).count()
    }

    /// Count total achievements.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.achievements.len()
    }

    /// Get completion percentage.
    #[must_use]
    pub fn completion_percent(&self) -> f32 {
        if self.achievements.is_empty() {
            return 0.0;
        }
        self.unlocked_count() as f32 / self.achievements.len() as f32 * 100.0
    }

    /// Get recently unlocked achievements.
    #[must_use]
    pub fn recently_unlocked(&self, since: f64) -> Vec<&AchievementProgressSave> {
        self.achievements
            .values()
            .filter(|a| a.unlocked_at.is_some_and(|t| t >= since))
            .collect()
    }

    /// Add to showcase.
    pub fn add_to_showcase(&mut self, achievement_id: impl Into<String>) {
        let id = achievement_id.into();
        if !self.showcase.contains(&id) && self.showcase.len() < 5 {
            self.showcase.push(id);
        }
    }

    /// Remove from showcase.
    pub fn remove_from_showcase(&mut self, achievement_id: &str) {
        self.showcase.retain(|id| id != achievement_id);
    }
}

// ============================================================================
// G-56: Statistics Save
// ============================================================================

/// Combat statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CombatStatsSave {
    /// Total enemies killed.
    pub enemies_killed: u64,
    /// Kills by enemy type.
    pub kills_by_type: HashMap<u32, u64>,
    /// Total damage dealt.
    pub damage_dealt: f64,
    /// Total damage received.
    pub damage_received: f64,
    /// Total deaths.
    pub deaths: u32,
    /// Longest kill streak.
    pub best_kill_streak: u32,
    /// Current kill streak.
    pub current_kill_streak: u32,
    /// Critical hits landed.
    pub critical_hits: u64,
    /// Total attacks made.
    pub attacks_made: u64,
    /// Total attacks hit.
    pub attacks_hit: u64,
    /// Bosses defeated.
    pub bosses_defeated: u32,
}

impl CombatStatsSave {
    /// Create new combat stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a kill.
    pub fn record_kill(&mut self, enemy_type: u32) {
        self.enemies_killed += 1;
        *self.kills_by_type.entry(enemy_type).or_insert(0) += 1;
        self.current_kill_streak += 1;
        self.best_kill_streak = self.best_kill_streak.max(self.current_kill_streak);
    }

    /// Record damage dealt.
    pub fn record_damage_dealt(&mut self, amount: f64, is_crit: bool) {
        self.damage_dealt += amount;
        self.attacks_made += 1;
        self.attacks_hit += 1;
        if is_crit {
            self.critical_hits += 1;
        }
    }

    /// Record damage received.
    pub fn record_damage_received(&mut self, amount: f64) {
        self.damage_received += amount;
    }

    /// Record death.
    pub fn record_death(&mut self) {
        self.deaths += 1;
        self.current_kill_streak = 0;
    }

    /// Record boss defeat.
    pub fn record_boss_defeat(&mut self) {
        self.bosses_defeated += 1;
    }

    /// Get accuracy percentage.
    #[must_use]
    pub fn accuracy(&self) -> f32 {
        if self.attacks_made == 0 {
            return 0.0;
        }
        self.attacks_hit as f32 / self.attacks_made as f32 * 100.0
    }

    /// Get critical hit rate.
    #[must_use]
    pub fn crit_rate(&self) -> f32 {
        if self.attacks_hit == 0 {
            return 0.0;
        }
        self.critical_hits as f32 / self.attacks_hit as f32 * 100.0
    }
}

/// Crafting statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CraftingStatsSave {
    /// Total items crafted.
    pub items_crafted: u64,
    /// Crafts by item type.
    pub crafts_by_type: HashMap<u32, u64>,
    /// Total resources gathered.
    pub resources_gathered: u64,
    /// Resources by type.
    pub resources_by_type: HashMap<u32, u64>,
    /// Buildings constructed.
    pub buildings_constructed: u32,
    /// Upgrades performed.
    pub upgrades_performed: u32,
    /// Failed crafts.
    pub craft_failures: u32,
}

impl CraftingStatsSave {
    /// Create new crafting stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record item craft.
    pub fn record_craft(&mut self, item_type: ItemTypeId, quantity: u32) {
        self.items_crafted += quantity as u64;
        *self.crafts_by_type.entry(item_type.raw()).or_insert(0) += quantity as u64;
    }

    /// Record resource gather.
    pub fn record_gather(&mut self, item_type: ItemTypeId, quantity: u32) {
        self.resources_gathered += quantity as u64;
        *self.resources_by_type.entry(item_type.raw()).or_insert(0) += quantity as u64;
    }

    /// Record building construction.
    pub fn record_building(&mut self) {
        self.buildings_constructed += 1;
    }

    /// Record craft failure.
    pub fn record_failure(&mut self) {
        self.craft_failures += 1;
    }
}

/// Exploration statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExplorationStatsSave {
    /// Total distance walked (units).
    pub distance_walked: f64,
    /// Total distance sprinted.
    pub distance_sprinted: f64,
    /// Total distance swam.
    pub distance_swam: f64,
    /// Total distance climbed.
    pub distance_climbed: f64,
    /// Total distance fallen.
    pub distance_fallen: f64,
    /// Highest point reached.
    pub highest_altitude: f32,
    /// Deepest point reached.
    pub lowest_altitude: f32,
    /// Total jumps.
    pub jumps: u64,
    /// Secrets discovered.
    pub secrets_found: u32,
    /// Treasures found.
    pub treasures_found: u32,
}

impl ExplorationStatsSave {
    /// Create new exploration stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record distance.
    pub fn record_distance(&mut self, distance: f64, sprinting: bool, swimming: bool) {
        if swimming {
            self.distance_swam += distance;
        } else if sprinting {
            self.distance_sprinted += distance;
        } else {
            self.distance_walked += distance;
        }
    }

    /// Record altitude.
    pub fn record_altitude(&mut self, altitude: f32) {
        self.highest_altitude = self.highest_altitude.max(altitude);
        self.lowest_altitude = self.lowest_altitude.min(altitude);
    }

    /// Record jump.
    pub fn record_jump(&mut self) {
        self.jumps += 1;
    }

    /// Get total distance.
    #[must_use]
    pub fn total_distance(&self) -> f64 {
        self.distance_walked + self.distance_sprinted + self.distance_swam + self.distance_climbed
    }
}

/// Social statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SocialStatsSave {
    /// NPCs talked to.
    pub npcs_talked_to: u32,
    /// Unique NPCs met.
    pub unique_npcs: HashSet<u64>,
    /// Quests completed.
    pub quests_completed: u32,
    /// Quests failed.
    pub quests_failed: u32,
    /// Items traded.
    pub items_traded: u64,
    /// Gold spent.
    pub gold_spent: u64,
    /// Gold earned.
    pub gold_earned: u64,
    /// Factions joined.
    pub factions_joined: u32,
    /// Reputation gained.
    pub reputation_gained: i64,
    /// Reputation lost.
    pub reputation_lost: i64,
}

impl SocialStatsSave {
    /// Create new social stats.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record NPC interaction.
    pub fn record_npc_interaction(&mut self, npc_id: EntityId) {
        self.npcs_talked_to += 1;
        self.unique_npcs.insert(npc_id.raw());
    }

    /// Record quest completion.
    pub fn record_quest_complete(&mut self) {
        self.quests_completed += 1;
    }

    /// Record quest failure.
    pub fn record_quest_fail(&mut self) {
        self.quests_failed += 1;
    }

    /// Record trade.
    pub fn record_trade(&mut self, items: u32, gold_spent: u32, gold_earned: u32) {
        self.items_traded += items as u64;
        self.gold_spent += gold_spent as u64;
        self.gold_earned += gold_earned as u64;
    }

    /// Record reputation change.
    pub fn record_reputation_change(&mut self, delta: i32) {
        if delta > 0 {
            self.reputation_gained += delta as i64;
        } else {
            self.reputation_lost += (-delta) as i64;
        }
    }
}

/// Complete statistics save.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StatisticsSave {
    /// Combat stats.
    pub combat: CombatStatsSave,
    /// Crafting stats.
    pub crafting: CraftingStatsSave,
    /// Exploration stats.
    pub exploration: ExplorationStatsSave,
    /// Social stats.
    pub social: SocialStatsSave,
    /// Custom numeric stats.
    pub custom_numeric: HashMap<String, f64>,
    /// Custom counter stats.
    pub custom_counters: HashMap<String, u64>,
}

impl StatisticsSave {
    /// Create new statistics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom stat.
    pub fn set_stat(&mut self, key: impl Into<String>, value: f64) {
        self.custom_numeric.insert(key.into(), value);
    }

    /// Get custom stat.
    #[must_use]
    pub fn get_stat(&self, key: &str) -> f64 {
        self.custom_numeric.get(key).copied().unwrap_or(0.0)
    }

    /// Increment counter.
    pub fn increment_counter(&mut self, key: impl Into<String>, amount: u64) {
        *self.custom_counters.entry(key.into()).or_insert(0) += amount;
    }

    /// Get counter.
    #[must_use]
    pub fn get_counter(&self, key: &str) -> u64 {
        self.custom_counters.get(key).copied().unwrap_or(0)
    }
}

// ============================================================================
// G-56: Playtime Save
// ============================================================================

/// Playtime tracking save.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlaytimeSave {
    /// Total playtime in seconds.
    pub total_seconds: f64,
    /// Current session start time (real time timestamp).
    pub session_start: Option<f64>,
    /// Current session duration.
    pub session_seconds: f64,
    /// Number of sessions.
    pub session_count: u32,
    /// Longest session duration.
    pub longest_session: f64,
    /// Average session duration.
    pub average_session: f64,
    /// Playtime by day of week.
    pub by_day_of_week: [f64; 7],
    /// Last played timestamp.
    pub last_played: f64,
}

impl PlaytimeSave {
    /// Create new playtime tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new session.
    pub fn start_session(&mut self, timestamp: f64) {
        self.session_start = Some(timestamp);
        self.session_seconds = 0.0;
        self.session_count += 1;
    }

    /// Update session time.
    pub fn update(&mut self, delta: f64, day_of_week: usize) {
        self.session_seconds += delta;
        self.total_seconds += delta;
        self.last_played = self.session_start.unwrap_or(0.0) + self.session_seconds;

        if day_of_week < 7 {
            self.by_day_of_week[day_of_week] += delta;
        }
    }

    /// End session.
    pub fn end_session(&mut self) {
        self.longest_session = self.longest_session.max(self.session_seconds);

        if self.session_count > 0 {
            self.average_session = self.total_seconds / self.session_count as f64;
        }

        self.session_start = None;
    }

    /// Get formatted total playtime.
    #[must_use]
    pub fn formatted_total(&self) -> String {
        Self::format_duration(self.total_seconds)
    }

    /// Get formatted session time.
    #[must_use]
    pub fn formatted_session(&self) -> String {
        Self::format_duration(self.session_seconds)
    }

    /// Format duration as string.
    #[must_use]
    pub fn format_duration(seconds: f64) -> String {
        let total = seconds as u64;
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        let secs = total % 60;

        if hours > 0 {
            format!("{hours}h {minutes}m {secs}s")
        } else if minutes > 0 {
            format!("{minutes}m {secs}s")
        } else {
            format!("{secs}s")
        }
    }
}

// ============================================================================
// G-56: Complete Progress Save
// ============================================================================

/// Complete game progress save data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameProgressSave {
    /// Save format version.
    pub version: u32,
    /// Map discovery data.
    pub map: MapDiscoverySave,
    /// Achievements.
    pub achievements: AchievementsSave,
    /// Statistics.
    pub statistics: StatisticsSave,
    /// Playtime.
    pub playtime: PlaytimeSave,
    /// Tutorial progress flags.
    pub tutorial_flags: HashMap<String, bool>,
    /// Game mode.
    pub game_mode: String,
    /// Difficulty level.
    pub difficulty: String,
    /// New game plus count.
    pub new_game_plus: u32,
    /// Unlocked game modes.
    pub unlocked_modes: HashSet<String>,
    /// Custom progress flags.
    pub flags: HashMap<String, String>,
}

impl Default for GameProgressSave {
    fn default() -> Self {
        Self {
            version: 1,
            map: MapDiscoverySave::new(),
            achievements: AchievementsSave::new(),
            statistics: StatisticsSave::new(),
            playtime: PlaytimeSave::new(),
            tutorial_flags: HashMap::new(),
            game_mode: "normal".to_string(),
            difficulty: "normal".to_string(),
            new_game_plus: 0,
            unlocked_modes: HashSet::new(),
            flags: HashMap::new(),
        }
    }
}

impl GameProgressSave {
    /// Create new game progress.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set game mode.
    #[must_use]
    pub fn with_game_mode(mut self, mode: impl Into<String>) -> Self {
        self.game_mode = mode.into();
        self
    }

    /// Set difficulty.
    #[must_use]
    pub fn with_difficulty(mut self, difficulty: impl Into<String>) -> Self {
        self.difficulty = difficulty.into();
        self
    }

    /// Set tutorial flag.
    pub fn set_tutorial_flag(&mut self, flag: impl Into<String>, value: bool) {
        self.tutorial_flags.insert(flag.into(), value);
    }

    /// Get tutorial flag.
    #[must_use]
    pub fn get_tutorial_flag(&self, flag: &str) -> bool {
        self.tutorial_flags.get(flag).copied().unwrap_or(false)
    }

    /// Set progress flag.
    pub fn set_flag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.flags.insert(key.into(), value.into());
    }

    /// Get progress flag.
    #[must_use]
    pub fn get_flag(&self, key: &str) -> Option<&str> {
        self.flags.get(key).map(String::as_str)
    }

    /// Unlock game mode.
    pub fn unlock_mode(&mut self, mode: impl Into<String>) {
        self.unlocked_modes.insert(mode.into());
    }

    /// Start new game plus.
    pub fn start_new_game_plus(&mut self) {
        self.new_game_plus += 1;
        // Keep achievements, stats, and playtime
        // Reset map discovery
        self.map = MapDiscoverySave::new();
        self.tutorial_flags.clear();
    }

    /// Get overall completion percentage.
    #[must_use]
    pub fn completion_percent(&self) -> f32 {
        let map_weight = 0.4;
        let achievement_weight = 0.4;
        let quest_weight = 0.2;

        let map_progress = self.map.overall_exploration() / 100.0;
        let achievement_progress = self.achievements.completion_percent() / 100.0;
        let quest_progress = if self.statistics.social.quests_completed > 0 {
            1.0 // Simplified - just check if any quests done
        } else {
            0.0
        };

        (map_progress * map_weight
            + achievement_progress * achievement_weight
            + quest_progress * quest_weight)
            * 100.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_discovery() {
        let mut region = RegionDiscoverySave::new("forest", 0.0).with_total_pois(5);
        region.find_poi("cave_1");
        region.find_poi("shrine_1");

        assert_eq!(region.pois_found.len(), 2);
        assert_eq!(region.exploration_percent(), 40.0);
    }

    #[test]
    fn test_chunk_discovery() {
        let chunk = ChunkDiscoverySave::new(ChunkCoord::new(5, 10), 100.0);
        assert_eq!(chunk.coord(), ChunkCoord::new(5, 10));
        assert!(!chunk.revealed);
    }

    #[test]
    fn test_map_discovery() {
        let mut map = MapDiscoverySave::new();
        map.discover_chunk(ChunkCoord::new(0, 0));
        map.reveal_chunk(ChunkCoord::new(1, 1));

        assert!(map.is_chunk_discovered(ChunkCoord::new(0, 0)));
        assert!(map.is_chunk_revealed(ChunkCoord::new(1, 1)));
        assert_eq!(map.chunk_count(), 2);
    }

    #[test]
    fn test_map_marker() {
        let marker = MapMarkerSave::new("m1", 100.0, 200.0, "Camp")
            .with_icon("tent")
            .with_color(0xFF0000);

        assert_eq!(marker.label, "Camp");
        assert_eq!(marker.icon, "tent");
    }

    #[test]
    fn test_achievement_progress() {
        let mut achievement =
            AchievementProgressSave::new("first_kill", 10).with_tier(AchievementTier::Silver);

        assert!(!achievement.add_progress(5, 100.0));
        assert_eq!(achievement.progress_percent(), 50.0);

        assert!(achievement.add_progress(5, 200.0));
        assert!(achievement.unlocked);
        assert_eq!(achievement.unlocked_at, Some(200.0));
    }

    #[test]
    fn test_achievements_save() {
        let mut achievements = AchievementsSave::new();
        achievements.add_achievement(
            AchievementProgressSave::new("kill_10", 10).with_tier(AchievementTier::Bronze),
        );

        assert!(achievements.update_progress("kill_10", 10, 0.0));
        assert_eq!(achievements.unlocked_count(), 1);
        assert_eq!(achievements.total_points, 10);
    }

    #[test]
    fn test_combat_stats() {
        let mut stats = CombatStatsSave::new();
        stats.record_kill(1);
        stats.record_kill(1);
        stats.record_kill(2);
        stats.record_damage_dealt(100.0, true);
        stats.record_death();

        assert_eq!(stats.enemies_killed, 3);
        assert_eq!(stats.kills_by_type.get(&1), Some(&2));
        assert_eq!(stats.current_kill_streak, 0);
        assert_eq!(stats.best_kill_streak, 3);
    }

    #[test]
    fn test_crafting_stats() {
        let mut stats = CraftingStatsSave::new();
        stats.record_craft(ItemTypeId::new(1), 5);
        stats.record_gather(ItemTypeId::new(100), 50);

        assert_eq!(stats.items_crafted, 5);
        assert_eq!(stats.resources_gathered, 50);
    }

    #[test]
    fn test_exploration_stats() {
        let mut stats = ExplorationStatsSave::new();
        stats.record_distance(100.0, false, false);
        stats.record_distance(50.0, true, false);
        stats.record_distance(25.0, false, true);

        assert_eq!(stats.distance_walked, 100.0);
        assert_eq!(stats.distance_sprinted, 50.0);
        assert_eq!(stats.distance_swam, 25.0);
        assert_eq!(stats.total_distance(), 175.0);
    }

    #[test]
    fn test_social_stats() {
        let mut stats = SocialStatsSave::new();
        stats.record_npc_interaction(EntityId::from_raw(1));
        stats.record_npc_interaction(EntityId::from_raw(1));
        stats.record_npc_interaction(EntityId::from_raw(2));

        assert_eq!(stats.npcs_talked_to, 3);
        assert_eq!(stats.unique_npcs.len(), 2);
    }

    #[test]
    fn test_playtime() {
        let mut playtime = PlaytimeSave::new();
        playtime.start_session(0.0);
        playtime.update(3600.0, 0); // 1 hour
        playtime.end_session();

        assert_eq!(playtime.total_seconds, 3600.0);
        assert_eq!(playtime.formatted_total(), "1h 0m 0s");
        assert_eq!(playtime.session_count, 1);
    }

    #[test]
    fn test_game_progress() {
        let progress = GameProgressSave::new()
            .with_game_mode("survival")
            .with_difficulty("hard");

        assert_eq!(progress.game_mode, "survival");
        assert_eq!(progress.difficulty, "hard");
    }

    #[test]
    fn test_new_game_plus() {
        let mut progress = GameProgressSave::new();
        progress.map.discover_chunk(ChunkCoord::new(0, 0));
        progress
            .achievements
            .add_achievement(AchievementProgressSave::new("test", 1));

        progress.start_new_game_plus();

        assert_eq!(progress.new_game_plus, 1);
        assert_eq!(progress.map.chunk_count(), 0);
        assert_eq!(progress.achievements.total_count(), 1); // Kept
    }
}
