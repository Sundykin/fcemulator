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
3. **ROM 头**:工具栏「ROM 头」可视化编辑 iNES 头(Mapper/PRG/CHR/镜像/电池)。头部由 `project.toml` 权威生成并在打包时前置——**不写在汇编里**。
4. **构建**:工具栏「构建」运行 `ca65 → ld65`,产出 `build/<name>.nes`。
   - 「问题」标签列出错误/警告,**点击跳转到对应源码行**。
   - 「输出」标签是完整构建日志。
5. **运行**:工具栏「运行」把刚打包的 `.nes` 载入内嵌模拟器预览;默认演示工程会显示一个可用方向键移动的 16×16 精灵与目标。预览画面获得焦点时,方向键/Z/X/Enter/Space 会作为手柄输入送入当前 ROM。
6. **调试**:在编辑器**左侧断点槽点击**即可在该行下/清断点(基于构建产出的 dbgfile 行映射);命中时自动切回 IDE 并高亮所在行。

## IDE 创作 MCP

Tauri IDE 启动时会同时启动一个语义级 MCP socket:`/tmp/fc-tauri-ide-mcp.sock`。项目级智能体应连接
`.mcp.json` 中的 `fc-ide` 服务器,通过 `ide_*` 工具直接操作**同一个** Tauri 后端工程状态:

- `ide_new_project` / `ide_open_project` 新建或打开当前 IDE 工程。
- `ide_read_file` / `ide_write_file` 写源码;写入 `src/*.s` / `.asm` 会自动登记进 `sources`。
- `ide_read_chr` / `ide_write_chr` 写 CHR 像素资源。
- `ide_read_map` / `ide_write_map` / `ide_bind_map_chr` 写地图和资源绑定。
- `ide_read_song` / `ide_write_song` 写 tracker 乐曲资源。
- `ide_build` / `ide_run` 调用 IDE 构建链并把产物加载进当前预览。
- `ide_press_buttons` / `ide_read_memory` 对当前预览 ROM 做运行验证。

这些工具执行后会通过 Tauri event 通知前端刷新文件树、工程清单、构建面板和预览状态。也就是说,
`fc-ide` 不是一个不可见的离线核心;它是 live IDE 的编程接口。`fc-tauri` MCP 仍保留为 DOM/窗口调试桥,
主要用于验证 UI 是否刷新正确。

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
