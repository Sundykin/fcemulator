# rom-packaging Specification

## Purpose
TBD - created by archiving change nes-ide-milestone-plan. Update Purpose after archive.
## Requirements
### Requirement: iNES 头可视化编辑
系统 SHALL 提供 iNES 头的可视化编辑,至少覆盖 Mapper 编号、PRG ROM 大小、CHR ROM 大小、镜像方式(水平/垂直)、电池 SRAM 标志。编辑结果 MUST 写入工程并用于打包产出的 `.nes` 头。

#### Scenario: 编辑并应用 iNES 头
- **WHEN** 用户在头编辑器修改 Mapper / 镜像 / 大小等字段并应用
- **THEN** 系统将这些字段持久化到工程,后续打包产出的 `.nes` 头与之一致

#### Scenario: 头字段校验
- **WHEN** 用户填入非法头字段(如不支持的 Mapper 或非法大小)
- **THEN** 系统 SHALL 阻止应用并报告字段级错误

### Requirement: PRG / CHR 装配产出 .nes
系统 SHALL 把 ld65 链接产出的 PRG 与工程的 CHR 数据按 iNES 头装配为完整可运行的 `.nes` 文件。打包逻辑 MUST 复用既有的 iNES 解析约定(项目已能读取 iNES 头)。

#### Scenario: 装配可运行 ROM
- **WHEN** 构建链接成功且 CHR 资源就绪
- **THEN** 系统装配出带正确 iNES 头的 `.nes`,可被内嵌模拟器加载运行

#### Scenario: CHR 缺失的处理
- **WHEN** 工程声明使用 CHR ROM 但 CHR 资源缺失
- **THEN** 系统 SHALL 报告 CHR 缺失错误,而非产出损坏的 `.nes`

### Requirement: 打包产物可被内嵌运行
打包产出的 `.nes` SHALL 能直接交给已有 `ControlDeck` 内嵌运行,无需用户手动导出再导入。

#### Scenario: 构建后一键运行
- **WHEN** 用户在构建成功后触发运行
- **THEN** 系统把刚打包的 `.nes` 加载进内嵌模拟器并开始运行

