# fc — AI 原生的 FC/NES 模拟器与游戏创作平台

<p align="center">
  <strong>从零自研 · 周期精确 · 多前端 · MCP 原生 · 一体化创作 IDE</strong>
</p>

**fc** 是一套从零基于纯 Rust 构建的周期精确（Cycle-Accurate）FC/NES 红白机模拟器系统，
同时也是一个 **AI 原生的 NES 游戏一体化创作工作台**。

它不仅是一台高精度模拟器——更是一个以"可编程内核 + 硬件级调试 + AI/MCP"为护城河的
**NES homebrew 全栈 IDE**，让人类开发者和 AI 智能体都能在同一套工具链里完成
「编码 → 资源制作 → 汇编编译 → ROM 打包 → 内嵌运行 → 硬件级调试」的完整创作闭环。

---

## 项目定位

```
┌─────────────────────────────────────────────────────────┐
│               fc-tauri · AI 原生 IDE                     │
│  Vue3 + Pinia + PixiJS + CodeMirror + dockview           │
│  工程管理 | 6502 编辑器 | CHR 图形 | 地图编辑 | 2A03 Tracker │
├─────────────────────────────────────────────────────────┤
│               fc-core · 自研护城河                        │
│  CPU 2A03 | PPU 2C02 | APU 五声道 | Mapper 40+           │
│  调试器 | 断点/追踪 | 金手指 | 存档系统                    │
├─────────────────────────────────────────────────────────┤
│               fc-mcp · AI 原生接口                        │
│  MCP JSON-RPC 2.0 | emu_* 工具 | ide_* 工具              │
│  AI 可编程驱动模拟器 + 编排 IDE 工具链                     │
├─────────────────────────────────────────────────────────┤
│    fc-cli (无头/测试)  │  fc-gui (egui 桌面)              │
│    4 个前端 × 1 套内核 = 全场景覆盖                        │
└─────────────────────────────────────────────────────────┘
```

### 核心差异化

| 能力 | fc | Mesen/FCEUX | cc65+VSCode |
|------|:--:|:-----------:|:-----------:|
| 自研周期精确模拟器 | ✅ | ✅ | ❌ |
| 硬件级 AI 调试（MCP） | ✅ | ❌ | ❌ |
| 一体化创作 IDE | ✅ | ❌ | 🔶 需拼装 |
| AI 可编程编排工具链 | ✅ | ❌ | ❌ |
| 内置 CHR/地图/Tracker 编辑器 | ✅ | ❌ | ❌ |
| cc65 汇编工具链捆绑 | ✅ | ❌ | ✅ 独立安装 |
| 跨平台（macOS/Windows/Linux） | ✅ | ✅ | 🔶 |
| 开源 + 可商用授权 | GPLv3 / 商业双授权 | GPL | zlib |

**没有其他项目把"自研模拟器 + 硬件级调试 + AI/MCP + 创作 IDE"四者合一。**

---

## 工作区结构

```
fc/
├── Cargo.toml                  # Cargo workspace 根配置
├── fc-core/                    # 🔧 纯逻辑核心库（无 IO、无渲染）
│   └── src/
│       ├── cpu/                #   2A03 CPU：官方 + 非官方指令，周期驱动
│       ├── ppu/                #   2C02 PPU：逐扫描线流水线，精灵/背景/滚动
│       ├── apu/                #   APU：Pulse×2 / Triangle / Noise / DMC
│       ├── mapper/             #   Mapper 工厂 + 40+ 种型号实现
│       ├── bus.rs              #   总线地址译码与锁步时钟推进
│       ├── control_deck.rs     #   顶层统一 API（所有前端的唯一调用入口）
│       ├── save_state.rs       #   全量快照存档 + 电池 SRAM 持久化
│       ├── input.rs            #   手柄/光枪输入
│       ├── debug/              #   调试器、反汇编、断点、内存观察
│       └── cheat.rs            #   金手指（Game Genie 解码）
├── fc-cli/                     # 🖥 命令行前端（fc 二进制）
│                               #   run | test | testsuite | disasm | info
│                               #   | dbg | mcp | tauri-bridge
├── fc-gui/                     # 🎮 egui + wgpu 桌面 GUI
│                               #   音频时钟节拍、整数缩放、调试面板
├── fc-mcp/                     # 🤖 MCP AI 服务端（JSON-RPC 2.0 / stdio）
│                               #   模拟器全量能力封装为 AI 可调用工具
├── fc-tauri/                   # 🏗 Tauri 2 桌面应用（独立构建，非 workspace 成员）
│   ├── src/                    #   Vue3 + Pinia + PixiJS + CodeMirror 前端
│   │   └── stores/             #   Pinia 状态管理
│   ├── src-tauri/              #   Rust 后端
│   │   ├── src/
│   │   │   ├── emu.rs          #     模拟器工作线程（cpal 音频时钟驱动）
│   │   │   ├── ide.rs          #     IDE 构建编排器、工程模型
│   │   │   ├── mcp.rs          #     内嵌 MCP socket 服务
│   │   │   └── lib.rs          #     Tauri 插件注册
│   │   └── vendor/cc65/        #     捆绑 cc65 工具链（ca65/ld65）
│   └── package.json
├── nes-test-roms/              # 标准化测试 ROM 套件（准确度验收）
├── docs/                       # 需求文档、路线图、策划案、使用说明
└── ui设计/                     # UI 设计稿（作为前端实现权威参考）
```

