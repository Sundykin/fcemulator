## Why

M1 已交付"工程 → 编辑 → cc65 构建 → 打包 → 内嵌运行 → 调试"的创作最小闭环(已归档,5 个能力规格已并入 `openspec/specs/`)。但一个 NES 游戏还需要**图形(CHR)、地图、音乐**资源,目前 IDE 完全没有资源创作能力——用户只能手写字节或借外部工具再手工塞进工程。

M2 把"资源 + 音频"补齐,让创作闭环从"能写能跑"升级到"能配齐一个真正可玩的小游戏"。战略仍遵循《策划案》v2 终版:**护城河自研(用自研 APU 内核做音频试听/校验),卡脖子环节借力(FamiStudio 文件对接,不打包)**。本里程碑按用户决策采用**全量 M2**——含内置精简 2A03 tracker。

## What Changes

- **内置 CHR 图形编辑器**:Canvas 画 8×8 图块、4 色调色板,导出 `.chr`(NES 2bpp)/`.inc`,工程内 `chr/` 落盘,asm 经 `.incbin` 引用。
- **内置地图/命名表编辑器**:图块拼接 + 属性表 + 碰撞层,导出字节数组(`.bin`/`.inc`)。
- **FamiStudio 文件对接(不打包)**:导出规范文档 + 一键导入 `.s`/`.dmc` → `chr|music/` 登记进工程;`music/` 文件监听 → 自动重建;用自研 APU/内核试听校验。
- **格式转换器**:图片(PNG)→ CHR;Tiled 导出 → 字节数组。
- **内置精简 2A03 tracker(自研主体,可分批)**:乐曲工程模型 + 播放引擎(音符/乐器/效果 → 逐帧写 APU,驱动 `fc-core` APU)+ 乐器系统(音量/琶音/音高/占空比包络 + DPCM)+ 音序/Pattern 视图 + 钢琴卷帘 + 基础效果 + 汇编导出(对齐 FamiTone2/FamiStudio 引擎)+ 基础 FTM 文本导入。

构建链路扩展:M1 的 build-pipeline 增加"资源装配"——CHR/地图/音乐源(`.s`/`.incbin`)纳入 ca65/ld65 流水线;`music/`/`chr/`/`map/` 变更触发增量重建。

非目标(Non-goals):不全量对标 FamiStudio(不做 6 颗扩展芯片 VRC6/VRC7/FDS/MMC5/N163/5B、不做 NSF/FTM 二进制保真导入、钢琴卷帘只做"够用"版);不打包 FamiStudio(.NET 自包含发布留 M4);自研汇编器仍不做(M4 可选)。

## Capabilities

### New Capabilities
- `chr-editor`: 内置 CHR 图形编辑器(8×8 图块网格、4 色调色板、铅笔/填充、导出 `.chr`/`.inc`,工程登记)。
- `map-editor`: 内置地图/命名表编辑器(图块拼接、属性/调色板、碰撞层,导出字节数组并登记)。
- `famistudio-integration`: FamiStudio 导出物对接(导入 `.s`/`.dmc` → `music/`、工程登记、`music/` 监听自动重建、内核试听)。
- `asset-converters`: 资源格式转换器(PNG → CHR 2bpp;Tiled 地图 → 字节数组)。
- `audio-tracker`: 内置精简 2A03 tracker(乐曲模型、驱动自研 APU 的播放引擎、乐器/DPCM、音序+钢琴卷帘编辑、效果、汇编导出、FTM 文本导入)。

### Modified Capabilities
- `build-pipeline`: 构建编排扩展为"资源装配 + 增量重建"——把 CHR/地图/音乐源纳入 ca65/ld65 流水线,并支持 `chr|map|music/` 文件监听触发重建。
- `rom-packaging`: 支持 CHR 来自工程 `chr/` 资源(`.chr`/`.incbin`),而非仅嵌入 asm。

## Impact

- **`fc-tauri/src-tauri/`**:新增资源 I/O 与转换(CHR/map/PNG/Tiled 编解码)、FamiStudio 导入与 `music/` 监听(`notify` crate)、增量构建;tracker 播放引擎需驱动 `fc-core` APU 做预览。
- **`fc-tauri/src/`**:新增 CHR 编辑器、地图编辑器、tracker(音序/卷帘 Canvas)、资源面板,纳入 dockview IDE。
- **`fc-core`**:复用现有 APU(`apu.rs`,周期级五声道)做 tracker 预览/校验;尽量不改核心(若需"按帧写寄存器并取样"接口,经 `ControlDeck` 暴露,保持 IO-free)。
- **外部依赖(集成)**:FamiStudio 仅**文件对接不打包**;新增 `notify`(文件监听)、可能的 PNG 解码(已有 `png` crate)。
- **承接**:M2 的资源/音乐工程化为 M3(工程层 MCP / AI 编排 / 内核即校验)提供更丰富的工程上下文。
