# PROMPT — Infra Agent — Iteration 6

> **Branch**: `infra-agent`
> **Focus**: Wire egui into render loop, multi-chunk terrain, player z-index fix, performance profiling

## Your Mission

Integrate all the new systems into the main engine. The player should render above terrain, egui UI should appear on top of everything, and multi-chunk rendering should be wired up.

---

## Tasks

### I-20: Egui in Main Render Loop (P0)
**File**: `crates/genesis-engine/src/renderer.rs`

Integrate egui rendering into the main render pipeline:

```rust
pub struct Renderer {
    // ... existing fields ...
    
    /// Egui integration for UI
    egui: EguiIntegration,
}

impl Renderer {
    pub async fn new(window: &Window) -> Result<Self> {
        // ... existing init ...
        
        // Initialize egui
        let egui = EguiIntegration::new(
            &device,
            config.format,
            window,
            1, // No MSAA for now
        );
        
        // ...
    }
    
    /// Handle window events - returns true if egui consumed event
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui.handle_event(window, event)
    }
    
    /// Main render with UI
    pub fn render_with_ui<F>(
        &mut self,
        window: &Window,
        camera: &Camera,
        gameplay: &GameState,
        ui_callback: F,
    ) -> Result<()>
    where
        F: FnOnce(&egui::Context),
    {
        // 1. Begin egui frame
        self.egui.begin_frame(window);
        
        // 2. Call UI callback (HUD, inventory, etc.)
        ui_callback(self.egui.context());
        
        // 3. End egui frame
        let egui_output = self.egui.end_frame(window);
        
        // 4. Get surface texture
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        
        // 5. Create encoder
        let mut encoder = self.device.create_command_encoder(&Default::default());
        
        // 6. Render game world (terrain, entities)
        self.render_world(&mut encoder, &view, camera, gameplay);
        
        // 7. Render player marker ABOVE terrain
        self.render_player(&mut encoder, &view, camera, gameplay);
        
        // 8. Render egui UI on top of everything
        self.egui.render(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            &self.screen_descriptor(),
            egui_output,
        );
        
        // 9. Submit and present
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}
```

---

### I-21: Multi-Chunk Terrain Integration (P0)
**File**: `crates/genesis-engine/src/renderer.rs`

Replace single chunk rendering with ChunkManager:

```rust
pub struct Renderer {
    // Remove: cell_buffer: CellBuffer,
    // Add:
    chunk_manager: ChunkManager,
    world_generator: Box<dyn WorldGenerator>,
}

impl Renderer {
    /// Render all visible chunks
    fn render_world(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera: &Camera,
        gameplay: &GameState,
    ) {
        // Update visible chunks based on camera
        self.chunk_manager.update_visible(camera, self.world_generator.as_ref());
        
        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("World Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1, g: 0.15, b: 0.2, a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        
        // Render each visible chunk
        for chunk in self.chunk_manager.visible_chunks() {
            self.render_chunk(&mut render_pass, chunk, camera);
        }
    }
}
```

---

### I-22: Player Z-Index Fix (P0)
**File**: `crates/genesis-kernel/src/render.rs`

Fix player marker to render above terrain. Options:
1. Render player in a separate pass after terrain
2. Use depth buffer with player at lower depth
3. Render player as a UI element

**Recommended approach** - Add player rendering uniform:

```rust
/// Render params now include player position
#[repr(C)]
pub struct RenderParams {
    // ... existing fields ...
    
    /// Player world position X
    pub player_x: f32,
    /// Player world position Y  
    pub player_y: f32,
    /// Player marker radius (in cells)
    pub player_radius: f32,
    /// Whether to show player marker
    pub show_player: u32,
}
```

