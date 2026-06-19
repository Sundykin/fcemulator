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
   - 三个模板都是可直接构建的最小 NROM 骨架。
2. **编辑**:左侧文件树点开 `src/main.s`;编辑器有 6502/ca65 高亮、补全(助记符/寄存器/伪指令)、折叠;`Cmd/Ctrl+S` 保存。
3. **ROM 头**:工具栏「ROM 头」可视化编辑 iNES 头(Mapper/PRG/CHR/镜像/电池)。头部由 `project.toml` 权威生成并在打包时前置——**不写在汇编里**。
4. **构建**:工具栏「构建」运行 `ca65 → ld65`,产出 `build/<name>.nes`。
   - 「问题」标签列出错误/警告,**点击跳转到对应源码行**。
   - 「输出」标签是完整构建日志。
5. **运行**:工具栏「运行」把刚打包的 `.nes` 载入内嵌模拟器并切到游戏页。
6. **调试**:在编辑器**左侧断点槽点击**即可在该行下/清断点(基于构建产出的 dbgfile 行映射);命中时自动切回 IDE 并高亮所在行。

## cc65 工具链

`ca65`/`ld65` 作为捆绑 sidecar,位于 `fc-tauri/src-tauri/vendor/cc65/<target-triple>/`。
解析顺序:`FC_CC65_DIR` 环境变量 → 捆绑目录 → `PATH`。其它平台二进制用
`vendor/cc65/build-cc65.sh` 在对应机器生成。

## 已知限制(M1)

- 仅保证**开发主机 target**;跨平台二进制矩阵与 macOS 签名/公证留到 M4。
- 链接脚本默认仅内置 **NROM(Mapper 0)**;其它 Mapper 需在 `project.toml` 用 `linker_cfg` 指定自定义 `.cfg`。
- 行级断点依赖构建产出的 dbgfile;**未成功构建前**无源码映射,届时仅支持文件级诊断跳转。
- CHR 目前随汇编 `CHARS` 段嵌入;独立 `.chr` 资源装配留到 M2。

## 第三方组件与许可

| 组件 | 用途 | 许可证 |
|---|---|---|
| cc65(ca65/ld65 V2.19) | 汇编/链接 sidecar | zlib(见 `fc-tauri/src-tauri/vendor/cc65/LICENSE`) |
| CodeMirror 6(@codemirror/*) | 代码编辑器内核 | MIT |
| dockview-vue | IDE 分栏布局 | MIT |

以上均允许商业闭源打包。发行时需随附各组件 LICENSE。
