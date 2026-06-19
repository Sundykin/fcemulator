//! FC/NES emulator GUI — egui + wgpu (winit 0.30).
//!
//! Controls: Arrows = D-Pad, Z = A, X = B, Enter = Start, Space = Select,
//! F1 pause, F5 reset, F8 open ROM, F2/F3 save/load state, Esc quit.

mod audio;

use audio::Audio;
use egui::{Color32, ColorImage, TextureHandle, TextureOptions};
use fc_core::{Button, ControlDeck, Region};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const NES_W: usize = 256;
const NES_H: usize = 240;

/// Upload an RGBA image into a (re-used) egui texture, returning its id.
fn upload(
    ctx: &egui::Context,
    tex: &mut Option<TextureHandle>,
    name: &str,
    size: [usize; 2],
    rgba: &[u8],
) -> egui::TextureId {
    let img = ColorImage::from_rgba_unmultiplied(size, rgba);
    match tex {
        Some(t) => {
            t.set(img, TextureOptions::NEAREST);
            t.id()
        }
        None => {
            let h = ctx.load_texture(name, img, TextureOptions::NEAREST);
            let id = h.id();
            *tex = Some(h);
            id
        }
    }
}

struct Gfx {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl Gfx {
    fn new(window: Arc<Window>) -> Gfx {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).expect("surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("fc-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .expect("device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            Some(window.scale_factor() as f32),
            None,
            Some(2048),
        );
        let renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);

