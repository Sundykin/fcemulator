# 2A03 DMA phase 级仲裁设计说明

本文是给后续实现者的设计结论，不包含代码实现。目标是把当前分散在 `Bus::tick()`、
`Bus::oam_dma()` 和 APU DMC 取样里的 DMA 行为，收敛成一个真正按 CPU cycle 调度的
统一仲裁器，避免继续用“多 tick 一拍”“把 DMC 小段插进 OAM 循环”这类局部补丁追测试。

## 结论

必须建立一个统一的 DMA 仲裁状态机。OAM DMA、DMC DMA、CPU RDY halt、DMC dummy
read、DMC get、OAM get/put 都要在同一条 CPU cycle 时间线上逐拍决策。

当前模型的根本问题不是某个测试差 1 cycle，而是 DMA 被拆成了两套不相干的机制：

- OAM DMA 在 `$4014` 写入时由 `Bus::oam_dma()` 一次性循环完成。
- DMC DMA 在 `Bus::tick()` 内看到 APU 请求后，立即对 cartridge 做一次 PRG 读取并喂给 APU。

这会绕过真实硬件上的 RDY halt 与 get/put cadence，也无法表达 DMC 与 OAM DMA 的重叠：
DMC get 只抢占 OAM get，不应该把整段 DMC DMA 作为小块插入 OAM DMA；OAM put 仍可能继续发生。

实现方向应当是：

1. `$4014` 写入只登记 OAM DMA 请求，不立即复制 256 字节。
2. APU DMC 只登记“需要取样字节”的请求，不在 APU tick 或 Bus tick 里直接读 cartridge。
3. CPU 每一个物理 CPU cycle 都调用同一个 arbiter，由 arbiter 决定本拍是正常 CPU
   访问、RDY halt、DMC dummy、DMC get、OAM get 还是 OAM put。
4. CPU 被 RDY halt 时，当前 CPU 微操作不能前进；如果它是读周期，地址线上的读可能被重复，
   这些重复读必须触发 `$4016/$4017`、`$2007` 等寄存器副作用。
5. DMA 期间仍然按每个 CPU cycle 推进 PPU ×3、APU ×1，并维护 NMI/IRQ 线采样；但 CPU
   指令边界的 interrupt poll 不能因为 RDY-halted retry 而提前执行。

## 非目标

- 不做按 ROM、按测试名、按 mapper 的特殊分支。
- 不再用全局 “IRQ 延后 N 拍”“OAM DMA 前面多 tick N 次” 来掩盖架构问题。
- 不把 DMC DMA 当成一个独立的 3/4 cycle 子过程塞入 OAM DMA 循环。
- 不为了旧 save-state 兼容牺牲状态结构。若仲裁器需要新增序列化字段，应直接更新状态格式。

## 参考行为

NESdev 的 DMA 页面把关键点讲得很清楚：

- DMA 单元交替处在 get/put cadence 上；这通常被近似说成 CPU 偶/奇周期，但严格来说它与
  APU 半周期对齐，不能只靠 `cpu.cycles % 2` 随意判断。
- OAM DMA 会 halt CPU，必要时做 alignment，然后执行 256 组 get/put。
- DMC DMA 会 halt CPU，做 dummy cycle，必要时做 alignment，然后执行一次 get。
- DMC get 优先于 OAM get；两者重叠时，DMC 只延迟 OAM get，不应该阻塞所有 OAM 行为。
- DMC DMA 可能导致 `$4016/$4017` controller read、`$2007` PPUDATA read 出现额外读副作用。

推荐实现者直接读这些页面：

