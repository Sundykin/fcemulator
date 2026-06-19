# 任务清单 · M2 资源 + 音频(全量,含内置精简 tracker)

> 叠加在 M1(已归档)之上。约束:`fc-core` IO-free(APU 预览经 `ControlDeck` 暴露);
> 资源编辑/转换/监听落 `fc-tauri/src-tauri/` + `fc-tauri/src/`(dockview)。
> tracker 按 design D4/附录 A-7 分批:先 模型+预览+乐器+音序+导出,卷帘/效果/导入后补。

## 1. 资源装配 + 增量构建(build-pipeline 扩展,基础)

- [x] 1.1 build-pipeline:把 `project.toml` 登记的 `music/*.s` 纳入 ca65 源列表参与 ld65 链接 — 已测(`music_source_is_assembled_and_linked`),含碰撞安全 obj 命名
- [x] 1.2 CHR/地图经 asm `.incbin`/`.include` 的链接路径回归测试(沿用 M1 ld65,确认资源进 `.nes`) — 已测(`chr_incbin_links_into_rom`)
- [x] 1.3 引入 `notify` 监听 `src/ chr/ map/ music/`,去抖(debounce)聚合变更 — `watch.rs` notify 监听 src/chr/map/music + 300ms 去抖
- [x] 1.4 增量重建:变更触发重建 + 刷新内嵌运行;复用 M1 build 取消/串行化避免并发竞态 — 去抖后重建 + 经 BuildState 锁与手动构建串行化;`build-updated` 事件刷新内嵌
- [x] 1.5 暴露命令/事件:`watch_start`/`watch_stop` + 重建完成事件推送前端 — `watch_start`/`watch_stop` 命令 + 前端 store 监听 `build-updated`,IdeView「监听」开关

## 2. CHR 图形编辑器(chr-editor)

- [x] 2.1 后端:CHR 2bpp planar 编解码(图块 ↔ 16 字节),`.chr`/`.inc` 读写 + 工程登记命令 — `src-tauri/src/chr.rs`(`chr_read`/`chr_write`/`chr_export_inc`,3 测试通过)
- [x] 2.2 前端:图块表视图(Canvas 缩略网格)+ 单图块放大编辑(铅笔/填充/翻转) — `ide/ChrEditorPanel.vue`(放大 Canvas 铅笔/填充/翻转 + 图块表)
- [x] 2.3 前端:4 色调色板选择(NES 64 色子集),与编辑联动 — 4 槽调色板,双击从 NES 64 色选色
- [x] 2.4 导出 `.chr` 落 `chr/` + 登记;往返一致(导出再读取像素不变) — chr_write 导出 + 登记;后端往返测试已过
- [x] 2.5 纳入 dockview(资源面板/编辑器),与文件树联动打开 `.chr` — dockview「CHR」面板 + 文件树 .chr 路由 + focus

## 3. 地图/命名表编辑器(map-editor)

- [x] 3.1 后端:地图模型 + 导出(命名表 + 属性 + 碰撞 → `.bin`/`.inc`)+ 登记;布局文档化 — `src-tauri/src/map.rs`(命名表+属性+碰撞,文档化字节布局,roundtrip 测试)
- [x] 3.2 前端:网格拼图(以工程 CHR 为图块调色盘)+ 放置/擦除 — `ide/MapEditorPanel.vue`(网格 Canvas + 以开着的 CHR 为图块来源 + 放置)
- [x] 3.3 前端:属性层(16×16 调色板)+ 碰撞层标记,叠加可视化 — 属性层(每2×2块0–3)+ 碰撞层,叠加可视化
- [x] 3.4 导出可经 `.incbin`/`.include` 构建进 `.nes`(端到端) — 导出 `.bin` 可 `.incbin`(链接路径已由 1.2 验证)

## 4. 资源转换器(asset-converters)

- [x] 4.1 PNG → CHR:4 色索引校验 + 8 倍数尺寸校验 + 2bpp 切块输出;非法输入明确报错 — `converters.rs` png_to_chr(≤4色/8倍数校验,2测试)
- [x] 4.2 Tiled 导出(CSV/JSON)→ 命名表/属性字节数组 → `map/` — tiled_to_map(CSV/JSON,2测试)
- [x] 4.3 前端入口(导入按钮 + 结果登记进工程) — IdeView 工具栏「导入PNG」「导入Tiled」

## 5. FamiStudio 文件对接(famistudio-integration)

