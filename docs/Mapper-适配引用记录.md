# Mapper 适配引用记录

这份记录只用于当前 mapper 适配面的来源回溯和后续开源致谢。
如果将来需要做闭源分支，优先替换下面列出的代码范围。

## 当前代码范围

- `fc-core/src/mapper/basic/unlicensed.rs:1-884`
  - 新增 Mapper 43 / 60 / 83 / 106 / 183 / 212 / 222 / 235。
  - 覆盖 PRG/CHR bank 译码、低地址 PRG-ROM 窗口、mapper register read、open-bus 读侧效应、reset hook、CPU clock IRQ、A12 IRQ。
- `fc-core/src/mapper/basic/latch/discrete.rs`
  - 新增 Mapper 36 / 72 / 79 / 92 / 122。
  - 覆盖 TXC/Micro Genius 简化 latch、$4100 读回、bus conflict、Jaleco 2-in-1/JF-17 PRG/CHR 写位规则、NINA-003/006 扩展区 latch，以及 Mapper 122 双 4KB CHR latch。
- `fc-core/src/mapper/basic/latch/sachen.rs:1-135`
  - 新增 Mapper 133 / Sachen SA72008、Mapper 146 / Sachen SA016-1M、Mapper 148 / Sachen SA0037 与 Mapper 149 / Sachen SA0036。
  - 覆盖 SA72008 PRG32/CHR8 latch、SA016-1M/SA0037 PRG32/CHR8 latch 和 SA0036 CHR bit7 latch。
- `fc-core/src/mapper/basic/core.rs:163-217,373-403`
  - 扩展 Mapper 11 / Color Dreams，并新增 Mapper 144 / AGCI 50282 变体。
  - 覆盖 Color Dreams 4-bit PRG/CHR latch、bus conflict，以及 Mapper 144 奇地址写窗口与 bit0-only conflict 规则。
- `fc-core/src/mapper/basic/taito.rs:81-222,305-335`
  - 新增 Mapper 80 / 207 / 82。
  - 覆盖 Taito X1-005/X1-017 的低地址寄存器、8KB PRG、混合 2KB/1KB CHR、mirroring、Mapper 80 的 256B gated WRAM，以及 Mapper 207 的 CHR register bit7 到 per-nametable CIRAM A10 映射。
- `fc-core/src/mapper/basic/multicart.rs`
  - 新增 Mapper 59 / 63 / 201 / 217 / 228 / 255。
  - 覆盖 address latch PRG/CHR/mirroring、Mapper 63 越界 PRG open-bus 读、Mapper 228 Action Enterprises 地址线 bank + nibble RAM、Mapper 255 BMC 地址 latch。
- `fc-core/src/mapper/basic/core.rs:278-359`
  - 新增 Mapper 232 / Codemasters BF9096。
  - 覆盖 BF9096/Quattro 16KB PRG block + page register 和 submapper 1 outer bank bit swap。
- `fc-core/src/mapper/basic/konami.rs:1-74`
  - 新增 Mapper 75 / VRC1。
  - 覆盖 8KB PRG、4KB CHR、CHR 高位模式和 mirroring。
- `fc-core/src/mapper/basic/latch/sunsoft.rs:1-159`
  - 新增 Mapper 68 / Sunsoft-4。
  - 覆盖 16KB PRG、四个 2KB CHR、mirroring 控制，以及 nametable 到 CHR-ROM/CHR-RAM 1KB page 的映射。
- `fc-core/src/mapper/basic/jy.rs:1-133`
  - 新增 Mapper 91 / JY Company。
  - 覆盖 2KB CHR、8KB PRG、submapper 1 outer bank/mirroring latch，以及 FCEUX/FCEUmm 风格 HBlank IRQ。
- `fc-core/src/mapper/basic/namco.rs:1-302`
  - 新增 Mapper 95 / Namco 108 Rev. B 与 Mapper 206 / Namco 108 子集。
  - 覆盖 Namco108 风格高区寄存器、固定 PRG/CHR mode、CHR register bit5 到 per-nametable CIRAM A10 映射，以及 Mapper 206 的无 IRQ PRG8/CHR2+1 bank mask。
- `fc-core/src/mapper/basic/sl12.rs:1-344`
  - 新增 Mapper 116 / Someri Team SL12。
  - 覆盖 VRC2/MMC3/MMC1 三模式切换、VRC2 nibble CHR、MMC3 A12 IRQ、MMC1 serial register、CHR outer bank bit。
- `fc-core/src/mapper/basic/waixing.rs:1-295`
  - 新增 Mapper 253 / Waixing Dragon Ball pirate。
  - 覆盖 8KB PRG、1KB CHR nibble register、2KB mapper-owned CHR-RAM window、mirroring、CPU-clock IRQ。
- `fc-core/src/mapper/mmc3.rs:9-1238`
  - 新增 Mapper 37 / 44 / 45 / 47 / 49 / 52 / 76 / 114 / 115 / 118 / 119 / 121 / 192 / 195 的 MMC3 变体布局。
  - 复用 MMC3 PRG/IRQ 核心，扩展 outer PRG/CHR bank latch、Mapper 45 serial outer registers、Mapper 49 outer latch、Mapper 76 自定义 2KB CHR cwrap、Mapper 114/115/121 写协议与保护寄存器、Mapper 118 TxSROM per-nametable CIRAM A10，以及 Mapper 119 TQROM CHR-ROM/CHR-RAM window。
