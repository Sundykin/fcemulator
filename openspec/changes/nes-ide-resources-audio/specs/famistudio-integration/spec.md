## ADDED Requirements

### Requirement: 导入 FamiStudio 导出物
系统 SHALL 提供导入入口,接收 FamiStudio 的 CA65 导出(乐曲 `.s` 与可选 DPCM `.dmc`),拷入工程 `music/` 目录并在 `project.toml` 登记乐曲文件与曲名。系统 SHALL 提供推荐导出配置文档(Sound Engine=FamiStudio/FamiTone2、Format=CA65、命名约定)。

#### Scenario: 导入乐曲
- **WHEN** 用户选择一个 FamiStudio 导出的 `.s`(及同名 `.dmc`)导入
- **THEN** 系统把文件拷入 `music/`,在 `project.toml` 登记,并将该 `.s` 加入工程的 ca65 源列表

#### Scenario: 校验导出格式
- **WHEN** 导入的 `.s` 不是受支持的 CA65 引擎导出(签名/标签不符)
- **THEN** 系统 SHALL 报告明确的格式不符诊断,而非静默接受

### Requirement: music 目录监听与自动重建
系统 SHALL 监听工程 `music/` 目录变化,变化时(去抖后)触发构建,使用户在外部 FamiStudio 改完音乐重新导出后,内嵌模拟器即可听到更新。

#### Scenario: 改音乐即重建
- **WHEN** `music/` 下已登记的乐曲文件被外部更新
- **THEN** 系统去抖后自动重建工程,并刷新内嵌运行

### Requirement: 内核试听校验
系统 SHALL 能用自研模拟器/APU 内核试听链接后的乐曲(或导出的 NSF),并读取 APU 通道电平用于"爆音/缺声"类校验。

#### Scenario: 试听已链接乐曲
- **WHEN** 用户对已纳入构建的乐曲触发试听
- **THEN** 系统经内核播放并可报告各 APU 通道是否有电平输出
