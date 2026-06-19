# 任务清单 · M1 详细 + M2–M4 里程碑占位

> 战略=集成优先(v2 终版)。M1 = 创作最小闭环,本变更细化到可实施;M2–M4 为里程碑占位,待 M1 验证后细化为独立 change。
> 实施约束:`fc-core` 保持 IO-free 不改核心;编排层落 `fc-tauri/src-tauri/`,IDE 外壳落 `fc-tauri/src/`;Pinia store 底部 `acceptHMRUpdate`;`tauri dev` orphan 进程按裸名 kill。

## 1. M1 · 工程模型(project-model)

- [x] 1.1 在 `fc-tauri/src-tauri/` 定义 `project.toml` 数据模型(name/mapper/源码列表/CHR/音乐/地图/链接 cfg/输出 nes)与 serde 解析
- [x] 1.2 实现工程加载:解析 `project.toml`,字段缺失补默认、非法字段返回字段级错误
- [x] 1.3 实现新建工程:从模板生成 `project.toml` + 约定目录骨架(`src/ chr/ music/ map/ build/`)
- [x] 1.4 实现打开/保存工程:活动工程单例,保存回写 `project.toml` 且读回一致
- [x] 1.5 暴露 Tauri 命令:`project_new` / `project_open` / `project_save` / `project_get`(返回结构化 JSON)
- [x] 1.6 制作 3 个内置模板(空白/横版/演示),每个含可直接成功构建的最小骨架 + 链接脚本
- [x] 1.7 文件树后端:列目录/新建/重命名/删除文件命令,变更同步磁盘

## 2. M1 · IDE 外壳 + 文件树前端

- [x] 2.1 在 `fc-tauri/src/` 搭 IDE 布局(dockview/分栏:文件树 | 编辑器 | 构建面板 | 调试面板),复用现有播放器/调试组件 — `views/IdeView.vue`(dockview-vue) + FooterNav「创作」入口
- [x] 2.2 新建 Pinia `project` store(活动工程/文件树状态;底部 `acceptHMRUpdate`) — `stores/project.ts`
- [x] 2.3 文件树组件:渲染工程目录、右键新建/重命名/删除、与后端命令联动 — `ide/FileTreePanel.vue` + `PromptDialog.vue`
- [x] 2.4 新建/打开工程的 UI 入口(模板选择 + 目录选择对话框) — `ide/NewProjectDialog.vue`

## 3. M1 · 代码编辑器(code-editor)

- [x] 3.1 集成 CodeMirror 6 到 IDE,支持多标签打开/编辑/保存工程内文件 — `ide/EditorPanel.vue` + `editor/cm6502.ts`(Cmd/Ctrl+S 保存)
- [x] 3.2 未保存标识 + 关闭标签时未保存提示 — 脏标记圆点 + 关闭确认
- [x] 3.3 接入 6502/ca65 语法高亮(自写 StreamLanguage)+ 代码折叠(.proc/.scope 等)
- [x] 3.4 补全:6502 助记符 + PPU/APU 寄存器常量 + ca65 伪指令
- [x] 3.5 编辑器与文件树联动:仅开活动工程内文件;文件树重命名/删除同步打开的标签页(store onRenamed/onDeleted)

## 4. M1 · cc65 sidecar 构建编排(build-pipeline)

- [x] 4.1 按开发主机 target-triple 预编译/捆绑 ca65、ld65 为 Tauri sidecar;捆绑标准 `.cfg` 链接脚本与 cc65 头/库 — vendored arm64 ca65/ld65 (V2.19) + nrom.cfg + LICENSE + 复现脚本于 `vendor/cc65/`
- [x] 4.2 sidecar 进程管理:spawn、设 `CC65_INC/LD65_LIB/LD65_CFG`、捕获 stdout/stderr、超时/取消 — spawn/排空/超时/取消已实现;env 注入待模板用到 cc65 include/lib 时再加(M1 纯 asm 模板不需要)
- [x] 4.3 构建编排器:`ca65`(各源)→ `ld65`(链接,优先工程自定义 cfg,否则按 Mapper 选捆绑 cfg)
- [x] 4.4 sidecar 缺失/不可执行 → 返回工具链不可用的明确错误
- [x] 4.5 日志解析:把 stdout/stderr 解析为结构化构建结果(成功/失败 + 错误/警告诊断列表,含 `file:line`)
- [x] 4.6 暴露 Tauri 命令:`build_run` / `build_cancel`;构建面板展示日志 + 诊断摘要 — 后端命令完成;面板展示属前端(Group 2)

