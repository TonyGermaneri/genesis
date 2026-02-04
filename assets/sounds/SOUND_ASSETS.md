# Sound Assets Manifest

This document lists all sound files required by the game. Replace stub files with your custom audio.

## Directory Structure

```
assets/sounds/
├── music/           # Background music tracks (MP3, looping)
├── ambient/         # Environmental ambient sounds (MP3, looping)
└── sfx/             # Sound effects (WAV or MP3, one-shot)
```

---

## 🎵 MUSIC (Replace with Custom MP3)

These files should be replaced with your custom music. Recommended format: MP3, 128-192kbps, stereo.

| File | Duration | Description | Loop |
|------|----------|-------------|------|
| `music/menu_theme.mp3` | 2-4 min | Main menu music, calm and inviting | Yes |
| `music/exploration.mp3` | 3-5 min | General exploration, peaceful | Yes |
| `music/forest.mp3` | 3-5 min | Forest biome ambient music | Yes |
| `music/desert.mp3` | 3-5 min | Desert biome ambient music, mysterious | Yes |
| `music/combat.mp3` | 2-3 min | Combat encounter music, intense | Yes |
| `music/boss.mp3` | 2-3 min | Boss fight music, epic | Yes |
| `music/night.mp3` | 3-5 min | Nighttime ambient music, eerie | Yes |
| `music/village.mp3` | 3-5 min | Village/safe area music, cheerful | Yes |

---

## 🌿 AMBIENT SOUNDS (Replace with Custom MP3)

These loop continuously based on biome/environment. Recommended: MP3, mono or stereo, seamless loop.

| File | Description | Trigger |
|------|-------------|---------|
| `ambient/forest_day.mp3` | Birds, rustling leaves, wind | Forest biome, daytime |
| `ambient/forest_night.mp3` | Crickets, owls, night wind | Forest biome, nighttime |
| `ambient/desert_day.mp3` | Hot wind, distant sand | Desert biome, daytime |
| `ambient/desert_night.mp3` | Cool wind, coyotes | Desert biome, nighttime |
| `ambient/lake.mp3` | Water lapping, frogs | Lake/water biome |
| `ambient/mountain.mp3` | High wind, eagles | Mountain biome |
| `ambient/swamp.mp3` | Bubbling, insects, croaks | Swamp biome |
| `ambient/rain.mp3` | Rain on ground/leaves | During rain weather |
| `ambient/storm.mp3` | Heavy rain, thunder | During storm weather |
| `ambient/cave.mp3` | Dripping water, echoes | Inside caves |

---

## 🔊 SFX - Sound Effects (Included or Replace)

Short one-shot sounds. Can be WAV (better quality) or MP3. These may already exist or need creation.

### Player Actions
| File | Description |
|------|-------------|
| `sfx/footstep_grass_1.wav` | Walking on grass |
| `sfx/footstep_grass_2.wav` | Walking on grass (variant) |
| `sfx/footstep_sand_1.wav` | Walking on sand |
| `sfx/footstep_sand_2.wav` | Walking on sand (variant) |
| `sfx/footstep_stone_1.wav` | Walking on stone |
| `sfx/footstep_stone_2.wav` | Walking on stone (variant) |
| `sfx/footstep_water.wav` | Splashing in shallow water |
| `sfx/jump.wav` | Player jump |
| `sfx/land.wav` | Player landing |
| `sfx/swing_sword.wav` | Melee attack swing |
| `sfx/hit_enemy.wav` | Attack connects |
| `sfx/player_hurt.wav` | Player takes damage |
| `sfx/player_death.wav` | Player dies |

### Inventory/Items
| File | Description |
|------|-------------|
| `sfx/pickup_item.wav` | Generic item pickup |
| `sfx/pickup_coin.wav` | Coin/currency pickup |
| `sfx/equip.wav` | Equip item |
| `sfx/inventory_open.wav` | Open inventory |
| `sfx/inventory_close.wav` | Close inventory |

### Environment
| File | Description |
|------|-------------|
| `sfx/grass_cut.wav` | Cutting grass |
| `sfx/tree_chop.wav` | Chopping tree |
| `sfx/rock_break.wav` | Breaking rock |
| `sfx/water_splash.wav` | Entering water |
| `sfx/door_open.wav` | Door opening |
| `sfx/door_close.wav` | Door closing |
| `sfx/chest_open.wav` | Opening chest |

### NPCs
| File | Description |
|------|-------------|
| `sfx/npc_greet.wav` | NPC greeting |
| `sfx/npc_goodbye.wav` | NPC farewell |
| `sfx/merchant_buy.wav` | Purchase sound |
| `sfx/merchant_sell.wav` | Sell sound |

### Monsters
| File | Description |
|------|-------------|
| `sfx/slime_bounce.wav` | Slime movement |
| `sfx/slime_death.wav` | Slime defeated |
| `sfx/skeleton_rattle.wav` | Skeleton movement |
| `sfx/monster_growl.wav` | Generic monster aggro |

### UI
| File | Description |
|------|-------------|
| `sfx/ui_click.wav` | Button click |
| `sfx/ui_hover.wav` | Button hover |
| `sfx/ui_error.wav` | Invalid action |
| `sfx/ui_confirm.wav` | Confirm action |
| `sfx/ui_cancel.wav` | Cancel action |
| `sfx/level_up.wav` | Level up fanfare |

---

## File Format Guidelines

### Music & Ambient
- **Format:** MP3 (preferred) or OGG
- **Bitrate:** 128-192 kbps for music, 96-128 kbps for ambient
- **Sample Rate:** 44100 Hz
- **Channels:** Stereo for music, mono acceptable for ambient
- **Looping:** Files should loop seamlessly (no click at loop point)

### Sound Effects
- **Format:** WAV (preferred for quality) or MP3
- **Bitrate:** 192+ kbps if MP3
- **Sample Rate:** 44100 Hz
- **Channels:** Mono (for spatial positioning)
- **Duration:** Typically 0.1 - 2 seconds

---

## Status

| Category | Status | Files |
|----------|--------|-------|
| Music | ⚪ STUB - Replace with custom | 8 files |
| Ambient | ⚪ STUB - Replace with custom | 10 files |
| SFX | ⚪ To be added | ~40 files |

**Total files to provide:** ~58 audio files
