//! FC/NES emulator CLI.
//!
//!   fc run <rom> [--frames N] [--region ntsc|pal|dendy] [--palette f.pal] [--remove-sprite-limit]
//!   fc test <rom> [--entry C000] [--frames N]   # CPU test ROMs (nestest etc.)
//!   fc disasm <rom> <addr-hex> [--count N]
//!   fc info <rom>
//!   fc mcp [--rom <rom>]                          # start MCP server on stdio

mod tauri_bridge;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use fc_core::{Button, ControlDeck, Region};

#[derive(Parser)]
#[command(name = "fc", version, about = "FC/NES emulator with LLM co-play (MCP)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a ROM headless for N frames and print a summary.
    Run {
        rom: String,
        #[arg(short, long, default_value = "ntsc")]
        region: String,
        #[arg(short, long, default_value_t = 600)]
        frames: u64,
        #[arg(short, long)]
        palette: Option<String>,
        /// Write the final frame to a PNG file.
        #[arg(short, long)]
        shot: Option<String>,
        /// Tap Start around frame 60 (to enter gameplay for demos/tests).
        #[arg(long)]
        autostart: bool,
        /// Script controller input as FRAME:DURATION:BUTTON[+BUTTON...].
        #[arg(long = "press", value_name = "FRAME:DURATION:BUTTONS")]
        presses: Vec<String>,
        /// Visual enhancement: render sprites beyond the NES 8-per-scanline limit.
        #[arg(long)]
        remove_sprite_limit: bool,
        /// Write all emulated audio to a mono 16-bit WAV file.
        #[arg(long)]
        wav: Option<String>,
    },
    /// Run a CPU test ROM (sets entry PC, prints result code at $0002).
    Test {
        rom: String,
        #[arg(short, long, default_value = "C000")]
        entry: String,
        #[arg(short, long, default_value_t = 200)]
        frames: u64,
    },
    /// Disassemble a ROM at an address.
    Disasm {
        rom: String,
        addr: String,
        #[arg(short, long, default_value_t = 30)]
        count: usize,
    },
    /// Print ROM header info.
    Info { rom: String },
    /// Trace executed instructions in nestest/Nintendulator layout (for accuracy
    /// diffing against Mesen2). Off the hot path unless this command is used.
    Trace {
        rom: String,
        #[arg(short, long, default_value = "ntsc")]
        region: String,
        /// Stop after N instructions (0 = run until --frames).
        #[arg(long, default_value_t = 0)]
        instrs: u64,
        /// Stop after N frames (0 = run until --instrs).
        #[arg(short, long, default_value_t = 0)]
        frames: u64,
        /// Set PC to this hex entry before tracing (e.g. C000 for nestest).
        #[arg(short, long)]
        entry: Option<String>,
    },
    /// Headless performance benchmark: emulation fps over a fixed run.
    Bench {
        rom: String,
        #[arg(short, long, default_value = "ntsc")]
        region: String,
        #[arg(short, long, default_value_t = 3000)]
        frames: u64,
        /// Warmup frames run before timing starts.
        #[arg(long, default_value_t = 120)]
        warmup: u64,
        /// Per-subsystem attribution: re-run with PPU render-output and APU
        /// resample ablated, reporting each one's share of frame time.
        #[arg(long, default_value_t = false)]
        profile: bool,
    },
    /// Run self-checking test ROMs and score them.
    Testsuite {
        roms: Vec<String>,
        #[arg(short, long, default_value = "ntsc")]
        region: String,
        #[arg(short, long, default_value_t = 3000)]
        frames: u64,
        #[arg(long, value_enum, default_value_t = TestProtocol::Blargg)]
        protocol: TestProtocol,
        #[arg(long)]
        expect_text: Vec<String>,
        /// Emit a machine-readable JSON report instead of the human table.
        #[arg(long)]
        json: bool,
        /// Compare against a baseline JSON; exit non-zero if any baseline-pass ROM regresses.
        #[arg(long)]
        baseline: Option<String>,
        /// Write the current results as a baseline JSON file.
        #[arg(long)]
        record_baseline: Option<String>,
    },
    /// Dump PPU debug views (pattern tables + nametables) to PNGs.
    Dbg {
        rom: String,
        #[arg(short, long, default_value_t = 120)]
        frames: u64,
        #[arg(long)]
        autostart: bool,
        /// Script controller input as FRAME:DURATION:BUTTON[+BUTTON...].
        #[arg(long = "press", value_name = "FRAME:DURATION:BUTTONS")]
        presses: Vec<String>,
    },
    /// Start the MCP server (stdio) for LLM agents.
    Mcp {
        #[arg(short, long)]
        rom: Option<String>,
    },
    /// MCP bridge (stdio) to the running fc-tauri dev app (eval JS / screenshot).
    TauriBridge,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TestProtocol {
    /// blargg $6000 signature/status protocol.
    Blargg,
    /// Text-console ROMs that print Passed/Failed/Error into nametable memory.
    Console,
    /// Validation ROMs that report final result in zero page $00F8 (1 = pass).
    Validation,
}