---

## 已实现功能

### 🧠 模拟器内核（fc-core）

- **CPU**：Ricoh 2A03，官方指令集 + 常用非官方指令，**周期驱动**。
  nestest 全指令测试通过（`$0002 == 0x0000`）。
- **PPU**：真流水线逐扫描线渲染，背景移位寄存器、精灵 OAM 评估、
  sprite-0 精确命中、A12 边沿检测。Super Mario Bros. 标题画面与游戏画面正确。
- **APU**：双矩形波 + 三角波 + 噪声 + **DMC DPCM 全声道**，
  帧序列器 + 帧 IRQ，重采样 + DC 阻断 → `f32` 输出。
- **Mapper**：40+ 种 Mapper 支持，包括：
  - 核心：NROM(0)、MMC1(1)、UNROM(2)、CNROM(3)、MMC3(4)、AxROM(7)
  - MMC3 含 **A12 边沿扫描线 IRQ**（通过 blargg mmc3_test 3/5）
  - MMC2/MMC4(9/10)、ColorDreams(11)、GxROM(66)
  - 长尾覆盖：VRC 系列、Namco 163、Sunsoft、Cameria、Codemasters 等
- **存档**：全量机器快照（Save State）+ 电池 SRAM 持久化（`.sav`）
- **金手指**：Game Genie 解码 + 条件写入
- **调试器**：执行/读/写断点、单步、反汇编、CPU 寄存器查看、
  PPU/APU 寄存器与通道电平实时预览、运行事件追踪
- **区域支持**：NTSC / PAL / Dendy 三制式

### 🖥 CLI 前端（fc-cli）

```sh
fc run    rom.nes --frames 600 --shot out.png   # 无头运行 + PNG 截图
fc test   nestest.nes --entry C000              # 测试 ROM 自动判分
fc testsuite blargg*.nes                        # blargg $6000 协议批量测试
fc disasm rom.nes 8000 --count 40               # 6502 反汇编
fc info   rom.nes                               # ROM 信息解析
fc mcp    --rom rom.nes                         # 启动 MCP 服务端
```

支持 `--shot` PNG 截图、`--wav` 音频导出、`--autostart` 跳过开始画面。

### 🎮 egui 桌面 GUI（fc-gui）

- egui + wgpu 渲染，**整数缩放**无模糊
- **音频时钟帧节拍**：声卡时钟驱动模拟速度，无欠载、无音画漂移
- 调试面板：CPU 寄存器、Pattern Table、Nametable（2×2）、
  Palette RAM、OAM Sprite 列表
- 键盘 + 手柄（gilrs 原生轮询）双输入，按键去乱序
- F1 暂停 / F5 重置 / F8 打开 ROM / F2/F3 存读档

### 🤖 MCP AI 服务（fc-mcp）

将模拟器全量能力封装为标准 MCP 工具，AI 智能体可直接调用：

`emu_load_rom` · `emu_press_button` · `emu_read_memory` · `emu_write_memory` ·
`emu_get_state` · `emu_step_frame` · `emu_run_until_break` · `emu_capture_screen`（真 PNG）·
`emu_save_state` · `emu_load_state` · `emu_reset` · `emu_disassemble` ·
`emu_set_breakpoint` · `emu_trace` · `emu_heatmap` · `emu_event_dump` · `emu_set_event_breakpoint`

**已打通双端口**：
- `fc-emu`（fc mcp）：无头核心驱动，适合研究/测试
- `fc-tauri`（tauri-bridge MCP）：**实时操控运行中的 Tauri 窗口**，
  `tauri_eval` 读 DOM/Pinia 状态，`tauri_screenshot` 截屏

### 🏗 Tauri 创作 IDE（fc-tauri）

完整的 NES 游戏一体化创作工作台，已在 M1+M2 里程碑交付：