Update the shader to draw player marker AFTER terrain color is determined:

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ... calculate terrain color ...
    
    // Draw player marker on top of terrain
    // Convert player world position to screen position
    let player_screen_x = (params.player_x - f32(params.camera_x)) * params.zoom;
    let player_screen_y = (params.player_y - f32(params.camera_y)) * params.zoom;
    
    // Offset to screen center
    let center_offset_x = f32(params.screen_width) / 2.0;
    let center_offset_y = f32(params.screen_height) / 2.0;
    
    let final_player_x = player_screen_x + center_offset_x;
    let final_player_y = player_screen_y + center_offset_y;
    
    let pixel_x = in.uv.x * f32(params.screen_width);
    let pixel_y = in.uv.y * f32(params.screen_height);
    
    let dist = sqrt(
        (pixel_x - final_player_x) * (pixel_x - final_player_x) +
        (pixel_y - final_player_y) * (pixel_y - final_player_y)
    );
    
    let marker_size = params.player_radius * params.zoom;
    
    if dist < marker_size * 0.5 {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // White core
    } else if dist < marker_size * 0.75 {
        return vec4<f32>(0.0, 1.0, 1.0, 1.0); // Cyan ring
    } else if dist < marker_size {
        return vec4<f32>(0.0, 0.3, 0.3, 1.0); // Dark outline
    }
    
    return terrain_color;
}
```

---

### I-23: Performance Profiling (P1)
**File**: `crates/genesis-engine/src/perf.rs` (new)

Add performance monitoring:

```rust
/// Performance metrics collector
pub struct PerfMetrics {
    frame_times: VecDeque<f32>,
    update_times: VecDeque<f32>,
    render_times: VecDeque<f32>,
    chunk_count: u32,
    cell_count: u64,
}

impl PerfMetrics {
    pub fn new(history_size: usize) -> Self;
    
    /// Record frame timing
    pub fn record_frame(&mut self, total: f32, update: f32, render: f32);
    
    /// Get average FPS
    pub fn avg_fps(&self) -> f32;
    
    /// Get 1% low FPS
    pub fn low_fps(&self) -> f32;
    
    /// Get average frame time
    pub fn avg_frame_time(&self) -> f32;
    
    /// Update chunk/cell counts
    pub fn set_world_stats(&mut self, chunks: u32, cells: u64);
    
    /// Get summary for debug display
    pub fn summary(&self) -> PerfSummary;
}

pub struct PerfSummary {
    pub fps: f32,
    pub fps_1_percent_low: f32,
    pub frame_time_ms: f32,
    pub update_time_ms: f32,
    pub render_time_ms: f32,
    pub chunks_loaded: u32,
    pub cells_simulated: u64,
}
```

**Integration**: Display in debug overlay (F3):
```
FPS: 60 (1% low: 55)
Frame: 16.6ms (Update: 2.1ms, Render: 8.3ms)
Chunks: 9 loaded (589,824 cells)
Camera: (128.5, 100.2) Zoom: 4.0x
Player: (128.5, 100.2) vel: (0.0, 0.0)
```

---

## App.rs Integration

Update `crates/genesis-engine/src/app.rs` to use new systems:

```rust
fn render(&mut self) {
    if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
        // Render with UI callback
        let result = renderer.render_with_ui(
            window,
            &self.camera,
            &self.gameplay,
            |ctx| {
                // Render HUD
                self.hud.render(ctx, &HUDState {
                    player: &self.gameplay.player,
                    health: &self.player_health,
                    inventory: &self.player_inventory,
                    hotbar_selection: self.hotbar_slot,
                    fps: self.current_fps,
                    player_position: self.gameplay.player.position().into(),
                    current_material: self.selected_material,
                });
                
                // Render inventory if open
                if self.show_inventory {
                    self.inventory_panel.render(ctx, &mut self.player_inventory);
                }
                
                // Render crafting if open
                if self.show_crafting {
                    self.crafting_panel.render(ctx, &self.recipes, &self.player_inventory);
                }
            },
        );
        
        if let Err(e) = result {
            warn!("Render error: {e}");
        }
    }
}
```

---

## Validation

After each task:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p genesis-engine
```

## Commit Format
```
[infra] feat: I-XX description
```

## Done Criteria
- [ ] Egui UI renders on top of game world
- [ ] Multi-chunk terrain renders seamlessly
- [ ] Player marker is visible above terrain
- [ ] Performance metrics displayed in debug overlay
- [ ] No regressions in frame rate