enum TestOutcome {
    Pass(String),
    Fail(String),
    Timeout(String),
}

#[derive(Debug, Clone)]
struct InputPress {
    start: u64,
    end: u64,
    buttons: u8,
}

fn parse_presses(specs: &[String]) -> Result<Vec<InputPress>> {
    let mut out = Vec::new();
    for spec in specs {
        let mut parts = spec.split(':');
        let start = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing input start frame in {spec:?}"))?
            .parse::<u64>()?;
        let duration = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing input duration in {spec:?}"))?
            .parse::<u64>()?;
        let buttons = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing input buttons in {spec:?}"))?;
        if parts.next().is_some() {
            anyhow::bail!("input spec has too many ':' fields: {spec:?}");
        }

        let mut mask = 0u8;
        for name in buttons.split(['+', ',']) {
            let Some(button) = Button::from_name(name.trim()) else {
                anyhow::bail!("unknown button {name:?} in {spec:?}");
            };
            mask |= 1 << button.bit();
        }
        out.push(InputPress {
            start,
            end: start.saturating_add(duration),
            buttons: mask,
        });
    }
    Ok(out)
}

fn scripted_buttons(frame: u64, autostart: bool, presses: &[InputPress]) -> u8 {
    let mut buttons = 0u8;
    if autostart {
        if (60..64).contains(&frame) {
            buttons |= 1 << Button::Start.bit();
        }
        if frame >= 90 {
            buttons |= 1 << Button::Right.bit();
        }
    }
    for press in presses {
        if (press.start..press.end).contains(&frame) {
            buttons |= press.buttons;
        }
    }
    buttons
}

fn format_mem_size(bytes: usize) -> String {
    if bytes == 0 {
        "0".to_string()
    } else if bytes % (1024 * 1024) == 0 {
        format!("{} MB", bytes / (1024 * 1024))
    } else if bytes % 1024 == 0 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{bytes} bytes")
    }
}

/// Run a blargg-style self-checking ROM until it reports a result via $6000.
fn run_blargg(deck: &mut ControlDeck, max_frames: u64) -> TestOutcome {
    let mut f = 0;
    let mut resets = 0;
    while f < max_frames {
        for _ in 0..10 {
            deck.run_frame();
        }
        f += 10;
        let sig = [
            deck.read_memory(0x6001),
            deck.read_memory(0x6002),
            deck.read_memory(0x6003),
        ];
        if sig == [0xDE, 0xB0, 0x61] {
            let s = deck.read_memory(0x6000);
            if s == 0x81 {
                if resets >= 8 {
                    return TestOutcome::Timeout("too many reset requests".into());
                }
                for _ in 0..6 {
                    deck.run_frame();
                }
                f += 6;
                deck.reset();
                resets += 1;
                continue;
            }
            if s < 0x80 {
                let mut msg = String::new();
                for i in 0..512u16 {
                    let b = deck.read_memory(0x6004 + i);
                    if b == 0 {
                        break;
                    }
                    if (0x20..0x7F).contains(&b) {
                        msg.push(b as char);
                    }
                }
                return if s == 0 {
                    TestOutcome::Pass(msg)
                } else {
                    TestOutcome::Fail(format!("code ${s:02X} {}", msg.trim()))
                };
            }
        }
    }
    TestOutcome::Timeout("timeout".into())
}

