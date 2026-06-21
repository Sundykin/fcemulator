## ADDED Requirements

### Requirement: 按事件类型触发断点
调试器 SHALL 支持「事件断点」:当被配置的调试事件发生(PPU/APU/mapper 寄存器读写、手柄读、NMI、IRQ、sprite-0 命中、OAM/DMC DMA)时,立即在该事件发生处暂停模拟。事件断点 MUST 复用现有断点的「帧内 halt」路径——`run_frame()` 在命中处返回 `false`、`is_halted()` 报告命中点,与 exec/read/write 断点一致。

#### Scenario: 命中寄存器写事件断点
- **WHEN** 设置「写 `$2006`」事件断点并运行
- **THEN** 当 CPU 写 `$2006` 时模拟于该指令处暂停,`run_frame()` 返回 `false`

#### Scenario: 命中 sprite-0 事件断点
- **WHEN** 设置「sprite-0 命中」事件断点并运行
- **THEN** 在 sprite-0 命中那一拍暂停,且可读取当时 `(scanline, dot)`

### Requirement: 可选 (scanline,dot) 窗口约束
事件断点 SHALL 支持可选的 `(scanline, dot)` 窗口:仅当事件发生在该光栅窗口内才触发,以便聚焦分屏/光栅特效的某一区域。

#### Scenario: 仅在指定扫描线区间触发
- **WHEN** 设置「写 `$2005`,限 scanline 30–32」事件断点
- **THEN** 仅当 `$2005` 写发生在 scanline 30–32 时暂停;区间外的 `$2005` 写不触发

### Requirement: 经 ControlDeck 与 MCP 暴露
事件断点 SHALL 通过 `ControlDeck` facade 增删/启停,并经 MCP `emu_set_event_breakpoint` 暴露,使 AI agent 无需 UI 即可下事件断点并配合 `emu_run_until_break`/`emu_event_dump` 定位。

#### Scenario: 经 MCP 下事件断点并跑到命中
- **WHEN** 经 `emu_set_event_breakpoint` 设「mapper IRQ」断点,再 `emu_run_until_break`
- **THEN** 在首个 mapper IRQ 处停下并返回命中信息(类型 + `(scanline,dot)` + 地址)

#### Scenario: 清除事件断点后不再触发
- **WHEN** 清除某事件断点后继续运行
- **THEN** 该事件不再导致暂停
