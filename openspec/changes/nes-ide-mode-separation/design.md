## Context

fc-tauri 前端现状(已核实):导航是扁平的 `store.view = main|library|saves|cheats|debug|settings|ide`(`stores/emu.ts`),`AppShell.vue` 以一串 `v-if` 平铺;`Toolbar`(游戏 transport)+ `FooterNav`(6 个并排标签,含「创作」)是**全局游戏外壳**,套在每个视图外。会话功能(存档/金手指/调试)被提为顶层目的地,且入口重复(金手指 ×3:`Toolbar.vue:36` + `ControlPanel` 链接 + 底栏标签;"存档"=快存 vs 管理页两义;"调试"=机器级 `DebugView` vs 断点跳 IDE 两义)。离开游戏页时 `setView` 用隐藏的 `navPaused` 自动暂停(`emu.ts:46-59`)。`IdeView.doRun()` 构建成功后 `emu.openPath(...)` + `emu.setView("main")` 跳玩家整页。`useHaltWatch` 命中断点时 `setView("ide")`。

约束:`fc-core` 保持 IO-free,本次纯前端再架构 + 复用既有 `ControlDeck`/渲染/调试接口,不动核心时序;玩家 5 页严格比对 `ui设计/*` mockup;创作模式与模式选择页**无 mockup**(创作属既有"无 mockup 例外",选择页为新增,其形态在此定)。Pinia store 末尾的 `acceptHMRUpdate` 必须保留。

## Goals / Non-Goals

**Goals:**
- 立起 `mode` 顶层(launcher/player/studio),玩家与创作彻底隔离,各有专属外壳。
- 把存档/金手指/机器级调试降为**会话级**抽屉/覆盖层面板,单一入口,无会话时隐藏。
- 创作模式以 IDE 为主体,模拟器降为**可停靠、可分离**的运行预览;构建运行灌入预览而非跳玩家页。
- 消歧两个"调试"与两义的"存档";显式化离场暂停(运行中药丸)。
- 玩家 5 页严格忠于 mockup,仅改"关系/层级"与面板呈现形态,不重画画面。

**Non-Goals:**
- 不改 `fc-core` 时序/锁步时钟,不新增核心接口(沿用现有渲染/调试/存档/金手指接口)。
- 不重做玩家页、存档页、金手指页、断点调试页的视觉(只改其归属与呈现容器)。
- 不在本次重定义 `source-debug-link` 的断点/映射行为(原样复用,仅固定其宿主为 studio)。
- 不做多用户/云存档/账号等新功能;不引入新外部依赖(浮窗优先用 dockview 既有能力)。

## Decisions

### D1 顶层 `mode` 凌驾于 `view`,AppShell 按模式分壳
- 在 `stores/emu.ts` 新增 `mode: "launcher" | "player" | "studio"`,与现有 `view` 解耦:`view` 退化为**玩家模式内部**的场景选择(game/library/settings + 会话面板态),创作模式由 `project` store + dockview 自治。
- `AppShell.vue` 改为:先按 `mode` 选 `LauncherView` / 玩家外壳 / 创作外壳;玩家外壳内再选场景。玩家外壳(Toolbar+FooterNav)**只在 player 模式渲染**。
- **备选**:引入 vue-router 做真正的路由。**否决**——现有纯 Pinia 状态导航简单且快,引入 router 改动面更大、与 dockview/会话覆盖层模型不契合;`mode/view` 双层状态已足够。

### D2 模式选择页 + 标题栏常驻切换器
- 新增 `views/LauncherView.vue`:两张大卡片(游戏/创作)+ 最近游戏/最近工程快捷项。无 mockup → 采用与现有 dark token 一致的极简卡片式(本设计确立形态)。
- `TitleBar.vue` 加常驻模式切换器(段控/下拉),任一模式可切到另一模式或返回选择页。
- 记住上次模式:持久化到本地(localStorage 或 tauri store);启动策略 = 进选择页但高亮上次模式(保留首次清晰 + 后续顺手)。
- **备选**:每次启动强制选择页(摩擦大,否决)/ 直接进上次模式无选择页(分界感弱,否决)。

### D3 会话(session)概念替代"会话功能即顶层目的地"
- "载入 ROM"= 开启会话;会话级面板(存档管理/金手指/机器级调试)的**存在性**由 `hasRom` 驱动,而非平级路由项。无会话时这些入口从玩家外壳中**移除**(非禁用)。
- 面板呈现:`SavesView/CheatsView/DebugView` 由"整页 view"改造为**右侧抽屉/覆盖层**容器中的内容,游戏画面留背景。内部表单/列表(忠于 mockup)基本复用,仅换承载容器与打开/关闭交互。
- **备选**:保留整页但仅会话内可达(改动小但弱化"对象=这局游戏",否决)/ 底部弹出(与现有右侧 ControlPanel 布局冲突,否决)。