/// Run a validation-style ROM for a fixed budget and read its final result.
/// These ROMs initialize $00F8 to 0, store an error code while testing, then
/// store 1 only when `tests_passed` is reached.
fn run_validation(deck: &mut ControlDeck, max_frames: u64) -> TestOutcome {
    for _ in 0..max_frames {
        deck.run_frame();
    }
    let result = deck.read_memory(0x00F8);
    match result {
        1 => TestOutcome::Pass("validation $00F8=01".into()),
        0 => TestOutcome::Timeout("no validation result ($00F8=00)".into()),
        n => TestOutcome::Fail(format!("validation failed #{n} ($00F8=${n:02X})")),
    }
}

fn run_console(deck: &mut ControlDeck, max_frames: u64, expected: &[String]) -> TestOutcome {
    for _ in 0..max_frames {
        deck.run_frame();
    }
    let text = console_text(deck);
    let compact = compact_console_text(&text);
    if let Some(missing) = expected
        .iter()
        .find(|needle| !compact.contains(needle.as_str()))
    {
        return TestOutcome::Fail(format!(
            "missing expected text {missing:?}; console: {compact}"
        ));
    }
    if !expected.is_empty() {
        return TestOutcome::Pass(compact);
    }
    if compact.contains("Passed") {
        TestOutcome::Pass(compact)
    } else if let Some(i) = compact.find("Failed").or_else(|| compact.find("Error ")) {
        TestOutcome::Fail(compact[i..].chars().take(120).collect())
    } else {
        TestOutcome::Timeout("no console result".into())
    }
}

fn console_text(deck: &ControlDeck) -> String {
    let mut text = String::new();
    for table in 0..4u16 {
        let base = 0x2000 + table * 0x400;
        for row in 0..30u16 {
            for col in 0..32u16 {
                let b = deck.read_ppu_memory(base + row * 32 + col);
                let ch = if (0x20..=0x7E).contains(&b) {
                    b as char
                } else {
                    ' '
                };
                text.push(ch);
            }
            text.push('\n');
        }
        text.push('\n');
    }
    text
}

