# Mapper 架构优化计划

目标：把后续 mapper 适配从“每块板单独接入”推进到“参考项目代码机械翻译 + 本项目精修”的流水线。

## 问题判断

当前扩展慢，不是因为 mapper 本身必须复杂，而是本项目缺少一层类似 FCEUX/FCEUmm/Mesen2 的 board API。参考项目里一块 mapper 看起来只有几百行，是因为它们已经有成熟的底座：

- CPU read/write handler 注册。
- `setprg8/16/32`、`setchr1/2/4/8` 这类 bank helper。
- IRQ counter/A12/HBlank hook。
- reset/power hook。
- mapper register read 和 open-bus/side-effect 读。
- CHR-ROM/CHR-RAM 混合窗口。
- expansion audio 接口。

本项目现在的 `MapperOps` 更偏 `prg_index()` / `chr_index()` / `write_register()` 的纯函数式模型。这个模型高性能、容易序列化，也适合常规 mapper，但长尾盗版板和复合 ASIC 会把大量特殊行为挤进单个 mapper 实现。

## 设计原则

1. 不破坏 `fc-core` 纯核心边界，不引入 IO、音频设备或前端依赖。
2. 不破坏 CPU/PPU/APU lock-step：CPU 访存前 tick bus 的顺序不改。
3. mapper hot path 保持缓存能力位，普通 mapper 不付出额外动态分发成本。
4. 先补通用板卡能力，再批量翻译参考项目 mapper。
5. 每个从参考项目翻译的行为都继续记录到 `docs/Mapper-适配引用记录.md`。

## 阶段规划

### A. BankMap helper

目的：把参考项目里的 `setprg8` / `setchr1` 语义搬到本项目。

交付：
- 新增轻量 bank table/helper，支持 PRG 8K/16K/32K、CHR 1K/2K/4K/8K。
- 支持固定 bank、可切换 bank、outer bank OR/AND、bus conflict 前后处理。
- 支持 CHR-ROM/CHR-RAM 混合窗口，例如 mapper 74/119/192/195。
- 保持最终 `prg_index()` / `chr_index()` 可内联读取，不把每次访问变成 handler 链。

优先迁移对象：
- address latch / Sachen / ColorDreams / Codemasters 小 mapper。
- MMC3 CHR-RAM window 变体。

### B. CPU 地址 handler 层

目的：让 `$4020-$5FFF`、`$6000-$7FFF`、`$8000-$FFFF` 的读写行为更接近参考项目 handler。

交付：
- 统一 expansion/low/high read/write helper。
- 明确 PRG-RAM、PRG-ROM、mapper register、open-bus 的优先级。
- 支持 mapper register read side-effect。
- 支持低地址 PRG-ROM 映射和低地址 WRAM 映射。

优先服务对象：
- `103/120` 低地址 PRG-ROM 类。
- `170/234` register read/side-effect 类。
- `228/235` open-bus aware 类的后续精修。

### C. IRQ 单元库

目的：减少每个 mapper 重写 IRQ 计数器。

交付：
- `Mmc3A12Irq`：复用现有 MMC3 A12 filter 与 reload 语义。
- `CpuCycleIrq`：支持 Mesen2/FCEUmm 常见 CPU counter。
- `HBlankIrq`：复用当前 `hblank_clock()` 机制。
- `VrcIrq` / `RamboIrq`：抽取 VRC4/RAMBO-1 这类可复用计数器。

优先迁移对象：
- VRC4 family。
- RAMBO-1。
- mapper 91/106/183/222/253 精修。

### D. MMC3 variant layer

目的：把 49/114/115/121 这类写协议变体继续收敛，避免 `mmc3.rs` 变成不可维护的大文件。

交付：
- 保留 `write_bank_select` / `write_bank_data` / `write_standard_register`。
- 抽出 high-register remap protocol。
- 抽出 outer PRG/CHR bank policy。
- 抽出 protection register/readback policy。
- 对 mapper 49/114/115/121 做二次整理，作为后续 MMC3 variant 模板。

优先后续对象：
- `90/96/105` 等 MMC3-like 变体。
- `191/193/196` 等 CHR-RAM/MMC3 派生板。

### E. Reset/power 和 side-effect 标准化