| 模块 | 能力 |
|------|------|
| **工程管理** | 标准化目录 + `project.toml` + 3 种模板（空白/横版/演示） |
| **代码编辑器** | CodeMirror 6 + 6502/ca65 语法高亮/补全/折叠 |
| **CHR 图形编辑器** | 8×8 图块、4 色调色板、铅笔/擦除/填充/取样、旋转/翻转/平移、撤销/重做、PNG 导入 |
| **地图编辑器** | 图块层/属性层/碰撞层、刷子/矩形/填充/取样/选区、2×2/4×4 刷子、缩放/拖拽 |
| **2A03 Tracker** | 音序/Pattern + 卷帘、琶音效果、**驱动自研 APU 内核试听**、导出汇编播放器引擎 |
| **构建系统** | 捆绑 cc65（ca65/ld65）sidecar、一键编译打包 `.nes`、报错跳转源码行 |
| **ROM 头编辑器** | 可视化 iNES 头（Mapper/PRG/CHR/镜像/电池） |
| **内嵌运行** | 复用自研模拟器内核，编译完成一键运行预览 |
| **行级断点调试** | 编辑器左侧断点槽 ↔ 构建产物 dbgfile 映射，命中自动定位 |
| **资源快速切换** | `Cmd/Ctrl+P` 跨源码/CHR/地图/Tracker 即时跳转 + 历史后退/前进 |
| **文件监听** | 资源变更自动重链 + 刷新预览 |
| **FamiStudio 对接** | 外部作曲 → 导出 CA65 → 落入工程 → 纳入构建（可选，不打包） |
| **IDE MCP** | 24 个 `ide_*` 工具：AI 可创建工程、读写资源、运行构建、游戏验证 |

---

## 正在开发 & 路线图

> 详细规划见 [`docs/路线图.md`](docs/路线图.md) 与 [`docs/策划案-基于现状-可行性评估.md`](docs/策划案-基于现状-可行性评估.md)

### 短期（M3 · 深度调试 + AI 编排）

- [ ] PPU 显存预览/精灵列表深度细化
- [ ] CPU 运行轨迹记录与回放
- [ ] 调用栈可视化
- [ ] 工程层 MCP 扩展：AI 可编排整条构建流水线
- [ ] AI 编码助手：6502 代码生成 + 编译排错
- [ ] AI 硬件级调试智能体：读快照 → 区分代码/资源/硬件 Bug

### 中期（M4 · 精度 + 生态）

- [ ] PPU 逐点精确（dot-accurate sprite fetching）
- [ ] DMC DMA 周期窃取冲突精确化（`dmc_dma_during_read4`）
- [ ] PAL 16:5 精确分频、Dendy 制式完善
- [ ] 更多 Mapper 覆盖（目标 95%+ 商业游戏兼容率）
- [ ] FDS 磁碟机系统支持
- [ ] 案例库、教程、新手引导
- [ ] 跨平台二进制矩阵 + macOS 签名公证

### 长期愿景

- [ ] 自研 6502 汇编器（可选，替代 cc65 sidecar）
- [ ] 多智能体调度与权限隔离
- [ ] AI 自动闯关/解谜 Agent（「暂停-思考-注入」闭环）
- [ ] AI 辅助逆向：自动分析游戏逻辑/数据结构
- [ ] 社区生态：自定义智能体接入、模板市场

---

## 快速开始

### 环境要求

- **Rust** 1.80+
- **Node.js** 20+（仅 fc-tauri 需要）
- **macOS** / **Windows** / **Linux**

### 构建

```sh
# 克隆仓库
git clone https://github.com/Sundykin/fcemulator.git
cd fcemulator

# 构建整个 workspace（fc-core + fc-cli + fc-gui + fc-mcp）
cargo build --release

# 仅构建核心库
cargo build -p fc-core

# 运行测试
cargo test -p fc-core
```

### 体验模拟器

```sh
# 桌面 GUI（推荐）—— 方向键 = D-Pad，Z = A，X = B，Enter = Start，Space = Select
cargo run -p fc-gui --release -- roms/SuperMarioBro.nes

# 无头运行 600 帧并截图
target/release/fc run roms/SuperMarioBro.nes --frames 600 --shot out.png --autostart

# ROM 信息查看
target/release/fc info roms/SuperMarioBro.nes

# CPU 指令测试
target/release/fc test nes-test-roms/other/nestest.nes --entry C000
# 通过标准：$0002 == 0x0000

# blargg 测试套件
target/release/fc testsuite nes-test-roms/instr_test-v5/official_only.nes
```

### 启动创作 IDE

```sh
# 构建并启动 fc-tauri（独立工具链，不影响 workspace）
npm --prefix fc-tauri run tauri dev

# 前端类型检查
( cd fc-tauri && npx vue-tsc --noEmit )

# 仅编译 Rust 后端
cargo build --manifest-path fc-tauri/src-tauri/Cargo.toml
```