fn compact_console_text(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Suite name = the test ROM's containing directory (e.g. `cpu_interrupts_v2`),
/// skipping a `rom_singles` leaf so sibling suites group sensibly.
fn suite_name(path: &str) -> String {
    let p = std::path::Path::new(path);
    let mut dir = p.parent();
    if dir
        .and_then(|d| d.file_name())
        .map(|s| s == "rom_singles")
        .unwrap_or(false)
    {
        dir = dir.and_then(|d| d.parent());
    }
    dir.and_then(|d| d.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn short(path: &str) -> String {
    let p = std::path::Path::new(path);
    let parent = p
        .parent()
        .and_then(|x| x.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let name = p
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    format!("{parent}/{name}")
}

fn write_png(path: &str, rgba: &[u8]) -> Result<()> {
    write_png_sized(path, rgba, 256, 240)
}

fn write_png_sized(path: &str, rgba: &[u8], w: u32, h: u32) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(rgba)?;
    Ok(())
}

fn write_wav(path: &str, samples: &[f32], rate: u32) -> Result<()> {
    use std::io::Write;
    let n = samples.len() as u32;
    let data_bytes = n * 2;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_bytes).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?; // fmt chunk size
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&1u16.to_le_bytes())?; // mono
    f.write_all(&rate.to_le_bytes())?;
    f.write_all(&(rate * 2).to_le_bytes())?; // byte rate
    f.write_all(&2u16.to_le_bytes())?; // block align
    f.write_all(&16u16.to_le_bytes())?; // bits/sample
    f.write_all(b"data")?;
    f.write_all(&data_bytes.to_le_bytes())?;
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        f.write_all(&v.to_le_bytes())?;
    }
    Ok(())
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            rom,
            region,
            frames,
            palette,
            shot,
            autostart,
            presses,
            remove_sprite_limit,
            wav,
        } => {
            let data = std::fs::read(&rom)?;
            let mut deck = ControlDeck::new(Region::from_str(&region));
            deck.load_rom(&data)?;
            deck.set_remove_sprite_limit(remove_sprite_limit);
            if let Some(p) = palette {
                let pd = std::fs::read(&p)?;
                if deck.load_palette_file(&pd) {
                    log::info!("loaded palette {p}");
                }
            }
            let mut audio: Vec<f32> = Vec::new();
            let presses = parse_presses(&presses)?;
            let start = std::time::Instant::now();
            for f in 0..frames {
                deck.set_controller_state(0, scripted_buttons(f, autostart, &presses));
                deck.run_frame();
                let samples = deck.drain_audio();
                if wav.is_some() {
                    audio.extend_from_slice(&samples);
                }
            }
            let secs = start.elapsed().as_secs_f64();
            let fb = deck.frame_buffer();
            let non_black = fb.chunks(4).filter(|c| c[0] | c[1] | c[2] != 0).count();
            println!(
                "frames={frames} time={secs:.3}s fps={:.1}",
                frames as f64 / secs
            );
            println!("non-black pixels: {non_black}/{}", 256 * 240);
            println!("{}", deck.cpu_state_string());
            if let Some(path) = shot {
                write_png(&path, deck.frame_buffer())?;
                println!("wrote {path}");
            }
            if let Some(path) = wav {
                write_wav(&path, &audio, 44_100)?;
                let rms =
                    (audio.iter().map(|s| s * s).sum::<f32>() / audio.len().max(1) as f32).sqrt();
                println!("wrote {path} ({} samples, RMS {rms:.4})", audio.len());
            }
        }
        Commands::Test { rom, entry, frames } => {
            let data = std::fs::read(&rom)?;
            let mut deck = ControlDeck::new(Region::Ntsc);
            deck.load_rom(&data)?;
            let entry = u16::from_str_radix(&entry, 16)?;
            deck.cpu.pc = entry;
            println!("running test from ${entry:04X} for {frames} frames");
            for _ in 0..frames {
                deck.run_frame();
            }
            let code = deck.read_memory(0x0002) as u16 | (deck.read_memory(0x0003) as u16) << 8;
            println!("{}", deck.cpu_state_string());
            println!(
                "result $0002={code:04X} ({})",
                if code == 0 { "PASS" } else { "see code" }
            );
        }
        Commands::Trace {
            rom,
            region,
            instrs,
            frames,
            entry,
        } => {
            use std::io::Write as _;
            let data = std::fs::read(&rom)?;
            let mut deck = ControlDeck::new(Region::from_str(&region));
            deck.load_rom(&data)?;
            if let Some(e) = &entry {
                deck.cpu.pc = u16::from_str_radix(e, 16)?;
            }
            deck.set_trace(true);
            let stdout = std::io::stdout();
            let mut w = std::io::BufWriter::new(stdout.lock());
            let mut emitted: u64 = 0;
            let mut done_frames: u64 = 0;
            // Default (no bounds): trace one frame.
            let frame_cap = if instrs == 0 && frames == 0 {
                1
            } else {
                frames
            };
            'trace: loop {
                deck.run_frame();
                done_frames += 1;
                for r in deck.take_trace() {
                    let mut bytes = String::new();
                    for i in 0..(r.len as usize).min(3) {
                        bytes.push_str(&format!("{:02X} ", r.bytes[i]));
                    }
                    // nestest / Nintendulator layout (diffable against Mesen2).
                    writeln!(
                        w,
                        "{:04X}  {:<8}  {:<31} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3},{:3} CYC:{}",
                        r.pc, bytes.trim_end(), r.op_text,
                        r.a, r.x, r.y, r.p, r.sp, r.scanline, r.dot, r.cycle
                    )?;
                    emitted += 1;
                    if instrs != 0 && emitted >= instrs {
                        break 'trace;
                    }
                }
                if frame_cap != 0 && done_frames >= frame_cap {
                    break;
                }
            }
            w.flush()?;
        }
        Commands::Bench {
            rom,
            region,
            frames,
            warmup,
            profile,
        } => {
            let data = std::fs::read(&rom)?;
            let region = Region::from_str(&region);

            // Time `frames` frames with the given ablation, after `warmup`.
            let time_run = |no_render: bool, no_apu: bool| -> f64 {
                let mut deck = ControlDeck::new(region);
                deck.load_rom(&data).expect("load rom");
                deck.set_profile_ablation(no_render, no_apu);
                for _ in 0..warmup {
                    deck.run_frame();
                    deck.drain_audio();
                }
                let t0 = std::time::Instant::now();
                for _ in 0..frames {
                    deck.run_frame();
                    deck.drain_audio();
                }
                t0.elapsed().as_secs_f64()
            };

            let realtime = ControlDeck::new(region).region_frame_rate();
            let elapsed = time_run(false, false);
            let fps = frames as f64 / elapsed;
            println!(
                "bench {rom}: {frames} frames in {elapsed:.3}s = {fps:.1} fps ({:.1}x realtime @ {realtime:.2})",
                fps / realtime
            );

            if profile {
                // Ablation deltas: each variant skips one subsystem's *output*
                // work only (behavior identical), so `full − variant` per-frame
                // time is that subsystem's cost. The remainder is CPU + PPU core
                // (dot machine, fetch, sprite eval) + mapper.
                let t_full = elapsed / frames as f64;
                let t_no_render = time_run(true, false) / frames as f64;
                let t_no_apu = time_run(false, true) / frames as f64;
                let render = (t_full - t_no_render).max(0.0);
                let apu = (t_full - t_no_apu).max(0.0);
                let rest = (t_full - render - apu).max(0.0);
                let us = |t: f64| t * 1e6;
                let pct = |t: f64| t / t_full * 100.0;
                println!(
                    "  per-frame {:.0}µs — subsystem attribution (ablation):",
                    us(t_full)
                );
                println!(
                    "    PPU render-output : {:6.0}µs  {:4.1}%",
                    us(render),
                    pct(render)
                );
                println!(
                    "    APU resample      : {:6.0}µs  {:4.1}%",
                    us(apu),
                    pct(apu)
                );
                println!(
                    "    CPU + PPU-core + mapper (remainder): {:6.0}µs  {:4.1}%",
                    us(rest),
                    pct(rest)
                );
            }
        }
        Commands::Disasm { rom, addr, count } => {
            let data = std::fs::read(&rom)?;
            let mut deck = ControlDeck::new(Region::Ntsc);
            deck.load_rom(&data)?;
            let addr = u16::from_str_radix(&addr, 16)?;
            for line in deck.disassemble(addr, count) {
                println!("  {line}");
            }
        }
        Commands::Info { rom } => {
            let data = std::fs::read(&rom)?;
            let mut deck = ControlDeck::new(Region::Ntsc);
            deck.load_rom(&data)?;
            let c = &deck.bus.cartridge;
            println!("mapper:   {}", c.mapper_number);
            if c.is_nes20 {
                println!("submap:   {}", c.submapper);
            }
            println!("format:   {}", if c.is_nes20 { "NES 2.0" } else { "iNES" });
            println!("PRG-ROM:  {} KB", c.prg_rom.len() / 1024);
            println!(
                "PRG-RAM:  {} work + {} save",
                format_mem_size(c.prg_ram_size),
                format_mem_size(c.prg_nvram_size)
            );
            println!(
                "CHR:      {} KB ({})",
                if c.uses_chr_ram {
                    c.chr_ram.len()
                } else {
                    c.chr_rom.len()
                } / 1024,
                if c.uses_chr_ram { "CHR-RAM" } else { "CHR-ROM" }
            );
            println!(
                "CHR-RAM:  {} work + {} save",
                format_mem_size(c.chr_ram_size),
                format_mem_size(c.chr_nvram_size)
            );
            println!("mirroring:{:?}", c.mirroring());
            println!("battery:  {}", c.has_battery);
        }
        Commands::Testsuite {
            roms,
            region,
            frames,
            protocol,
            expect_text,
            json,
            baseline,
            record_baseline,
        } => {
            let mut pass = 0;
            let mut total = 0;
            let region = Region::from_str(&region);
            // (path, suite, status, message) — status ∈ pass|fail|timeout|error.
            let mut results: Vec<(String, String, &'static str, String)> = Vec::new();
            for rom in &roms {
                total += 1;
                let suite = suite_name(rom);
                let data = match std::fs::read(rom) {
                    Ok(d) => d,
                    Err(e) => {
                        if !json {
                            println!("  {:<48} ERR  {e}", short(rom));
                        }
                        results.push((rom.clone(), suite, "error", e.to_string()));
                        continue;
                    }
                };
                let mut deck = ControlDeck::new(region);
                if deck.load_rom(&data).is_err() {
                    if !json {
                        println!("  {:<48} ERR  bad ROM", short(rom));
                    }
                    results.push((rom.clone(), suite, "error", "bad ROM".into()));
                    continue;
                }
                let outcome = match protocol {
                    TestProtocol::Blargg => run_blargg(&mut deck, frames),
                    TestProtocol::Console => run_console(&mut deck, frames, &expect_text),
                    TestProtocol::Validation => run_validation(&mut deck, frames),
                };
                let (verdict, status, msg) = match outcome {
                    TestOutcome::Pass(msg) => {
                        pass += 1;
                        ("PASS", "pass", msg)
                    }
                    TestOutcome::Fail(msg) => ("FAIL", "fail", msg),
                    TestOutcome::Timeout(msg) => ("TIMEOUT", "timeout", msg),
                };
                if !json {
                    println!("  {:<48} {verdict:<8} {}", short(rom), msg.trim());
                }
                results.push((rom.clone(), suite, status, msg.trim().to_string()));
            }
            if json {
                let arr: Vec<_> = results
                    .iter()
                    .map(|(path, suite, status, message)| {
                        serde_json::json!({"path": path, "suite": suite, "status": status, "message": message, "frames": frames})
                    })
                    .collect();
                let doc = serde_json::json!({"passed": pass, "total": total, "roms": arr});
                println!("{}", serde_json::to_string_pretty(&doc)?);
            } else {
                println!("\n  {pass}/{total} passed");
            }
            if let Some(path) = &record_baseline {
                let arr: Vec<_> = results
                    .iter()
                    .map(|(p, _, status, _)| serde_json::json!({"path": p, "status": status}))
                    .collect();
                std::fs::write(path, serde_json::to_string_pretty(&serde_json::json!(arr))?)?;
                eprintln!("recorded baseline → {path} ({total} roms)");
            }
            if let Some(path) = &baseline {
                let base: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
                let mut regressed = Vec::new();
                let now: std::collections::HashMap<&str, &str> = results
                    .iter()
                    .map(|(p, _, s, _)| (p.as_str(), *s))
                    .collect();
                for entry in base.as_array().into_iter().flatten() {
                    let p = entry["path"].as_str().unwrap_or("");
                    // Only ROMs run THIS session can regress; a baseline ROM that
                    // wasn't run is simply untested, not a regression.
                    if entry["status"].as_str() == Some("pass") {
                        if let Some(&cur) = now.get(p) {
                            if cur != "pass" {
                                regressed.push((p.to_string(), cur.to_string()));
                            }
                        }
                    }
                }
                if regressed.is_empty() {
                    eprintln!("baseline OK: no regressions vs {path}");
                } else {
                    eprintln!("REGRESSION vs {path}:");
                    for (p, s) in &regressed {
                        eprintln!("  {} : pass → {}", short(p), s);
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Dbg {
            rom,
            frames,
            autostart,
            presses,
        } => {
            let data = std::fs::read(&rom)?;
            let mut deck = ControlDeck::new(Region::Ntsc);
            deck.load_rom(&data)?;
            let presses = parse_presses(&presses)?;
            for f in 0..frames {
                deck.set_controller_state(0, scripted_buttons(f, autostart, &presses));
                deck.run_frame();
            }
            write_png_sized("/tmp/dbg_pattern0.png", &deck.pattern_table(0, 0), 128, 128)?;
            write_png_sized("/tmp/dbg_pattern1.png", &deck.pattern_table(1, 0), 128, 128)?;
            write_png_sized("/tmp/dbg_nametables.png", &deck.nametables(), 512, 480)?;
            println!("wrote /tmp/dbg_pattern0.png /tmp/dbg_pattern1.png /tmp/dbg_nametables.png");
        }
        Commands::Mcp { rom } => {
            let mut deck = ControlDeck::new(Region::Ntsc);
            if let Some(rp) = rom {
                let data = std::fs::read(&rp)?;
                deck.load_rom(&data)?;
                eprintln!("# fc-mcp: loaded {rp}");
            }
            eprintln!("# fc-mcp: serving MCP over stdio");
            let shared = fc_mcp::shared(deck);
            let mut server = fc_mcp::McpServer::new(shared);
            server.run_stdio()?;
        }
        Commands::TauriBridge => {
            tauri_bridge::run()?;
        }
    }
    Ok(())
}
