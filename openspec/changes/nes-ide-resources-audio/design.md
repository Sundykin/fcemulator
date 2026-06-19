## Context

M1(已归档)交付了创作最小闭环:`project-model` / `code-editor` / `build-pipeline`(cc65 sidecar)/ `rom-packaging`(manifest 权威 iNES 头)/ `source-debug-link`,IDE 外壳基于 dockview,后端编排在 `fc-tauri/src-tauri/`,内核 `fc-core` 保持 IO-free 且已有周期级 APU(`apu.rs`:Pulse×2/Triangle/Noise/DMC/帧序列器/混音)。M2 在此之上补齐**图形、地图、音乐**资源创作。

约束延续 M1:`fc-core` IO-free;资源编辑/转换/监听落 `fc-tauri/src-tauri/`(Rust)+ `fc-tauri/src/`(Vue3,dockview);沿用 `project.toml` 工程模型;cc65 sidecar 已就绪。音频战略遵循 design 附录 A:**自研精简 tracker 用同一台 `fc-core` APU 做预览/校验(护城河),FamiStudio 只做文件对接不打包(借力)**。

## Goals / Non-Goals

**Goals:**
- 内置 CHR 与地图轻量编辑器,产出物经 asm `.incbin`/`.include` 直接进现有构建链。
- FamiStudio 导出物(CA65 格式 `.s`+`.dmc`)无缝接入:导入登记 + `music/` 监听自动重建 + 内核试听。
- 内置精简 2A03 tracker:用 `fc-core` APU 逐帧预览;乐曲可导出为 ca65 汇编纳入构建。
- build-pipeline 扩展为"资源装配 + 增量重建";rom-packaging 支持 CHR 来自 `chr/`。
- 资源转换器:PNG→CHR、Tiled→字节数组。

**Non-Goals:**
- 不对标 FamiStudio 全量:**不做扩展音频芯片**(VRC6/VRC7/FDS/MMC5/N163/5B)、不做 NSF/FTM 二进制保真导入(只做 FTM 文本导入)、钢琴卷帘只做"够用"。
- 不打包 FamiStudio(.NET 自包含/签名留 M4)。
- 不改 `fc-core` 时序/锁步时钟;APU 预览经 `ControlDeck` 暴露,不破坏内核 IO-free。
- 不自研汇编器(沿用 cc65)。

## Decisions

### D1 资源即文件,经 asm 引用进构建(最小耦合)
- CHR 编辑器导出 `chr/*.chr`(NES 2bpp planar,16 字节/图块);asm 用 `.incbin "chr/x.chr"` 放入 `CHARS` 段。
- 地图编辑器导出 `map/*.bin`(命名表 + 属性 + 碰撞,布局文档化);asm `.incbin` 或转 `.inc` `.include`。
- 理由:M1 的 ld65 流水线已能链接 asm 引用的资源,**新增资源无需改链接器逻辑**,只需把资源文件落到约定目录 + 登记 `project.toml`。

### D2 CHR 2bpp 编解码与编辑
- 一个图块 = 8×8 像素,每像素 2 bit(0–3 调色板索引),NES planar:低位平面 8 字节 + 高位平面 8 字节 = 16 字节。
- 编辑器:Canvas 网格(图块表 + 单图块放大编辑),4 色调色板(取 NES 64 色子集),铅笔/填充/翻转。
- 理由:格式简单且确定,自研成本低(附录 B 列为"轻自研")。

### D3 FamiStudio 文件对接(下游集成点,不做宿主)
- 约定:`music/` 存 FamiStudio 的 **CA65 导出**(`Sound Engine=FamiStudio` 或 `FamiTone2`,`Format=CA65`),DPCM 同名 `.dmc` 一并落盘;`project.toml` 登记乐曲文件与曲名。
- 导入入口:选 `.s`(+`.dmc`)→ 拷入 `music/` → 登记 → 加入 ca65 源列表(音乐数据作为额外 ca65 源参与链接)。
- `music/` 监听(`notify` crate)→ 触发增量重建 → 内嵌模拟器即时试听。
- 理由:附录 A-5/A-6——纯文件对接省掉 .NET 跨平台/签名;主路径单向(FamiStudio→工程),不回写 `.fms`。

