# source-debug-link Specification

## Purpose
TBD - created by archiving change nes-ide-milestone-plan. Update Purpose after archive.
## Requirements
### Requirement: 编译诊断 ↔ 代码行跳转
系统 SHALL 把构建诊断(错误/警告)关联到具体的源码文件与行号,使用户可从构建面板的诊断条目一键跳转到编辑器对应行。

#### Scenario: 点击诊断跳转到源码行
- **WHEN** 用户在构建面板点击一条含 `file:line` 的诊断
- **THEN** 编辑器打开该文件并将光标/高亮定位到对应行

#### Scenario: 诊断缺少精确行号时降级
- **WHEN** 某条诊断无法解析出精确行号
- **THEN** 系统 SHALL 至少跳转到对应文件(文件级定位),而非无响应

### Requirement: 地址 ↔ 源码映射
系统 SHALL 在构建时获取 ld65 产出的符号/行号映射,并建立 `CPU 地址 ↔ {文件, 行}` 的双向映射,供断点与暂停时的源码定位使用。映射 MUST 随每次成功构建刷新。

#### Scenario: 构建后建立映射
- **WHEN** 一次构建成功并产出符号/行号信息
- **THEN** 系统解析出地址↔源码行映射并供调试使用

#### Scenario: 映射不可用时降级
- **WHEN** 链接产物缺少足够的调试/符号信息
- **THEN** 系统 SHALL 降级为"文件级 + 最近符号"提示,且不阻断运行与调试

### Requirement: 行级断点驱动 ControlDeck 调试器
系统 SHALL 允许用户在源码行设置/清除断点;设置时经地址映射转为 PC 地址并调用已有 `ControlDeck` 断点接口。命中断点暂停后,系统 MUST 把当前 PC 反查回源码行并在编辑器高亮。

#### Scenario: 源码行下断点
- **WHEN** 用户在某源码行切换断点开
- **THEN** 系统经映射求得对应 PC 地址并向 `ControlDeck` 注册执行断点

#### Scenario: 命中断点高亮源码
- **WHEN** 内嵌运行命中已注册断点而暂停
- **THEN** 系统把当前 PC 反查为源码行,在编辑器高亮该行并联动调试面板(寄存器/内存)

#### Scenario: 清除断点
- **WHEN** 用户在已下断的源码行再次切换断点关
- **THEN** 系统从 `ControlDeck` 移除对应断点,该行不再触发暂停