- `fc-core/src/mapper/rambo1.rs:1-309`
  - 新增 Mapper 64 / Tengen RAMBO-1。
  - 覆盖 8KB PRG bank mode、2KB/1KB CHR mode、CHR A12 inversion、mapper-controlled mirroring、CPU/PPU A12 双模式 IRQ、IRQ 延迟与 CPU-mode force-clock。
- `fc-core/src/mapper/vrc4.rs:1-329`
  - 扩展 Mapper 21 / 22 / 23 / 25 的 VRC2/VRC4 共用实现。
  - 覆盖 mapper/submapper 地址线变体、VRC2a CHR 右移、VRC2 无 IRQ、VRC4 CPU-clock IRQ。
- `fc-core/src/bus.rs:262-268` 与 `fc-core/src/cartridge.rs:64-67,267-313,748-790`
  - 新增 mapper HBlank clock 架构钩子和缓存能力位。
- `fc-core/src/mapper.rs:129-141, 215-225, 236-322, 335-482, 504-690`
  - mapper facade 的导出、枚举、构造表和 dispatch 接入。
- `fc-core/src/mapper.rs:680-998, 1373-1961`
  - mapper capability 快路径守门测试和新增 unlicensed mapper 行为测试。
- `fc-core/src/cartridge.rs:76-79, 269-321, 524-560, 840-860`
  - 新增 nametable-to-CHR 映射缓存与 Cartridge 侧 CHR-ROM/CHR-RAM 解析。
- `fc-core/src/cartridge.rs:343-430`
  - CPU 读路径新增 open-bus aware high-register read、低区 PRG-RAM 合成读、扩展区 PRG-ROM 映射。
- `fc-core/src/bus.rs:478-479`
  - Bus 将 CPU open bus 传入 Cartridge 读路径。

## 对照来源

