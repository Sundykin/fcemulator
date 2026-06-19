## Context

`fc` 现状(基线):一台自研 NES 模拟器内核(`fc-core`,~4.3k 行,CPU 过 nestest、PPU dot 级、APU 原生 cpal、11 个 Mapper、存档、金手指),四个前端(`fc-cli`/`fc-gui`/`fc-mcp`/`fc-tauri`),所有前端经 `ControlDeck`(`control_deck.rs`)单一门面驱动机器。`fc-tauri` 已有完整播放器 + 调试器 UI(6 张设计稿全实现,Vue3+Pinia+PixiJS,后端 worker 线程按 cpal 音频时钟节拍跑 `ControlDeck`,Tauri 命令回传裸二进制帧)。硬件层 MCP 已打通(`fc-mcp` + `fc tauri-bridge` 调真实窗口)。

缺口:愿景中的"创作工具链"为 0。《策划案-基于现状-可行性评估.md》v2 终版已拍板**集成优先**:复用 cc65/FamiStudio/CodeMirror,自研只留护城河。本设计聚焦**如何落地 M1 创作最小闭环**,并锁定 M1→M4 的架构主线,使后续里程碑沿同一编排层自然延展。

约束:
- `fc-core` 必须保持 IO-free(file/audio/render/dialog 只在前端);M1 不改核心,只复用 `ControlDeck`。
- IDE 外壳与编排层落在 `fc-tauri`(`src-tauri/` Rust + `src/` Vue3);沿用 Pinia(store 底部必须 `acceptHMRUpdate`)。
- `fc-tauri` 独立工具链(npm + Tauri),不进根 `cargo` workspace。
- cc65 许可证 zlib、FamiStudio/CodeMirror MIT —— 均允许商业闭源打包。

## Goals / Non-Goals

**Goals:**
- 固化"集成优先 + 护城河变现"为产品主线,定义 M1→M4 阶段、验收口径与关键技术决策。
- M1 端到端闭环:**新建工程 → 写汇编 → ca65 编译 → ld65 链接 → 拼 .nes → 内嵌运行 → 调试**,报错/断点 ↔ 代码行双向跳转。
- 建立可重放、可被 AI 编排的统一工程模型 + 构建编排器(M2/M3 的技术内核)。
- sidecar(cc65)集成范式可复用,使 M2 的 FamiStudio CLI、转换器以相同方式接入。

**Non-Goals:**
- 不自研汇编器/编译器(M1 封装 cc65;自研留 M4 可选)。
- 不自研对标 Famitracker 的音频编辑器(M2 内置精简 tracker / FamiStudio 文件对接)。
- 不重写编辑器内核(用 CodeMirror 6)。
- 不打包 FamiStudio(纯文件对接,M2)。
- M1 不做跨平台签名/公证(M4)、不做工程层 MCP / AI 编排(M3)。
- 不改 `fc-core` 时序/读写顺序(锁步时钟是核心不变量)。

## Decisions

### D1 关键技术决策(对应文档第五节,已拍板)
| 决策点 | 选择 | 理由 / 备选 |
|---|---|---|
| 汇编器 | **封装 ca65/ld65 sidecar**(M1);自研留 M4 可选 | cc65 是几十年积累,"自研全兼容"≈ 不可达;封装 ~1–1.5 人周、零兼容风险。备选:自研子集(慢、是后期卖点)。 |
| 音频编辑器 | **B 内置精简 tracker + C FamiStudio 文件对接**(M2) | 对标 Famitracker 6–12 人月不现实;自研 APU 内核可做试听/AI 校验。备选:全量对标(不做)。 |
| 编辑器内核 | **CodeMirror 6** | 轻、易嵌 webview、现成 6502/ca65 语法。备选:Monaco(重)、从零(不做)。 |
| 前端框架 | **沿用 Vue3 + Pinia**,IDE 布局加 dockview/分栏 | 复用现有 3.8k 行前端与 6 张设计稿。 |
| 产品定位 | **押"AI 硬件级调试的创作 IDE"wedge**,播放器作为内嵌运行子集 | 护城河变现路径最短。 |
| "零依赖"约束 | **务实混合**:护城河自研,卡脖子环节封装 | 把自研预算押在模拟器/调试/AI-MCP。 |