- [NESdev Wiki: DMA](https://www.nesdev.org/wiki/DMA)
- [NESdev Wiki: PPU registers / OAMDMA](https://www.nesdev.org/wiki/PPU_registers)
- [NESdev Wiki: Controller reading](https://www.nesdev.org/wiki/Controller_reading)
- [NESdev Wiki: Errata / DMC DMA controller read bug](https://www.nesdev.org/wiki/Errata)

## 当前代码落点

需要改造的现有边界：

- `fc-core/src/bus.rs`
  - `Bus::tick()` 目前会立刻处理 `apu.dmc_dma()`，这是错误抽象。DMC 应变成请求，不应在 tick
    里直接完成 PRG read。
  - `Bus::write(0x4014, value)` 目前直接调用 `oam_dma(value)`，应改成登记 OAM 请求。
  - `oam_dma()` 这种 bulk copy 函数应消失，或者只保留为测试辅助且不参与真实执行路径。
- `fc-core/src/cpu.rs`
  - `rd/wr/io` 目前是 “begin poll -> bus.tick -> 访问 -> end poll”。
  - 后续应变成 CPU 向 bus/arbiter 描述本拍想做的访问；arbiter 返回本拍是否完成 CPU 微操作。
  - 如果 RDY halt 生效，CPU cycle 计数和 PPU/APU 时间前进，但 CPU 当前读/写/内部微操作不能提交。
- `fc-core/src/apu.rs`
  - DMC 需要暴露“请求产生时间”和“取样地址”，由 arbiter 在 DMC get phase 完成读取后调用 supply。

## 统一状态机

建议新增一个明确的 `DmaArbiter` 概念，由 Bus 持有。它不应该拥有 cartridge/ppu/apu，
但它拥有 DMA 调度状态，并通过 Bus 执行具体读写。

仲裁器至少需要记录：

| 状态字段 | 目的 |
|---|---|
| get/put phase | 当前 CPU cycle 是 DMA get 允许相位还是 put 允许相位。应来自 2A03/APU cadence，而不是散落的奇偶判断。 |
| pending OAM request | `$4014` 写入后的 page、是否已 halt、当前 index、临时 latch、是否等待 get 或 put。 |
| pending DMC request | DMC 取样地址、请求类型、是否已 halt、是否完成 dummy、是否等待 alignment/get。 |
| CPU halt 状态 | CPU 当前微操作是否被 RDY hold，是否需要 retry 同一个 CPU read。 |
| interrupted CPU read | 被 halt 的 CPU 读地址，用于 dummy/repeated read 副作用。 |
| DMA 标记 | 调试器/watchpoint/open-bus 需要区分 CPU 正常访问与 DMA 访问时使用。 |

状态机的核心输入是 “CPU 本拍想做什么”，而不是让 Bus 自己猜：

| CPU cycle 类型 | 是否可被 RDY halt | 说明 |
|---|---:|---|
| CPU read(addr) | 是 | 包括 opcode fetch、operand read、数据 read、实际有外部地址的 dummy read。 |
| CPU write(addr, value) | 否 | DMA 请求遇到写周期必须等待，CPU 写先完成。 |
| CPU internal/no-access | 取决于建模 | 如果真实 6502 此拍仍驱动读地址，应按 read 建模；如果当前核心暂时无法细分，至少不要把 write 当成可 halt。 |

长期方向是减少模糊的 `io()`：凡是硬件上会驱动 CPU 地址线的“内部周期”，都应带上地址和
read/write 性质。DMC `$4016/$2007` 测试依赖的正是“被 halt 的读会被重复看到”。

## 每 CPU cycle 的调度顺序

建议把每个物理 CPU cycle 拆成同一个固定流程：

1. CPU 提交本拍意图：读、写或内部周期，以及地址/数据。
2. APU/PPU/mapper 按上一拍状态已经可能提出 DMA/IRQ/NMI 线变化；新请求进入 arbiter pending。
3. Arbiter 判断是否要对当前 CPU cycle 拉低 RDY。
   - 如果 DMA pending 且当前 CPU cycle 是可 halt 的读，进入 halt/retry。
   - 如果当前 CPU cycle 是写，DMA 等待下一拍再尝试。
4. 本拍执行一个 bus action：
   - 正常 CPU read/write；
   - RDY halt/repeated read；
   - DMC dummy read；
   - DMC sample get；
   - OAM source get；
   - OAM OAMDATA put；
   - alignment/no-op。
5. 无论本拍执行哪种 action，都推进 PPU ×3、APU ×1，并更新 NMI/IRQ 线采样。
6. 如果本拍是正常 CPU action，CPU 微操作提交；如果是 RDY halt 或 DMA action，CPU 保持在同一微操作，下一拍继续 retry。

重点是“每拍只做一个仲裁后的 action”。不要在一个函数调用里先 tick 一次、再额外跑完 DMC
3/4 拍、再回到 OAM 循环；那会再次丢失 phase 关系。

## OAM DMA 行为

OAM DMA 请求来自 CPU 写 `$4014`。写周期本身必须先完成；DMA 从下一 CPU cycle 开始尝试 halt。

OAM DMA 状态应按下面的 phase 推进：

| Phase | 行为 |
|---|---|
| Halt attempt | 只在 CPU read cycle 可成功；遇到 CPU write cycle 继续等待。成功后 CPU 当前微操作被 hold。 |
| Optional alignment | 如果当前 DMA cadence 不允许马上做 OAM get，则等待到 get phase。 |
| OAM get | 从 `(page << 8) | index` 通过 CPU bus read 通路读取一个字节，存入 OAM latch。 |
| OAM put | 在 put phase 把 latch 写入 PPU OAM，随后 `index += 1`。 |
| Done | 256 次 put 后释放 RDY，CPU 从被 hold 的微操作继续。 |

注意：

- `index` 只能在 put 完成后增加。若 OAM get 被 DMC get 抢占，不能提前增加。
- OAM get 应使用统一 CPU bus read 语义，至少要更新 open bus；是否触发 debugger watchpoint
  可以由调试器策略决定，但必须能标记 `is_dma`，避免把 DMA 误报为 CPU 指令访问。
- OAM put 写的是 PPU OAM，不是 CPU `$2004` 正常寄存器写。它应走 PPU 专用的 DMA 写入口，
  但仍处在同一个 CPU cycle 时间线上。

## DMC DMA 行为

DMC DMA 请求来自 APU DMC channel 的 sample buffer refill。APU 只负责在正确 APU tick
产生请求和给出 sample address；真正读取由 DMA arbiter 在 DMC get phase 完成。

DMC DMA 状态应按下面的 phase 推进：

| Phase | 行为 |
|---|---|
| Halt attempt | 与 OAM 一样，只能在 CPU read cycle 成功；遇到 CPU write cycle 继续尝试。 |
| Dummy read | 重复被 halt 的 CPU read 地址，并触发该地址的真实读副作用；该结果不提交给 CPU。 |
| Optional alignment | 如果下一拍不是 DMC get 允许相位，则等待。 |
| DMC get | 从 sample address 读取 PRG 字节，调用 APU supply。 |
| Done | 若没有 OAM DMA 等其它 pending，释放 RDY；否则继续由 arbiter 处理剩余 DMA。 |

关键点：

- DMC dummy/repeated read 不能是空 tick。它必须能导致 `$4016/$4017` controller shift、
  `$2007` PPUDATA buffer/VRAM address 前进等副作用。
- 被 halt 的 CPU read 本身不能完成。DMA 结束后，CPU 会重新执行同一个 read，并把那一次结果提交给指令。
- DMC sample get 是 DMC 的地址，不是 interrupted CPU 地址；它给 APU sample buffer，不给 CPU 寄存器。
- DMC get 可以更新 CPU open bus；更细的 APU internal bus/open-bus 差异可作为后续精度项，但不要因此绕过主状态机。

## DMC 与 OAM 重叠仲裁

这部分是本次设计的核心。DMC/OAM overlap 不能用嵌套循环或整段插入模拟。

统一规则：

| 同一 CPU cycle 可选 action | 仲裁结论 |
|---|---|
| DMC get 与 OAM get 同时想用 get phase | DMC get 优先，OAM get 等下一个可用 get phase。 |
| DMC get 与 OAM put | DMC 用 get phase；OAM put 只在 put phase，二者天然不在同一 phase。 |
| DMC dummy/alignment 与 OAM get/put | 按 phase 和已 halt 状态合并调度；不要把 DMC dummy 当成会整体暂停 OAM 的小段。 |
| OAM 已有 latch 等待 put | 到 put phase 应尽快 put，除非本拍被更高优先级规则明确占用。 |
| OAM 等待 get 且 DMC 抢占 get | OAM index 不变，latch 不变，下一 get phase 再取同一个 index。 |

实现者可以把 arbiter 看成每拍在回答两个问题：

1. CPU 是否已经被 DMA hold 住，或者本拍能否被新 DMA halt？
2. 在当前 get/put phase 上，DMC 和 OAM 哪个 DMA 子动作有资格占用总线？

只要这两个问题在一个状态机里回答，就不会出现 “DMC 已经取了 sample，但 OAM 的 index 也偷偷前进了”
或 “DMC 插入 4 拍导致 OAM put phase 整体错位” 这类问题。

## 与 interrupt sampling 的关系

近期 CPU IRQ/NMI poll-point 模型已经修过，后续 DMA 仲裁不能把这部分冲掉。

建议约束：

- 每个物理 CPU cycle 结束时仍采样 NMI edge/IRQ level，因为 PPU/APU/mapper 在 DMA 期间继续走时钟。
- CPU 指令的 interrupt poll point 只在 CPU 微操作正常推进到对应位置时发生。
- RDY-halted retry 不应让 CPU 误以为完成了一个 opcode fetch、operand read 或指令末尾 poll。
- DMA 期间 mapper IRQ 仍可能变化，例如 MMC3 IRQ；但 CPU 是否响应，取决于 CPU 恢复后到达的真实 poll point。

这能避免把 DMA 精度修复变成新的 MMC3 A12 或 CPU IRQ sampling 回归。

## Bus 读写语义

需要区分 “谁发起了访问”，但不能复制三套互相不一致的读写逻辑。

建议 Bus 提供统一内部访问层：

| 发起者 | 读写语义 |
|---|---|
| CPU normal read/write | 当前 `read/write` 的语义，含寄存器副作用、open bus、watchpoint。 |
| CPU repeated read / DMC dummy | 与 CPU normal read 一样触发副作用，但结果丢弃，并标记为 DMA/halt 导致。 |
| OAM get | CPU 地址空间读，结果进入 OAM latch；通常也应更新 open bus。 |
| DMC get | 读取 DMC sample address，结果进入 APU DMC buffer；必要时更新 open bus。 |
| OAM put | PPU OAM DMA 写入口，非 CPU `$2004` 普通写。 |

调试器后续可以用 `is_dma` 条件区分断点：

- 用户想看所有 bus read，可以包含 DMA。
- 用户想看 CPU 指令访问，应排除 DMA。
- 类似 Mesen 的 `IsDma` 条件可以后续暴露给 IDE。

## 序列化与存档

不用做旧存档兼容。新的 phase 级 arbiter 必须把完整 DMA 状态纳入 save-state：

- get/put phase 或 APU half-cycle alignment。
- OAM pending/page/index/latch/是否已 halt。
- DMC pending/address/dummy/alignment/get 状态。
- CPU 是否处于 RDY-halted retry，以及被 retry 的 CPU cycle 描述。
- open bus 和 NMI/IRQ sampling 相关既有状态保持一致。

如果状态不完整，读档后 DMA overlap 会随机错位，测试 ROM 可能偶发通过但实际不可复现。

## 验证计划

第一层：确保现有绿项不回归。

- `cargo test -p fc-core`
- `target/debug/fc testsuite nes-test-roms/cpu_interrupts_v2/rom_singles/*.nes --frames 12000`
- `target/debug/fc testsuite nes-test-roms/apu_test/rom_singles/*.nes`
- `target/debug/fc testsuite nes-test-roms/ppu_vbl_nmi/rom_singles/*.nes`
- `target/debug/fc testsuite nes-test-roms/mmc3_test/rom_singles/*.nes`

重点看：

- `cpu_interrupts_v2/4-irq_and_dma.nes` 不能退回失败。
- `mmc3_test/3-A12_clocking.nes` 必须保持 PASS。
- `mmc3_test/4-scanline_timing.nes` 当前若仍是已知失败，可以不作为本次 DMA 的完成门槛，
  但不能引入新的 MMC3 失败项。

第二层：DMA 本体目标。

- `target/debug/fc testsuite nes-test-roms/sprdma_and_dmc_dma/*.nes`
- `target/debug/fc testsuite nes-test-roms/dmc_dma_during_read4/*.nes`

其中 `dmc_dma_during_read4` 的核心验收点：

- `dma_4016_read.nes` 应看到 DMC DMA 对 `$4016` read 造成额外 controller read。
- `dma_2007_read.nes` 应看到 DMC DMA 对 `$2007` read 造成额外 PPUDATA read；源注释允许因 reset
  CPU/PPU 相位不同出现两种输出/CRC。

第三层：扩大回归。

- `instr_test-v5`、`instr_misc`、`instr_timing`：确认 CPU cycle retry 没改坏普通指令时序。
- 常见游戏人工 smoke：`roms/SuperMarioBro.nes` 跑若干帧截图，确认画面和音频没有明显退化。
- 长期把上述 test ROM 分类纳入 CI，至少记录每个 suite 的 PASS/FAIL/TIMEOUT 基线。

## 实施顺序建议

1. 先引入 arbiter 数据结构和请求登记，但保持行为尽量接近旧模型，测试确保没有大面积回归。
2. 改 `$4014`：从 bulk OAM DMA 改为 pending request，由每 CPU cycle 推进 OAM halt/get/put。
3. 改 DMC：APU tick 只产生 pending request，删除 `Bus::tick()` 内立即 fetch。
4. 改 CPU cycle 接口：让 `rd/wr/io` 能被 arbiter hold/retry；先保证 CPU 写周期不能被 halt。
5. 接入 DMC dummy repeated read，优先打通 `$4016/$2007` 副作用测试。
6. 接入 DMC/OAM overlap 优先级，目标通过 `sprdma_and_dmc_dma`。
7. 最后清理旧 `oam_dma()`、直接 `dmc_dma()` fetch 路径和所有临时 timing 补丁。

## 最容易踩的坑

- 只用 `cpu.cycles % 2` 判定 get/put，而没有一个可保存、可复位策略明确的 APU phase。
- DMC 请求一出现就立即读 PRG，导致 `$4016/$2007` dummy read 永远不可能正确。
- OAM DMA 仍在一个 for 循环里复制 256 字节，DMC 只能被“插进去”，无法真正 overlap。
- RDY halt 时让 CPU 当前 read 提交了结果，导致 dummy/retry 与真实 CPU read 混在一起。
- OAM get 被 DMC 抢占后仍然增加 index，造成后续 OAM 数据整体错位。
- DMA cycle 没有推进 PPU/APU，或推进了 PPU/APU 但没有维护 NMI/IRQ line sampling。
- 为了过单个测试增加固定 tick，破坏 CPU interrupt poll 或 MMC3 A12 这类已稳定路径。

## 最终验收标准

这次重构完成后，代码结构上应能明确看到：

- Bus tick 不再偷偷完成 DMC sample fetch。
- `$4014` 不再直接执行整段 OAM copy。
- CPU cycle 执行路径能表达 “本拍被 RDY hold，当前微操作下拍重试”。
- OAM DMA 和 DMC DMA 共享同一个 phase/cadence 状态。
- DMC dummy/repeated read 能走真实副作用读路径。
- DMC/OAM overlap 的优先级在一个地方实现，而不是散落在 OAM、APU、CPU 三处。

功能上应至少达到：

- 当前已通过的 CPU interrupt、APU、PPU vblank/NMI、MMC3 A12 测试不回归。
- `dmc_dma_during_read4` 通过或只剩明确记录的非 DMA 架构问题。
- `sprdma_and_dmc_dma` 输出进入可解释、可逐拍追踪的范围，并最终通过。

