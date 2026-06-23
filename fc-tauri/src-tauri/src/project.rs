//! M1 创作最小闭环 · 工程模型(project-model capability)。
//!
//! 一个 NES 工程由根目录下的 `project.toml` 声明其全部构建上下文,作为可重放
//! 构建的唯一事实源(见 design.md D3)。本模块负责:工程清单的数据模型与
//! 校验、新建/打开/保存、内置模板、以及受工程根约束的文件树操作。
//!
//! 约定目录:`src/`(.s/.asm) `chr/` `music/` `map/` `build/`(产物)。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

/// 项目内核已实现的 Mapper(见 CLAUDE.md / mapper.rs)。打包/构建只允许这些。
const SUPPORTED_MAPPERS: &[u16] = &[0, 1, 2, 3, 4, 7, 9, 10, 11, 66, 71];

/// 约定的工程子目录,新建工程时一并创建。
const SCAFFOLD_DIRS: &[&str] = &["src", "chr", "music", "map", "build"];

// ----------------------------------------------------------------- manifest

/// iNES 头字段(与 rom-packaging 共享;打包时据此写 `.nes` 头)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InesHeader {
    /// Mapper 编号(必须是内核支持的)。
    pub mapper: u16,
    /// PRG ROM 大小,单位 16KB bank,至少 1。
    pub prg_banks: u8,
    /// CHR ROM 大小,单位 8KB bank;0 表示使用 CHR RAM。
    pub chr_banks: u8,
    /// 镜像方式:`"horizontal"` 或 `"vertical"`。
    pub mirroring: String,
    /// 电池 SRAM。
    pub battery: bool,
}

impl Default for InesHeader {
    fn default() -> Self {
        InesHeader {
            mapper: 0,
            prg_banks: 2,
            chr_banks: 1,
            mirroring: "vertical".into(),
            battery: false,
        }
    }
}

/// 工程清单 = `project.toml` 的内存模型。
///
/// 字段顺序刻意把标量/数组放前、`[ines]` 表放最后,以满足 TOML "表必须在
/// 值之后" 的序列化约束。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    /// 工程名。
    pub name: String,
    /// 源码文件(相对工程根),交给 ca65 汇编。
    #[serde(default)]
    pub sources: Vec<String>,
    /// CHR 资源(相对工程根)。
    #[serde(default)]
    pub chr: Vec<String>,
    /// 音乐资源(相对工程根;M2 FamiStudio 导出落点)。
    #[serde(default)]
    pub music: Vec<String>,
    /// 地图/命名表资源(相对工程根)。
    #[serde(default)]
    pub maps: Vec<String>,
    /// 自定义 ld65 链接脚本(相对工程根);缺省则按 Mapper 选捆绑 `.cfg`。
    #[serde(default)]
    pub linker_cfg: Option<String>,
    /// 输出 `.nes` 路径(相对工程根)。
    #[serde(default = "default_output")]
    pub output: String,
    /// 地图资源 → CHR 资源的编辑器绑定,用于恢复地图编辑预览上下文。
    #[serde(default)]
    pub map_chr: BTreeMap<String, String>,
    /// iNES 头。
    #[serde(default)]
    pub ines: InesHeader,
}

fn default_output() -> String {
    "build/game.nes".into()
}

impl ProjectManifest {
    /// 校验字段合法性,返回首个字段级错误(见 spec: 非法字段返回字段级错误)。
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name: 工程名不能为空".into());
        }
        if !SUPPORTED_MAPPERS.contains(&self.ines.mapper) {
            return Err(format!(
                "ines.mapper: 不支持的 Mapper {}(内核支持 {:?})",
                self.ines.mapper, SUPPORTED_MAPPERS
            ));
        }
        if self.ines.prg_banks == 0 {
            return Err("ines.prg_banks: PRG 至少 1 个 16KB bank".into());
        }
        match self.ines.mirroring.as_str() {
            "horizontal" | "vertical" => {}
            other => {
                return Err(format!(
                    "ines.mirroring: 非法镜像 \"{other}\"(应为 horizontal/vertical)"
                ))
            }
        }
        if self.output.trim().is_empty() {
            return Err("output: 输出路径不能为空".into());
        }
        for (map, chr) in &self.map_chr {
            if map.trim().is_empty() || chr.trim().is_empty() {
                return Err("map_chr: 地图与 CHR 绑定路径不能为空".into());
            }
        }
        Ok(())
    }
}

