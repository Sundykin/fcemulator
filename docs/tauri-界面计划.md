# fc-tauri —— 现代化模拟器界面开发计划（Tauri v2 + Vue 3）

> 目标：在现有 `fc-core` 之上新增一个 **Tauri v2 + Vue 3** 桌面前端 `fc-tauri`，
> 提供精致、可定制的现代 UI（游戏库 / 设置面板 / 金手指 / 存档槽 / 调试器），
> 解决 egui「渲染卡顿 + 无标准组件、难定制」的问题。
> **`fc-gui`(egui) 保留** 作为轻量调试前端，不删除。

## 0. 核心决策

| 项 | 选择 | 理由 |
|---|---|---|
| 模拟器核心 | **复用 `fc-core`**（不用第三方 `nes-sim`） | 已通过 nestest、SMB、MMC1/3，全量存档、调试渲染器都有现成 API |
| 桌面框架 | Tauri v2 | 体积小（系统 WebView）、Rust 后端、跨平台 |
| 前端框架 | Vue 3 + TypeScript + Vite | 生态成熟、组件库丰富 |
| 组件库 | **Naive UI** | TS 原生、暗色主题极佳、轻量、适合工具/仪表盘类界面（可换 Element Plus） |
| 状态管理 | Pinia | 轻量、TS 友好 |
| 画面渲染 | Canvas 2D `putImageData` + `image-rendering: pixelated`（整数缩放） | 256×240 毫无压力；后续可上 WebGL 做 CRT 滤镜 |
| 音频 | Web Audio + AudioWorklet 环形缓冲 | 音画同步、低延迟 |
| 图标 | @vicons/tabler | 配 Naive UI |

## 1. 架构与数据流

```
┌─────────────────────────── fc-tauri ───────────────────────────┐
│  src-tauri (Rust 后端)                  src/ (Vue 前端, WebView) │
│  ┌──────────────────────┐               ┌────────────────────┐  │
│  │ Emulator Worker 线程  │               │  Pinia store        │  │
│  │  fc_core::ControlDeck │◀──set_input───│  键盘/手柄 → 位掩码  │  │
│  │  60fps 定时跑帧        │               │                     │  │
│  │  最新帧→Mutex<Vec<u8>>│──poll_frame──▶│  rAF → Canvas 绘制   │  │
│  │  音频样本→环形缓冲      │──poll_audio──▶│  AudioWorklet 播放   │  │
│  └──────────────────────┘               │  Naive UI 面板       │  │
│   commands: open_rom/reset/pause/        │  库/设置/存档/金手指 │  │
│   save_state/load_state/set_speed/        └────────────────────┘  │
│   debug(pattern/nametable/oam/cpu)                                │
└────────────────────────────────────────────────────────────────┘
```

**帧/音频传输（避开 IPC 瓶颈）**：命令返回 `tauri::ipc::Response`（**原始二进制**，
非 JSON 数组），前端 `invoke` 得到 `ArrayBuffer`。前端 `requestAnimationFrame`
按显示器刷新率拉取最新帧（256×240×4≈245KB/次），音频按定时拉取样本块灌入
AudioWorklet。模拟时钟完全由后端 60fps 驱动，前端只做展示——音画同步可控。

## 2. 目录结构

```
fc-tauri/
├── package.json            # vue, naive-ui, pinia, @tauri-apps/api, @tauri-apps/cli
├── vite.config.ts
├── tsconfig.json
├── index.html
├── src/                    # Vue 前端
│   ├── main.ts
│   ├── App.vue             # 整体布局（侧栏 + 主区 + 状态栏）
│   ├── store.ts            # Pinia：运行态、设置、存档槽
│   ├── emu.ts              # 封装 invoke 命令 + 帧/音频轮询循环
│   ├── audio.ts            # AudioWorklet 装配
│   ├── audio-worklet.js    # 环形缓冲 worklet
│   └── components/
│       ├── Screen.vue      # Canvas 游戏画面 + 缩放/全屏
│       ├── Library.vue     # 游戏库（网格 + 封面 + 搜索）
│       ├── Toolbar.vue     # 播放/暂停/复位/变速/存读档/截图
│       ├── Settings.vue    # 按键映射/视频滤镜/音频/区域
│       ├── SaveSlots.vue   # 多存档槽
│       ├── Cheats.vue      # 金手指（地址/值/启停）
│       └── debug/          # PatternTable/Nametable/Oam/CpuRegs 面板
└── src-tauri/              # Rust 后端
    ├── Cargo.toml          # tauri=2, fc-core={path="../../fc-core"}
    ├── tauri.conf.json
    ├── build.rs
    ├── capabilities/default.json
    └── src/
        ├── main.rs
        ├── lib.rs          # Tauri builder + 命令注册
        └── emu.rs          # Worker 线程 + 共享状态 + 命令实现
```