## 5. M1 · ROM 打包(rom-packaging)

- [x] 5.1 iNES 头模型 + 可视化编辑(Mapper/PRG 大小/CHR 大小/镜像/电池),持久化到工程 — `ide/HeaderEditor.vue`
- [x] 5.2 头字段校验:非法值阻止应用并报字段级错误 — 后端 validate() + UI 回滚并显示字段级错误
- [x] 5.3 装配器:manifest 权威 iNES 头 + 链接产物按头部装配为 `.nes`;大小不一致(CHR 缺失)报错不产损坏 ROM — `build_ines_header` + 尺寸守卫(已测)
- [x] 5.4 构建末步产出 `build/<name>.nes`,并暴露"构建后一键运行"命令(加载进 `ControlDeck`) — IdeView「运行」

## 6. M1 · 源码映射与调试联动(source-debug-link)

- [x] 6.1 构建时 `ca65 -g` + ld65 `--dbgfile`,解析 line→span→seg 得 `地址 ↔ {文件,行}` 映射,每次成功构建刷新(`parse_dbgfile`,已测)
- [x] 6.2 诊断 ↔ 代码行:构建面板诊断点击跳转编辑器对应行;无精确行号时降级为文件级定位
- [x] 6.3 行级断点:编辑器断点槽下断 → 经映射转 PC → 调 `dbg_add_breakpoint`;清断点同步移除(`breakpointGutter` + store)
- [x] 6.4 命中暂停:`useHaltWatch` 轮询→ PC 反查源码行,切回 IDE + 高亮(活动行);调试面板复用现有 DebugView。*实时行为待 live 验收*
- [x] 6.5 映射不可用时降级为"文件级"提示(无映射给出明确状态),不阻断运行/调试

## 7. M1 · 端到端验收

- [x] 7.1 闭环冒烟:用演示模板新建工程 → 编辑 → 一键构建 → 打包 `.nes` → 内嵌运行,全程无手动导出/导入 — live 验收通过(ROM 载入 mapper0/32K/8K/Vertical)
- [x] 7.2 报错跳转验收:故意引入汇编错误,确认诊断可点击跳到正确源码行 — live 验收通过(诊断 src/main.s:4 → 跳转活动行 = bogusop foo)
- [x] 7.3 断点验收:源码行下断,运行命中后暂停并高亮对应行,寄存器/内存可查 — live 验收通过(行8→$8005 命中,PC 反查行8,A=1、内存 A9 01 可读,切回 IDE;断点点 1 个)
- [x] 7.4 模板验收:三个模板均可"新建即构建成功" — `all_templates_build_from_scratch` 测试通过
- [x] 7.5 捆绑"第三方许可"清单(cc65 zlib 等)+ 文档:M1 使用说明与已知限制 — `docs/M1-创作IDE-使用说明.md` + `vendor/cc65/LICENSE`

## M2–M4 路线图(后续里程碑,各自独立 change)

M1 验证通过后,M2–M4 按计划拆为独立 OpenSpec change 推进(路线总览见
`proposal.md` / `design.md`):
- **M2 · 资源 + 音频**:CHR/地图轻量编辑器、FamiStudio 文件对接、`music/` 监听自动重建、内置精简 2A03 tracker、格式转换器。→ change `nes-ide-resources-audio`
- **M3 · AI 一栈式**:工程层 MCP、AI 编排、"内核即校验"闭环、IDE AI 面板。
- **M4 · 打磨/生态**:跨平台打包签名、模板/案例库、外部 pro 工具对接、更多 Mapper/FDS、多智能体。