// --------------------------------------------------------------- load / save

const MANIFEST_NAME: &str = "project.toml";

/// 解析工程根下的 `project.toml`:缺失字段由 serde default 补全,语法错误与
/// 字段非法都返回明确错误,且不会进入"已加载工程"状态。
pub fn load_manifest(root: &Path) -> Result<ProjectManifest, String> {
    let p = root.join(MANIFEST_NAME);
    let text = std::fs::read_to_string(&p)
        .map_err(|e| format!("读取 {} 失败: {e}", p.display()))?;
    let manifest: ProjectManifest =
        toml::from_str(&text).map_err(|e| format!("解析 project.toml 失败: {e}"))?;
    manifest.validate()?;
    Ok(manifest)
}

/// 写回 `project.toml`,保证读回一致(序列化—反序列化往返)。
pub fn save_manifest(root: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    manifest.validate()?;
    let text = toml::to_string_pretty(manifest)
        .map_err(|e| format!("序列化 project.toml 失败: {e}"))?;
    std::fs::write(root.join(MANIFEST_NAME), text)
        .map_err(|e| format!("写入 project.toml 失败: {e}"))
}

// ----------------------------------------------------------------- templates

/// 内置工程模板。每个模板含一份可直接成功构建的最小 NROM 骨架 + 链接脚本。
pub struct Template {
    pub manifest: ProjectManifest,
    /// (相对路径, 内容)——除约定目录外要落盘的文件。
    pub files: Vec<(&'static str, String)>,
    /// (相对路径, 内容)——模板附带的二进制资源。
    pub binary_files: Vec<(&'static str, Vec<u8>)>,
}

/// 标准 NROM ld65 链接脚本(ca65 + ld65 可直接链接)。
fn nrom_cfg() -> String {
    String::from(
        r#"# 捆绑 NROM 链接脚本(M1 模板用)。
# 注意:不含 HEADER 段——iNES 头由构建编排器据 project.toml 权威生成并前置。
MEMORY {
    ZP:     start = $0000, size = $0100, type = rw, file = "";
    PRG:    start = $8000, size = $8000, type = ro, file = %O, fill = yes;
    CHR:    start = $0000, size = $2000, type = ro, file = %O, fill = yes;
}
SEGMENTS {
    CODE:     load = PRG,    type = ro,  start = $8000;
    RODATA:   load = PRG,    type = ro;
    VECTORS:  load = PRG,    type = ro,  start = $FFFA;
    CHARS:    load = CHR,    type = ro;
    ZEROPAGE: load = ZP,     type = zp;
}
"#,
    )
}

/// 一个最小、可汇编/链接通过的 NROM 主程序骨架(ca65 语法)。
fn nrom_main(comment: &str) -> String {
    format!(
        r#"; {comment}
; M1 模板:最小 NROM 骨架,ca65 + ld65(nrom.cfg)可直接构建。
; iNES 头不写在这里——由构建编排器据 project.toml 的 [ines] 权威生成。

.segment "CODE"
reset:
    sei                ; 关中断
    cld                ; 关十进制
    ldx #$ff
    txs                ; 初始化栈
forever:
    jmp forever        ; 主循环占位

nmi:
    rti

irq:
    rti

.segment "VECTORS"
    .word nmi
    .word reset
    .word irq

.segment "CHARS"
    .res 8192, $00     ; 空 CHR(占位 8KB)
"#
    )
}

fn encode_chr_tile(pixels: &[u8; 64]) -> [u8; 16] {
    let mut out = [0u8; 16];
    for row in 0..8 {
        let mut lo = 0u8;
        let mut hi = 0u8;
        for x in 0..8 {
            let p = pixels[row * 8 + x] & 0b11;
            lo |= (p & 1) << (7 - x);
            hi |= ((p >> 1) & 1) << (7 - x);
        }
        out[row] = lo;
        out[8 + row] = hi;
    }
    out
}

fn solid_tile(value: u8) -> [u8; 64] {
    [value & 0b11; 64]
}

fn circle_tile() -> [u8; 64] {
    let mut px = [0u8; 64];
    for y in 0..8 {
        for x in 0..8 {
            let dx = x as i32 - 3;
            let dy = y as i32 - 3;
            px[y * 8 + x] = if dx * dx + dy * dy <= 13 { 1 } else { 0 };
        }
    }
    px
}

fn diamond_tile() -> [u8; 64] {
    let mut px = [0u8; 64];
    for y in 0..8 {
        for x in 0..8 {
            let dx = (x as i32 - 3).abs();
            let dy = (y as i32 - 3).abs();
            px[y * 8 + x] = if dx + dy <= 4 { 2 } else { 0 };
        }
    }
    px
}

fn border_tile() -> [u8; 64] {
    let mut px = [0u8; 64];
    for y in 0..8 {
        for x in 0..8 {
            px[y * 8 + x] = if x == 0 || y == 0 || x == 7 || y == 7 { 1 } else { 0 };
        }
    }
    px
}

fn dot_tile() -> [u8; 64] {
    let mut px = [0u8; 64];
    for y in 0..8 {
        for x in 0..8 {
            px[y * 8 + x] = if (x + y) % 4 == 0 { 1 } else { 0 };
        }
    }
    px
}

fn demo_chr_bytes() -> Vec<u8> {
    let mut out = Vec::with_capacity(8192);
    let mut push = |tile: [u8; 64]| out.extend_from_slice(&encode_chr_tile(&tile));
    push(solid_tile(0)); // 0 blank
    for _ in 0..4 {
        push(circle_tile()); // 1-4 player sprite quadrants
    }
    for _ in 0..4 {
        push(diamond_tile()); // 5-8 target sprite quadrants
    }
    push(border_tile()); // 9 map frame tile
    push(dot_tile()); // 10 subtle floor/detail tile
    out.resize(8192, 0);
    out
}

fn demo_map_bytes() -> Vec<u8> {
    let w = 32usize;
    let h = 30usize;
    let mut tiles = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            tiles[idx] = if x == 0 || x == w - 1 || y == 0 || y == h - 1 {
                9
            } else if (x + y) % 7 == 0 {
                10
            } else {
                0
            };
        }
    }
    let attrs = vec![0u8; w.div_ceil(2) * h.div_ceil(2)];
    let collision = vec![0u8; w * h];
    let mut out = Vec::with_capacity(4 + tiles.len() + attrs.len() + collision.len());
    out.extend_from_slice(&(w as u16).to_le_bytes());
    out.extend_from_slice(&(h as u16).to_le_bytes());
    out.extend_from_slice(&tiles);
    out.extend_from_slice(&attrs);
    out.extend_from_slice(&collision);
    out
}