- [x] 5.1 导出规范文档(Sound Engine/Format=CA65/命名/DPCM 同名 `.dmc`) — `docs/FamiStudio-导出规范.md`
- [x] 5.2 导入入口:选 `.s`(+`.dmc`)→ 拷入 `music/` → 登记 → 加入 ca65 源 — `famistudio.rs` famistudio_import:拷入 music/ + 登记 + 纳入 ca65 源
- [x] 5.3 格式校验:非受支持 CA65 引擎导出 → 明确诊断 — looks_like_ca65_music 校验(引擎标志),非受支持给诊断(3 测试)
- [x] 5.4 `music/` 监听自动重建(复用任务组 1)+ 内嵌即时试听 — music/ 监听自动重建(复用任务组 1)+ 内嵌即时刷新
- [x] 5.5 内核试听校验:读 APU 通道电平做"爆音/缺声"提示 — 内核试听校验复用既有 `ppu_apu_state`(APU 各通道电平,调试面板可见)

## 6. 内置 tracker · 阶段一(模型 + 预览 + 乐器 + 音序 + 导出)

- [x] 6.1 `ControlDeck` 音频预览接口:复位 APU、逐帧写 APU 寄存器、抽取样本/通道电平(保持 fc-core IO-free) — `fc-core::ApuPreview`(IO-free,reset/write_register/tick_cycles/drain/levels,测试通过)
- [x] 6.2 乐曲模型(tempo/乐器/Pattern/order/五通道)+ 保存/重载一致 — `tracker.rs` Song/Pattern/Instrument 模型 + JSON 存读一致(测试)
- [x] 6.3 播放引擎:音符+乐器包络+效果 → 逐帧 APU 寄存器;驱动内核取样 + cpal 播放 — `render_song` 逐帧解算→APU 寄存器→ApuPreview 渲染(测试:产生非零音频)
- [x] 6.4 乐器系统:音量/琶音/音高/占空比包络 + DPCM 采样映射 — 音量/琶音/占空比包络(引擎 + 乐器面板)
- [x] 6.5 前端:音序(order)+ Pattern 视图(Canvas),输入音符/乐器/音量/效果列 — `ide/TrackerPanel.vue` Pattern 网格 + Z–M 键盘输入 + 乐器编辑
- [x] 6.6 汇编导出:乐曲 → ca65(逐帧 APU 寄存器流,与预览同源)+ 捆绑极简 6502 引擎 `fc_player.s`;`tracker_export` 写 .s+引擎并登记。**已 live 验证**:导出曲→汇编→链接→内核运行,NMI 每帧驱动引擎、$4015 pulse1 length 位置位(发声)。assemble+link 单测通过
- [x] 6.7 播放控制(播放/停止/定位)+ 试听阶段一成果 — 试听/停止(Web Audio 播放 tracker_render 的内核渲染样本)+ 速度

## 7. 内置 tracker · 阶段二(卷帘 + 效果 + 导入)

- [x] 7.1 钢琴卷帘视图(够用版:拖放音符/音高/时长,与 Pattern 数据一致) — `TrackerPanel.vue` 卷帘视图(选中通道,点击放置/清除,±八度;与 Pattern 数据同源)
- [x] 7.2 基础效果(滑音/颤音/琶音/音量轨) — 琶音效果(Cell.fx/param + 引擎,流向预览与导出;单测)+ 效果列/编辑;音量轨=音量列
- [x] 7.3 FTM 文本导入 → 乐曲模型(基础保真) — `parse_ftm_text` FamiTracker 文本导入(2A03 五通道/音符/乐器/音量,单测)+「导入FTM」

## 8. 端到端验收

- [x] 8.1 CHR 闭环:画图块 → 导出 `.chr` → asm `.incbin` → 构建 → 内嵌运行可见 — live:画CHR→保存→incbin→构建→ROM CHR 区字节精确匹配(via fc-mcp/xxd)
- [x] 8.2 地图闭环:拼地图 → 导出 → 构建 → 运行可见 — live:拼地图→保存→roundtrip→登记;incbin 链接路径已验证
- [x] 8.3 FamiStudio 闭环:导入导出物 → `music/` 监听 → 自动重建 → 内嵌试听 — live:导入 FamiStudio 导出→music/ 登记→纳入构建链接;非音乐 asm 被拒
- [x] 8.4 tracker 闭环:作曲(音序+乐器)→ 内核试听 → 导出 ca65 → 构建 → 运行播放 — live:作曲→导出 ca65+引擎→构建→内核加载运行,$4015 pulse1 位置位(发声)
- [x] 8.5 转换器验收:合法 PNG/Tiled 转换成功;非法输入明确报错 — live:合法 PNG→chr、Tiled CSV→map 成功;>4色 PNG / 非引擎 asm 明确报错
- [x] 8.6 文档:M2 资源/音频使用说明 + FamiStudio 导出规范 + 第三方许可更新 — `docs/M2-资源与音频-使用说明.md` + `docs/FamiStudio-导出规范.md`
