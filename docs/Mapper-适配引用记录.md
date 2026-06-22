# Mapper 适配引用记录

这份记录只用于当前 mapper 适配面的来源回溯和后续开源致谢。
如果将来需要做闭源分支，优先替换下面列出的代码范围。

## 当前代码范围

- `fc-core/src/mapper/basic/unlicensed.rs:1-884`
  - 新增 Mapper 43 / 60 / 83 / 106 / 183 / 212 / 222 / 235。
  - 覆盖 PRG/CHR bank 译码、低地址 PRG-ROM 窗口、mapper register read、open-bus 读侧效应、reset hook、CPU clock IRQ、A12 IRQ。
- `fc-core/src/mapper/basic/latch/discrete.rs`
  - 新增 Mapper 36 / 72 / 79 / 92。
  - 覆盖 TXC/Micro Genius 简化 latch、$4100 读回、bus conflict、Jaleco 2-in-1/JF-17 PRG/CHR 写位规则、NINA-003/006 扩展区 latch。
- `fc-core/src/mapper/basic/taito.rs`
  - 新增 Mapper 80 / 82。
  - 覆盖 Taito X1-005/X1-017 的低地址寄存器、8KB PRG、混合 2KB/1KB CHR、mirroring、Mapper 80 的 256B gated WRAM。
- `fc-core/src/mapper/basic/multicart.rs`
  - 新增 Mapper 59 / 63 / 201 / 217。
  - 覆盖 address latch PRG/CHR/mirroring、Mapper 63 越界 PRG open-bus 读。
- `fc-core/src/mapper/basic/konami.rs:1-74`
  - 新增 Mapper 75 / VRC1。
  - 覆盖 8KB PRG、4KB CHR、CHR 高位模式和 mirroring。
- `fc-core/src/mapper/basic/jy.rs:1-133`
  - 新增 Mapper 91 / JY Company。
  - 覆盖 2KB CHR、8KB PRG、submapper 1 outer bank/mirroring latch，以及 FCEUX/FCEUmm 风格 HBlank IRQ。
- `fc-core/src/mapper/mmc3.rs:9-135,221-281,447-470`
  - 新增 Mapper 76 的 MMC3 变体布局。
  - 复用 MMC3 PRG/IRQ 核心，仅扩展自定义 2KB CHR cwrap。
- `fc-core/src/bus.rs:262-268` 与 `fc-core/src/cartridge.rs:64-67,267-313,748-790`
  - 新增 mapper HBlank clock 架构钩子和缓存能力位。
- `fc-core/src/mapper.rs:204-214, 223-305, 318-438, 490-657`
  - mapper facade 的导出、枚举、构造表和 dispatch 接入。
- `fc-core/src/mapper.rs:680-998, 1373-1961`
  - mapper capability 快路径守门测试和新增 unlicensed mapper 行为测试。
- `fc-core/src/cartridge.rs:343-430`
  - CPU 读路径新增 open-bus aware high-register read、低区 PRG-RAM 合成读、扩展区 PRG-ROM 映射。
- `fc-core/src/bus.rs:478-479`
  - Bus 将 CPU open bus 传入 Cartridge 读路径。

## 对照来源