/// A tiny playable NROM game template. It intentionally stays in one source file
/// so the first creative-mode project is easy for an AI/user to read, modify,
/// build, and validate.
fn nrom_simple_game(comment: &str) -> String {
    format!(
        r#"; {comment}
; Tiny NROM game template for ca65 + ld65.
; Goal: move the blue player with the D-pad and catch the gold target.
; iNES header is generated from project.toml by the build pipeline.

.segment "ZEROPAGE"
player_x:       .res 1
player_y:       .res 1
target_x:       .res 1
target_y:       .res 1
frame_counter:  .res 1
pad_state:      .res 1
nmi_ready:      .res 1
hit_flash:      .res 1
candidate_x:   .res 1
candidate_y:   .res 1
tile_x:         .res 1
tile_y:         .res 1
tile_index_lo:  .res 1
tile_index_hi:  .res 1
collision_hit:  .res 1

.segment "CODE"

PPUCTRL   = $2000
PPUMASK   = $2001
PPUSTATUS = $2002
OAMADDR   = $2003
PPUSCROLL = $2005
PPUADDR   = $2006
PPUDATA   = $2007
OAMDMA    = $4014
JOY1      = $4016

reset:
    sei
    cld
    ldx #$40
    stx $4017
    ldx #$ff
    txs
    inx
    stx PPUCTRL
    stx PPUMASK
    stx $4010

vblank_wait_1:
    bit PPUSTATUS
    bpl vblank_wait_1
vblank_wait_2:
    bit PPUSTATUS
    bpl vblank_wait_2

    jsr clear_ram
    jsr clear_nametable
    jsr load_palettes
    jsr init_game
    jsr write_sprites

    lda #%10000000      ; enable NMI
    sta PPUCTRL
    lda #%00011110      ; show background + sprites, incl. left edge
    sta PPUMASK

main_loop:
    lda nmi_ready
    beq main_loop
    lda #$00
    sta nmi_ready
    jsr read_controller
    jsr update_game
    jsr write_sprites
    jmp main_loop

clear_ram:
    lda #$00
    tax
clear_ram_loop:
    sta $0000,x
    sta $0200,x
    sta $0300,x
    sta $0400,x
    sta $0500,x
    sta $0600,x
    sta $0700,x
    inx
    bne clear_ram_loop
    rts

clear_nametable:
    bit PPUSTATUS
    lda #$20
    sta PPUADDR
    lda #$00
    sta PPUADDR
    ldx #$00
clear_nt_page0:
    lda map_room_tiles,x
    sta PPUDATA
    inx
    cpx #$f0
    bne clear_nt_page0
    ldx #$00
clear_nt_page1:
    lda map_room_tiles+$00f0,x
    sta PPUDATA
    inx
    cpx #$f0
    bne clear_nt_page1
    ldx #$00
clear_nt_page2:
    lda map_room_tiles+$01e0,x
    sta PPUDATA
    inx
    cpx #$f0
    bne clear_nt_page2
    ldx #$00
clear_nt_page3:
    lda map_room_tiles+$02d0,x
    sta PPUDATA
    inx
    cpx #$f0
    bne clear_nt_page3
    lda #$00
    ldx #$40
clear_attr_loop:
    sta PPUDATA
    dex
    bne clear_attr_loop
    rts

load_palettes:
    bit PPUSTATUS
    lda #$3f
    sta PPUADDR
    lda #$00
    sta PPUADDR
    ldx #$00
palette_loop:
    lda palettes,x
    sta PPUDATA
    inx
    cpx #$20
    bne palette_loop
    rts

init_game:
    lda #$78
    sta player_x
    lda #$b0
    sta player_y
    lda #$70
    sta target_x
    lda #$58
    sta target_y
    lda #$00
    sta frame_counter
    sta hit_flash

    lda #$f0            ; hide unused sprites
    ldx #$00
hide_oam_loop:
    sta $0200,x
    inx
    inx
    inx
    inx
    bne hide_oam_loop
    rts

read_controller:
    lda #$01
    sta JOY1
    lda #$00
    sta JOY1

    lda JOY1            ; A
    lda JOY1            ; B
    lda JOY1            ; Select
    lda JOY1            ; Start

    lda #$00
    sta pad_state

    lda JOY1            ; Up
    and #$01
    beq no_up
    lda pad_state
    ora #$10
    sta pad_state
no_up:
    lda JOY1            ; Down
    and #$01
    beq no_down
    lda pad_state
    ora #$20
    sta pad_state
no_down:
    lda JOY1            ; Left
    and #$01
    beq no_left
    lda pad_state
    ora #$40
    sta pad_state
no_left:
    lda JOY1            ; Right
    and #$01
    beq no_right
    lda pad_state
    ora #$80
    sta pad_state
no_right:
    rts

update_game:
    inc frame_counter

    lda pad_state
    and #$40
    beq skip_move_left
    lda player_x
    cmp #$08
    beq skip_move_left
    sec
    sbc #$02
    sta candidate_x
    lda player_y
    sta candidate_y
    jsr can_move_to
    bne skip_move_left
    lda candidate_x
    sta player_x
skip_move_left:
    lda pad_state
    and #$80
    beq skip_move_right
    lda player_x
    cmp #$f0
    bcs skip_move_right
    clc
    adc #$02
    sta candidate_x
    lda player_y
    sta candidate_y
    jsr can_move_to
    bne skip_move_right
    lda candidate_x
    sta player_x
skip_move_right:
    lda pad_state
    and #$10
    beq skip_move_up
    lda player_y
    cmp #$20
    beq skip_move_up
    sec
    sbc #$02
    sta candidate_y
    lda player_x
    sta candidate_x
    jsr can_move_to
    bne skip_move_up
    lda candidate_y
    sta player_y
skip_move_up:
    lda pad_state
    and #$20
    beq skip_move_down
    lda player_y
    cmp #$d8
    bcs skip_move_down
    clc
    adc #$02
    sta candidate_y
    lda player_x
    sta candidate_x
    jsr can_move_to
    bne skip_move_down
    lda candidate_y
    sta player_y
skip_move_down:

    lda frame_counter
    and #$03
    bne skip_target_drift
    inc target_x
    lda target_x
    cmp #$f0
    bcc skip_target_drift
    lda #$10
    sta target_x
skip_target_drift:

    lda hit_flash
    beq check_collision
    dec hit_flash
check_collision:
    jsr maybe_catch_target
    rts

can_move_to:
    lda #$00
    sta collision_hit

    lda candidate_x
    clc
    adc #$04
    sta tile_x
    lda candidate_y
    clc
    adc #$04
    sta tile_y
    jsr test_collision_point

    lda candidate_x
    clc
    adc #$13
    sta tile_x
    lda candidate_y
    clc
    adc #$04
    sta tile_y
    jsr test_collision_point

    lda candidate_x
    clc
    adc #$04
    sta tile_x
    lda candidate_y
    clc
    adc #$13
    sta tile_y
    jsr test_collision_point

    lda candidate_x
    clc
    adc #$13
    sta tile_x
    lda candidate_y
    clc
    adc #$13
    sta tile_y
    jsr test_collision_point

    lda collision_hit
    rts

test_collision_point:
    lda tile_x
    lsr a
    lsr a
    lsr a
    sta tile_x
    lda tile_y
    lsr a
    lsr a
    lsr a
    sta tile_y

    lda tile_y
    and #$07
    asl a
    asl a
    asl a
    asl a
    asl a
    clc
    adc tile_x
    sta tile_index_lo
    lda tile_y
    lsr a
    lsr a
    lsr a
    sta tile_index_hi

    ldx tile_index_lo
    lda tile_index_hi
    beq collision_page0
    cmp #$01
    beq collision_page1
    cmp #$02
    beq collision_page2
    lda map_room_collision+$0300,x
    bne collision_blocked
    rts
collision_page0:
    lda map_room_collision,x
    bne collision_blocked
    rts
collision_page1:
    lda map_room_collision+$0100,x
    bne collision_blocked
    rts
collision_page2:
    lda map_room_collision+$0200,x
    bne collision_blocked
    rts
collision_blocked:
    lda #$01
    sta collision_hit
    rts

maybe_catch_target:
    lda player_x
    sec
    sbc target_x
    bcs dx_positive
    eor #$ff
    clc
    adc #$01
dx_positive:
    cmp #$12
    bcs no_catch

    lda player_y
    sec
    sbc target_y
    bcs dy_positive
    eor #$ff
    clc
    adc #$01
dy_positive:
    cmp #$12
    bcs no_catch

    lda #$18
    sta hit_flash
    lda frame_counter
    eor #$a5
    and #$7f
    clc
    adc #$40
    sta target_x
    lda frame_counter
    eor #$5a
    and #$5f
    clc
    adc #$50
    sta target_y
no_catch:
    rts

write_sprites:
    lda hit_flash
    beq normal_player_attr
    lda #$02
    bne got_player_attr
normal_player_attr:
    lda #$00
got_player_attr:
    sta $0202
    sta $0206
    sta $020a
    sta $020e

    lda player_y
    sta $0200
    sta $0204
    clc
    adc #$08
    sta $0208
    sta $020c

    lda player_x
    sta $0203
    sta $020b
    clc
    adc #$08
    sta $0207
    sta $020f

    lda #$01
    sta $0201
    lda #$02
    sta $0205
    lda #$03
    sta $0209
    lda #$04
    sta $020d

    lda target_y
    sta $0210
    sta $0214
    clc
    adc #$08
    sta $0218
    sta $021c

    lda target_x
    sta $0213
    sta $021b
    clc
    adc #$08
    sta $0217
    sta $021f

    lda #$01
    sta $0212
    sta $0216
    sta $021a
    sta $021e

    lda #$05
    sta $0211
    lda #$06
    sta $0215
    lda #$07
    sta $0219
    lda #$08
    sta $021d
    rts

nmi:
    pha
    txa
    pha
    tya
    pha

    lda #$00
    sta OAMADDR
    lda #$02
    sta OAMDMA
    lda #$00
    sta PPUSCROLL
    sta PPUSCROLL
    lda #$01
    sta nmi_ready

    pla
    tay
    pla
    tax
    pla
    rti

irq:
    rti

palettes:
    .byte $0f,$11,$21,$30,  $0f,$06,$16,$27
    .byte $0f,$09,$19,$29,  $0f,$0c,$1c,$2c
    .byte $0f,$27,$16,$30,  $0f,$28,$18,$30
    .byte $0f,$2a,$1a,$30,  $0f,$15,$25,$30

.segment "VECTORS"
    .word nmi
    .word reset
    .word irq

.segment "RODATA"
map_room:
    .incbin "map/room.bin"
map_room_tiles = map_room + 4
map_room_collision = map_room + 4 + 960 + 240

.segment "CHARS"
    .incbin "chr/sprites.chr"
"#
    )
}

