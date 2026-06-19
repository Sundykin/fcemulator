## Why

fc-tauri 的前端是一个**扁平的 7 视图模型**(`stores/emu.ts` 的 `view: main | library | saves | cheats | debug | settings | ide`),且 **Toolbar(游戏 transport)+ FooterNav 这套游戏中心外壳是全局的**,套在每一个视图外面。后果:IDE(「创作」)只是并排的第 7 个 tab,天然继承了游戏外壳 → **没有"模式"边界,主次不分**;`存档/金手指/调试`本是"当前这局游戏"的子功能,却被提拔成与`游戏库/创作`平级的顶层目的地,且**入口重复**(金手指 ×3、"存档"与"调试"各自一词两义);没载入游戏时这些 tab 只是灰着,离开游戏页又有隐藏的自动暂停魔法。M1/M2 已把创作闭环做厚,继续往这个塌平的导航里塞功能只会更乱,**现在是把模式边界立起来的时机**。

## What Changes

- **引入顶层"模式"层**:`launcher | player | studio` 凌驾于现有 `view` 之上。启动先到**模式选择页**(🎮 游戏 / 🛠 创作),TitleBar 常驻**模式切换器**随时互通;记住上次模式。**BREAKING**:`AppShell` 从"7 选 1 v-if"改为"先按模式分壳,壳内再选场景"。
- **确立三层信息架构:模式 → 场景 → 面板**,并区分**应用级**(游戏库 · 设置 · 切换模式,恒在)与**会话级**(游戏页 · 存档 · 金手指 · 机器级调试,**仅载入 ROM 时存在,无则隐藏而非灰掉**)。
- **游戏模式收口**(忠于现有 5 张玩家 mockup):底栏**移除「创作」**(它本就不在 mockup 里);`存档/金手指/机器级调试`降为**右侧抽屉/覆盖层**形态的会话面板,游戏画面留背景,强化"对象=这局游戏";**入口去重**——金手指 3→1,"存档"区分「快速存档/读档(槽1)」与「存档管理页」,"调试"区分机器级与源码级。隐藏的 `navPaused` 自动暂停**显式化**为"▸ 运行中:X"药丸。
- **创作模式以 IDE 为主体,模拟器降为预览**:运行结果灌入 dockview 里一块**可停靠、可分离为浮窗**的"运行预览"面板(替代现在 `IdeView.doRun()` 的 `emu.setView("main")` 跳玩家整页);源码级断点调试与编辑器一体,机器级检视器也作为可停靠面板提供。**BREAKING**:`doRun` 不再切到玩家页。
- **统一两个"调试"的归属**:机器级(寄存器/内存/反汇编,`断点调试` mockup)= 玩家会话面板 +(创作)可停靠面板;源码级(断点↔代码行)= 创作模式,沿用 `source-debug-link` 既有行为。

## Capabilities

### New Capabilities
- `app-navigation`:顶层模式模型(launcher/player/studio)、模式选择页、TitleBar 常驻切换器、按模式分壳、以及"应用级 vs 会话级"的层级与可见性规则。
- `game-session`:玩家模式的"游戏会话"概念——载入 ROM 即开一会话;存档/金手指/机器级调试作为**会话级面板**(右侧抽屉/覆盖层)、单一入口、无会话时隐藏;空状态/运行中/翻库时的状态规则与"运行中"药丸。
- `studio-preview`:创作模式的模拟器"运行预览"——dockview 可停靠面板、可分离为浮窗;`构建→运行`灌入预览而非跳玩家页;机器级检视器作为创作侧可停靠面板。

### Modified Capabilities
<!-- 无。现有 specs(project-model/code-editor/build-pipeline/rom-packaging/source-debug-link)是后端/构建/调试行为向,其需求不变;本次只改"功能归属哪个模式/在哪里浮现",由上面三个新导航能力承载。source-debug-link 的断点↔源码行行为原样复用,仅其宿主固定为 studio 模式——属导航关注点,不构成需求变更。 -->

## Impact

- **`fc-tauri/src/stores/emu.ts`**:新增 `mode` 状态与"会话"概念;把隐藏的 `navPaused` 自动暂停显式化;会话级面板的存在性由"是否有 ROM"驱动。
- **`fc-tauri/src/layout/`**:`AppShell.vue` 按模式分壳;`FooterNav.vue` 玩家专用、去「创作」、收敛重复入口;`Toolbar.vue` 仅在会话内呈现 transport;`TitleBar.vue` 加模式切换器。
- **`fc-tauri/src/views/`**:新增模式选择页(launcher);`SavesView/CheatsView/DebugView` 改造为会话级抽屉/覆盖层;`IdeView.vue` 的 `doRun` 改为灌入预览面板。
- **`fc-tauri/src/ide/`**:新增"运行预览"停靠面板(+ 分离浮窗);机器级检视器作为可停靠面板复用。
- **`fc-core` / `src-tauri`**:无需求变更(沿用现有 `ControlDeck` 渲染/调试接口);浮窗预览若需第二 webview 帧传输,属前端 plumbing,不动核心。
- **设计稿**:玩家 5 页严格比对 `ui设计/*`;**模式选择页与创作模式布局无 mockup**(创作模式属既有"无 mockup 例外",启动页为新增,需在 design 里定其形态)。