### D4 内置 tracker 用 fc-core APU 预览(护城河)
- 乐曲模型:Song{ tempo, patterns, instruments, order };Channel = Pulse1/Pulse2/Triangle/Noise/DPCM。
- 播放引擎:逐帧把"音符+乐器包络+效果"解算为 APU 寄存器写入;驱动 `fc-core` APU 取样输出(经 `ControlDeck` 新增"音频预览"接口:reset APU、每帧写寄存器、抽取样本/通道电平),用现有 cpal 播放。
- 理由:附录 A-2——正确性内核(2A03 合成)已具备,用同一台 APU 预览,AI 还能在内核里"听"和断言,这是 FamiStudio 没有的闭环。
- **分批交付**(附录 A-7):先 乐曲模型 + 播放引擎 + 乐器 + 音序/Pattern + 汇编导出(够用),**钢琴卷帘 + 高级效果后补**。

### D5 汇编导出对齐 FamiStudio/FamiTone2 引擎格式
- 内置 tracker 导出 ca65 乐曲数据 + 捆绑 FamiTone2/FamiStudio 声音引擎 asm(作为工程模板),与 D3 的 FamiStudio 导出走同一条链接路径。
- 理由:统一"音乐数据 → ca65 源 → ld65 链接"路径;内置与外部产出对称。

### D6 build-pipeline 扩展:资源装配 + 增量重建
- 把 `music/*.s`(及内置 tracker 导出)纳入 ca65 源;CHR/map 经 asm `.incbin` 已被 ld65 纳入。
- 文件监听(`chr|map|music/` + `src/`)→ 仅重跑受影响步骤 → 重链接 → 内嵌运行刷新。
- 理由:把 M1 的"一键全量构建"升级为"改资源即听/即见"的回写闭环(附录 A-6 第 4 点)。

### D7 rom-packaging:CHR 来源多样化
- CHR 可来自 asm `CHARS` 段(M1 现状)**或**工程 `chr/*.chr`(经 `.incbin`)。打包尺寸守卫沿用 M1(链接产物大小 vs 头部声明)。

## Risks / Trade-offs

- **tracker 范围爆炸(最大风险,附录 A-1/A-4 的"三道悬崖")** → 严格 B 方案:不做扩展芯片、不做二进制导入、卷帘"够用";分批交付,先把"音序+乐器+导出"打通验证。
- **驱动 fc-core APU 预览可能诱使改核心** → 只经 `ControlDeck` 加预览接口(复位/逐帧写寄存器/取样),不动锁步时钟与读写顺序。
- **FamiStudio 导出配置不一致导致链接失败** → 固化推荐导出配置文档 + 导入时一键校验(引擎/格式/命名);失败给明确诊断。
- **文件监听抖动/重复重建** → `notify` 去抖(debounce)+ 构建串行化(复用 M1 的 build 取消/串行)。
- **PNG→CHR 颜色量化不可控** → 要求输入为已量化的 4 色索引图;非法颜色报错并提示。
- **增量构建正确性** → M2 先做"受影响目录 → 全量重链"的保守增量;真正的细粒度增量留后。

## Migration Plan

纯增量,叠加在 M1 之上,不破坏现有闭环。资源能力以新面板 + 新命令暴露;build-pipeline/rom-packaging 仅新增 ADDED 需求(不改 M1 既有行为)。tracker 分批落地见 tasks。

## Open Questions

- FamiStudio 声音引擎版本与导出标签命名以实测最新版为准。
- tracker 汇编导出对齐 FamiTone2 还是 FamiStudio 引擎格式——倾向 FamiStudio(功能更全),落地时按引擎模板可得性定。
- `ControlDeck` 音频预览接口的取样粒度(整帧样本 vs 通道电平)——实现时按 tracker UI 需求定。
