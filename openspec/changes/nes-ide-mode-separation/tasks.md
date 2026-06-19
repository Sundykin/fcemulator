# Tasks — nes-ide-mode-separation

> 阶段对齐 design 的 Migration Plan;每阶段可独立 `tauri dev` 活体验收并回退。
> 实施约束:`fc-core` 不动;Pinia store 保留末尾 `acceptHMRUpdate`;玩家 5 页逐张比对 `ui设计/*`。

## 1. 模式骨架与分壳(app-navigation)

- [x] 1.1 `stores/emu.ts` 新增 `mode: "launcher" | "player" | "studio"`;`view` 退化为玩家模式内部场景选择(保留 `acceptHMRUpdate`)
- [x] 1.2 新增 `views/LauncherView.vue`:游戏/创作两大卡片 + 最近游戏/最近工程快捷项(dark token 极简卡片式)
- [x] 1.3 `layout/TitleBar.vue` 加常驻模式切换器(切到另一模式 / 返回选择页)
- [x] 1.4 `layout/AppShell.vue` 改为按 `mode` 分壳:launcher → LauncherView;player → 玩家外壳(Toolbar+FooterNav+场景);studio → 创作外壳(IdeView)。玩家外壳仅在 player 渲染
- [x] 1.5 模式记忆:持久化上次模式(localStorage / tauri store),启动进选择页并高亮上次模式
- [x] 1.6 切换模式时 emu 与 project store 状态各自保活,切回不丢(Pinia store + Rust worker 保活;v-if 重挂的视图从 store 复原)
- [x] 1.7 活体验收(tauri dev + tauri_eval):启动落地 launcher(切换器隐藏、上次模式 player 高亮);player↔studio 互切,studio 下玩家 toolbar/footer 消失;切回 player ROM 保留(SuperMarioBro.nes)✅

## 2. 玩家模式收口与会话化(game-session)

- [x] 2.1 `layout/FooterNav.vue` 移除「创作」标签;玩家底栏仅含应用级(游戏库/设置)+ 会话级入口(随会话出现/隐藏)
- [x] 2.2 引入"会话"驱动:会话级入口(存档管理/金手指/机器级调试)存在性由 `hasRom` 驱动,无会话时从导航**移除**(非禁用)— footer `items` computed + store `panel` 状态
- [x] 2.3 `views/SavesView.vue` 改造为右侧抽屉/覆盖层容器(AppShell `.drawer.saves` 承载;`load/importState` 后 `closePanel()` 回游戏)
- [x] 2.4 `views/CheatsView.vue` 改造为会话抽屉/覆盖层(`.drawer.cheats`,内容复用)
- [x] 2.5 `views/DebugView.vue` 改造为会话抽屉/覆盖层(`.drawer.debug` 近全屏覆盖;机器级检视:寄存器/内存/反汇编)
- [x] 2.6 入口去重:删 `Toolbar` 金手指/设置按钮与 `ControlPanel` 金手指页签("打开金手指页→"),金手指仅 footer 一处
- [x] 2.7 存档消歧:transport 文案改「快存/快读(槽1)」;"存档管理"为会话抽屉;图标/文案区分
- [x] 2.8 离场暂停显式化:把 `navPaused` 暴露为标题栏"运行中:<游戏名>"药丸,点击返回游戏页恢复;保持 pause/resume 语义不变
- [x] 2.9 状态规则:无 ROM 落地游戏库(`setMode('player')`)且会话入口/transport 不出现(Toolbar `.group` v-if hasRom);载入 ROM 落地游戏页;切换 ROM 重置 `panel`
- [x] 2.10 玩家页比对 mockup,**记录的有意偏差**:① 存档/金手指/调试 由整页→会话抽屉/覆盖层(强化"对象=这局游戏");② 无 ROM 隐藏 transport(mockup 是禁用态);③ ControlPanel 去金手指页签;④ footer 去「创作」。视觉逐张比对留 2.11 活体
- [x] 2.11 活体验收:footer=[游戏库,存档,金手指,调试,设置](无创作);工具栏金手指/设置按钮已删、ControlPanel 仅 控制/信息;三抽屉(存档"存档管理"/金手指/调试机器检视含寄存器+反汇编)均叠在运行的游戏之上,Esc(window)/遮罩/× 三途径关闭;翻库显示"运行中：SuperMarioBro.nes"药丸,点击一键恢复运行 ✅

## 3. 创作模式:模拟器降为运行预览(studio-preview)

- [x] 3.1 `views/IdeView.vue` dockview 新增「运行预览」面板(`ide/PreviewPanel.vue`,复用 `useEmuLoop`/canvas 渲染路径),右侧运行列
- [x] 3.2 `doRun()` 改为:构建成功 → `emu.openPath(产物, keepMode=true)` → 聚焦运行预览面板;**已移除 `emu.setView("main")`**(不跳玩家页,模式仍 studio)
- [x] 3.3 机器级检视器复用:`DebugView` 本就自洽无玩家耦合,直接复用为机器检视组件(玩家会话抽屉 + 创作 dock 共用同一组件,免抽取)
- [x] 3.4 创作侧新增机器级检视「可停靠面板」(`inspect` = DebugView),与编辑器源码调试并存于右侧运行列
- [x] 3.5 `composables/useHaltWatch.ts` 命中断点改为 `setMode("studio")` + `onHalt` 编辑器高亮,不再跨壳 `setView("ide")`;沿用 `source-debug-link` 行为
- [x] 3.6 运行预览「分离预览」按钮 → `dockApi.addFloatingGroup`(浮动面板,可拖回停靠);默认停靠,分离为增量(try/catch 守卫,_真·OS popout 留后,活体核验_)
- [x] 3.7 活体验收:demo 工程 dockwrap 含 `.preview-panel`;点「运行」→ 构建成功(build/game.nes,0 错)→ **模式仍 studio**、ROM=game.nes、玩家外壳不出现、预览面板出现游戏画布 ✅(断点浮现/分离浮窗为 best-effort,后续可深验)

## 4. 打磨与收尾

- [x] 4.1 状态栏/键位文案随新模式与会话语义校订(复查:无残留旧视图/「创作 tab」文案;键盘输入已门控 player 模式)
- [x] 4.2 `vue-tsc --noEmit` + `vite build` 通过;`cargo build --manifest-path fc-tauri/src-tauri/Cargo.toml` 通过(后端无改动,确认无回归)
- [x] 4.3 全链路活体验收(`tauri dev` + `tauri_eval`)完成:launcher → player(SMB 运行/会话抽屉/药丸)→ studio(隔离/运行预览),逐条对照三份 spec 场景通过
- [x] 4.4 更新 `docs/`(模式分离使用说明)并 `openspec validate nes-ide-mode-separation --strict` 通过