fn base_manifest(name: &str) -> ProjectManifest {
    ProjectManifest {
        name: name.to_string(),
        sources: vec!["src/main.s".into()],
        chr: vec![],
        music: vec![],
        maps: vec![],
        linker_cfg: None,
        output: default_output(),
        map_chr: BTreeMap::new(),
        ines: InesHeader::default(),
    }
}

/// 解析模板 id → 模板内容。未知 id 返回错误。
pub fn template(id: &str, name: &str) -> Result<Template, String> {
    let (cfg_file, source): (&str, String) = match id {
        "blank" => ("nrom.cfg", nrom_main("空白模板")),
        "horizontal" => (
            "nrom.cfg",
            nrom_simple_game("横版模板(NROM 可玩起步工程)"),
        ),
        "demo" => (
            "nrom.cfg",
            nrom_simple_game("演示模板(Catch the Dot 可玩样例)"),
        ),
        other => return Err(format!("未知模板: {other}(可选 blank/horizontal/demo)")),
    };
    let mut manifest = base_manifest(name);
    manifest.linker_cfg = Some(cfg_file.to_string());
    let resource_backed = matches!(id, "horizontal" | "demo");
    if resource_backed {
        manifest.chr.push("chr/sprites.chr".into());
        manifest.maps.push("map/room.bin".into());
        manifest.map_chr.insert("map/room.bin".into(), "chr/sprites.chr".into());
    }
    Ok(Template {
        manifest,
        files: vec![
            ("src/main.s", source),
            ("nrom.cfg", nrom_cfg()),
        ],
        binary_files: if resource_backed {
            vec![
                ("chr/sprites.chr", demo_chr_bytes()),
                ("map/room.bin", demo_map_bytes()),
            ]
        } else {
            vec![]
        },
    })
}

