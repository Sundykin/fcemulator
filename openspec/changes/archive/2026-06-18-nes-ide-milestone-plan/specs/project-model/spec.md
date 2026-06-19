## ADDED Requirements

### Requirement: 工程模型与 project.toml
系统 SHALL 用单一 `project.toml` 文件声明一个 NES 工程的全部构建上下文,作为可重放构建的唯一事实源。该文件 MUST 至少包含:工程名、Mapper 编号、源码文件列表、CHR 资源、音乐资源、地图资源、链接脚本(`.cfg`)与输出 `.nes` 路径。缺失字段 MUST 由系统以模板默认值补全。

#### Scenario: 解析合法工程
- **WHEN** 用户打开一个含合法 `project.toml` 的目录
- **THEN** 系统加载工程模型,前端可读取工程名、Mapper、各资源列表与输出路径

#### Scenario: 字段缺失或非法
- **WHEN** `project.toml` 缺少必填字段或字段类型非法
- **THEN** 系统 SHALL 报告具体的字段级错误(字段名 + 原因),且不进入"已加载工程"状态

### Requirement: 新建 / 打开 / 保存工程
系统 SHALL 支持从模板新建工程、打开已有工程目录、以及保存对工程模型的修改回 `project.toml`。同一时刻 MUST 只有一个活动工程。

#### Scenario: 从模板新建工程
- **WHEN** 用户选择模板与目标目录新建工程
- **THEN** 系统在目标目录生成 `project.toml` 与约定目录骨架(`src/`、`chr/`、`music/`、`map/`、`build/`),并将其设为活动工程

#### Scenario: 打开已有工程
- **WHEN** 用户选择一个含 `project.toml` 的目录
- **THEN** 系统加载该工程为活动工程,关闭先前活动工程(若有)

#### Scenario: 保存工程修改
- **WHEN** 用户在 IDE 中修改了工程配置(如增删源码、改 Mapper)并触发保存
- **THEN** 系统将变更写回 `project.toml`,且重新读取后内容一致

### Requirement: 文件树
系统 SHALL 以工程根为基准提供文件树,展示约定目录与文件,并支持在树中新建/重命名/删除文件,变更 MUST 同步反映到磁盘与文件树。

#### Scenario: 展示工程文件树
- **WHEN** 工程已加载
- **THEN** 文件树展示 `src/`、`chr/`、`music/`、`map/`、`build/` 等目录及其文件

#### Scenario: 在文件树中操作文件
- **WHEN** 用户在文件树中新建、重命名或删除一个文件
- **THEN** 对应磁盘文件随之创建、改名或删除,文件树刷新显示最新状态

### Requirement: 工程模板
系统 SHALL 至少提供三个工程模板:空白、横版、演示。每个模板 MUST 含一份可直接成功编译/链接的最小骨架(源码 + 链接脚本)。

#### Scenario: 模板可直接构建
- **WHEN** 用户用任一内置模板新建工程后立即触发构建
- **THEN** 构建成功产出可运行的 `.nes`(无需用户先写任何代码)