IDE 底部导航栏点击**「创作」**进入：
1. 新建工程（空白/横版/演示模板）
2. 编写 6502 汇编代码（语法高亮、补全、折叠）
3. Ctrl/Cmd+Shift+B 一键构建（ca65 → ld65 → .nes）
4. 工具栏「运行」→ 内嵌模拟器预览
5. 编辑器左侧点击设断点 → 命中自动定位

### 启动 MCP 服务（供 AI 智能体调用）

```sh
# 无头模式
target/release/fc mcp --rom roms/SuperMarioBro.nes

# IDE 启动时会自动开启内嵌 MCP socket
# fc-emu → /tmp/fc-tauri-emu-mcp.sock
# fc-ide → /tmp/fc-tauri-ide-mcp.sock
```

在 Claude Code 等 MCP 客户端中配置 `.mcp.json` 即可让 AI 直接操控模拟器运行态。

---

## 准确度验证

本项目以标准化测试 ROM 客观验收准确度，**零针对单游戏的 hack**。

| 测试套件 | 状态 | 说明 |
|----------|:----:|------|
| nestest（全指令） | ✅ PASS | `$0002 == 0x0000` |
| blargg instr_test-v5（官方指令） | ✅ PASS | `$6000` 协议 |
| blargg instr_timing-v6 | 🔶 进行中 | 指令周期精确计时 |
| blargg mmc3_test (3-A12 / 5-MMC3) | ✅ PASS | MMC3 IRQ 逻辑 |
| blargg mmc3_test (1-clocking / 2-details) | ✅ PASS | MMC3 时钟/细节 |
| blargg mmc3_test (4-scanline_timing) | ⏳ | 需 dot-accurate PPU |
| Super Mario Bros. | ✅ 可玩 | 标题 + 1-1 画面与音乐正确 |

---

## 架构原则

贯穿整个项目的设计约束：

- **`fc-core` 纯逻辑、零 IO**：文件/渲染/音频/对话框代码归属前端，永不在核心出现
- **`ControlDeck` 单一外观**：所有前端（CLI/egui/Tauri/MCP）都通过同一套 API 驱动内核
- **锁步时钟是核心不变量**：CPU 每次总线访问前推进 PPU×3 + APU×1，
  所有组件在子指令粒度保持同步
- **一次实现，四端复用**：金手指、断点、调试视图等能力在 core 实现，
  所有前端共享同一套 API
- **以测试 ROM 为客观准绳**：准确度以 `nes-test-roms` 量化验收

---

## 第三方组件与许可

| 组件 | 用途 | 许可证 |
|------|------|--------|
| cc65 (ca65/ld65 V2.19) | 汇编/链接 sidecar | zlib |
| CodeMirror 6 | 代码编辑器内核 | MIT |
| dockview-vue | IDE 分栏布局 | MIT |
| egui / wgpu / cpal | GUI 渲染与音频 | MIT / Apache-2.0 |
| Tauri 2 | 桌面应用框架 | MIT |
| notify | 文件监听 | MIT/Apache-2.0 |

内置 tracker、`fc_player.s` 引擎、CHR/地图编辑器、所有转换器、模拟器内核、
调试器、MCP 服务均为本项目自研。

---

## 许可

本项目采用**双授权**模式：

- **开源使用**：[GNU GPL v3.0](LICENSE)。自由使用、修改、再分发，
  但衍生作品也须以 GPLv3 开源发布完整源码。
- **商业使用**：无法遵循 GPLv3 的闭源/商业使用场景，需获取
  **独立商业授权** —— 详见 [`COMMERCIAL.md`](COMMERCIAL.md)。

`nes-test-roms/`（如有）保留其上游许可证。ROM 文件与开发参照用的模拟器
**不随本仓库分发**。

> 桌面应用的 macOS（`.dmg`）和 Windows（`.msi`/`.exe`）安装包在
> [GitHub Releases](https://github.com/Sundykin/fcemulator/releases)
> 页面发布，由 CI 自动构建。

---

## 贡献与反馈

项目处于活跃开发期。欢迎 Issue 和 PR。

- 📖 项目文档：[`docs/`](docs/)
- 🗺 路线图：[`docs/路线图.md`](docs/路线图.md)
- 📋 需求规范：[`docs/需求文档.md`](docs/需求文档.md)
- 🏗 IDE 使用说明：[`docs/M1-创作IDE-使用说明.md`](docs/M1-创作IDE-使用说明.md) | [`docs/M2-资源与音频-使用说明.md`](docs/M2-资源与音频-使用说明.md)
- 📐 可行性评估：[`docs/策划案-基于现状-可行性评估.md`](docs/策划案-基于现状-可行性评估.md)
