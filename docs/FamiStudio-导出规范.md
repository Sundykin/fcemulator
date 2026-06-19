# FamiStudio → 本工程 导出对接规范

> 本工程**不打包** FamiStudio,只做"下游集成点":你在 FamiStudio 里作曲,按下面规范
> 导出 CA65 文件,用 IDE 工具栏「导入音乐」拖进 `music/`,即被纳入构建并可在内嵌
> 模拟器试听。日常 80% 的曲子也可用 IDE 内置 tracker 完成(无需 FamiStudio)。

## 推荐导出配置(FamiStudio)

在 FamiStudio: **Export → NES / FamiStudio Sound Engine** 或 **FamiTone2**:

| 选项 | 取值 |
|---|---|
| Sound Engine | **FamiStudio**(功能全)或 **FamiTone2**(轻量) |
| Format | **CA65**(`.s` 汇编,**不要**选 NSF/二进制) |
| Generate list include | 可选(便于按曲名引用) |
| DPCM | 若使用采样,会同时生成同名 `.dmc`,**一并放入 `music/`** |
| 命名 | 建议 `{project}_{song}`,便于工程内识别 |

导出得到一个 `.s`(乐曲数据 + 引擎)与可选的 `.dmc`(DPCM 采样)。

## 导入步骤

1. IDE 工具栏点 **「导入音乐」**,选导出的 `.s`。
2. 系统校验它是受支持的 CA65 引擎导出(含 `famistudio`/`famitone`/`music_data` 等标志),
   拷入 `music/` 并登记进 `project.toml`(从而被 ca65 汇编、ld65 链接)。
3. 若有 DPCM:把同名 `.dmc` 放入 `music/`(导入按钮目前只搬运 `.s`)。
4. 在主程序里按引擎 API 调用播放(`FamiToneInit`/`FamiStudioMusicPlay` 等,见各引擎文档)。

## 改完即听(文件监听)

开启工具栏 **「监听」** 后,`music/`(以及 `src/ chr/ map/`)的变更会**去抖后自动重建**
并刷新内嵌运行——在 FamiStudio 里改完重新导出覆盖 `music/` 下文件,IDE 即自动重链、可试听。

## 校验

- 导入时若文件不是受支持的 CA65 引擎导出,会给出明确诊断。
- 链接后可用 IDE 的内核试听 / 调试面板读 **APU 各通道电平**,做"爆音/缺声"检查
  (自研模拟器内核即 ground truth)。

> 许可证:FamiStudio 为 MIT;其声音引擎 asm 可随工程分发(遵循其许可)。本工程不冒认作者。
