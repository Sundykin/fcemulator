## ADDED Requirements

### Requirement: CHR 图块编辑
系统 SHALL 提供内置 CHR 图形编辑器,以 8×8 图块为单位编辑 NES 图案数据,每像素为 0–3 的 4 色调色板索引。编辑器 MUST 支持铅笔、填充、水平/垂直翻转,并提供图块表(多图块)与单图块放大编辑两种视图。

#### Scenario: 编辑一个图块
- **WHEN** 用户在放大视图中用铅笔修改某像素的调色板索引
- **THEN** 该图块对应像素更新,图块表中的缩略图同步反映

#### Scenario: 选择调色板颜色
- **WHEN** 用户从 4 色调色板选一个颜色再作画
- **THEN** 后续绘制使用该颜色对应的索引(0–3)

### Requirement: CHR 导出与工程登记
系统 SHALL 把编辑的图块导出为 NES 2bpp planar 格式的 `.chr`(每图块 16 字节:低位平面 8 字节 + 高位平面 8 字节),落盘到工程 `chr/` 目录,并在 `project.toml` 登记。系统 SHOULD 同时可导出 `.inc`(ca65 可 `.include` 的字节定义)。

#### Scenario: 导出 .chr 并登记
- **WHEN** 用户保存/导出一组图块
- **THEN** 系统写出 `chr/<name>.chr`(字节数 = 图块数 × 16),并在 `project.toml` 的 chr 列表登记该文件

#### Scenario: 往返一致
- **WHEN** 导出的 `.chr` 再被编辑器读取
- **THEN** 解析出的图块像素与导出前一致(planar 编解码可逆)

### Requirement: CHR 可进构建
导出的 CHR 资源 SHALL 能被工程汇编经 `.incbin` 引用并由现有 build-pipeline 链接进 `.nes`,无需用户手工转换字节。

#### Scenario: incbin 引用构建通过
- **WHEN** 工程 asm 以 `.incbin "chr/<name>.chr"` 引用导出的 CHR 并触发构建
- **THEN** 构建成功,产出 `.nes` 的 CHR 区包含该图块数据