/// 从模板在 `root` 新建工程:写 project.toml + 约定目录骨架 + 模板文件。
pub fn create_from_template(root: &Path, name: &str, template_id: &str) -> Result<ProjectManifest, String> {
    if root.join(MANIFEST_NAME).exists() {
        return Err(format!("{} 已存在 project.toml,拒绝覆盖", root.display()));
    }
    let tpl = template(template_id, name)?;
    std::fs::create_dir_all(root).map_err(|e| format!("创建工程目录失败: {e}"))?;
    for d in SCAFFOLD_DIRS {
        std::fs::create_dir_all(root.join(d)).map_err(|e| format!("创建目录 {d} 失败: {e}"))?;
    }
    for (rel, content) in &tpl.files {
        let dst = root.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::write(&dst, content).map_err(|e| format!("写入 {rel} 失败: {e}"))?;
    }
    for (rel, bytes) in &tpl.binary_files {
        let dst = root.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::write(&dst, bytes).map_err(|e| format!("写入 {rel} 失败: {e}"))?;
    }
    save_manifest(root, &tpl.manifest)?;
    Ok(tpl.manifest)
}

// ----------------------------------------------------------------- file tree

/// 文件树节点(相对工程根)。
#[derive(Debug, Serialize)]
pub struct FileNode {
    pub name: String,
    /// 相对工程根的路径(POSIX 风格,根为 "")。
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

/// 把用户给的相对路径安全地解析到工程根内,拒绝 `..`/绝对路径越界。
fn resolve_within(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        return Err("路径必须相对工程根".into());
    }
    let mut out = root.to_path_buf();
    for comp in rel_path.components() {
        match comp {
            Component::Normal(c) => out.push(c),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("路径越出工程根".into());
            }
        }
    }
    Ok(out)
}