| Mapper | 当前实现范围 | 参考来源 | 行号 | 主要用途 |
|---:|---|---|---:|---|
| 36 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/36.cpp` | 28-65 | TXC/Micro Genius 简化 mapper、PRG/CHR latch、$4100 读回、bus conflict |
| 36 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/txcchip.c` | 177-194 | TXC mapper 36 cross-check；FCEUmm 更完整 TXC 芯片模型留作后续精修参考 |
| 43 | `unlicensed.rs:5-121` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper43.h` | 16-20, 22-33, 49-82 | PRG/CHR page size、$5000/$6000 映射、寄存器地址译码、4096 CPU-cycle IRQ |
| 43 | `unlicensed.rs:5-121` | `/Users/sunmeng/workspace/fc/fceux/src/boards/43.cpp` | 38-58, 72-78 | `transo` LUT、swap bank、IRQ counter cross-check |
| 59 | `multicart.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/addrlatch.cpp` | 164-179 | address latch PRG32/CHR/mirroring、bit8 read gate 参考 |
| 59 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 158-173 | 历史实现 cross-check；FCEUmm 注释其旧实现曾误归类 |
| 60 | `unlicensed.rs:123-163` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper60.h` | 11-29 | reset counter 选择 PRG/CHR bank |
| 60 | `unlicensed.rs:123-163` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/60.c` | 23-35 | reset hook 行为 cross-check |
| 63 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 203-235 | NTDEC multicart PRG mode、submapper mask、越界 PRG open-bus、mirroring |
| 83 | `unlicensed.rs:165-337` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper83.h` | 18-23, 57-68, 71-108, 111-156 | YOKO/30-in-1 PRG/CHR 模式、低寄存器、CPU IRQ |
| 83 | `unlicensed.rs:165-337` | `/Users/sunmeng/workspace/fc/fceux/src/boards/yoko.cpp` | 73-99, 118-139, 164-176, 197-204 | FCEUX 旧 mapper 83 译码和 IRQ cross-check |
| 83 | `unlicensed.rs:165-337` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/83_264.c` | 41-117, 130-157, 159-189, 219-231 | 新 submapper 设计参考；当前仅落地基础 mapper 83 行为 |
| 72 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/72.cpp` | 36-53, 66-72 | Jaleco mapper 72 PRG fixed-high/CHR latch 写位 |
| 72 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/72.c` | 35-53, 59-64 | mapper 72 `$6000-$FFFF` 写窗口 cross-check |
| 75 | `konami.rs:1-74` | `/Users/sunmeng/workspace/fc/fceux/src/boards/vrc1.cpp` | 35-52, 66-70 | VRC1 PRG8/CHR4/mode/mirroring |
| 75 | `konami.rs:1-74` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/vrc1.c` | 35-52, 66-70 | VRC1 行为 cross-check |
| 76 | `mmc3.rs:9-135,221-281` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 696-706 | Mapper 76 MMC3 core + custom CHR cwrap |
| 76 | `mmc3.rs:9-135,221-281` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 772-781 | `M76CW` CHR2 映射 cross-check |
| 79 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/79.cpp` | 32-49, 57-60 | NINA-003/006 PRG32/CHR8 latch 和高区写兼容行为 |
| 79 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/79.c` | 32-49, 56-60 | `$4100-$5FFF` 且 `A&0x100` 写门控 cross-check |
| 80 | `taito.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/80.cpp` | 58-103, 136-176 | Taito X1-005 PRG/CHR/mirroring/256B WRAM enable |
| 80 | `taito.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/80.c` | 58-103, 136-176 | mapper 80 行为 cross-check |
| 82 | `taito.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/82.cpp` | 37-63, 66-96 | Taito X1-017 PRG/CHR swap/mirroring |
| 82 | `taito.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/82_552.c` | 38-71, 92-107 | mapper 82 与 552 PRG bit 译码差异 cross-check；当前只落地 mapper 82 |
| 91 | `jy.rs:1-133` | `/Users/sunmeng/workspace/fc/fceux/src/boards/91.cpp` | 35-48, 51-57, 67-74, 80-83 | JY PRG/CHR low-register decode、HBlank IRQ hook |
| 91 | `jy.rs:1-133` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/91.c` | 48-61, 63-87, 97-104, 110-118 | submapper 1 outer bank/mirroring latch 与 HBlank IRQ |
| 91 | `jy.rs:1-133` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/JyCompany/Mapper91.h` | 5-18, 25-42 | Mapper 91 PRG/CHR register decode cross-check；Mesen2 用 MMC3 IRQ 复用 |
| 92 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/72.cpp` | 35-80 | Jaleco 2-in-1 mapper 92 PRG fixed-low/CHR latch 写位 |
| 92 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 247-263 | address-latch 形式 cross-check；保留后续按地址高位差异精修入口 |
| 106 | `unlicensed.rs:339-431` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper106.h` | 14-16, 18-26, 36-73 | PRG/CHR register decode、CPU-cycle IRQ |
| 106 | `unlicensed.rs:339-431` | `/Users/sunmeng/workspace/fc/fceux/src/boards/106.cpp` | 36-59, 81-87 | FCEUX PRG/CHR sync 和 IRQ overflow cross-check |
| 183 | `unlicensed.rs:433-579` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper183.h` | 17-21, 46-83, 86-105 | 低区 PRG-ROM、VRC-like CHR nibble register、mirroring、IRQ scaler |
| 183 | `unlicensed.rs:433-579` | `/Users/sunmeng/workspace/fc/fceux/src/boards/183.cpp` | 41-47, 49-61, 70-96 | FCEUX Gimmick bootleg bank/IRQ cross-check |
| 201 | `multicart.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/addrlatch.cpp` | 229-237 | 21-in-1 address latch PRG32/CHR bank |
| 201 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 284-289 | Mapper 201 fixed-horizontal note 与 bank 译码 cross-check |
| 212 | `unlicensed.rs:581-650` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper212.h` | 8-15, 18-38 | address latch PRG/CHR/mirroring、低区读 OR bit7 |
| 212 | `unlicensed.rs:581-650` | `/Users/sunmeng/workspace/fc/fceux/src/boards/addrlatch.cpp` | 272-293 | M212Sync / M212Read cross-check |
| 217 | `multicart.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/addrlatch.cpp` | 325-332 | address latch PRG32/CHR bank |
| 217 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 363-368 | Mapper 217 bank 译码 cross-check |
| 222 | `unlicensed.rs:652-764` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper222.h` | 15-23, 33-64 | PRG/CHR register decode、A12 IRQ counter |
| 222 | `unlicensed.rs:652-764` | `/Users/sunmeng/workspace/fc/fceux/src/boards/222.cpp` | 52-81, 84-98 | FCEUX VRC4-like bootleg sync 和 IRQ cross-check |
| 222 | `unlicensed.rs:652-764` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/222.c` | 39-65, 67-90 | 新 VRC2-style IRQ 参考；当前实现先按 Mesen2 A12 行为 |
| 235 | `unlicensed.rs:766-884` | `/Users/sunmeng/workspace/fc/fceux/src/boards/235.cpp` | 42-77, 80-95, 102-109 | open-bus latch、reset-selected UNROM mode、raw PRG size 比较 |
| 235 | `unlicensed.rs:766-884` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/235.c` | 38-73, 76-89, 96-101 | mapper 235 行为 cross-check |

## 当前实现和来源的对应关系

- `Mapper43::write_any_register()` 对应 Mesen2 `Mapper43.h:69-82` 与 FCEUX `43.cpp:51-58`。
- `Mapper43::expansion_prg_index()` / `low_prg_index()` 对应 Mesen2 `Mapper43.h:29-33,62-67` 与 FCEUX `43.cpp:38-48`。
- `Mapper36::write_register()` / `$4100` 读回对应 FCEUX `36.cpp:35-53`。
- `Mapper72::write_register()` / `write_low_register()` 对应 FCEUX `72.cpp:36-53` 与 FCEUmm `72.c:41-53`。
- `Vrc1::write_register()` / `chr_page()` 对应 FCEUX/FCEUmm `vrc1.cpp`/`vrc1.c:35-52,66-70`。
- `Mmc3::new_76()` / `mapper76_chr_write()` 对应 FCEUX `mmc3.cpp:696-706` 与 FCEUmm `mmc3.c:772-781`。
- `Mapper79::write_expansion()` 对应 FCEUmm `79.c:37-42`；高区 `write_register()` 兼容 FCEUX `79.cpp:37-49` 的 register write 路径。
- `AddrLatchVariant::Mapper59` 对应 FCEUX `addrlatch.cpp:164-179` 的 M59Sync/M59Read，包括 bit8 置位时高区读返回 0。
- `Mapper60::reset()` 对应 Mesen2 `Mapper60.h:22-30` 与 FCEUmm `60.c:32-35`。
- `Mapper63::set_from_addr()` / open-bus high read 对应 FCEUmm `addrlatch.c:203-235`。
- `Mapper83` 的 PRG/CHR 译码对应 Mesen2 `Mapper83.h:71-99`，低寄存器读写对应 `Mapper83.h:102-114`，IRQ 对应 `Mapper83.h:57-68,146-154`。
- `Mapper91::write_low_register()` / `hblank_clock()` 对应 FCEUX `91.cpp:46-57,67-74` 和 FCEUmm `91.c:63-81,97-104`。
- `Mapper91` submapper 1 的 `outer_bank` / `mirroring_latch` 对应 FCEUmm `91.c:48-61,84-87`。
- `Mapper92::write_register()` 对应 FCEUX `72.cpp:35-80` 的 mapper 92 变体。
- `Mapper106::write_register()` / `cpu_clock()` 对应 Mesen2 `Mapper106.h:36-73`。
- `Mapper183::write_any_register()` / `cpu_clock()` 对应 Mesen2 `Mapper183.h:52-83,86-105`。
- `AddrLatchVariant::Mapper201` / `Mapper217` 对应 FCEUX `addrlatch.cpp:229-237,325-332` 与 FCEUmm `addrlatch.c:284-289,363-368`。
- `Mapper212::select_from_addr()` / low read OR bit7 对应 Mesen2 `Mapper212.h:18-38` 和 FCEUX `addrlatch.cpp:274-289`。
- `Mapper222::notify_a12()` / register decode 对应 Mesen2 `Mapper222.h:33-64`。
- `Mapper235::read_register_with_open_bus()` / reset UNROM mode 对应 FCEUX `235.cpp:42-95`。
- `TaitoX1005::write_low_register()` / gated WRAM 对应 FCEUX/FCEUmm `80.cpp`/`80.c:75-103,136-142`。
- `TaitoX1017::chr_index()` 的 pattern half swap 对应 FCEUX `82.cpp:37-50` 与 FCEUmm `82_552.c:45-58`。

## 以后替换时的删除边界

- 先替换 `fc-core/src/mapper/basic/unlicensed.rs:1-884`。
- 同批替换 `fc-core/src/mapper/basic/latch/discrete.rs` 里 Mapper 36 / 72 / 79 / 92 的新增段、`fc-core/src/mapper/basic/taito.rs` 里 Mapper 80 / 82 的新增段，以及 `fc-core/src/mapper/basic/multicart.rs` 里 Mapper 59 / 63 / 201 / 217 的新增段。
- 同批替换 `fc-core/src/mapper/basic/konami.rs` 的 VRC1 段、`fc-core/src/mapper/basic/jy.rs` 的 Mapper91 段，以及 `fc-core/src/mapper/mmc3.rs` 的 Mapper76 CHR layout 变体段。
- 再处理 `fc-core/src/mapper.rs` 里 Mapper 43/60/75/76/83/91/106/183/212/222/235 的导出、枚举、构造和 dispatch 分支。
- 若替换 Mapper91，请同步检查 `MapperOps::hblank_clock`、`Cartridge::mapper_clocks_hblank` 与 `Bus::clock_ppu_dot()` 的 HBlank hook 是否仍有使用者。
- 最后检查 `fc-core/src/cartridge.rs` 的 open-bus aware 读钩子是否仍被其他 mapper 使用；如果无使用者，可收窄接口。
