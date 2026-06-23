## ADDED Requirements

### Requirement: 逐帧事件记录(scanline×dot 标注)
核心 SHALL 在启用记录时,于现有 lock-step 插桩点采集调试事件,每条事件 MUST 标注其发生时的 `(scanline, dot, master_cycle)` 位置。事件类型 MUST 覆盖:PPU 寄存器读写(`$2000–$2007`)、APU 寄存器读写(`$4000–$4017`)、手柄读(`$4016/$4017`)、mapper 寄存器写(`$4020+`)、NMI 触发、IRQ 触发(按来源区分 APU 帧计数器 / DMC / mapper)、sprite-0 命中、OAM DMA(`$4014`)、DMC DMA 取样。

#### Scenario: 记录一帧的寄存器写与中断
- **WHEN** 启用事件记录并运行一帧含滚动分屏的游戏(如 SMB 状态栏)
- **THEN** 该帧事件流包含其 `$2005/$2006` 写,且每条写事件标注的 `(scanline, dot)` 落在该写实际发生的拍上;若该帧产生 sprite-0 命中或 NMI,对应事件 MUST 出现且位置正确

#### Scenario: 事件位置取自实时 PPU 拍
- **WHEN** CPU 在某拍写 `$2006`
- **THEN** 该事件记录的 `(scanline, dot)` 等于写访问发生时(`bus.tick` 已推进 PPU 后)的 `ppu.scanline` / `ppu.dot`

### Requirement: 记录默认关闭且关闭时零开销零行为变化
事件记录 SHALL 默认关闭,并由热路径上的单个运行期布尔门控。关闭时 MUST 不产生任何事件、不改变 CPU/PPU/APU 的时序或输出。开启记录本身 MUST NOT 改变模拟行为(仅旁路观测)。

#### Scenario: 关闭时无事件且 trace 不变
- **WHEN** 事件记录关闭,运行 SMB / 双截龙3 / 忍者神龟3
- **THEN** `event_log()` 为空;且与未引入本特性的二进制相比,`fc trace` 逐指令 0-diff

#### Scenario: 开启记录不扰动模拟流
- **WHEN** 同一 ROM 分别在「开记录」与「关记录」下运行相同输入相同帧数
- **THEN** 两次的 CPU/PPU 状态轨迹与帧缓冲逐字节一致(记录是纯旁路)

#### Scenario: 性能门
- **WHEN** 用 `fc bench` 对比关记录 vs 开记录
- **THEN** 关记录路径 fps 在噪声范围内;开记录 fps 差 MUST ≤5%(对齐路线图调试开关门)

### Requirement: 双缓冲,保证读到完整且稳定的一帧
系统 SHALL 双缓冲事件:下一帧记录进行时,刚完成的那一帧事件 MUST 保持可查询且稳定不被覆盖,使暂停/断点态的前端总能读到一整帧。

#### Scenario: 暂停态读取完整帧
- **WHEN** 在断点处暂停后调用 `event_log()`
- **THEN** 返回最近一帧**完整**的事件集合(不含正在记录的半帧)

#### Scenario: 跨帧不串
- **WHEN** 连续运行多帧
- **THEN** 每次查询只反映最近一个完整帧,不混入更早帧的残留事件

### Requirement: 按类型过滤
系统 SHALL 提供按事件类型的开关配置,仅记录被启用的类型,以控制噪声与开销。

#### Scenario: 仅记录寄存器写
- **WHEN** 配置为只启用「PPU/APU/mapper 寄存器写」类型
- **THEN** 该帧事件流只含写类事件,不含读、NMI/IRQ、DMA 等

### Requirement: 经 ControlDeck 暴露
所有事件能力 SHALL 仅通过 `ControlDeck` facade 暴露:启用/禁用记录、类型过滤配置、读取最近完整帧事件——遵循各前端只经 `ControlDeck` 驱动内核的项目约定。

#### Scenario: 通过 facade 开关与读取
- **WHEN** 前端调用 `ControlDeck` 的事件开关与 `event_log()`
- **THEN** 能开启记录、配置过滤、并取回最近完整帧的结构化事件,无需触达 `bus`/`ppu` 内部

### Requirement: MCP 工具 `emu_event_dump`
系统 SHALL 新增 MCP 工具 `emu_event_dump`,返回最近完整帧的事件为结构化 JSON(至少含 `type, scanline, dot, addr, value, rw, source`),并尊重当前类型过滤,使 AI agent 无需 UI 即可读取事件以定位时序 bug。

#### Scenario: dump 返回结构化事件
- **WHEN** 已加载 ROM、启用记录并运行若干帧后调用 `emu_event_dump`
- **THEN** 返回该帧事件数组,每项含类型与 `(scanline, dot)` 及访问详情

#### Scenario: 未启用或未加载时的明确响应
- **WHEN** 未加载 ROM 或记录未启用时调用 `emu_event_dump`
- **THEN** 返回空事件集合并附带状态说明,而非报错或返回脏数据

### Requirement: fc-tauri scanline×dot 画布
fc-tauri 调试视图 SHALL 提供一个 `scanline × dot` 网格画布(尺寸取当前 region 的扫描线/点数,NTSC 为 262×341),按事件类型着色标注,且可逐帧步进查看。该可视化 MUST 建立在与 MCP 同一份核心事件数据之上。

#### Scenario: 按类型着色
- **WHEN** 启用记录并在画布查看某帧
- **THEN** 不同事件类型以不同颜色标注于对应 `(scanline, dot)` 网格点

#### Scenario: 逐帧步进
- **WHEN** 单步一帧
- **THEN** 画布刷新为新一帧的事件分布

### Requirement: 事件日志不进存档
事件日志 SHALL 为瞬态状态(不纳入 save-state 序列化),存档/读档 MUST NOT 携带或依赖事件日志。

#### Scenario: 存读档不携带事件日志
- **WHEN** 在启用记录时存档,随后读档
- **THEN** 读档成功且不包含旧的事件数据;记录于读档后照常从新帧开始累积