/// 递归列目录,跳过 `build/` 产物与隐藏文件,生成文件树。
pub fn file_tree(root: &Path) -> Result<FileNode, String> {
    fn walk(abs: &Path, rel: &str) -> FileNode {
        let name = abs
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let is_dir = abs.is_dir();
        let mut children = Vec::new();
        if is_dir {
            if let Ok(entries) = std::fs::read_dir(abs) {
                let mut items: Vec<_> = entries.flatten().collect();
                // 目录在前,再按名称排序,保证稳定显示。
                items.sort_by_key(|e| {
                    let p = e.path();
                    (!p.is_dir(), e.file_name().to_string_lossy().to_lowercase())
                });
                for e in items {
                    let fname = e.file_name().to_string_lossy().to_string();
                    if fname.starts_with('.') {
                        continue; // 跳过隐藏文件
                    }
                    let child_rel = if rel.is_empty() {
                        fname.clone()
                    } else {
                        format!("{rel}/{fname}")
                    };
                    children.push(walk(&e.path(), &child_rel));
                }
            }
        }
        FileNode {
            name,
            path: rel.to_string(),
            is_dir,
            children,
        }
    }
    if !root.exists() {
        return Err(format!("工程目录不存在: {}", root.display()));
    }
    Ok(walk(root, ""))
}