        Gfx {
            surface,
            device,
            queue,
            config,
            egui_ctx,
            egui_state,
            renderer,
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

struct App {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    deck: ControlDeck,
    audio: Option<Audio>,
    nes_tex: Option<TextureHandle>,
    dbg_pt: [Option<TextureHandle>; 2],
    dbg_nt: Option<TextureHandle>,
    pal_row: u8,
    running: bool,
    paused: bool,
    rom_path: Option<String>,
    last: Instant,
    acc: f64,
    fps: f64,
    fps_count: u32,
    fps_timer: Instant,
    show_debug: bool,
    show_ppu: bool,
}

impl App {
    fn new() -> App {
        App {
            window: None,
            gfx: None,
            deck: ControlDeck::new(Region::Ntsc),
            audio: None,
            nes_tex: None,
            dbg_pt: [None, None],
            dbg_nt: None,
            pal_row: 0,
            running: false,
            paused: false,
            rom_path: None,
            last: Instant::now(),
            acc: 0.0,
            fps: 0.0,
            fps_count: 0,
            fps_timer: Instant::now(),
            show_debug: false,
            show_ppu: false,
        }
    }

    fn load_rom_path(&mut self, path: String) {
        match std::fs::read(&path) {
            Ok(data) => {
                if self.deck.load_rom(&data).is_ok() {
                    if let Some(a) = &self.audio {
                        self.deck.set_audio_sample_rate(a.sample_rate);
                    }
                    // restore battery save if present
                    if self.deck.has_battery() {
                        if let Ok(sram) = std::fs::read(format!("{path}.sav")) {
                            self.deck.load_battery_ram(&sram);
                        }
                    }
                    self.running = true;
                    self.paused = false;
                    self.rom_path = Some(path);
                } else {
                    log::error!("failed to parse ROM");
                }
            }
            Err(e) => log::error!("read ROM: {e}"),
        }
    }

    fn open_dialog(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("NES ROMs", &["nes", "NES"])
            .pick_file()
        {
            self.load_rom_path(p.to_string_lossy().to_string());
        }
    }

    fn save_path(&self) -> Option<String> {
        self.rom_path.as_ref().map(|p| format!("{p}.state"))
    }

    fn save_state(&mut self) {
        if let Some(path) = self.save_path() {
            let _ = std::fs::write(path, self.deck.save_state());
        }
    }
    fn load_state(&mut self) {
        if let Some(path) = self.save_path() {
            if let Ok(data) = std::fs::read(path) {
                self.deck.load_state(&data);
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode, pressed: bool) {
        let button = match code {
            KeyCode::KeyZ => Some(Button::A),
            KeyCode::KeyX => Some(Button::B),
            KeyCode::Enter => Some(Button::Start),
            KeyCode::Space => Some(Button::Select),
            KeyCode::ArrowUp => Some(Button::Up),
            KeyCode::ArrowDown => Some(Button::Down),
            KeyCode::ArrowLeft => Some(Button::Left),
            KeyCode::ArrowRight => Some(Button::Right),
            _ => None,
        };
        if let Some(b) = button {
            self.deck.set_button(0, b, pressed);
            return;
        }
        if pressed {
            match code {
                KeyCode::F1 => self.paused = !self.paused,
                KeyCode::F2 => self.save_state(),
                KeyCode::F3 => self.load_state(),
                KeyCode::F5 => self.deck.reset(),
                KeyCode::F8 => self.open_dialog(),
                _ => {}
            }
        }
    }

    fn emulate(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        if self.running && !self.paused {
            let mut ran = 0;
            if let Some(rate) = self.audio.as_ref().map(|a| a.sample_rate) {
                // Sync to the audio clock: run frames to keep the output buffer
                // near `target` (~50 ms). This makes the sound card's stable
                // clock drive emulation — no underruns, no long-term A/V drift.
                let target = (rate * 0.05) as usize;
                while self.audio.as_ref().unwrap().buffered() < target && ran < 6 {
                    self.deck.run_frame();
                    let samples = self.deck.drain_audio();
                    self.audio.as_ref().unwrap().queue(&samples);
                    ran += 1;
                    self.fps_count += 1;
                }
            } else {
                // No audio device: fall back to wall-clock pacing.
                self.acc += dt.min(0.1);
                let frame_period = 1.0 / self.deck.region_frame_rate();
                while self.acc >= frame_period && ran < 4 {
                    self.deck.run_frame();
                    let _ = self.deck.drain_audio();
                    self.acc -= frame_period;
                    ran += 1;
                    self.fps_count += 1;
                }
            }
        }
        if self.fps_timer.elapsed().as_secs_f64() >= 1.0 {
            self.fps = self.fps_count as f64 / self.fps_timer.elapsed().as_secs_f64();
            log::info!("emulated {:.1} frames/sec (wall clock)", self.fps);
            self.fps_count = 0;
            self.fps_timer = Instant::now();
        }
    }

    fn render(&mut self) {
        let Some(window) = self.window.clone() else { return };
        let Some(gfx) = self.gfx.as_mut() else { return };

        // Upload the latest frame into an egui texture.
        let tex_ctx = gfx.egui_ctx.clone();
        let nes_id = upload(&tex_ctx, &mut self.nes_tex, "nes", [NES_W, NES_H], self.deck.frame_buffer());

        // Debug textures + snapshots (only when the PPU viewer is open).
        let mut pt_ids: [Option<egui::TextureId>; 2] = [None, None];
        let mut nt_id: Option<egui::TextureId> = None;
        let mut oam_snapshot = [0u8; 256];
        let mut pal_snapshot = [fc_core::Rgb::new(0, 0, 0); 32];
        if self.show_ppu {
            let pt0 = self.deck.pattern_table(0, self.pal_row);
            pt_ids[0] = Some(upload(&tex_ctx, &mut self.dbg_pt[0], "pt0", [128, 128], &pt0));
            let pt1 = self.deck.pattern_table(1, self.pal_row);
            pt_ids[1] = Some(upload(&tex_ctx, &mut self.dbg_pt[1], "pt1", [128, 128], &pt1));
            let nt = self.deck.nametables();
            nt_id = Some(upload(&tex_ctx, &mut self.dbg_nt, "nt", [512, 480], &nt));
            oam_snapshot = *self.deck.oam();
            pal_snapshot = self.deck.palette_swatches();
        }

        let raw_input = gfx.egui_state.take_egui_input(&window);
        let ctx = gfx.egui_ctx.clone();

        // UI-driven actions captured here, applied after the closure.
        let mut act_open = false;
        let mut act_reset = false;
        let paused = self.paused;
        let running = self.running;
        let fps = self.fps;
        let cpu = self.deck.cpu.clone();
        let ppu_line = self.deck.bus.ppu.scanline;
        let frame = self.deck.frame_count();
        let mut show_debug = self.show_debug;
        let mut show_ppu = self.show_ppu;
        let mut pal_row = self.pal_row;
        let mut toggle_pause = false;

        let full = ctx.run(raw_input, |ctx| {
            egui::TopBottomPanel::top("menu").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    if ui.button("📂 Open").clicked() {
                        act_open = true;
                    }
                    if ui.button(if paused { "▶ Resume" } else { "⏸ Pause" }).clicked() {
                        toggle_pause = true;
                    }
                    if ui.button("⟲ Reset").clicked() {
                        act_reset = true;
                    }
                    ui.checkbox(&mut show_debug, "🐞 CPU");
                    ui.checkbox(&mut show_ppu, "🖼 PPU");
                    ui.separator();
                    ui.label(format!("{fps:.0} FPS"));
                    if !running {
                        ui.separator();
                        ui.label("no ROM — File ▸ Open or F8");
                    }
                });
            });

            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(Color32::BLACK))
                .show(ctx, |ui| {
                    let avail = ui.available_size();
                    // Integer scaling keeps pixel-art text crisp (non-integer
                    // nearest-neighbor sampling garbles thin 1px strokes).
                    let scale = (avail.x / NES_W as f32)
                        .min(avail.y / NES_H as f32)
                        .floor()
                        .max(1.0);
                    let size = egui::vec2(NES_W as f32 * scale, NES_H as f32 * scale);
                    ui.centered_and_justified(|ui| {
                        ui.add(
                            egui::Image::new(egui::load::SizedTexture::new(nes_id, size))
                                .texture_options(TextureOptions::NEAREST),
                        );
                    });
                });

            if show_debug {
                egui::Window::new("CPU").default_pos([20.0, 60.0]).show(ctx, |ui| {
                    ui.monospace(format!("PC {:04X}", cpu.pc));
                    ui.monospace(format!("A {:02X}  X {:02X}  Y {:02X}", cpu.a, cpu.x, cpu.y));
                    ui.monospace(format!("SP {:02X}  P {:02X} ({})", cpu.sp, cpu.p, ControlDeck::flags_string(cpu.p)));
                    ui.monospace(format!("scanline {ppu_line}  frame {frame}"));
                    ui.monospace(format!("cycles {}", cpu.cycles));
                });
            }

            if show_ppu {
                egui::Window::new("PPU Viewer").default_pos([300.0, 60.0]).show(ctx, |ui| {
                    ui.label("Pattern tables ($0000 / $1000)");
                    ui.horizontal(|ui| {
                        for id in pt_ids.into_iter().flatten() {
                            ui.add(
                                egui::Image::new(egui::load::SizedTexture::new(id, egui::vec2(256.0, 256.0)))
                                    .texture_options(TextureOptions::NEAREST),
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("BG palette:");
                        for r in 0..4u8 {
                            ui.selectable_value(&mut pal_row, r, format!("{r}"));
                        }
                    });
                    ui.separator();
                    ui.label("Palette RAM (BG | Sprite)");
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        for (i, c) in pal_snapshot.iter().enumerate() {
                            if i == 16 {
                                ui.add_space(8.0);
                            }
                            let (rect, _) = ui.allocate_exact_size(egui::vec2(13.0, 13.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, egui::Color32::from_rgb(c.r, c.g, c.b));
                        }
                    });
                    ui.separator();
                    ui.label("Nametables (2×2)");
                    if let Some(id) = nt_id {
                        ui.add(
                            egui::Image::new(egui::load::SizedTexture::new(id, egui::vec2(512.0, 480.0)))
                                .texture_options(TextureOptions::NEAREST),
                        );
                    }
                });

                egui::Window::new("OAM").default_pos([300.0, 60.0]).show(ctx, |ui| {
                    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                        egui::Grid::new("oam_grid").striped(true).show(ui, |ui| {
                            for h in ["#", "X", "Y", "Tile", "Attr"] {
                                ui.monospace(h);
                            }
                            ui.end_row();
                            for i in 0..64 {
                                let o = i * 4;
                                ui.monospace(format!("{i:02}"));
                                ui.monospace(format!("{:3}", oam_snapshot[o + 3]));
                                ui.monospace(format!("{:3}", oam_snapshot[o]));
                                ui.monospace(format!("{:02X}", oam_snapshot[o + 1]));
                                ui.monospace(format!("{:02X}", oam_snapshot[o + 2]));
                                ui.end_row();
                            }
                        });
                    });
                });
            }
        });

