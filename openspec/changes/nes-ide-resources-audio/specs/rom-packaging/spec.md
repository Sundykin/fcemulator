## ADDED Requirements

### Requirement: CHR 来自工程资源
打包 SHALL 支持 CHR 数据来自工程 `chr/` 资源(经汇编 `.incbin "chr/*.chr"` 进入 `CHARS` 段),而不仅限于内联在汇编里的字节。打包尺寸守卫沿用既有规则(链接产物大小须与 iNES 头声明的 PRG/CHR 一致)。

#### Scenario: 使用 chr/ 资源打包
- **WHEN** 工程的 CHR 来自 `chr/tiles.chr`(经 `.incbin`)且头部声明 1×8KB CHR
- **THEN** 产出 `.nes` 的 CHR 区为该资源内容,且尺寸守卫通过

#### Scenario: CHR 资源缺失
- **WHEN** 汇编 `.incbin` 引用的 `chr/*.chr` 文件不存在
- **THEN** 构建 SHALL 因缺失资源报错(ca65/ld65 诊断),不产出损坏 `.nes`