/// 在工程内新建文件或目录。
pub fn create_entry(root: &Path, rel: &str, is_dir: bool) -> Result<(), String> {
    let target = resolve_within(root, rel)?;
    if target.exists() {
        return Err(format!("已存在: {rel}"));
    }
    if is_dir {
        std::fs::create_dir_all(&target).map_err(|e| format!("创建目录失败: {e}"))
    } else {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {e}"))?;
        }
        std::fs::write(&target, b"").map_err(|e| format!("创建文件失败: {e}"))
    }
}

/// 重命名/移动工程内文件或目录。
pub fn rename_entry(root: &Path, from_rel: &str, to_rel: &str) -> Result<(), String> {
    let from = resolve_within(root, from_rel)?;
    let to = resolve_within(root, to_rel)?;
    if !from.exists() {
        return Err(format!("源不存在: {from_rel}"));
    }
    if to.exists() {
        return Err(format!("目标已存在: {to_rel}"));
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目标父目录失败: {e}"))?;
    }
    std::fs::rename(&from, &to).map_err(|e| format!("重命名失败: {e}"))
}

/// 删除工程内文件或目录(目录递归删除)。拒绝删除工程根与 project.toml。
pub fn delete_entry(root: &Path, rel: &str) -> Result<(), String> {
    if rel.trim().is_empty() {
        return Err("拒绝删除工程根".into());
    }
    let target = resolve_within(root, rel)?;
    if target == root.join(MANIFEST_NAME) {
        return Err("拒绝删除 project.toml".into());
    }
    if !target.exists() {
        return Err(format!("不存在: {rel}"));
    }
    if target.is_dir() {
        std::fs::remove_dir_all(&target).map_err(|e| format!("删除目录失败: {e}"))
    } else {
        std::fs::remove_file(&target).map_err(|e| format!("删除文件失败: {e}"))
    }
}

// --------------------------------------------------------------- Tauri state

/// 活动工程单例:同一时刻只有一个活动工程(其根目录)。
#[derive(Default)]
pub struct ProjectState {
    root: Mutex<Option<PathBuf>>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self::default()
    }
    fn set(&self, root: PathBuf) {
        *self.root.lock().unwrap() = Some(root);
    }
    fn root(&self) -> Result<PathBuf, String> {
        self.root
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "没有活动工程,请先新建或打开工程".into())
    }
    /// 活动工程根(供 build_pipeline 等其它模块使用)。
    pub fn active_root(&self) -> Result<PathBuf, String> {
        self.root()
    }
}

// --------------------------------------------------------------- commands

#[tauri::command]
pub fn project_new(
    dir: String,
    name: String,
    template: String,
    state: State<ProjectState>,
) -> Result<ProjectManifest, String> {
    let root = PathBuf::from(&dir);
    let manifest = create_from_template(&root, &name, &template)?;
    state.set(root);
    Ok(manifest)
}

#[tauri::command]
pub fn project_open(dir: String, state: State<ProjectState>) -> Result<ProjectManifest, String> {
    let root = PathBuf::from(&dir);
    let manifest = load_manifest(&root)?;
    state.set(root);
    Ok(manifest)
}

#[tauri::command]
pub fn project_get(state: State<ProjectState>) -> Result<ProjectManifest, String> {
    let root = state.root()?;
    load_manifest(&root)
}