| Mapper | 当前实现范围 | 参考来源 | 行号 | 主要用途 |
|---:|---|---|---:|---|
| 21 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/21_22_23_25.c` | 33-39 | VRC4a/VRC4c submapper 到地址线 mask 映射 |
| 21 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Konami/VRC2_4.h` | 37-48, 223-233 | mapper 21 variant detection 与 submapper 0 OR heuristics cross-check |
| 22 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/21_22_23_25.c` | 42-48 | VRC2a 地址线和 CHR bank 右移 |
| 22 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Konami/VRC2_4.h` | 51, 121-130, 251-256 | VRC2a 变体识别、CHR 低位忽略、地址翻译 |
| 23 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/21_22_23_25.c` | 51-58 | VRC4f/VRC4e/VRC2b submapper 地址线映射 |
| 23 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Konami/VRC2_4.h` | 53-60, 235-245 | mapper 23 VRC2b/VRC4e variant detection 与 OR heuristics |
| 25 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/21_22_23_25.c` | 61-67 | VRC4b/VRC4d/VRC2c submapper 地址线映射 |
| 25 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Konami/VRC2_4.h` | 63-71, 210-221 | mapper 25 variant detection 与 submapper 0 OR heuristics |
| 36 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/36.cpp` | 28-65 | TXC/Micro Genius 简化 mapper、PRG/CHR latch、$4100 读回、bus conflict |
| 36 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/txcchip.c` | 177-194 | TXC mapper 36 cross-check；FCEUmm 更完整 TXC 芯片模型留作后续精修参考 |
| 37 | `mmc3.rs:15-21,99-136,240-274,534-550` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 418-457 | Mapper 37 outer bank PRG/CHR wrapping 与低地址 latch |
| 37 | `mmc3.rs:15-21,99-136,240-274,534-550` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 412-451 | Mapper 37 FCEUmm cross-check |
| 44 | `mmc3.rs:15-21,106-110,240-274,381-390,552-565` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 462-497 | Mapper 44 A001 outer bank select、block >=6 PRG/CHR mask |
| 44 | `mmc3.rs:15-21,106-110,240-274,381-390,552-565` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 456-500 | Mapper 44 reset/power cross-check；当前 block 7 按 FCEUX/FCEUmm 不 clamp |
| 45 | `mmc3.rs:15-23,114-122,248-294,432-446,540-545,604-635` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 502-565 | Mapper 45 PRG/CHR wrapper、低区 serial register 写、锁定位、reset/power defaults、可选 open-bus read 记录 |
| 45 | `mmc3.rs:15-23,114-122,248-294,432-446,540-545,604-635` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 505-589 | Mapper 45 PRG/CHR mask/OR、低区 serial register 写、reset/power、可选 open-bus high/low read 记录 |
| 45 | `mmc3.rs:15-23,114-122,248-294,432-446,540-545,604-635` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_45.h` | 40-80 | Mapper 45 reset、PRG/CHR bank wrapper、低区 serial write 与 lock 行为 cross-check |
| 45 | `mmc3.rs:15-23,114-122,248-294,432-446,540-545,604-635` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardBmcHero.cpp` | 97-129 | BMC Hero serial register 写入与 PRG/CHR 更新 cross-check |
| 47 | `mmc3.rs:15-21,113-126,240-274,400-414,567-596` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 570-601 | Mapper 47 1-bit outer bank latch |
| 47 | `mmc3.rs:15-21,113-126,240-274,400-414,567-596` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 602-644 | Mapper 47 submapper lock 与 low write fall-through |
| 49 | `mmc3.rs:15-26,165-178,382-674,1095-1110` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 604-647 | Mapper 49 outer latch、PRG32/MMC3 mode、CHR bit extension、reset/power defaults |
| 49 | `mmc3.rs:15-26,165-178,382-674,1095-1110` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 647-705 | Mapper 49 submapper 1 `$41` default 与 partial write preserve 行为 |
| 52 | `mmc3.rs:15-21,128-136,240-274,400-414,598-616` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 652-693 | Mapper 52 one-shot low latch、outer PRG/CHR mask |
| 52 | `mmc3.rs:15-21,128-136,240-274,400-414,598-616` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 710-769 | Mapper 52 submapper 14/CHR-RAM alternate path 记录；当前只实现基础路径 |
| 43 | `unlicensed.rs:5-121` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper43.h` | 16-20, 22-33, 49-82 | PRG/CHR page size、$5000/$6000 映射、寄存器地址译码、4096 CPU-cycle IRQ |
| 43 | `unlicensed.rs:5-121` | `/Users/sunmeng/workspace/fc/fceux/src/boards/43.cpp` | 38-58, 72-78 | `transo` LUT、swap bank、IRQ counter cross-check |
| 59 | `multicart.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/addrlatch.cpp` | 164-179 | address latch PRG32/CHR/mirroring、bit8 read gate 参考 |
| 59 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 158-173 | 历史实现 cross-check；FCEUmm 注释其旧实现曾误归类 |
| 60 | `unlicensed.rs:123-163` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper60.h` | 11-29 | reset counter 选择 PRG/CHR bank |
| 60 | `unlicensed.rs:123-163` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/60.c` | 23-35 | reset hook 行为 cross-check |
| 63 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 203-235 | NTDEC multicart PRG mode、submapper mask、越界 PRG open-bus、mirroring |
| 64 | `rambo1.rs:1-309` | `/Users/sunmeng/workspace/fc/fceux/src/boards/tengen.cpp` | 23-139 | Tengen RAMBO-1 state、bank sync、register decode、旧 IRQ hook 和 init cross-check |
| 64 | `rambo1.rs:1-309` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/tengen.c` | 21-195 | Mapper 64/158 共用 RAMBO-1、现代 trigger-on-reach-zero IRQ 注释、PRG/CHR/mirroring/register 行为 |
| 64 | `rambo1.rs:1-309` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Tengen/Rambo1.h` | 11-177 | RAMBO-1 PRG/CHR mode、CPU/PPU IRQ source、IRQ delay、force-clock quirk、A12 watcher |
| 64 | `rambo1.rs:1-309` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/A12Watcher.h` | 26-54 | PPU A12 low-time filter semantics |
| 64 | `rambo1.rs:1-309` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardTengenRambo1.cpp` | 75-96, 190-225, 233-344 | RAMBO-1 register map、IRQ unit、PRG/CHR update、write handlers |
| 64 | `rambo1.rs:1-309` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardTengenRambo1.hpp` | 82-106 | CPU M2 divisor、A12 filter、IRQ delay/source constants |
| 68 | `latch/sunsoft.rs:1-159; mapper.rs:129-141; cartridge.rs:524-560` | `/Users/sunmeng/workspace/fc/fceux/src/boards/68.cpp` | 41-76, 96-119, 130-134 | Sunsoft-4 nametable CHR mapping、四个 2KB CHR register、16KB PRG register、mirroring/control 写窗口 |
| 68 | `latch/sunsoft.rs:1-159; mapper.rs:129-141; cartridge.rs:524-560` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/68.c` | 41-76, 96-119, 130-134 | FCEUmm mapper 68 cross-check；记录同源 PRG/CHR/NT 译码 |
| 68 | `latch/sunsoft.rs:1-159; mapper.rs:129-141; cartridge.rs:524-560` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Sunsoft/Sunsoft4.h` | 17-35, 106-132 | `UpdateNametables()` 的 CHR-backed nametable page 选择、control bit4、C/D/E/F 寄存器写入 |
| 68 | `latch/sunsoft.rs:1-159; mapper.rs:129-141; cartridge.rs:524-560` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardSunsoft4.cpp` | 102-129 | `CTRL_CROM` nametable source 与 V/H/单屏 page select cross-check |
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
| 95 | `namco.rs:34-203` | `/Users/sunmeng/workspace/fc/fceux/src/boards/80.cpp` | 106-122, 124-134, 153-184 | Mapper 95 写寄存器、CHR bit5 到 mirroring cache、PPU hook 模型 cross-check |
| 95 | `namco.rs:34-203` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/80.c` | 106-122, 124-134, 153-184 | FCEUmm Mapper 95 与 FCEUX 同源行为 cross-check |
| 95 | `namco.rs:34-203` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Namco/Namco108.h` | 5-23 | Namco108 固定 PRG/CHR mode 与 hardwired mirroring 行为 |
| 95 | `namco.rs:34-203` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Namco/Namco108_95.h` | 5-18 | Mapper 95 reg0/reg1 bit5 到四个 nametable 页映射；当前实现采用此模型 |
| 206 | `namco.rs:86-154,275-301` | `/Users/sunmeng/workspace/fc/fceux/src/boards/206.cpp` | 33-78 | Mapper 206 Namco108 subset：2KB/1KB CHR、8KB PRG、cmd/data 写译码、power defaults |
| 206 | `namco.rs:86-154,275-301` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/206_486.c` | 33-78 | FCEUmm Mapper 206 cross-check；同文件 80-98 记录相关 mapper 486 直写变体 |
| 206 | `namco.rs:86-154,275-301` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Namco/Namco108.h` | 6-28 | Namco108 写地址 mask、固定最后两个 PRG bank、hardwired mirroring 行为 |
| 206 | `namco.rs:86-154,275-301` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 479 | mapper 206 归类到 Namco108 |
| 106 | `unlicensed.rs:339-431` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper106.h` | 14-16, 18-26, 36-73 | PRG/CHR register decode、CPU-cycle IRQ |
| 106 | `unlicensed.rs:339-431` | `/Users/sunmeng/workspace/fc/fceux/src/boards/106.cpp` | 36-59, 81-87 | FCEUX PRG/CHR sync 和 IRQ overflow cross-check |
| 116 | `sl12.rs:1-344` | `/Users/sunmeng/workspace/fc/fceux/src/boards/116.cpp` | 64-163, 165-260, 264-305 | SL12 VRC2/MMC3/MMC1 PRG/CHR/mirroring、mode write、MMC3 HBlank IRQ、power defaults |
| 116 | `sl12.rs:1-344` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/116.c` | 42-73, 79-120, 122-145 | 新版 ASIC 复用设计、submapper/game 轮换记录；当前仅实现主线行为 |
| 116 | `sl12.rs:1-344` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper116.h` | 31-78, 102-123, 126-291 | mapper 116 register range、A12 IRQ、三模式 PRG/CHR/mirroring 与写寄存器 |
| 118 | `mmc3.rs:26-32,169-183,241-264,295-302,444-449,459-480,590-615,863-895` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 828-845, 1412-1426 | TxSROM/TLSROM/TKSROM CHR bit7 到 nametable mirroring、禁用普通 A000 mirroring |
| 118 | `mmc3.rs:26-32,169-183,241-264,295-302,444-449,459-480,590-615,863-895` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 938-953, 1702-1717 | FCEUmm TKSPPU/TKSWRAP 与 init 注册 cross-check |
| 118 | `mmc3.rs:26-32,169-183,241-264,295-302,444-449,459-480,590-615,863-895` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Nintendo/TxSRom.h` | 5-41 | TxSROM 在 MMC3 `8001` CHR 写时设置 nametable 页；当前实现采用此 per-page 模型 |
| 118 | `mmc3.rs:26-32,169-183,241-264,295-302,444-449,459-480,590-615,863-895` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardTxRom.cpp` | 38-56 | Nestopia TksRom `UpdateChr` 仅低 pattern table address 更新 nametable bank cross-check |
| 119 | `mmc3.rs:26-30,56-64,162-181,246-259,742-773` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 847-860, 1428-1435 | TQROM CHR bank bit 6 选择 8KB CHR-RAM mapping，低 6 位选择 CHR page |
| 119 | `mmc3.rs:26-30,56-64,162-181,246-259,742-773` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_ChrRam.h` | 5-24, 34-37 | 通用 MMC3 CHR-RAM bank range 机制 |
| 119 | `mmc3.rs:26-30,56-64,162-181,246-259,742-773` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 404 | mapper 119 使用 `MMC3_ChrRam(0x40, 0x7F, 8)` cross-check |
| 114 | `mmc3.rs:16-27,191-199,393-458,569-603,792-799,849-856,1098-1116` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 726-777 | Mapper 114 高区写协议重映射、cmd_pending 门控、PRG/CHR 强制位与 IRQ 地址重映射 |
| 114 | `mmc3.rs:16-27,191-199,393-458,569-603,792-799,849-856,1098-1116` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 802-881 | Mapper 114 security perm、low-register force path、CHR 扩展位 cross-check |
| 114 | `mmc3.rs:16-27,191-199,393-458,569-603,792-799,849-856,1098-1116` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_114.h` | 5-71 | Mapper 114 register remap、forced PRG32/16K、CHR extension 位与 IRQ register decode |
| 115 | `mmc3.rs:16-27,201-205,393-458,666-673,800-830,857-860,1099-1115` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 782-824 | Mapper 115 PRG/CHR/protection 低区寄存器译码与保护读回 |
| 115 | `mmc3.rs:16-27,201-205,393-458,666-673,800-830,857-860,1099-1115` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 886-931 | Mapper 115 低区寄存器写入、PRG/CHR OR 扩展与保护寄存器 cross-check |
| 115 | `mmc3.rs:16-27,201-205,393-458,666-673,800-830,857-860,1099-1115` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_115.h` | 6-69 | Mapper 115 PRG/CHR 扩展、保护寄存器与 readback 规则 |
| 121 | `mmc3.rs:16-27,208-214,446-494,614-663,833-870,1099-1114` | `/Users/sunmeng/workspace/fc/fceux/src/boards/121.cpp` | 28-127 | Mapper 121 `$5000-$5FFF` 保护寄存器、scramble、PRG/CHR override 与 `$8003` 译码 |
| 121 | `mmc3.rs:16-27,208-214,446-494,614-663,833-870,1099-1114` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/121.c` | 28-125 | Mapper 121 protection register LUT、`exRegs[3/4/5/6/7]` sync cross-check |
| 121 | `mmc3.rs:16-27,208-214,446-494,614-663,833-870,1099-1114` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_121.h` | 6-119 | Mapper 121 protection regs、A9713 extension register、PRG/CHR override 与 reset 默认 |
| 122 | `latch/discrete.rs:259-316` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/122.c` | 23-55 | 固定 PRG32、地址 A0 选择两个 4KB CHR latch |
| 144 | `core.rs:163-217,373-403` | `/Users/sunmeng/workspace/fc/fceux/src/boards/datalatch.cpp` | 222-233 | Mapper 144 复用 Mapper 11 sync，写窗口从 `$8001-$FFFF` 开始 |
| 144 | `core.rs:163-217,373-403` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/datalatch.c` | 157-167 | FCEUmm Mapper 144 cross-check |
| 144 | `core.rs:163-217,373-403` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/ColorDreams.h` | 5-29 | ColorDreams 4-bit PRG/CHR latch；mapper 144 只让 ROM bit0 参与 bus conflict |
| 146 | `latch/sachen.rs:70-135,218-251` | `/Users/sunmeng/workspace/fc/fceux/src/boards/sachen.cpp` | 258-284 | SA016-1M PRG32=`value>>3`、CHR8=`value&7`，低区 `$4100-$5FFF` 写 |
| 146 | `latch/sachen.rs:70-135,218-251` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/sachen.c` | 165-190 | FCEUmm SA016-1M cross-check |
| 146 | `latch/sachen.rs:70-135,218-251` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Nina03_06.h` | 5-35 | mapper 146 归入非 multicart Nina03_06：`(addr & 0xE100) == 0x4100` gate |
| 148 | `latch/sachen.rs:70-135,253-271` | `/Users/sunmeng/workspace/fc/fceux/src/boards/sachen.cpp` | 258-264, 306-316 | SA0037 复用 SA016-1M bank sync，但高区 `$8000-$FFFF` 写 |
| 148 | `latch/sachen.rs:70-135,253-271` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/sachen.c` | 165-171, 213-222 | FCEUmm SA0037 cross-check |
| 133 | `latch/sachen.rs:1-62,105-121` | `/Users/sunmeng/workspace/fc/fceux/src/boards/sachen.cpp` | 273-296 | SA72008 PRG32=`value>>2`、CHR8=`value&3` |
| 133 | `latch/sachen.rs:1-62,105-121` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/sachen.c` | 180-203 | FCEUmm SA72008 cross-check |
| 133 | `latch/sachen.rs:1-62,105-121` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Sachen/Sachen_133.h` | 10-25 | `$4100-$FFFF` 写窗口与 `(addr & 0x6100) == 0x4100` 门控 |
| 149 | `latch/sachen.rs:64-135` | `/Users/sunmeng/workspace/fc/fceux/src/boards/sachen.cpp` | 306-310 | SA0036 复用 SA72007 CHR=`value>>7` |
| 149 | `latch/sachen.rs:64-135` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/sachen.c` | 213-215 | FCEUmm SA0036 cross-check |
| 149 | `latch/sachen.rs:64-135` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Sachen/Sachen_149.h` | 10-19 | mapper 149 固定 PRG、CHR bit7 latch |
| 192 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 990-1008 | Mapper 192 CHR banks 8..B 路由到 4KB CHR-RAM |
| 192 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1026-1045 | FCEUmm Mapper 192 cross-check |
| 192 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_ChrRam.h` | 5-38 | 通用 MMC3 CHR-RAM window 机制 |
| 192 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 465 | mapper 192 使用 `MMC3_ChrRam(0x08, 0x0B, 4)` |
| 195 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1028-1045 | Mapper 195 基础 CHR banks 0..3 路由到 4KB CHR-RAM；FCEUX 还记录 `$5000` 4KB PRG-RAM |
| 195 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/195.c` | 30-97 | 新 FCEUmm Mapper 195 CHR-RAM + PPU write intercept 保护逻辑；当前仅落地 Mesen/FCEUX 基础 CHR-RAM window |
| 195 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_ChrRam.h` | 5-38 | 通用 MMC3 CHR-RAM window 机制 |
| 195 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 468 | mapper 195 使用 `MMC3_ChrRam(0x00, 0x03, 4)` |
| 183 | `unlicensed.rs:433-579` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper183.h` | 17-21, 46-83, 86-105 | 低区 PRG-ROM、VRC-like CHR nibble register、mirroring、IRQ scaler |
| 183 | `unlicensed.rs:433-579` | `/Users/sunmeng/workspace/fc/fceux/src/boards/183.cpp` | 41-47, 49-61, 70-96 | FCEUX Gimmick bootleg bank/IRQ cross-check |
| 201 | `multicart.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/addrlatch.cpp` | 229-237 | 21-in-1 address latch PRG32/CHR bank |
| 201 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 284-289 | Mapper 201 fixed-horizontal note 与 bank 译码 cross-check |
| 207 | `taito.rs:81-222,305-335` | `/Users/sunmeng/workspace/fc/fceux/src/boards/80.cpp` | 58-103,145-190 | Mapper 207 复用 Taito X1-005 写寄存器，并启用 PPU hook 风格 per-nametable mirroring cache |
| 207 | `taito.rs:81-222,305-335` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/80.c` | 58-103,145-188 | FCEUmm Mapper 207 行为 cross-check |
| 207 | `taito.rs:81-222,305-335` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Taito/TaitoX1005.h` | 50-97,110-113 | `TaitoX1005(true)` alternate mirroring：`7EF0/7EF1` bit7 选择 nametable 页，`7EF6/7EF7` mirroring 写被忽略 |
| 207 | `taito.rs:81-222,305-335` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 480 | mapper 207 归类到 `TaitoX1005(true)` |
| 212 | `unlicensed.rs:581-650` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper212.h` | 8-15, 18-38 | address latch PRG/CHR/mirroring、低区读 OR bit7 |
| 212 | `unlicensed.rs:581-650` | `/Users/sunmeng/workspace/fc/fceux/src/boards/addrlatch.cpp` | 272-293 | M212Sync / M212Read cross-check |
| 217 | `multicart.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/addrlatch.cpp` | 325-332 | address latch PRG32/CHR bank |
| 217 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 363-368 | Mapper 217 bank 译码 cross-check |
| 222 | `unlicensed.rs:652-764` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper222.h` | 15-23, 33-64 | PRG/CHR register decode、A12 IRQ counter |
| 222 | `unlicensed.rs:652-764` | `/Users/sunmeng/workspace/fc/fceux/src/boards/222.cpp` | 52-81, 84-98 | FCEUX VRC4-like bootleg sync 和 IRQ cross-check |
| 222 | `unlicensed.rs:652-764` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/222.c` | 39-65, 67-90 | 新 VRC2-style IRQ 参考；当前实现先按 Mesen2 A12 行为 |
| 228 | `multicart.rs:465-558,859-880` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/ActionEnterprises.h` | 5-41 | Action Enterprises 地址线 PRG/CHR/mirroring 译码与 reset 默认 |
| 228 | `multicart.rs:465-558,859-880` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/228.c` | 23-85 | Mapper 228 nibble RAM、地址 latch、mirroring、reset/power cross-check |
| 228 | `multicart.rs:465-558,859-880` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 499 | mapper 228 归类到 `ActionEnterprises` |
| 235 | `unlicensed.rs:766-884` | `/Users/sunmeng/workspace/fc/fceux/src/boards/235.cpp` | 42-77, 80-95, 102-109 | open-bus latch、reset-selected UNROM mode、raw PRG size 比较 |
| 235 | `unlicensed.rs:766-884` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/235.c` | 38-73, 76-89, 96-101 | mapper 235 行为 cross-check |
| 232 | `core.rs:278-359` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Codemasters/BF9096.h` | 5-52 | BF9096 PRG block/page 寄存器、固定 high page、submapper 1 bit swap |
| 232 | `core.rs:278-359` | `/Users/sunmeng/workspace/fc/fceux/src/boards/232.cpp` | 31-68 | FCEUX mapper 232 block/page 写窗口与 CHR fixed cross-check |
| 232 | `core.rs:278-359` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/232.c` | 31-69 | FCEUmm mapper 232 cross-check |
| 232 | `core.rs:278-359` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 503 | mapper 232 归类到 `BF9096` |
| 253 | `waixing.rs:1-295` | `/Users/sunmeng/workspace/fc/fceux/src/boards/253.cpp` | 44-89, 110-145 | Mapper 253 PRG/CHR/mirroring、IRQ、2KB CHR-RAM 与 8KB WRAM |
| 253 | `waixing.rs:1-295` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Waixing/Mapper253.h` | 17-21, 54-80, 83-130 | Mapper 253 page size、CHR-RAM window、114-cycle IRQ scaler、register decode |
| 253 | `waixing.rs:1-295` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/252_253.c` | 37-104 | 252/253 later VRC4-style CHR-RAM mask path；当前仅作为后续精修参考 |
| 255 | `multicart.rs:146-166,396-407,882-896` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Bmc255.h` | 5-25 | BMC255 地址 latch PRG/CHR/mirroring 公式；当前按 Mesen2 独立实现 |
| 255 | `multicart.rs:146-166,396-407,882-896` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 521 | mapper 255 归类到 `Bmc255` |

## 当前实现和来源的对应关系

- `Mapper43::write_any_register()` 对应 Mesen2 `Mapper43.h:69-82` 与 FCEUX `43.cpp:51-58`。
- `Mapper43::expansion_prg_index()` / `low_prg_index()` 对应 Mesen2 `Mapper43.h:29-33,62-67` 与 FCEUX `43.cpp:38-48`。
- `Mapper36::write_register()` / `$4100` 读回对应 FCEUX `36.cpp:35-53`。
- `Mapper72::write_register()` / `write_low_register()` 对应 FCEUX `72.cpp:36-53` 与 FCEUmm `72.c:41-53`。
- `Vrc1::write_register()` / `chr_page()` 对应 FCEUX/FCEUmm `vrc1.cpp`/`vrc1.c:35-52,66-70`。
- `Mmc3::new_76()` / `mapper76_chr_write()` 对应 FCEUX `mmc3.cpp:696-706` 与 FCEUmm `mmc3.c:772-781`。
- `Mapper79::write_expansion()` 对应 FCEUmm `79.c:37-42`；高区 `write_register()` 兼容 FCEUX `79.cpp:37-49` 的 register write 路径。
- `AddrLatchVariant::Mapper59` 对应 FCEUX `addrlatch.cpp:164-179` 的 M59Sync/M59Read，包括 bit8 置位时高区读返回 0。
- `Rambo1::write_register()` / `prg_index()` / `chr_index()` 对应 Mesen2 `Rambo1.h:96-167` 与 Nestopia `NstBoardTengenRambo1.cpp:233-344`；CPU-mode IRQ 每 4 CPU cycle、PPU-mode IRQ 走 A12 filter，分别对应 Mesen2 `Rambo1.h:55-75,170-177` 和 Nestopia `NstBoardTengenRambo1.hpp:82-106`。
- `MapperOps::nametable_chr_index()` 与 `Cartridge::nametable_read/peek/write()` 的 CHR 解析对应 Mesen2 `Sunsoft4.h:17-35` / Nestopia `NstBoardSunsoft4.cpp:102-119` 这类“nametable source = CHR”模型；该接口保持 mapper 只返回索引，实际 CHR-ROM/RAM 仍由 Cartridge 负责。
- `Sunsoft4::write_register()` / `nametable_chr_index()` 对应 FCEUX/FCEUmm `68.cpp`/`68.c:41-76,96-119` 与 Mesen2 `Sunsoft4.h:106-132`；外部 PRG licensing timer 暂未落地，留作后续 mapper 68 精修。
- `Mapper60::reset()` 对应 Mesen2 `Mapper60.h:22-30` 与 FCEUmm `60.c:32-35`。
- `Mapper63::set_from_addr()` / open-bus high read 对应 FCEUmm `addrlatch.c:203-235`。
- `Mapper83` 的 PRG/CHR 译码对应 Mesen2 `Mapper83.h:71-99`，低寄存器读写对应 `Mapper83.h:102-114`，IRQ 对应 `Mapper83.h:57-68,146-154`。
- `Mapper91::write_low_register()` / `hblank_clock()` 对应 FCEUX `91.cpp:46-57,67-74` 和 FCEUmm `91.c:63-81,97-104`。
- `Mapper91` submapper 1 的 `outer_bank` / `mirroring_latch` 对应 FCEUmm `91.c:48-61,84-87`。
- `Vrc4::config_for()` / `reg_select()` 对应 FCEUmm `21_22_23_25.c:33-67` 与 Mesen2 `VRC2_4.h:37-78,204-245`；submapper 0 使用参考实现中的 OR 地址线启发式。
- `Vrc4::chr_index()` 的 mapper 22 右移对应 Mesen2 `VRC2_4.h:121-130` 与 FCEUmm `21_22_23_25.c:42-48`。
- `Mmc3OuterBank::{Mapper37,Mapper44,Mapper47,Mapper49,Mapper52}` 对应 FCEUX `mmc3.cpp:418-497,570-647,652-693` 与 FCEUmm `mmc3.c:412-500,602-705,710-769`。
- `Mmc3OuterBank::{Mapper114,Mapper115,Mapper121}` 对应 FCEUX `mmc3.cpp:726-824`、FCEUmm `mmc3.c:802-931`、FCEUX/FCEUmm `121.cpp`/`121.c:28-127`，以及 Mesen2 `MMC3_114.h:5-71`、`MMC3_115.h:6-69`、`MMC3_121.h:6-119`。
- `Mmc3::write_bank_select()` / `write_bank_data()` / `write_standard_register()` 对应 MMC3 基础写协议；`mapper114_write()` / `mapper115_write_extra()` / `mapper121_write()` 则分别对应 114/115/121 的 protocol remap、protection register 和 scrambled extension register。
- `Mmc3OuterBank::Mapper45` 的 PRG/CHR AND/OR wrapper、低区 serial register 和 reset defaults 对应 FCEUX `mmc3.cpp:502-565`、FCEUmm `mmc3.c:505-589`、Mesen2 `MMC3_45.h:40-80` 与 Nestopia `NstBoardBmcHero.cpp:97-129`；FCEUX/FCEUmm 的可选 open-bus read 侧效应暂未落地，留作需要具体 ROM 证据时精修。
- `Mmc3OuterBank::Mapper114` / `Mapper115` / `Mapper121` 目前先按参考项目的第一版实现，后续若遇到特定 ROM 证据，可继续补齐更细的 reset/protection/read side-effect 行为。
- `Mapper253::write_chr_register()` / `chr_ram_index()` / `cpu_clock()` 对应 FCEUX `253.cpp:44-89,110-145` 与 Mesen2 `Mapper253.h:54-130`。
- `Mapper92::write_register()` 对应 FCEUX `72.cpp:35-80` 的 mapper 92 变体。
- `Mapper122::write_register()` 对应 FCEUmm `122.c:25-33` 的 A0 选择两个 4KB CHR latch。
- `ColorDreams::write_register()` / `apply_bus_conflict()` 对应 FCEUX/FCEUmm `datalatch.cpp`/`datalatch.c:222-233,157-167` 与 Mesen2 `ColorDreams.h:5-29`；mapper 144 只接受奇地址写并使用 bit0-only conflict。
- `MapperOps::apply_bus_conflict()` 是为 mapper 144 增加的 bus-conflict 细分钩子；默认仍保持原有 AND 语义。
- `Sachen133::write_low_register()` / `write_register()` 对应 Mesen2 `Sachen_133.h:19-25` 与 FCEUX/FCEUmm `sachen.cpp`/`sachen.c:273-296,180-203`。
- `SachenSa0161m::write_expansion()` / `write_register()` 对应 FCEUX/FCEUmm `sachen.cpp`/`sachen.c:258-284,306-316,165-190,213-222` 与 Mesen2 `Nina03_06.h:5-35`。
- `Sachen149::write_register()` 对应 Mesen2 `Sachen_149.h:16-19` 与 FCEUX/FCEUmm SA0036 的 SA72007 sync 路径。
- `Mapper106::write_register()` / `cpu_clock()` 对应 Mesen2 `Mapper106.h:36-73`。
- `Mapper116::write_expansion()` / `write_low_register()` 的 mode select 对应 FCEUX `116.cpp:165-181` 与 Mesen2 `Mapper116.h:282-291`。
- `Mapper116::write_vrc2()` / VRC2 PRG/CHR/mirroring 对应 FCEUX `116.cpp:184-199` 与 Mesen2 `Mapper116.h:225-239`。
- `Mapper116::write_mmc3()` / `notify_a12()` 对应 Mesen2 `Mapper116.h:102-123,241-268`；FCEUX 用 `GameHBIRQHook` 近似，当前实现按 Mesen2 A12 watcher 语义。
- `Mapper116::write_mmc1()` / MMC1 PRG/CHR/mirroring 对应 FCEUX `116.cpp:239-258` 与 Mesen2 `Mapper116.h:271-280`。
- `Mmc3ChrRamWindow` / `Mmc3::new_119()` 对应 FCEUX `TQWRAP` (`mmc3.cpp:849-850`) 与 Mesen2 `MMC3_ChrRam(0x40, 0x7F, 8)`；同时保留旧 `chr_ram_bank_base` 字段作为 mapper 74/194 存档兼容 fallback。
- `Mmc3::new_192()` / `Mmc3::new_195()` 复用 `Mmc3ChrRamWindow`，分别对应 Mesen2 `MapperFactory.cpp:465,468` 的 `MMC3_ChrRam(0x08,0x0B,4)` 与 `MMC3_ChrRam(0x00,0x03,4)`。
- `ActionEnterprises::sync()` / nibble expansion RAM 对应 FCEUmm `228.c:34-65` 与 Mesen2 `ActionEnterprises.h:21-40`。
- `Bf9096::write_register()` 对应 Mesen2 `BF9096.h:26-44`；FCEUX/FCEUmm `232.cpp`/`232.c:31-40` 用于 cross-check block/page 行为。
- `AddrLatchVariant::Mapper255` 对应 Mesen2 `Bmc255.h:16-24`。
- `Mapper183::write_any_register()` / `cpu_clock()` 对应 Mesen2 `Mapper183.h:52-83,86-105`。
- `AddrLatchVariant::Mapper201` / `Mapper217` 对应 FCEUX `addrlatch.cpp:229-237,325-332` 与 FCEUmm `addrlatch.c:284-289,363-368`。
- `Mapper212::select_from_addr()` / low read OR bit7 对应 Mesen2 `Mapper212.h:18-38` 和 FCEUX `addrlatch.cpp:274-289`。
- `Mapper222::notify_a12()` / register decode 对应 Mesen2 `Mapper222.h:33-64`。
- `Mapper235::read_register_with_open_bus()` / reset UNROM mode 对应 FCEUX `235.cpp:42-95`。
- `TaitoX1005::write_low_register()` / gated WRAM 对应 FCEUX/FCEUmm `80.cpp`/`80.c:75-103,136-142`。
- `Namco108Mapper206::write_register()` / `chr_index()` / `prg_index()` 对应 FCEUX/FCEUmm `206.cpp`/`206_486.c:33-78`；Mesen2 `Namco108.h:15-27` 用于确认地址 mask 与固定 PRG/CHR mode。
- `TaitoX1005::new_207()` / `set_chr_2k()` alternate mirroring 对应 FCEUX/FCEUmm `80.cpp`/`80.c:87-103,145-190` 与 Mesen2 `TaitoX1005.h:50-97,110-113`。
- `TaitoX1017::chr_index()` 的 pattern half swap 对应 FCEUX `82.cpp:37-50` 与 FCEUmm `82_552.c:45-58`。

## 以后替换时的删除边界

- 先替换 `fc-core/src/mapper/basic/unlicensed.rs:1-884`。
- 同批替换 `fc-core/src/mapper/basic/latch/discrete.rs` 里 Mapper 36 / 72 / 79 / 92 / 122 的新增段、`fc-core/src/mapper/basic/latch/sachen.rs` 的新增段、`fc-core/src/mapper/basic/core.rs` 里 ColorDreams/Mapper144 的扩展段、`fc-core/src/mapper/basic/taito.rs` 里 Mapper 80 / 207 / 82 的新增段，以及 `fc-core/src/mapper/basic/multicart.rs` 里 Mapper 59 / 63 / 201 / 217 / 228 / 255 的新增段。
- 同批替换 `fc-core/src/mapper/basic/latch/sunsoft.rs` 里 Mapper 68 的新增段，并同步检查 `MapperOps::nametable_chr_index` 与 `Cartridge::mapper_has_nametable_chr_mapping` 是否仍有其他使用者。
- 同批替换 `fc-core/src/mapper/basic/core.rs` 里 Mapper 232 的新增段。
- 同批替换 `fc-core/src/mapper/basic/konami.rs` 的 VRC1 段、`fc-core/src/mapper/basic/jy.rs` 的 Mapper91 段、`fc-core/src/mapper/basic/sl12.rs` 的 Mapper116 段、`fc-core/src/mapper/basic/waixing.rs` 的 Mapper253 段、`fc-core/src/mapper/vrc4.rs` 的 VRC2/VRC4 段、`fc-core/src/mapper/rambo1.rs` 的 Mapper64 段，以及 `fc-core/src/mapper/mmc3.rs` 的 Mapper37/44/45/47/52/76/119 变体段。
- 再处理 `fc-core/src/mapper.rs` 里 Mapper 43/60/75/76/83/91/106/183/212/222/235 的导出、枚举、构造和 dispatch 分支。
- 若替换 Mapper91，请同步检查 `MapperOps::hblank_clock`、`Cartridge::mapper_clocks_hblank` 与 `Bus::clock_ppu_dot()` 的 HBlank hook 是否仍有使用者。
- 最后检查 `fc-core/src/cartridge.rs` 的 open-bus aware 读钩子是否仍被其他 mapper 使用；如果无使用者，可收窄接口。
