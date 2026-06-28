# M1 · 创作最小闭环 —— 使用说明

> 对应 OpenSpec 变更 `nes-ide-milestone-plan` 的 M1 目标:
> **新建工程 → 写汇编 → 一键构建(ca65→ld65)→ 打包 .nes → 内嵌运行 → 调试**,
> 报错/断点 ↔ 代码行联动。

## 启动

```sh
npm --prefix fc-tauri run tauri dev
```

底部导航栏点 **「创作」** 进入 IDE。

## 工作流

1. **新建/打开工程**:工具栏「工程」→ 选模板(空白/横版/演示)+ 父目录 + 工程名 → 创建。
   - 工程目录含 `project.toml` 与约定子目录 `src/ chr/ music/ map/ build/`。
   - `空白` 是最小 NROM 骨架;`横版`/`演示` 是可直接构建运行的 "Catch the Dot" 小游戏样例,方向键移动角色追目标。
   - `横版`/`演示` 会同时生成可编辑资源 `chr/sprites.chr` 与 `map/room.bin`;源码默认 `.incbin` 这些资源,所以改 CHR/地图后重新运行即可看到变化进入 ROM。
2. **编辑**:左侧文件树点开 `src/main.s`;编辑器有 6502/ca65 高亮、补全(助记符/寄存器/伪指令)、折叠;`Cmd/Ctrl+S` 保存。
   - 顶栏「资源」或 `Cmd/Ctrl+P` 可快速打开工程内源码、CHR、地图和音乐资源,不用先展开文件树;空查询会先列出最近资源,并保留源码行、CHR 图块、地图格或 tracker 行等位置。相邻的资源历史按钮或 `Cmd/Ctrl+[` / `Cmd/Ctrl+]` 可在最近打开的创作资源间后退/前进。
   - CHR 资源编辑器会把 8×8 图块放大到可操作区域,并支持旋转、翻转、按像素平移、撤销/重做,适合边画边把图块送回地图里验证。
3. **ROM 头**:工具栏「ROM 头」可视化编辑 iNES 头(Mapper/PRG/CHR/镜像/电池)。头部由 `project.toml` 权威生成并在打包时前置——**不写在汇编里**。
4. **构建**:工具栏「构建」运行 `ca65 → ld65`,产出 `build/<name>.nes`。
   - 「问题」标签列出错误/警告,**点击跳转到对应源码行**。
   - 「输出」标签是完整构建日志。
5. **运行**:工具栏「运行」把刚打包的 `.nes` 载入内嵌模拟器预览;默认演示工程会显示一个可用方向键移动的 16×16 精灵与目标。预览画面获得焦点时,方向键/Z/X/Enter/Space 会作为手柄输入送入当前 ROM。
6. **验证**:工具栏闭环状态里的手柄芯片可一键执行游戏验证:自动保存、构建、运行,并检查可见预览运行态和非空画面。Build 面板「体检」也会显示 `游戏验证` 状态,过期时可直接重新验证。
7. **调试**:在编辑器**左侧断点槽点击**即可在该行下/清断点(基于构建产出的 dbgfile 行映射);命中时自动切回 IDE 并高亮所在行。

## IDE 创作 MCP

Tauri IDE 启动时会同时启动一个语义级 MCP socket:`/tmp/fc-tauri-ide-mcp.sock`。项目级智能体应连接
`.mcp.json` 中的 `fc-ide` 服务器,通过 `ide_*` 工具直接操作**同一个** Tauri 后端工程状态:

- `ide_new_project` / `ide_open_project` 新建或打开当前 IDE 工程。
- `ide_scaffold_game` 一次生成可玩的简单游戏蓝图:源码、CHR、地图、tracker 乐曲、导出音乐引擎,并可选直接构建/运行到真实预览。
- `ide_get_state` 查询 live IDE 工程雷达:资源分类/缺失、map↔CHR 绑定、构建产物新旧状态、最近诊断、源码映射摘要,以及 `ui.active_editor` 当前编辑上下文、`ui.resource_history` 资源后退/前进/最近资源状态和 `ui.game_verify` 游戏验证状态。
- `ide_create_resource` 创建空白源码/CHR/地图/乐曲资源,登记到 `project.toml`,并让真实 IDE 打开对应编辑器。
- `ide_read_file` / `ide_write_file` 写源码;写入 `src/*.s` / `.asm` 会自动登记进 `sources`。
- `ide_patch_source` 按 1-based 行号小范围替换/插入源码,自动登记 `src/*.s` / `.asm`,并让真实 IDE 跳到修改行。
- `ide_read_chr` / `ide_write_chr` 写 CHR 像素资源。
- `ide_read_map` / `ide_write_map` / `ide_bind_map_chr` 写地图和资源绑定。
- `ide_patch_source` / `ide_patch_chr_tile` / `ide_patch_chr_pixels` / `ide_transform_chr_tile` / `ide_patch_map_cells` / `ide_patch_song_cell` / `ide_patch_song_cells` 小粒度修改源码、CHR 图块或像素、CHR 图块变换、地图格或 tracker 单元格/短旋律,适合智能体迭代资源而不整文件重写。
- `ide_transform_chr_tile` 支持 `rotate_cw`/`rotate_ccw`/`flip_h`/`flip_v`/`shift_left`/`shift_right`/`shift_up`/`shift_down`,平移可传 `wrap=true` 循环卷绕边缘像素;执行后真实 CHR 编辑器会聚焦该 tile。
- `ide_patch_active_context` 读取 `ui.active_editor` 作为默认目标,直接修改当前可见源码行、CHR 图块/悬停像素、地图 `focus_cell` 当前格或 tracker 单元格;源码可用 `scope=selection` 替换当前可见选区行范围,Map 可用 `scope=brush|selection` 扩展到当前画笔范围或选区,音乐可用 `scope=phrase` 从当前 Pattern 行/声道写入 `notes` 短旋律,或用 `scope=selection` 将 `note/instrument/volume/fx/param` 批量写入当前可见 Pattern 选区。地图批量 patch 后会把修改矩形保留为可见选区,tracker 批量 patch 后会保留 Pattern 行/声道范围,便于连续区域编辑。智能体已聚焦资源后只需提供本次要写入的内容。
- `ide_read_song` / `ide_write_song` 写 tracker 乐曲资源;`ide_patch_song_cell` 会定位到对应 Pattern 行/声道,`ide_patch_song_cells` 可用 `cells[]` 精确批量改多格,或用 `notes[] + start_row/start_channel/row_step` 一次写入旋律。
- `ide_export_song` 将 `.song.json` 导出为 `music/*.s` + `music/fc_player.s`,登记为音乐构建输入,并刷新真实 IDE 文件树/工程清单。
- `ide_wire_song_player` 将已导出的 tracker 播放器接入源码:补 `.import fc_player_init, fc_player_tick`,在 `reset` 调 init,在 `nmi` 调 tick,并跳到插入行。
- `ide_open_resource` 打开并聚焦源码/CHR/地图/音乐编辑器,让智能体写完资源后直接把真实 IDE 切到对应上下文。
- `ide_focus_resource` 打开并定位资源:源码可跳到 `line`,CHR 可选中 `tile`,地图可跳到 `x/y` 与 `layer`,音乐可跳到 `pattern/row/channel`。
- `ide_wait_ui_context` 等待 `ui.active_editor` 匹配期望的源码行/源码选区、CHR 图块、地图格/图层/选区矩形或音乐 Pattern 单元格/短句选区,用于确认异步前端已经完成打开与聚焦。
- `ide_build` / `ide_run` 调用 IDE 构建链并把产物加载进当前预览。
- `ide_verify_game` 在同一个 Tauri IDE 进程内构建/运行并读取可见预览证据:运行态、非空画面统计、可选手柄输入导致的内存变化;结果会进入 IDE 顶栏验证芯片和 `ide_get_state.ui.game_verify`。
- `ide_press_buttons` / `ide_read_memory` 对当前预览 ROM 做运行验证。

这些工具执行后会通过 Tauri event 通知前端刷新文件树、工程清单、构建面板和预览状态。智能体开始工作、
修复构建错误或决定能否运行旧 ROM 前,应先读 `ide_get_state`:其中 `build.output_status=current` 才表示
磁盘 ROM 对应最近一次成功构建;`stale_after_failed_build` 表示旧 ROM 仍在但当前代码构建失败。也就是说,
`fc-ide` 不是一个不可见的离线核心;它是 live IDE 的编程接口。`ui.active_editor` 由前端通过 IPC 回写,
会报告源码行、CHR 图块、地图 `focus_cell`/图层/画笔或 tracker Pattern 单元格等语义状态。`fc-tauri` MCP 仍保留为
DOM/窗口调试桥,主要用于验证 UI 是否刷新正确。
`ui.game_verify.stale=true` 表示验证发生后又有新的构建或预览载入,应重新调用 `ide_verify_game`。
人类用户也可以直接点击顶栏手柄验证芯片或 Build「体检」里的 `游戏验证` 动作,它复用同一个 Tauri
进程内验证路径。

同时,Tauri 启动时也会启动 live emulator MCP socket:`/tmp/fc-tauri-emu-mcp.sock`。`.mcp.json`
中的 `fc-emu` 默认连接这个 socket,因此 `emu_*` 工具会操作**可见的**播放器/IDE 预览,而不是另开一个
隐藏模拟器核心:

- `emu_load_rom` 会把 ROM 载入当前 Tauri 模拟器界面。
- `emu_step_frame` / `emu_press_button` / `emu_control` 会驱动界面里的同一台机器。
- `emu_capture_screen` / `emu_read_memory` / `emu_trace` / `emu_event_dump` 读取的是当前可见运行态。

需要纯核心无界面的调试时,再连接 `.mcp.json` 中的 `fc-emu-core`。

## cc65 工具链

`ca65`/`ld65` 作为捆绑 sidecar,位于 `fc-tauri/src-tauri/vendor/cc65/<target-triple>/`。
解析顺序:`FC_CC65_DIR` 环境变量 → 捆绑目录 → `PATH`。其它平台二进制用
`vendor/cc65/build-cc65.sh` 在对应机器生成。

## 已知限制(M1)

- 仅保证**开发主机 target**;跨平台二进制矩阵与 macOS 签名/公证留到 M4。
- 链接脚本默认仅内置 **NROM(Mapper 0)**;其它 Mapper 需在 `project.toml` 用 `linker_cfg` 指定自定义 `.cfg`。
- 行级断点依赖构建产出的 dbgfile;**未成功构建前**无源码映射,届时仅支持文件级诊断跳转。
- `横版`/`演示` 模板已使用独立 `.chr` 与 `.bin` 地图资源;`空白` 模板仍保留最小内联 CHR 占位。

## 第三方组件与许可

| 组件 | 用途 | 许可证 |
|---|---|---|
| cc65(ca65/ld65 V2.19) | 汇编/链接 sidecar | zlib(见 `fc-tauri/src-tauri/vendor/cc65/LICENSE`) |
| CodeMirror 6(@codemirror/*) | 代码编辑器内核 | MIT |
| dockview-vue | IDE 分栏布局 | MIT |

以上均允许商业闭源打包。发行时需随附各组件 LICENSE。