### D2 集成架构(M1 落点,M2–M4 延展)
```
前端(Vue3 IDE 外壳:文件树 / CodeMirror 编辑器 / 构建面板 / 调试面板 / [M3 AI 面板])
        │ Tauri IPC(命令返回结构化 JSON;帧数据仍走裸二进制)
Rust 编排层(本变更新增,fc-tauri/src-tauri/):
   · 统一工程模型(project.toml:源码/CHR/音乐/地图/Mapper/链接配置)
   · 构建编排器:ca65 → ld65 → 拼 .nes(M2 再加 FamiStudio CLI 导出)
   · sidecar 进程管理 + 日志解析(报错跳转)+ [M2 文件监听/增量构建]
   · [M3 工程层 MCP:把"工程上下文 + 构建 + 运行"暴露给 AI]
        │
   sidecars: ca65 / ld65 / (cc65) / [M2 FamiStudio CLI]
   内嵌:自研模拟器 + 调试器(ControlDeck,已有)
```
理由:把"造编译器"换成"粘合 + 编排";工程模型 + 构建编排器是一栈式的技术内核,也是 M3 让 AI 端到端编排流水线的接口面。

### D3 工程模型与磁盘布局
- 工程根 `project.toml`(声明:`name`/`mapper`/源码列表/CHR/音乐/地图/链接 `.cfg`/输出 `.nes`)。
- 约定目录:`src/`(.s/.asm)、`chr/`、`music/`(M2 FamiStudio 导出落点)、`map/`、`build/`(产物)。
- 模板:空白 / 横版 / 演示(各含最小可编译骨架 + 链接脚本)。
- 理由:可重放构建的"单一事实源";AI(M3)读 `project.toml` 即得全工程上下文。

### D4 cc65 sidecar 集成范式
- 按 target-triple 预编译/捆绑 ca65、ld65(必要时 cc65);捆绑标准 `.cfg` 链接脚本。
- 通过 `CC65_INC` / `LD65_LIB` / `LD65_CFG` 环境变量指向捆绑资源,headless CLI 调用。
- 进程管理:Rust 编排层 spawn sidecar、捕获 stdout/stderr、超时/取消、退出码 → 结构化构建结果。
- 理由:零兼容风险、生态/教程现成、用户零迁移成本;该范式 M2 直接复用给 FamiStudio CLI。

### D5 报错/断点 ↔ 代码行(source-debug-link)
- 构建时让 ld65 产出符号/映射(`-Ln` 标签文件 / map 文件 / dbg 信息),编排层解析为 `addr ↔ {file,line}`。
- 编译日志解析:正则提取 `file:line: error/warning` → 前端可点击跳转到编辑器对应行。
- 断点:UI 在源码行下断 → 经映射转 PC 地址 → 调 `ControlDeck.add_breakpoint`;命中暂停后反查回源码行高亮。
- 理由:这是"能调"的关键体验;复用已有调试器(`add_breakpoint`/`step_instruction`/`is_halted`),只加映射层。

### D6 复用而非重写运行/调试
- 内嵌运行 = 现有 `fc-tauri` worker(cpal 音频时钟节拍)直接加载编排器产出的 `.nes`。
- 调试面板 = 复用现有反汇编/寄存器/内存/PPU 预览组件,新增"源码视图 + 行级断点"。
- 跨前端能力一律加在 `ControlDeck`,前端只做展示/编排。

## Risks / Trade-offs

- **范围爆炸(最大风险)** → M1 最小闭环先验证产品价值,M2–M4 只占位、按验证结果再投入(本变更已据此分期)。
- **跨平台 sidecar 二进制** → M1 先支持开发主机 target;CI 预编译 ca65/ld65,M4 再做 macOS 签名/公证与全平台矩阵。
- **ld65 调试信息粒度不足以精确映射行** → 优先用标签/行号文件;映射不到时降级为"文件级 + 最近符号"提示,不阻塞闭环。
- **sidecar 体积/版本漂移** → 锁定 cc65 版本号、内置捆绑;提供"工具链版本"展示(M4 加升级入口)。
- **UX 像拼装几个独立软件** → 统一构建/运行/调试入口;80% 常用操作内置;重度场景(M2 音频/地图)才"外部打开 + 回写"。
- **许可证合规** → 捆绑各工具 LICENSE(cc65 zlib),"第三方许可"页;不冒认作者。
- **改动 fc-tauri 重启陷阱** → `tauri dev` 会 orphan `target/debug/fc-tauri`,按裸名 kill,避免双窗口/重复音频。
- **Pinia HMR** → store 底部必须 `acceptHMRUpdate`,否则 action 编辑不热更。

## Migration Plan

M1 为纯增量(新增编排层 + IDE 外壳),不破坏现有播放器/调试器路径;失败可回退到现有 fc-tauri 播放器形态。分期推进见 tasks.md(M1 细化任务 + M2–M4 里程碑占位)。

## Open Questions

- ld65 输出哪种调试/符号格式最利于行级映射(`-Ln` vs `--dbgfile`)——M1 实现时以实测为准。
- IDE 分栏用 dockview 还是自研轻量分栏——倾向 dockview,落地时按集成成本定。
- 工程模板的最小骨架取自哪个社区范例(cc65 nrom 模板)——M1 实现时选定并捆绑其 LICENSE。