### D4 入口去重与语义消歧
- 金手指:删 `Toolbar.vue:36` 按钮与 `ControlPanel` 的"打开金手指页→",只留游戏页一处控制簇入口。
- 存档:transport 保留「快速存档/快读」(`store.save/load` 槽1,改명确文案),"存档管理"作为会话面板;二者图标/文案区分。
- 调试:玩家会话面板 = 机器级检视(`DebugView` 内容);源码级断点调试归 studio。`useHaltWatch` 命中断点改为在 **studio** 编辑器浮现,不再 `setView("ide")` 跨壳跳。

### D5 离场暂停显式化
- 保留"离开游戏页则暂停、返回则恢复"的语义,但把隐藏的 `navPaused` 暴露为 UI:标题栏"▸ 运行中:<游戏名>"药丸,点击返回游戏页恢复。状态仍存于 emu store,只是新增可见指示与显式入口。

### D6 创作模式:模拟器为可停靠/可分离的运行预览
- `IdeView` 的 dockview 新增「运行预览」面板,复用现有 `useEmuLoop`/canvas 渲染路径(同一帧拉取逻辑,渲染到该面板的 canvas)。
- `doRun()` 改为:构建成功 → `emu.openPath(产物)` → **聚焦运行预览面板**(`dockApi.getPanel('preview').api.setActive()`),**移除 `setView("main")`**。
- 分离浮窗:优先用 dockview 的 floating group / popout 能力承载该面板;若 popout 用第二 webview,帧传输沿用现有"拉取最新帧覆盖渲染"模式(不排队),属前端 plumbing,不动核心。**默认停靠,分离为可选**(对齐用户决策)。
- 机器级检视器:把 `DebugView` 的寄存器/内存/反汇编抽成可复用组件,作为 studio 的一块可停靠面板,与源码级调试并存。

### D7 严格分壳,创作不继承玩家外壳
- 创作模式不渲染玩家 `Toolbar`/`FooterNav`;沿用 `IdeView` 自有顶栏(工程/构建/运行/监听)。两模式 store 状态各自独立保活(切回不丢)。

## Risks / Trade-offs

- **大改 AppShell/导航,回归面大** → 分阶段(先立 mode 骨架与隔离,再迁会话面板,再改预览),每阶段 `tauri dev` + `tauri_eval` 活体验收;玩家页逐张比对 mockup。
- **会话面板从"整页 view"改抽屉,可能漏接事件/状态** → 复用原组件内容,仅换容器;保留 `acceptHMRUpdate`。
- **dockview popout/浮窗用第二 webview 的帧传输与音频** → 默认停靠面板(单 webview,零新风险);浮窗作为增量、可后置;音频仍由 worker 线程 cpal 主控,不随浮窗迁移。
- **隐藏暂停显式化后语义变化** → 保持既有 pause/resume 语义不变,仅加可见药丸与入口,降低行为意外。
- **mockup 保真 vs 抽屉化的取舍** → 抽屉内仍用 mockup 的页内布局;若覆盖层与 mockup 整页观感冲突,以"对象=这局游戏"的层级目标优先,并记录偏差。

## Migration Plan

纯前端、增量、可分阶段灰度,不破坏 `fc-core`/后端命令:
1. 立 `mode` 状态 + `LauncherView` + TitleBar 切换器 + AppShell 分壳;玩家/创作隔离(行为等价,先不动面板)。
2. 玩家收口:底栏去「创作」;存档/金手指/机器级调试改会话抽屉;入口去重 + 存档/调试消歧 + 运行中药丸。
3. 创作预览:`doRun` 灌入预览面板;机器级检视器面板化;`useHaltWatch` 改在 studio 浮现;预览分离浮窗(增量)。
4. 打磨:空/运行/翻库状态规则、模式记忆、文案与键位。
回滚:每阶段独立可回退到前一阶段(改动集中在 emu store + 几个 layout/view 文件)。

## Open Questions

- 模式记忆的启动策略最终取"进选择页高亮上次"还是"可配置直接进上次"——倾向前者,设置项后补。
- 浮窗预览用 dockview popout(第二 webview)还是轻量"置顶子窗口";落地时按 dockview-vue 实际 popout 能力与帧传输开销定,默认停靠不阻塞主线。
- 机器级检视器在玩家会话与创作侧复用同一组件的取数来源(玩家会话 ROM vs 预览 ROM)如何参数化——实现时按组件 props 注入当前 deck 句柄。