> `fc-tauri` 从根 workspace **exclude**，用 `npm run tauri dev/build` 独立构建，
> 不影响现有 `cargo build` 的离线编译。

## 3. 后端命令（Tauri commands）

| 命令 | 入参 | 出参 | 说明 |
|---|---|---|---|
| `open_rom` | `path` | `RomInfo`(mapper/大小/镜像) | 加载并启动 worker |
| `poll_frame` | — | `Response`(RGBA 原始字节) | rAF 拉取最新帧 |
| `poll_audio` | — | `Response`(f32 原始字节) | 拉取并清空音频队列 |
| `set_input` | `p1:u8, p2:u8` | — | 两手柄按键位掩码 |
| `control` | `action`("reset"/"pause"/"resume"/"step") | — | 运行控制 |
| `set_speed` | `mult:f64` | — | 变速（0.5/1/2/快进） |
| `save_state`/`load_state` | `slot` / `slot,bytes` | — | 全量存读档 |
| `battery_save` | — | — | 退出时落盘 `.sav` |
| `dbg_pattern`/`dbg_nametable`/`dbg_oam`/`cpu_state` | … | `Response`/JSON | 调试视图（复用 fc-core 渲染器） |

Worker 线程：持有 `ControlDeck`，按 60.0988fps 节拍循环：读共享输入 → `run_frame`
→ 写共享最新帧 → `drain_audio` 入环形缓冲。命令通过 `Mutex`/原子量与 worker 通信。

## 4. 前端 UI（Naive UI）

- **主布局**：左侧可折叠侧栏（库/设置/调试切换）+ 中央画面 + 顶部工具栏 + 底部状态栏（FPS/帧/区域）。暗色主题。
- **游戏库**：`n-grid` 卡片网格、搜索框 `n-input`、最近游玩；拖拽 `.nes` 入窗加载。
- **工具栏**：播放/暂停/复位/变速/存读档/截图/全屏（`n-button` + 图标）。
- **设置**：`n-tabs`（按键映射可视化、视频缩放/滤镜、音频音量、区域 NTSC/PAL/Dendy）。
- **存档槽**：多槽位缩略图（`n-card`），快捷键 F1–F8。
- **金手指**：`n-data-table`（地址/值/描述/启停）→ 调 `write_memory`。
- **调试器**（复用 fc-core）：Pattern Tables / Nametables(2×2) / OAM 表 / CPU 寄存器，`n-modal` 或停靠面板。

## 5. 里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| T0 | 脚手架：Tauri+Vue+Naive 跑通空壳 | `npm run tauri dev` 出窗口 |
| T1 | 后端 worker + open_rom + poll_frame；前端 Canvas 渲染 | 打开 SMB 出画面 |
| T2 | 键盘输入 + Web Audio 出声 + 工具栏（暂停/复位/变速） | 能玩、有声 |
| T3 | 游戏库 + 设置（按键映射/缩放/区域）+ 存档槽 | 现代 UI 闭环 |
| T4 | 金手指 + 调试面板（pattern/nametable/oam/cpu）+ 手柄(gilrs) | 调试可用 |
| T5 | `tauri build` 三平台打包、CRT 滤镜(WebGL) | 安装包产出 |

## 6. 风险

- **帧数据 IPC 开销**：用 `Response` 原始二进制 + rAF 拉取已抹平；若仍紧，升级 Tauri Channel/共享内存。
- **音画同步**：后端 60fps 驱动 + 前端 AudioWorklet 环形缓冲 + 水位调节。
- **WebView 渲染差异**：256×240 小画面 + `pixelated` 整数缩放，差异可忽略。
- **构建链**：需 node + tauri 工具链；首次 `cargo tauri` 拉 tauri crates 较慢。
