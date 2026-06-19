//! FC/NES emulator CLI.
//!
//!   fc run <rom> [--frames N] [--region ntsc|pal|dendy] [--palette f.pal]
//!   fc test <rom> [--entry C000] [--frames N]   # CPU test ROMs (nestest etc.)
//!   fc disasm <rom> <addr-hex> [--count N]
//!   fc info <rom>
//!   fc mcp [--rom <rom>]                          # start MCP server on stdio

mod tauri_bridge;

use anyhow::Result;
use clap::{Parser, Subcommand};
use fc_core::{ControlDeck, Region};

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
    /// Run self-checking test ROMs (blargg $6000 protocol) and score them.
    Testsuite {
        roms: Vec<String>,
        #[arg(short, long, default_value_t = 3000)]
        frames: u64,
    },
    /// Dump PPU debug views (pattern tables + nametables) to PNGs.
    Dbg {
        rom: String,
        #[arg(short, long, default_value_t = 120)]
        frames: u64,
        #[arg(long)]
        autostart: bool,
    },
    /// Start the MCP server (stdio) for LLM agents.
    Mcp {
        #[arg(short, long)]
        rom: Option<String>,
    },
    /// MCP bridge (stdio) to the running fc-tauri dev app (eval JS / screenshot).
    TauriBridge,
}

/// Run a blargg-style self-checking ROM until it reports a result via $6000.
/// Returns (status, message): status 0x00 = pass, 0xFF = timeout, else fail code.
fn run_blargg(deck: &mut ControlDeck, max_frames: u64) -> (u8, String) {
    let mut f = 0;
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
                return (s, msg);
            }
        }
    }
    (0xFF, "timeout".into())
}

fn short(path: &str) -> String {
    let p = std::path::Path::new(path);
    let parent = p.parent().and_then(|x| x.file_name()).map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let name = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
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
            wav,
        } => {
            let data = std::fs::read(&rom)?;
            let mut deck = ControlDeck::new(Region::from_str(&region));
            deck.load_rom(&data)?;
            if let Some(p) = palette {
                let pd = std::fs::read(&p)?;
                if deck.load_palette_file(&pd) {
                    log::info!("loaded palette {p}");
                }
            }
            let mut audio: Vec<f32> = Vec::new();
            let start = std::time::Instant::now();
            for f in 0..frames {
                if autostart {
                    // Tap Start to enter the level, then hold Right to walk/scroll.
                    deck.set_button(0, fc_core::Button::Start, (60..64).contains(&f));
                    deck.set_button(0, fc_core::Button::Right, f >= 90);
                }
                deck.run_frame();
                let samples = deck.drain_audio();
                if wav.is_some() {
                    audio.extend_from_slice(&samples);
                }
            }
            let secs = start.elapsed().as_secs_f64();
            let fb = deck.frame_buffer();
            let non_black = fb.chunks(4).filter(|c| c[0] | c[1] | c[2] != 0).count();
            println!("frames={frames} time={secs:.3}s fps={:.1}", frames as f64 / secs);
            println!("non-black pixels: {non_black}/{}", 256 * 240);
            println!("{}", deck.cpu_state_string());
            if let Some(path) = shot {
                write_png(&path, deck.frame_buffer())?;
                println!("wrote {path}");
            }
            if let Some(path) = wav {
                write_wav(&path, &audio, 44_100)?;
                let rms = (audio.iter().map(|s| s * s).sum::<f32>() / audio.len().max(1) as f32).sqrt();
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
            println!("result $0002={code:04X} ({})", if code == 0 { "PASS" } else { "see code" });
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
            println!("format:   {}", if c.is_nes20 { "NES 2.0" } else { "iNES" });
            println!("PRG-ROM:  {} KB", c.prg_rom.len() / 1024);
            println!(
                "CHR:      {} KB ({})",
                if c.uses_chr_ram { c.chr_ram.len() } else { c.chr_rom.len() } / 1024,
                if c.uses_chr_ram { "CHR-RAM" } else { "CHR-ROM" }
            );
            println!("mirroring:{:?}", c.mirroring());
            println!("battery:  {}", c.has_battery);
        }
        Commands::Testsuite { roms, frames } => {
            let mut pass = 0;
            let mut total = 0;
            for rom in &roms {
                total += 1;
                let data = match std::fs::read(rom) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("  {:<48} ERR  {e}", short(rom));
                        continue;
                    }
                };
                let mut deck = ControlDeck::new(Region::Ntsc);
                if deck.load_rom(&data).is_err() {
                    println!("  {:<48} ERR  bad ROM", short(rom));
                    continue;
                }
                let (status, msg) = run_blargg(&mut deck, frames);
                let verdict = match status {
                    0x00 => {
                        pass += 1;
                        "PASS"
                    }
                    0xFF => "TIMEOUT",
                    _ => "FAIL",
                };
                println!("  {:<48} {verdict:<8} {}", short(rom), msg.trim());
            }
            println!("\n  {pass}/{total} passed");
        }
        Commands::Dbg { rom, frames, autostart } => {
            let data = std::fs::read(&rom)?;
            let mut deck = ControlDeck::new(Region::Ntsc);
            deck.load_rom(&data)?;
            for f in 0..frames {
                if autostart {
                    deck.set_button(0, fc_core::Button::Start, (60..64).contains(&f));
                    deck.set_button(0, fc_core::Button::Right, f >= 90);
                }
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
