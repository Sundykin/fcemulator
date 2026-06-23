## ADDED Requirements

### Requirement: 逐地址访问计数(read/write/exec)
核心 SHALL 在启用时,于 CPU 总线访问路径累计每个地址的 **读 / 写 / 执行** 次数。计数 MUST 覆盖 CPU 地址空间(`$0000–$FFFF`),并区分三类访问;默认关闭,关闭时 MUST 零计数、零热路径开销、零行为变化。

#### Scenario: 计数读写执行
- **WHEN** 启用热力图并运行若干帧
- **THEN** 频繁轮询的寄存器(如 `$2002`)读计数显著高;代码区地址有执行计数;被写的 RAM 地址有写计数

#### Scenario: 关闭时零计数零开销
- **WHEN** 热力图关闭
- **THEN** 无任何计数累积;且与未引入本特性的二进制相比,模拟时序/输出与 `fc trace` 0-diff

### Requirement: code/data 标志(CodeDataLogger 风格)
系统 SHALL 为每个地址维护「曾作为指令执行」与「曾作为数据读取」的标志(对齐 Mesen `CodeDataLogger`),以区分代码与数据区。

#### Scenario: 区分代码与数据
- **WHEN** 运行一段后查询某 PRG 地址
- **THEN** 落在已执行指令上的地址标记为 code,被当数据读的地址标记为 data,二者可并存

### Requirement: 最近热度衰减视图
系统 SHALL 提供「最近热度」视图:计数随时间(按帧)衰减,使近期高频访问区域可与历史累计区分。原始累计计数与衰减热度 MUST 都可获取。

#### Scenario: 近期访问更热
- **WHEN** 某地址上一帧被大量访问、之后停止
- **THEN** 其衰减热度随后续帧下降,而累计计数保持不变

### Requirement: 经 ControlDeck 与 MCP 暴露,且不进存档
访问计数与 code/data 标志 SHALL 通过 `ControlDeck` 暴露(启停、读取、重置),并经 MCP `emu_heatmap` 返回结构化数据。该状态 MUST 为瞬态,不纳入 save-state。

#### Scenario: 经 MCP 读取热力图
- **WHEN** 启用后调用 `emu_heatmap`
- **THEN** 返回各地址的 read/write/exec 计数(或区间聚合)+ code/data 标志 + 衰减热度

#### Scenario: 重置计数
- **WHEN** 调用重置
- **THEN** 所有计数与标志归零,后续从新累计

#### Scenario: 存读档不携带计数
- **WHEN** 启用计数时存档并读档
- **THEN** 读档成功且不含旧计数;计数于读档后从零重新累计