#[tauri::command]
pub fn project_save(
    manifest: ProjectManifest,
    state: State<ProjectState>,
) -> Result<(), String> {
    let root = state.root()?;
    save_manifest(&root, &manifest)
}

#[tauri::command]
pub fn project_file_tree(state: State<ProjectState>) -> Result<FileNode, String> {
    let root = state.root()?;
    file_tree(&root)
}

#[tauri::command]
pub fn project_create_file(
    rel_path: String,
    is_dir: bool,
    state: State<ProjectState>,
) -> Result<(), String> {
    let root = state.root()?;
    create_entry(&root, &rel_path, is_dir)
}

#[tauri::command]
pub fn project_rename_file(
    from: String,
    to: String,
    state: State<ProjectState>,
) -> Result<(), String> {
    let root = state.root()?;
    rename_entry(&root, &from, &to)
}

#[tauri::command]
pub fn project_delete_file(rel_path: String, state: State<ProjectState>) -> Result<(), String> {
    let root = state.root()?;
    delete_entry(&root, &rel_path)
}

#[tauri::command]
pub fn project_read_file(rel_path: String, state: State<ProjectState>) -> Result<String, String> {
    let root = state.root()?;
    let target = resolve_within(&root, &rel_path)?;
    std::fs::read_to_string(&target).map_err(|e| format!("读取 {rel_path} 失败: {e}"))
}

#[tauri::command]
pub fn project_write_file(
    rel_path: String,
    content: String,
    state: State<ProjectState>,
) -> Result<(), String> {
    let root = state.root()?;
    let target = resolve_within(&root, &rel_path)?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {e}"))?;
    }
    std::fs::write(&target, content).map_err(|e| format!("写入 {rel_path} 失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip_is_stable() {
        let mut m = base_manifest("demo");
        m.map_chr.insert("map/room.bin".into(), "chr/tiles.chr".into());
        let text = toml::to_string_pretty(&m).unwrap();
        let back: ProjectManifest = toml::from_str(&text).unwrap();
        assert_eq!(m.name, back.name);
        assert_eq!(m.ines.mapper, back.ines.mapper);
        assert_eq!(m.output, back.output);
        assert_eq!(m.map_chr, back.map_chr);
    }

    #[test]
    fn manifest_without_map_chr_defaults_to_empty_binding_table() {
        let text = r#"
name = "legacy"
sources = ["src/main.s"]
output = "build/game.nes"

[ines]
mapper = 0
prg_banks = 2
chr_banks = 1
mirroring = "vertical"
battery = false
"#;
        let back: ProjectManifest = toml::from_str(text).unwrap();
        assert!(back.map_chr.is_empty());
        back.validate().unwrap();
    }

    #[test]
    fn playable_templates_create_editable_resources() {
        let tmp = std::env::temp_dir().join(format!("fc-playable-res-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let manifest = create_from_template(&tmp, "playable", "demo").unwrap();
        assert_eq!(manifest.chr, vec!["chr/sprites.chr"]);
        assert_eq!(manifest.maps, vec!["map/room.bin"]);
        assert_eq!(manifest.map_chr.get("map/room.bin").map(String::as_str), Some("chr/sprites.chr"));
        let chr = std::fs::read(tmp.join("chr/sprites.chr")).unwrap();
        let map = std::fs::read(tmp.join("map/room.bin")).unwrap();
        assert_eq!(chr.len(), 8192);
        assert_eq!(&map[0..4], &[32, 0, 30, 0]);
        assert_eq!(map[4], 9, "top-left map tile should use the editable border tile");
        assert!(std::fs::read_to_string(tmp.join("src/main.s")).unwrap().contains(".incbin \"chr/sprites.chr\""));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn validate_rejects_bad_fields() {
        let mut m = base_manifest("x");
        m.ines.mapper = 999;
        assert!(m.validate().is_err());
        m.ines.mapper = 0;
        m.ines.mirroring = "diagonal".into();
        assert!(m.validate().is_err());
        m.ines.mirroring = "vertical".into();
        m.map_chr.insert("map/a.bin".into(), "".into());
        assert!(m.validate().is_err());
    }

    #[test]
    fn resolve_within_blocks_escape() {
        let root = Path::new("/tmp/proj");
        assert!(resolve_within(root, "../etc/passwd").is_err());
        assert!(resolve_within(root, "/etc/passwd").is_err());
        assert!(resolve_within(root, "src/main.s").is_ok());
    }
}