        gfx.egui_state
            .handle_platform_output(&window, full.platform_output);
        let tris = ctx.tessellate(full.shapes, full.pixels_per_point);
        let desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gfx.config.width, gfx.config.height],
            pixels_per_point: full.pixels_per_point,
        };
        for (id, delta) in &full.textures_delta.set {
            gfx.renderer.update_texture(&gfx.device, &gfx.queue, *id, delta);
        }

        let frame_tex = match gfx.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => {
                gfx.surface.configure(&gfx.device, &gfx.config);
                return;
            }
        };
        let view = frame_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let cmds = gfx
            .renderer
            .update_buffers(&gfx.device, &gfx.queue, &mut encoder, &tris, &desc);
        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut rpass = rpass.forget_lifetime();
            gfx.renderer.render(&mut rpass, &tris, &desc);
        }
        for id in &full.textures_delta.free {
            gfx.renderer.free_texture(id);
        }
        gfx.queue
            .submit(cmds.into_iter().chain(std::iter::once(encoder.finish())));
        frame_tex.present();

        // Apply UI-driven actions now that the `gfx` borrow has ended.
        self.show_debug = show_debug;
        self.show_ppu = show_ppu;
        self.pal_row = pal_row;
        if toggle_pause {
            self.paused = !self.paused;
        }
        if act_reset {
            self.deck.reset();
        }
        if act_open {
            self.open_dialog();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("FC Emulator — Famicom/NES")
            .with_inner_size(winit::dpi::LogicalSize::new(NES_W as f64 * 3.0, NES_H as f64 * 3.0 + 30.0));
        let window = Arc::new(event_loop.create_window(attrs).expect("window"));
        self.gfx = Some(Gfx::new(window.clone()));
        self.window = Some(window);
        self.audio = Audio::new();
        if let Some(a) = &self.audio {
            self.deck.set_audio_sample_rate(a.sample_rate);
        }
        // ROM passed on the command line.
        if let Some(p) = std::env::args().nth(1) {
            self.load_rom_path(p);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Let egui see the event first.
        if let (Some(gfx), Some(window)) = (self.gfx.as_mut(), self.window.as_ref()) {
            let _ = gfx.egui_state.on_window_event(window, &event);
        }
        match event {
            WindowEvent::CloseRequested => {
                // Persist battery SRAM on exit.
                if self.deck.has_battery() {
                    if let Some(p) = &self.rom_path {
                        let _ = std::fs::write(format!("{p}.sav"), self.deck.battery_ram());
                    }
                }
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::Escape && event.state == ElementState::Pressed {
                        event_loop.exit();
                        return;
                    }
                    self.handle_key(code, event.state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                self.emulate();
                self.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
