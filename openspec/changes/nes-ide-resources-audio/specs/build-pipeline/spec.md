## ADDED Requirements

### Requirement: 资源装配进构建
构建编排 SHALL 把工程音乐源(FamiStudio 导出或内置 tracker 导出的 `.s`)纳入 ca65 源列表与 ld65 链接;CHR/地图资源经汇编 `.incbin`/`.include` 引用时,SHALL 随现有链接流程进入 `.nes`。

#### Scenario: 音乐源参与链接
- **WHEN** 工程登记了 `music/*.s` 且触发构建
- **THEN** 系统对这些音乐源运行 ca65 并与主程序一同 ld65 链接进 `.nes`

#### Scenario: CHR 资源进构建
- **WHEN** 工程 asm `.incbin "chr/x.chr"` 且触发构建
- **THEN** 该 CHR 数据出现在产出 `.nes` 的 CHR 区

### Requirement: 资源变更增量重建
系统 SHALL 监听工程 `src/`、`chr/`、`map/`、`music/` 目录变化(去抖),变化时触发重建并刷新内嵌运行;并发重建 MUST 串行化/可取消,避免抖动导致的重复或竞态构建。

#### Scenario: 改资源即重建
- **WHEN** 受监听目录下的已登记资源被更新
- **THEN** 系统去抖后触发一次重建,完成后内嵌运行反映更新

#### Scenario: 重建串行化
- **WHEN** 在一次重建进行中又发生新的文件变化
- **THEN** 系统不并发启动第二次构建,而是取消/排队后再跑,最终以最新内容构建
