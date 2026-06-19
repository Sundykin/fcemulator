# build-pipeline Specification

## Purpose
TBD - created by archiving change nes-ide-milestone-plan. Update Purpose after archive.
## Requirements
### Requirement: cc65 sidecar 构建编排
系统 SHALL 提供一键构建,按 `ca65 → ld65 → 拼 .nes` 流水线把活动工程构建为可运行的 `.nes`。ca65/ld65 MUST 作为捆绑的 Tauri sidecar 以 headless CLI 方式调用,系统 MUST 负责其进程的启动、输出捕获、超时与取消。

#### Scenario: 成功构建
- **WHEN** 用户对一个合法工程触发一键构建
- **THEN** 系统依次运行 ca65 汇编各源文件、ld65 链接、装配产出 `.nes` 到 `build/`,并报告构建成功

#### Scenario: 取消进行中的构建
- **WHEN** 构建进行中用户取消
- **THEN** 系统终止 sidecar 进程并将构建状态置为已取消,不产出半成品 `.nes`

#### Scenario: sidecar 缺失或不可执行
- **WHEN** 捆绑的 ca65/ld65 sidecar 不存在或无法执行
- **THEN** 系统 SHALL 报告工具链不可用的明确错误,而非静默失败

### Requirement: 链接脚本与工具链环境捆绑
系统 SHALL 捆绑标准链接脚本(`.cfg`)与 cc65 头/库资源,并通过 `CC65_INC` / `LD65_LIB` / `LD65_CFG` 等环境变量指向捆绑资源,使构建不依赖用户机器上的系统级 cc65 安装。

#### Scenario: 使用捆绑链接脚本构建
- **WHEN** 工程未自定义链接脚本而触发构建
- **THEN** 系统使用与工程 Mapper 匹配的捆绑 `.cfg` 完成链接

#### Scenario: 工程自定义链接脚本优先
- **WHEN** 工程在 `project.toml` 指定了自定义 `.cfg`
- **THEN** 系统使用该自定义链接脚本而非捆绑默认值

### Requirement: 构建日志与结果解析
系统 SHALL 捕获 sidecar 的 stdout/stderr,解析为结构化构建结果(成功/失败、各步骤、诊断条目列表),并在构建面板展示。每条诊断 MUST 区分错误与警告。

#### Scenario: 展示构建日志
- **WHEN** 构建完成(无论成败)
- **THEN** 构建面板展示完整日志与按错误/警告分类的诊断摘要

#### Scenario: 失败时定位首个错误
- **WHEN** 构建因汇编/链接错误失败
- **THEN** 系统 SHALL 将失败原因解析为结构化诊断条目(供 source-debug-link 跳转),并标记构建失败

