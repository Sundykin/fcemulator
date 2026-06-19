//! M2 · 内置精简 2A03 tracker(audio-tracker capability)后端。
//!
//! 乐曲模型 + 播放引擎:把"音符 + 乐器包络"逐帧解算为 APU 寄存器写入,驱动
//! `fc_core::ApuPreview`(自研 APU 内核)渲染样本——这是"内核即试听/校验"的
//! 护城河。Stage 1 覆盖 Pulse1/Pulse2/Triangle/Noise + 音量/琶音/占空比包络;
//! DPCM 与高级效果留 stage 2。

use crate::project::{self, ProjectState};
use fc_core::{ApuPreview, Region};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

/// 单元格音符约定:0 = 空(不变),255 = 音符停止(note-off),1..=96 = 音符。
pub const NOTE_EMPTY: u8 = 0;
pub const NOTE_OFF: u8 = 255;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    /// 逐帧音量包络(0–15);到末尾保持最后值。
    #[serde(default)]
    pub volume: Vec<u8>,
    /// 逐帧琶音半音偏移;到末尾保持最后值。
    #[serde(default)]
    pub arpeggio: Vec<i8>,
    /// 脉冲占空比 0–3。
    #[serde(default)]
    pub duty: u8,
}

impl Default for Instrument {
    fn default() -> Self {
        Instrument { name: "inst".into(), volume: vec![15], arpeggio: vec![0], duty: 2 }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Cell {
    pub note: u8,
    pub instrument: u8,
    /// 0 = 无,1..=16 → 音量 0..=15。
    pub volume: u8,
    /// 效果类型:0 = 无,1 = 琶音(param = xy 半音偏移)。
    #[serde(default)]
    pub fx: u8,
    #[serde(default)]
    pub param: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// rows × 5 通道(P1 P2 TRI NOISE DMC)。
    pub rows: Vec<[Cell; 5]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub name: String,
    /// 每行帧数(速度);60fps 下 6 ≈ 默认。
    pub frames_per_row: u32,
    pub rows_per_pattern: usize,
    pub instruments: Vec<Instrument>,
    pub patterns: Vec<Pattern>,
    /// 播放顺序(pattern 下标)。
    pub order: Vec<usize>,
}

impl Song {
    pub fn blank() -> Song {
        let rows = 64;
        Song {
            name: "song".into(),
            frames_per_row: 6,
            rows_per_pattern: rows,
            instruments: vec![Instrument::default()],
            patterns: vec![Pattern { rows: vec![[Cell::default(); 5]; rows] }],
            order: vec![0],
        }
    }
}

// ----------------------------------------------------------- note → period

/// 音符(1..=96)→ 频率(12-TET,note 1 = C1)。
fn note_freq(note: u8) -> f64 {
    let midi = note as i32 + 23; // note 1 → MIDI 24 (C1)
    440.0 * 2f64.powf((midi as f64 - 69.0) / 12.0)
}

fn pulse_period(note: u8, cpu_hz: f64) -> u16 {
    let p = (cpu_hz / (16.0 * note_freq(note))).round() as i32 - 1;
    p.clamp(0, 2047) as u16
}
fn triangle_period(note: u8, cpu_hz: f64) -> u16 {
    let p = (cpu_hz / (32.0 * note_freq(note))).round() as i32 - 1;
    p.clamp(0, 2047) as u16
}

fn env_at<T: Copy>(env: &[T], frame: usize, default: T) -> T {
    if env.is_empty() {
        default
    } else {
        env[frame.min(env.len() - 1)]
    }
}

// ----------------------------------------------------------- playback engine

#[derive(Default, Clone, Copy)]
struct ChState {
    playing: bool,
    note: u8,
    inst: u8,
    frame: usize, // frames since note start (envelope index)
    last_hi: u8,
    fx: u8,
    param: u8,
}

fn cpu_hz_for(region: Region) -> f64 {
    match region {
        Region::Pal => 1_662_607.0,
        _ => 1_789_773.0,
    }
}

/// Per-frame APU register writes for the whole song (the single source of truth
/// shared by preview render AND asm export — so an exported ROM sounds identical
/// to the in-IDE preview). Each frame is a list of (register, value).
pub fn song_frames(song: &Song, region: Region) -> Vec<Vec<(u16, u8)>> {
    let cpu_hz = cpu_hz_for(region);
    let mut ch = [ChState::default(); 5];
    let mut frames = Vec::new();
    for &pat_idx in &song.order {
        let Some(pat) = song.patterns.get(pat_idx) else { continue };
        for row in &pat.rows {
            for c in 0..5 {
                let cell = row[c];
                if cell.note == NOTE_OFF {
                    ch[c].playing = false;
                } else if cell.note != NOTE_EMPTY {
                    ch[c] = ChState {
                        playing: true,
                        note: cell.note,
                        inst: cell.instrument,
                        frame: 0,
                        last_hi: 0xff,
                        fx: cell.fx,
                        param: cell.param,
                    };
                }
            }
            for _f in 0..song.frames_per_row {
                let mut fw = Vec::new();
                for c in 0..5 {
                    fw.extend(channel_writes(c, &mut ch[c], song, cpu_hz));
                }
                frames.push(fw);
                for c in 0..5 {
                    if ch[c].playing {
                        ch[c].frame += 1;
                    }
                }
            }
        }
    }
    frames
}

/// 渲染整首乐曲为 PCM 样本(驱动 ApuPreview)。供离线试听与单元测试。
pub fn render_song(song: &Song, region: Region, sample_rate: f64) -> Vec<f32> {
    let cycles_per_frame = (cpu_hz_for(region) / 60.0) as u32;
    let mut apu = ApuPreview::new(region, sample_rate);
    let mut out = Vec::new();
    for fw in song_frames(song, region) {
        for (reg, val) in fw {
            apu.write_register(reg, val);
        }
        apu.tick_cycles(cycles_per_frame);
        out.extend(apu.drain_samples());
    }
    out
}

/// 一个通道本帧的 APU 寄存器写入(纯函数,会更新 `st.last_hi`)。
fn channel_writes(c: usize, st: &mut ChState, song: &Song, cpu_hz: f64) -> Vec<(u16, u8)> {
    let mut w = Vec::new();
    let inst = song.instruments.get(st.inst as usize);
    let vol = if st.playing {
        inst.map(|i| env_at(&i.volume, st.frame, 15)).unwrap_or(15)
    } else {
        0
    };
    let arp = inst.map(|i| env_at(&i.arpeggio, st.frame, 0)).unwrap_or(0);
    let duty = inst.map(|i| i.duty & 3).unwrap_or(2);
    // cell effect: arpeggio (fx 1, param xy) cycles base / +x / +y each frame
    let mut note_off = arp as i32;
    if st.fx == 1 {
        let steps = [0i32, (st.param >> 4) as i32, (st.param & 0x0f) as i32];
        note_off += steps[st.frame % 3];
    }
    let eff_note = (st.note as i32 + note_off).clamp(1, 96) as u8;

    match c {
        0 | 1 => {
            let base = if c == 0 { 0x4000 } else { 0x4004 };
            w.push((base, ((duty as u16) << 6) as u8 | 0x30 | (vol & 0x0f)));
            if st.playing {
                let period = pulse_period(eff_note, cpu_hz);
                w.push((base + 2, (period & 0xff) as u8));
                let hi = ((period >> 8) & 0x07) as u8;
                if hi != st.last_hi {
                    w.push((base + 3, hi | (0x08 << 3)));
                    st.last_hi = hi;
                }
            }
        }
        2 => {
            if st.playing && vol > 0 {
                w.push((0x4008, 0x7f));
                let period = triangle_period(eff_note, cpu_hz);
                w.push((0x400a, (period & 0xff) as u8));
                let hi = ((period >> 8) & 0x07) as u8;
                if hi != st.last_hi {
                    w.push((0x400b, hi | (0x08 << 3)));
                    st.last_hi = hi;
                }
            } else {
                w.push((0x4008, 0x00));
            }
        }
        3 => {
            w.push((0x400c, 0x30 | (vol & 0x0f)));
            if st.playing {
                let pidx = ((eff_note as u16).wrapping_sub(1) % 16) as u8;
                w.push((0x400e, pidx));
                if st.last_hi == 0xff {
                    w.push((0x400f, 0x08 << 3));
                    st.last_hi = 0;
                }
            }
        }
        _ => {} // DMC: stage 2
    }
    w
}

/// 导出乐曲为 ca65 数据:逐帧寄存器写入流。格式(被 `fc_player.s` 引擎回放):
///   每帧:`.byte N, reg0, val0, reg1, val1, ...`(reg = $40xx 的低字节);
///   末尾:`.byte $FF`(引擎据此循环回到开头)。
pub fn export_ca65(song: &Song, region: Region) -> String {
    let frames = song_frames(song, region);
    let mut s = String::from(
        "; 自动生成:内置 tracker 乐曲(逐帧 APU 寄存器流)。由 fc_player.s 回放。\n\
         .export song_data\n.segment \"CODE\"\nsong_data:\n",
    );
    for fw in &frames {
        let n = fw.len().min(254);
        let mut line = format!("    .byte {}", n);
        for (reg, val) in fw.iter().take(n) {
            line.push_str(&format!(",${:02X},${:02X}", (reg & 0xff) as u8, val));
        }
        s.push_str(&line);
        s.push('\n');
    }
    s.push_str("    .byte $FF   ; end → loop\n");
    s
}

// --------------------------------------------------------------- persistence

fn resolve(root: &Path, rel: &str) -> Result<std::path::PathBuf, String> {
    if Path::new(rel).is_absolute() || rel.split('/').any(|c| c == "..") {
        return Err("路径必须相对工程根且不得越界".into());
    }
    Ok(root.join(rel))
}

// --------------------------------------------------------------- commands

#[tauri::command]
pub fn tracker_save(rel_path: String, song: Song, state: State<ProjectState>) -> Result<(), String> {
    let root = state.active_root()?;
    let p = resolve(&root, &rel_path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&song).map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(&p, json).map_err(|e| format!("写入 {rel_path} 失败: {e}"))
}

#[tauri::command]
pub fn tracker_load(rel_path: String, state: State<ProjectState>) -> Result<Song, String> {
    let root = state.active_root()?;
    let text = std::fs::read_to_string(resolve(&root, &rel_path)?)
        .map_err(|e| format!("读取 {rel_path} 失败: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("解析乐曲失败: {e}"))
}

/// 渲染乐曲为 PCM(f32 LE 裸字节),前端经 Web Audio 试听。
#[tauri::command]
pub fn tracker_render(song: Song, sample_rate: f64) -> tauri::ipc::Response {
    let samples = render_song(&song, Region::Ntsc, sample_rate);
    let mut bytes = Vec::with_capacity(samples.len() * 4);
    for s in samples {
        bytes.extend_from_slice(&s.to_le_bytes());
    }
    tauri::ipc::Response::new(bytes)
}

// --------------------------------------------------------- FTM text import

/// FamiTracker 文本音符记号 → 本模型音符值(0=空,255=停止)。
fn ftm_note(tok: &str) -> Option<u8> {
    match tok {
        "..." | "" => Some(NOTE_EMPTY),
        "---" | "===" => Some(NOTE_OFF), // halt / release → note-off
        _ => {
            let b = tok.as_bytes();
            if b.len() < 3 {
                return None;
            }
            let semi = match (b[0] as char, b[1] as char) {
                ('C', '-') => 0, ('C', '#') => 1, ('D', '-') => 2, ('D', '#') => 3,
                ('E', '-') => 4, ('F', '-') => 5, ('F', '#') => 6, ('G', '-') => 7,
                ('G', '#') => 8, ('A', '-') => 9, ('A', '#') => 10, ('B', '-') => 11,
                _ => return None,
            };
            let oct = (b[2] as char).to_digit(10)? as i32;
            Some((oct * 12 + semi + 1).clamp(1, 96) as u8)
        }
    }
}

/// 解析 FamiTracker 文本导出(基础保真:2A03 五通道、音符/乐器/音量;忽略
/// 扩展芯片与多数效果)。
pub fn parse_ftm_text(text: &str) -> Result<Song, String> {
    use std::collections::HashMap;
    let mut song = Song::blank();
    song.patterns.clear();
    let mut order_raw: Vec<u32> = Vec::new();
    let mut pat_map: HashMap<u32, usize> = HashMap::new();
    let mut cur: Option<usize> = None;
    let mut rows_per = 64usize;

    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("TRACK") {
            let p: Vec<&str> = rest.split_whitespace().collect();
            if p.len() >= 2 {
                rows_per = p[0].parse().unwrap_or(64);
                song.frames_per_row = p[1].parse().unwrap_or(6);
                song.rows_per_pattern = rows_per;
            }
        } else if t.starts_with("ORDER") {
            if let Some(rhs) = t.split(':').nth(1) {
                if let Some(first) = rhs.split_whitespace().next() {
                    if let Ok(n) = u32::from_str_radix(first, 16) {
                        order_raw.push(n);
                    }
                }
            }
        } else if let Some(rest) = t.strip_prefix("PATTERN") {
            let num = rest
                .split_whitespace()
                .next()
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .unwrap_or(pat_map.len() as u32);
            let idx = song.patterns.len();
            pat_map.insert(num, idx);
            song.patterns.push(Pattern { rows: vec![[Cell::default(); 5]; rows_per] });
            cur = Some(idx);
        } else if t.starts_with("ROW") {
            if let Some(idx) = cur {
                let segs: Vec<&str> = t.split(':').collect();
                let rownum = segs[0]
                    .split_whitespace()
                    .nth(1)
                    .and_then(|h| u32::from_str_radix(h, 16).ok())
                    .unwrap_or(0) as usize;
                if rownum < rows_per {
                    for c in 0..5 {
                        if let Some(field) = segs.get(1 + c) {
                            let toks: Vec<&str> = field.split_whitespace().collect();
                            if let Some(nt) = toks.first() {
                                if let Some(note) = ftm_note(nt) {
                                    let inst = toks
                                        .get(1)
                                        .and_then(|s| u8::from_str_radix(s, 16).ok())
                                        .unwrap_or(0);
                                    let vol = toks
                                        .get(2)
                                        .and_then(|s| u8::from_str_radix(s, 16).ok())
                                        .map(|v| v + 1)
                                        .unwrap_or(0);
                                    song.patterns[idx].rows[rownum][c] =
                                        Cell { note, instrument: inst, volume: vol, ..Default::default() };
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if song.patterns.is_empty() {
        return Err("未解析到任何 PATTERN(请确认是 FamiTracker 文本导出)".into());
    }
    // resolve order (raw FTM pattern numbers → our indices); default = all in order
    song.order = order_raw
        .iter()
        .filter_map(|n| pat_map.get(n).copied())
        .collect();
    if song.order.is_empty() {
        song.order = (0..song.patterns.len()).collect();
    }
    Ok(song)
}

#[tauri::command]
pub fn tracker_import_ftm(src_path: String) -> Result<Song, String> {
    let text = std::fs::read_to_string(&src_path).map_err(|e| format!("读取 {src_path} 失败: {e}"))?;
    parse_ftm_text(&text)
}

/// 捆绑的回放引擎 asm 路径。
fn engine_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/fc-player/fc_player.s")
}

/// 导出乐曲为 ca65(`out_rel`)+ 捆绑回放引擎到 `music/fc_player.s`,并登记进
/// 工程(→ 被 build-pipeline 汇编链接进 `.nes`)。主程序需 `jsr fc_player_init`
/// 于 reset、`jsr fc_player_tick` 于 NMI。
#[tauri::command]
pub fn tracker_export(out_rel: String, song: Song, state: State<ProjectState>) -> Result<(), String> {
    let root = state.active_root()?;
    let out = resolve(&root, &out_rel)?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&out, export_ca65(&song, Region::Ntsc))
        .map_err(|e| format!("写入 {out_rel} 失败: {e}"))?;

    let eng_rel = "music/fc_player.s";
    let eng_dst = resolve(&root, eng_rel)?;
    if let Some(parent) = eng_dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::copy(engine_path(), &eng_dst).map_err(|e| format!("拷入回放引擎失败: {e}"))?;

    let mut manifest = project::load_manifest(&root)?;
    for f in [out_rel.clone(), eng_rel.to_string()] {
        if !manifest.music.contains(&f) {
            manifest.music.push(f);
        }
    }
    project::save_manifest(&root, &manifest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_note_song() -> Song {
        let mut s = Song::blank();
        s.frames_per_row = 6;
        s.rows_per_pattern = 4;
        let mut rows = vec![[Cell::default(); 5]; 4];
        rows[0][0] = Cell { note: 49, instrument: 0, volume: 16, ..Default::default() }; // a mid note on pulse1
        s.patterns = vec![Pattern { rows }];
        s.order = vec![0];
        s
    }

    #[test]
    fn render_produces_audio() {
        let s = one_note_song();
        let samples = render_song(&s, Region::Ntsc, 44_100.0);
        assert!(!samples.is_empty(), "应产生样本");
        assert!(samples.iter().any(|&v| v.abs() > 0.001), "应有非零输出");
    }

    #[test]
    fn song_json_roundtrip() {
        let s = one_note_song();
        let j = serde_json::to_string(&s).unwrap();
        let back: Song = serde_json::from_str(&j).unwrap();
        assert_eq!(back.frames_per_row, s.frames_per_row);
        assert_eq!(back.patterns[0].rows[0][0].note, 49);
        assert_eq!(back.order, s.order);
    }

    #[test]
    fn note_periods_descend_with_pitch() {
        // higher note → smaller timer period
        let lo = pulse_period(20, 1_789_773.0);
        let hi = pulse_period(60, 1_789_773.0);
        assert!(hi < lo);
    }

    #[test]
    fn ftm_text_imports() {
        let ftm = "\
# FamiTracker text export\n\
TRACK  16   7 150 \"test\"\n\
ORDER 00 : 00 00 00 00 00\n\
PATTERN 00\n\
ROW 00 : C-4 00 F ... : ... .. . ... : E-3 00 . ... : ... .. . ... : ... .. . ...\n\
ROW 01 : --- .. . ... : ... .. . ... : ... .. . ... : ... .. . ... : ... .. . ...\n";
        let song = parse_ftm_text(ftm).unwrap();
        assert_eq!(song.frames_per_row, 7);
        assert_eq!(song.patterns.len(), 1);
        assert_eq!(song.order, vec![0]);
        // C-4 on pulse1 row0
        assert_eq!(song.patterns[0].rows[0][0].note, 4 * 12 + 0 + 1);
        assert_eq!(song.patterns[0].rows[0][0].volume, 0x0F + 1);
        // E-3 on triangle row0
        assert_eq!(song.patterns[0].rows[0][2].note, 3 * 12 + 4 + 1);
        // note-off on pulse1 row1
        assert_eq!(song.patterns[0].rows[1][0].note, NOTE_OFF);
    }

    #[test]
    fn arpeggio_effect_changes_pitch_per_frame() {
        let mut s = Song::blank();
        s.frames_per_row = 3;
        s.rows_per_pattern = 1;
        // arpeggio 0x0C07: base, +12, +7 each frame
        let mut rows = vec![[Cell::default(); 5]; 1];
        rows[0][0] = Cell { note: 40, instrument: 0, volume: 16, fx: 1, param: 0xC7 };
        s.patterns = vec![Pattern { rows }];
        s.order = vec![0];
        let frames = song_frames(&s, Region::Ntsc);
        assert_eq!(frames.len(), 3);
        // pulse1 timer-low ($4002) should differ across the 3 arpeggio frames
        let lo = |f: &Vec<(u16, u8)>| f.iter().find(|&&(r, _)| r == 0x4002).map(|&(_, v)| v);
        assert!(lo(&frames[0]) != lo(&frames[1]) || lo(&frames[1]) != lo(&frames[2]), "琶音应逐帧改变音高");
    }

    #[test]
    fn export_emits_frame_stream() {
        let frames = song_frames(&one_note_song(), Region::Ntsc);
        assert!(!frames.is_empty());
        // first frame should contain pulse1 writes (note triggered on row 0)
        assert!(frames[0].iter().any(|&(r, _)| r == 0x4000));
        let asm = export_ca65(&one_note_song(), Region::Ntsc);
        assert!(asm.contains("song_data:"));
        assert!(asm.contains(".byte $FF"));
    }

    /// 6.6 验收:导出的乐曲 + 捆绑引擎 + 驱动它的主程序能汇编+链接成 .nes。
    #[test]
    fn export_assembles_and_links() {
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;
        if !crate::build_pipeline::tool_available("ca65") || !crate::build_pipeline::tool_available("ld65") {
            eprintln!("跳过:本平台未 vendored cc65");
            return;
        }
        let tmp = std::env::temp_dir().join(format!("fc-trk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let mut manifest = crate::project::create_from_template(&tmp, "trk", "demo").unwrap();
        // main that drives the player (init at reset, tick in NMI)
        let main = "\
.import fc_player_init, fc_player_tick\n\
.segment \"CODE\"\n\
reset:\n  sei\n  cld\n  ldx #$ff\n  txs\n  jsr fc_player_init\n  lda #$80\n  sta $2000\n\
loop:\n  jmp loop\n\
nmi:\n  jsr fc_player_tick\n  rti\n\
irq:\n  rti\n\
.segment \"VECTORS\"\n  .word nmi, reset, irq\n\
.segment \"CHARS\"\n  .res 8192, $00\n";
        std::fs::write(tmp.join("src/main.s"), main).unwrap();
        std::fs::write(tmp.join("music/song.s"), export_ca65(&one_note_song(), Region::Ntsc)).unwrap();
        std::fs::copy(engine_path(), tmp.join("music/fc_player.s")).unwrap();
        manifest.music = vec!["music/song.s".into(), "music/fc_player.s".into()];
        let result = crate::build_pipeline::run_build(&tmp, &manifest, Arc::new(AtomicBool::new(false)));
        assert!(result.success, "导出曲应汇编链接成功,日志:\n{}", result.log);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