目的：让 reset hook、power default、读副作用不再散落。

交付：
- 区分 power-on 与 soft reset 的 mapper hook。
- 支持 reset counter / reset-selected bank。
- 支持 mapper read 触发 latch 或保护寄存器状态变化。
- 用单测覆盖 reset 后 bank 状态。

优先服务对象：
- `230/233` reset hook 类。
- `60/228/235` 已有 reset 行为精修。
- `170/234` register read side-effect。

### F. Expansion audio 接口

目的：让 FME7/N163/VRC6/VRC7 不被硬塞进普通 mapper bank 逻辑。

交付：
- mapper 可返回 expansion audio sample 或 clock audio sub-unit。
- APU mixing 路径接入 mapper audio，但 `fc-core` 不引入外部音频依赖。
- FME7/N163/VRC6/VRC7 的音频和 bank register 可以共享同一板卡状态。

优先对象：
- FME7。
- VRC6。
- N163。
- VRC7/OPLL，继续保留 `docs/VRC7-OPLL-引用记录.md` 的来源记录。

## 执行顺序

1. 收口当前 49/114/115/121：补引用记录、跑完整测试、提交。
2. 落地 `BankMap` helper，并迁移一个小 mapper 家族验证收益。
3. 落地 CPU handler helper，推进 `154/155/108` 与低地址映射类。
4. 抽 IRQ 单元，回收重复 mapper IRQ 逻辑。
5. 清理 MMC3 variant layer，把 49/114/115/121 二次整理成模板。
6. 开始扩展 FME7/N163/VRC6/VRC7 expansion audio。
7. 进入长尾机械翻译批次，每批按“引用记录 -> 实现 -> mapper tests -> fc-core tests -> commit”收口。

## 验收标准

- 新 mapper 增加时，通常只需要声明 bank table、handler 和少量特殊逻辑。
- mapper capability 表仍由测试锁住，不出现漏报 A12/CPU/HBlank hook。
- 普通 mapper hot path 不因为 handler 层变慢。
- 每批提交前至少通过：
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo test -p fc-core mapper::`
  - `cargo test -p fc-core`
- 参考来源继续记录具体文件和行号，便于开源致谢或闭源替换。

## 落地记录

- 2026-06-22：新增 `fc-core/src/mapper/bank.rs`，先提供无状态 PRG/CHR page index helper，并迁移 ColorDreams/GxROM 与 Sachen 133/146/148/149 作为第一批验证。
- 2026-06-22：扩展 `bank.rs` 的 `ChrRamWindow` / `ChrBankSource`，把 MMC3 派生 mapper 74/119/192/194/195 的 CHR-ROM/CHR-RAM 混合窗口迁移到通用 helper。CPU handler 层仍留在后续阶段。
- 2026-06-22：在 `Cartridge` 内落地 CPU 地址 helper 层，把 expansion、low、high 三段 read/peek/write 的 open-bus、PRG-RAM、低地址 PRG-ROM、mapper register read、patch 和 bus-conflict 优先级集中起来。当前保持私有 helper，不扩大 `MapperOps`，作为后续低地址映射、读副作用和 reset hook 类 mapper 的接入点。
- 2026-06-22：新增 `fc-core/src/mapper/irq.rs`，先抽出 `Mmc3A12Irq`，并用 `serde(flatten)` 迁移 MMC3 全家共享的 A12 filter、reload、enable、pending 和 MMC6 zero-reload 抑制逻辑。VRC/RAMBO/CPU counter/HBlank IRQ 保持后续独立收敛。
- 2026-06-22：继续抽出 `A12EdgeFilter`，迁移 MMC3、RAMBO-1、Mapper117 共享的 A12 低电平门限/上升沿检测状态。各 mapper 仍保留自己的 counter/reload/delay 语义，后续再分层抽 CPU counter/HBlank/VRC/RAMBO IRQ。
- 2026-06-22：新增 `CpuCycleIrq`，迁移 mapper 43/50/106 这类简单 CPU-cycle up-counter IRQ。当前 helper 只覆盖 enabled/counter/pending、阈值触发、wrap-to-zero 触发和 byte 写入；复杂 reload/prescaler/delay IRQ 后续单独抽取。
