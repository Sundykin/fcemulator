# Mapper 适配引用记录

这份记录只用于当前 mapper 适配面的来源回溯和后续开源致谢。
如果将来需要做闭源分支，优先替换下面列出的代码范围。

## 当前代码范围

- `fc-core/src/mapper/basic/unlicensed.rs:1-884`
  - 新增 Mapper 43 / 60 / 83 / 106 / 183 / 212 / 222 / 235。
  - 覆盖 PRG/CHR bank 译码、低地址 PRG-ROM 窗口、mapper register read、open-bus 读侧效应、reset hook、CPU clock IRQ、A12 IRQ。
- `fc-core/src/mapper/basic/latch/discrete.rs`
  - 新增 Mapper 8 / 29 / 31 / 36 / 72 / 79 / 81 / 92 / 96 / 99 / 122。
  - 覆盖 FFE/FJ-007 PRG16/CHR8 latch、Sealie Computing PRG16/CHR8 latch 与 32KB CHR-RAM 默认容量、NSF/INL 4KB PRG-ROM paging、TXC/Micro Genius 简化 latch、$4100 读回、bus conflict、NTDEC N715062 address/data latch、Jaleco 2-in-1/JF-17 PRG/CHR 写位规则、Mapper 96 的 PPU nametable latch、Mapper 99 的 `$4016` controller-strobe PRG/CHR latch、NINA-003/006 扩展区 latch，以及 Mapper 122 双 4KB CHR latch。
- `fc-core/src/mapper/basic/latch/sachen.rs:1-671`
  - 新增 Mapper 133 / Sachen SA72008、Mapper 146 / Sachen SA016-1M、Mapper 148 / Sachen SA0037、Mapper 149 / Sachen SA0036、Mapper 137 / 141 的 Sachen 8259D/8259A，以及 Mapper 150 / 243 的 Sachen 74LS374N。
  - 覆盖 SA72008 PRG32/CHR8 latch、SA016-1M/SA0037 PRG32/CHR8 latch、SA0036 CHR bit7 latch、Sachen 8259 的 current-register/data-register 写协议、PRG32、1KB/2KB CHR 与 mirroring 译码、Sachen 74LS374N 的 current-register/data-register 写协议、PRG32/CHR8/mirroring 译码，以及 Mapper150 DIP/open-bus 读回。
- `fc-core/src/mapper/basic/core.rs:163-217,373-403`
  - 扩展 Mapper 11 / Color Dreams，并新增 Mapper 144 / AGCI 50282 变体。
  - 覆盖 Color Dreams 4-bit PRG/CHR latch、bus conflict，以及 Mapper 144 奇地址写窗口与 bit0-only conflict 规则。
- `fc-core/src/mapper/basic/taito.rs:1-142,144-373`
  - 新增 Mapper 48 / 80 / 207 / 82。
  - 覆盖 Taito TC0190/Taito X1-005/X1-017 的 PRG/CHR/mirroring 寄存器、Mapper 48 的 HBlank IRQ、Mapper 80 的 256B gated WRAM，以及 Mapper 207 的 CHR register bit7 到 per-nametable CIRAM A10 映射。
- `fc-core/src/mapper/basic/multicart.rs:1-1411`
  - 新增 Mapper 28 / 51 / 59 / 63 / 128 / 201 / 217 / 221 / 228 / 236 / 237 / 239 / 255。
  - 覆盖 Action 53 多寄存器 PRG/CHR/mirroring 模式、Mapper 51 低区 PRG-ROM window + mode latch、address latch PRG/CHR/mirroring、Mapper 63 越界 PRG open-bus 读、Mapper 128 outer address latch + inner bank、Mapper 221 双地址 latch/UNROM-NROM 模式/open-bus 高区读、Mapper 228 Action Enterprises 地址线 bank + nibble RAM、Mapper 236/237 reset DIP 与读路径、Mapper 239 地址 latch，以及 Mapper 255 BMC 地址 latch。
- `fc-core/src/mapper/basic/core.rs:278-359`
  - 新增 Mapper 232 / Codemasters BF9096。
  - 覆盖 BF9096/Quattro 16KB PRG block + page register 和 submapper 1 outer bank bit swap。
- `fc-core/src/mapper/basic/konami.rs:1-74`
  - 新增 Mapper 75 / VRC1。
  - 覆盖 8KB PRG、4KB CHR、CHR 高位模式和 mirroring。
- `fc-core/src/mapper/basic/bandai.rs:1-515`
  - 新增 Mapper 16 / 153 / 159 的 Bandai FCG-1/FCG-2/LZ93D50 第一版。
  - 覆盖 16KB PRG、8 个 1KB CHR register、Mapper153 CHR low-bit PRG outer block、mirroring register、CPU-cycle IRQ、submapper 4/5 写窗口差异、FCG 直接 counter 与 LZ93D50 reload latch 差异、24C01/24C02 EEPROM bit4/open-bus 读，以及 Mapper153 `$800D` PRG-RAM gate。
  - 当前 factory 尚未把 NES 2.0 PRG-NVRAM byte 透传给 mapper，因此 mapper16 submapper 5 的“仅 header 声明 256B 时接 24C02”仍留作后续 factory 扩参精修；Mapper157/Datach 条码外设留待核心外设输入接口后补齐。
- `fc-core/src/mapper/basic/latch/sunsoft.rs:1-159`
  - 新增 Mapper 68 / Sunsoft-4。
  - 覆盖 16KB PRG、四个 2KB CHR、mirroring 控制，以及 nametable 到 CHR-ROM/CHR-RAM 1KB page 的映射。
- `fc-core/src/mapper/basic/jy.rs:1-624`
  - 新增 Mapper 35 / JY Company single-cart board、Mapper 90 / 209 / 211 JY ASIC 与 Mapper 91 / JY Company。
  - 覆盖 Mapper 35 的 8KB PRG、1KB CHR、A12 IRQ、mirroring register，Mapper 90/209/211 的 JY ASIC PRG/CHR/nametable/ALU/IRQ register model、低地址 PRG-ROM window、209/211 CHR latch、211 forced nametable control，以及 Mapper 91 的 2KB CHR、8KB PRG、submapper 1 outer bank/mirroring latch 和 FCEUX/FCEUmm 风格 HBlank IRQ。
  - JY ASIC 的 IRQ source 3（任意 CPU write clock）需要后续新增全局 CPU write notification hook；当前第一版覆盖 CPU-clock、HBlank 近似和 PPU-read/address-change source。
- `fc-core/src/mapper/basic/namco.rs:1-356`
  - 新增 Mapper 95 / Namco 108 Rev. B、Mapper 154 / Namco 108 单屏变体与 Mapper 206 / Namco 108 子集。
  - 覆盖 Namco108 风格高区寄存器、固定 PRG/CHR mode、CHR register bit5 到 per-nametable CIRAM A10 映射、Mapper 154 的 command bit6 单屏 mirroring，以及 Mapper 206 的无 IRQ PRG8/CHR2+1 bank mask。
- `fc-core/src/mapper/basic/special.rs:114-382,547-663`
  - 新增 Mapper 104 / Pegasus 5-in-1、Mapper 108 / FDS conversion、Mapper 168 / Racermate Challenge II、Mapper 175 delayed latch、Mapper 177 / Henggedianzi XH-32A 与 Mapper 190 / Magic Kid GooGoo。
  - 覆盖 Mapper 104 的双 16KB PRG register、固定 CHR8、固定垂直 mirroring、普通 `$6000-$7FFF` WRAM fallback，Mapper 108 的 `$6000-$7FFF` switchable PRG-ROM window、固定最后 32KB PRG-ROM、固定 CHR8 和 `$8000-$8FFF`/`$F000-$FFFF` 写窗口，Mapper 168 的 PRG16/CHR4 bank、64KB mapper-owned CHR-RAM、CPU-cycle IRQ 和 header mirroring，Mapper 175 的 `$FFFC` read side-effect 提交 PRG latch，以及 Mapper 177 的 PRG32 latch/固定 CHR8/WRAM fallback/mirroring bit。
- `fc-core/src/mapper/basic/opencorp.rs:1-121`
  - 新增 Mapper 156 / OpenCorp Daou306。
  - 覆盖 16KB PRG bank、固定最后 16KB PRG、8 个 1KB CHR low/high register、`$C014` mirroring register 与 reset hook。
- `fc-core/src/mapper/basic/subor.rs:1-124`
  - 新增 Mapper 166 / 167 Subor。
  - 覆盖四个高区 register、outer/inner PRG bank XOR、UNROM/inverted UNROM/NROM-like 三种 PRG 模式、mapper 167 bank order 变体，以及 FCEUmm 风格 mirroring bit。
- `fc-core/src/mapper/basic/discrete.rs:1-247`
  - 新增 Mapper 185 / CNROM copy-protection CHR disable、Mapper 188 / Karaoke Studio expansion cartridge 与 Mapper 193 / MEGA-SOFT War in the Gulf。
  - 覆盖 mapper-owned dummy CHR read/write gating、Mapper 188 PRG16 latch 与 `$6000-$7FFF` 固定设备读、固定首/尾 16KB PRG、低区 `$6000-$6003` register、固定尾部 8KB PRG pages，以及 4KB/2KB/2KB CHR bank 译码。
- `fc-core/src/mapper/basic/irq.rs:745-860`
  - 新增 Mapper 142 / Kaiser KS7032。
  - 覆盖 8KB PRG bank register、`$6000-$7FFF` PRG-ROM 低区窗口、固定 CHR8、nibble IRQ reload、CPU-cycle IRQ、`$E000/$F000` selected-register 写协议，以及 KS7032 风格 `$F000-$F003` PRG 高位写窗口。
- `fc-core/src/mapper/mmc1.rs:23-318`
  - 新增 Mapper 105 / Nintendo World Championships MMC1 变体与 Mapper 155 / MMC1 WRAM-always-enabled 变体入口。
  - 覆盖 Mapper 105 的 NWC 初始化状态机、CHR reg0 bit4 控制 CPU-cycle IRQ timer、CHR reg0 bit3 控制 MMC1/32KB PRG 模式、固定 CHR8、PRG bank bit3 强制，以及 Mapper 155 variant intent。
  - 当前本项目 MMC1 尚未实现 WRAM disable gating，因此 Mapper 155 先记录 variant 标记；后续补 PRG-RAM enable/disable 时可保留 mapper 155 的 always-enabled 语义。
- `fc-core/src/mapper/mmc3.rs:18-38,360-365,651,704,914-916,973-974,1326,1626-1654`
  - 新增 Mapper 250 / MMC3 地址线写协议变体。
  - 覆盖 register address remap、写入 data=`addr & 0xff`、普通 MMC3 PRG/CHR/mirroring/A12 IRQ 复用，以及 reset pass-through。
- `fc-core/src/mapper/basic/sl12.rs:1-344`
  - 新增 Mapper 116 / Someri Team SL12。
  - 覆盖 VRC2/MMC3/MMC1 三模式切换、VRC2 nibble CHR、MMC3 A12 IRQ、MMC1 serial register、CHR outer bank bit。
- `fc-core/src/mapper/basic/waixing.rs:1-804`
  - 新增 Mapper 252 / Waixing San Guo Zhi 与 Mapper 253 / Waixing Dragon Ball pirate。
  - 覆盖 Mapper252 的 8KB PRG、1KB CHR nibble register、8KB mapper-owned CHR-RAM mask/compare window、mirroring、VRC4-style CPU-clock IRQ 与 PPUDATA 写侧效应；覆盖 Mapper253 的 8KB PRG、1KB CHR nibble register、2KB mapper-owned CHR-RAM window、mirroring、CPU-clock IRQ。
- `fc-core/src/mapper.rs:182-185`、`fc-core/src/mapper/dispatch.rs:251-252` 与 `fc-core/src/ppu.rs:1192-1195`
  - 新增 `MapperOps::notify_ppudata_write()` 架构钩子，并在 CPU `$2007` 写入真实 CHR/nametable 前通知 mapper；当前用于 Mapper252 的 FCEUmm 风格 PPUDATA 写截获窗口。
- `fc-core/src/mapper/mmc3.rs:17-3031`
  - 新增 Mapper 12 / 14 / 37 / 44 / 45 / 47 / 49 / 52 / 76 / 114 / 115 / 118 / 119 / 121 / 126 / 134 / 176 / 182 / 187 / 189 / 191 / 192 / 195 / 196 / 197 / 198 / 199 / 205 / 208 / 215 / 224 / 238 / 245 / 249 / 250 / 254 的 MMC3 变体布局。
  - 复用 MMC3 PRG/IRQ 核心，扩展 outer PRG/CHR bank latch、Mapper 12 expansion CHR high-bit/language latch、Mapper 14 SL-1632 direct/MMC3 模式切换与 direct PRG/CHR latch、Mapper 45 serial outer registers、Mapper 49 outer latch、Mapper 76/197 自定义 2KB CHR cwrap、Mapper 114/115/121 写协议与保护寄存器、Mapper 118 TxSROM per-nametable CIRAM A10、Mapper 119 TQROM CHR-ROM/CHR-RAM window、Mapper 126 multicart PRG/CHR/mirroring/SL0 低区寄存器、Mapper 134 multicart outer PRG/CHR/DIP/lock 寄存器、Mapper 176 FK23C PRG/CHR outer regs、PRG mode 0-5、CNROM latch、extended MMC3 regs，Mapper 182 remapped writes/AX5202P outer bank、Mapper 187 保护读/PRG/CHR 扩展、Mapper 198 PRG mask/低区 WRAM、Mapper 199 fixed PRG slots/extra CHR slots/low bank CHR-RAM、Mapper 205 低区 outer block 与 PRG-RAM fall-through、Mapper 215 UNL-8237 register/address LUT 与 forced PRG modes、Mapper 224 `$5000` PRG outer bank、Mapper 238 security register、Mapper 249 security PRG/CHR bit permutation，以及 Mapper 208 PRG32 latch/保护 LUT。
- `fc-core/src/mapper/rambo1.rs:1-310`
  - 新增 Mapper 64 / Tengen RAMBO-1 与 Mapper 158 / Tengen 800037 变体。
  - 覆盖 8KB PRG bank mode、2KB/1KB CHR mode、CHR A12 inversion、mapper-controlled mirroring、Mapper 158 CHR bit7 到 nametable page 映射、CPU/PPU A12 双模式 IRQ、IRQ 延迟与 CPU-mode force-clock。
- `fc-core/src/mapper/vrc4.rs:1-329`
  - 扩展 Mapper 21 / 22 / 23 / 25 的 VRC2/VRC4 共用实现。
  - 覆盖 mapper/submapper 地址线变体、VRC2a CHR 右移、VRC2 无 IRQ、VRC4 CPU-clock IRQ。
- `fc-core/src/bus.rs:262-268` 与 `fc-core/src/cartridge.rs:64-67,267-313,748-790`
  - 新增 mapper HBlank clock 架构钩子和缓存能力位。
- `fc-core/src/mapper.rs:20-212, 215-371`
  - mapper facade 的 trait 扩展接口、模块声明、导出和 enum 定义。
- `fc-core/src/mapper/factory.rs:3-224`
  - mapper 构造表接入。
- `fc-core/src/mapper/dispatch.rs:3-249`
  - mapper enum dispatch 接入。
- `fc-core/src/mapper/tests.rs:1-1894`
  - mapper capability 快路径守门测试和新增 unlicensed mapper 行为测试。
- `fc-core/src/cartridge.rs:76-79, 269-321, 524-560, 840-860`
  - 新增 nametable-to-CHR 映射缓存与 Cartridge 侧 CHR-ROM/CHR-RAM 解析。
- `fc-core/src/cartridge.rs:343-450`
  - CPU 读路径新增 open-bus aware high-register read、低区 PRG-RAM/open-bus 合成读、扩展区 PRG-ROM 映射。
- `fc-core/src/cartridge.rs:449-515`
  - 低区默认 PRG-RAM 新增 mapper gate，用于 Mapper153 `$800D` 关闭 SRAM 读写时返回 open bus/忽略写入；默认开启，现有 mapper 行为不变。
- `fc-core/src/bus.rs:478-479`
  - Bus 将 CPU open bus 传入 Cartridge 读路径。
- `fc-core/src/mapper.rs:176-180`、`fc-core/src/cartridge.rs:407-409`、`fc-core/src/bus.rs:532-535`
  - 新增 `$4016` controller-strobe mapper write hook；Bus 先处理普通控制器 strobe，再通知 mapper，用于 VS System/Mapper99 这类复用 `$4016` 写入的板卡。

## 对照来源

| Mapper | 当前实现范围 | 参考来源 | 行号 | 主要用途 |
|---:|---|---|---:|---|
| 21 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/21_22_23_25.c` | 33-39 | VRC4a/VRC4c submapper 到地址线 mask 映射 |
| 21 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Konami/VRC2_4.h` | 37-48, 223-233 | mapper 21 variant detection 与 submapper 0 OR heuristics cross-check |
| 12 | `mmc3.rs:19-21,134-139,646-651,710-718,1110-1152,1258-1265,1330-1350; mapper.rs:379-382,821-824,981-984,1138-1141` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 377-413 | Mapper 12 MMC3 clone：expansion reg0/reg1 控制 pattern table 两半 CHR bit8，reg2 语言读回，reset toggle 并重置 MMC3 寄存器 |
| 12 | `mmc3.rs:19-21,134-139,646-651,710-718,1110-1152,1258-1265,1330-1350; mapper.rs:379-382,821-824,981-984,1138-1141` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 367-407 | FCEUmm mapper 12 cross-check；submapper 1 转 FFE_Init 记录为后续精修，不混入标准 mapper 12 |
| 16/153/159 | `basic/bandai.rs:1-515; factory.rs:32,150,154; tests.rs:945-1059` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Bandai/BandaiFcg.h` | 12-92,127-239 | Mapper 16/153/157/159 共用 Bandai FCG/LZ93D50：PRG/CHR/mirroring、CPU-cycle IRQ、submapper 4/5 register window 与 IRQ counter/reload 差异、Mapper153 PRG outer/SRAM gate、Mapper159 24C01、低区 read/open-bus 语义 |
| 16/159 | `basic/bandai.rs:14-276,400-415,463-478` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Bandai/BaseEeprom24C0X.h` | 8-69 | 24C0X mode/state、SCL/SDA line helper 和 output bit 语义 |
| 16 | `basic/bandai.rs:25-205,400-415,463-478` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Bandai/Eeprom24C02.h` | 9-144 | 24C02 START/STOP、chip-address compare、ACK、read/write byte 和 address auto-increment |
| 159 | `basic/bandai.rs:25-119,207-276,400-415,463-478` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Bandai/Eeprom24C01.h` | 9-127 | 24C01 LSB-first address/data/read、128-byte wrap、ACK/idle transitions |
| 16/153/159 | `basic/bandai.rs:317-515` | `/Users/sunmeng/workspace/fc/fceux/src/boards/bandai.cpp` | 244-310,316-386 | FCEUX Bandai cross-check；mapper16/159 init、mapper153 SRAM power/register window、register decode、mirroring、IRQ hook、read bit4/open-bus |
| 16/153/159 | `basic/bandai.rs:317-515` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/bandai.c` | 257-409 | FCEUmm Bandai cross-check；同源 mapper16/153/159 register/IRQ/read/init 行为 |
| 22 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/21_22_23_25.c` | 42-48 | VRC2a 地址线和 CHR bank 右移 |
| 22 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Konami/VRC2_4.h` | 51, 121-130, 251-256 | VRC2a 变体识别、CHR 低位忽略、地址翻译 |
| 23 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/21_22_23_25.c` | 51-58 | VRC4f/VRC4e/VRC2b submapper 地址线映射 |
| 23 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Konami/VRC2_4.h` | 53-60, 235-245 | mapper 23 VRC2b/VRC4e variant detection 与 OR heuristics |
| 25 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/21_22_23_25.c` | 61-67 | VRC4b/VRC4d/VRC2c submapper 地址线映射 |
| 25 | `vrc4.rs:1-329` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Konami/VRC2_4.h` | 63-71, 210-221 | mapper 25 variant detection 与 submapper 0 OR heuristics |
| 8 | `latch/discrete.rs:152-188; mapper.rs:228-240,253-268,369-392,571-589,1336-1346` | `/Users/sunmeng/workspace/fc/fceux/src/boards/datalatch.cpp` | 207-216 | Mapper 8 FFE/FJ-007：单 latch 控制低 16KB PRG bank 与 8KB CHR bank，高 16KB 固定 bank 1，固定垂直 mirroring |
| 28 | `multicart.rs:73-194; mapper.rs:228-240,253-269,370-394,572-590,1354-1382` | `/Users/sunmeng/workspace/fc/fceux/src/boards/28.cpp` | 20-166 | Action 53：`reg/chr/prg/mode/outer` 状态、`$5000-$5FFF` register select、PRG16 mode matrix、CHR8、mirroring 和 reset defaults |
| 28 | `multicart.rs:73-194; mapper.rs:228-240,253-269,370-394,572-590,1354-1382` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/28.c` | 21-187 | FCEUmm mapper 28 cross-check；同源 Action 53 PRG mode/mirroring/reset 逻辑 |
| 29 | `latch/discrete.rs:190-232; cartridge.rs:679-683,777-790; mapper.rs:228-240,253-270,371-396,580-594,846-848,1010-1012,1171-1173,1390-1402` | `/Users/sunmeng/workspace/fc/fceux/src/boards/datalatch.cpp` | 246-256 | Mapper 29 / Sealie Computing：latch bit2-4 选择低 16KB PRG，高 16KB 固定末 bank，latch bit0-1 选择 CHR8，普通 WRAM 在 `$6000-$7FFF` |
| 29 | `latch/discrete.rs:190-232; cartridge.rs:679-683,777-790; mapper.rs:228-240,253-270,371-396,580-594,846-848,1010-1012,1171-1173,1390-1402` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/datalatch.c` | 181-194 | FCEUmm mapper 29 cross-check；同样的 PRG/CHR latch，记录其 `$6000-$FFFF` 写窗口差异为后续精修项 |
| 29 | `latch/discrete.rs:190-232; cartridge.rs:679-683,777-790; mapper.rs:228-240,253-270,371-396,580-594,846-848,1010-1012,1171-1173,1390-1402` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Homebrew/SealieComputing.h` | 5-31 | PRG page 16KB、CHR page 8KB、8KB WRAM、32KB CHR-RAM、`$8000-$FFFF` register window 与 fixed high PRG page cross-check |
| 31 | `latch/discrete.rs:190-228; mapper.rs:228-240,253-268,369-392,571-589,1348-1359` | `/Users/sunmeng/workspace/fc/fceux/src/boards/inlnsf.cpp` | 23-61 | Mapper 31 NSF/INL：`$5000-$5FFF` 八个 4KB PRG-ROM window register，末 slot power-on 为 `0xFF` |
| 31 | `latch/discrete.rs:190-228; mapper.rs:228-240,253-268,369-392,571-589,1348-1359` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/31.c` | 26-65 | FCEUmm mapper 31 cross-check；同样使用 `$5000 + (addr & 7)` 选择 4KB PRG slot |
| 35 | `jy.rs:1-128; mapper.rs:235,276,395,588,831,989,1144,2246-2287` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/JyCompany/Mapper35.h` | 5-65 | Mapper 35 JY single-cart：PRG8/CHR1 register、`$C002/$C003/$C005` A12 IRQ、`$D001` mirroring |
| 35 | `jy.rs:1-128; mapper.rs:235,276,395,588,831,989,1144,2246-2287` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/jyasic.c` | 455-488 | FCEUmm JYASIC mapper 35 cross-check；记录 mapper 35 默认 8KB WRAM、single-cart sync 和 extended mirroring |
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
| 48 | `taito.rs:1-142,379-412` | `/Users/sunmeng/workspace/fc/fceux/src/boards/33.cpp` | 23-116 | Taito TC0190 共享 PRG/CHR bank 寄存器、Mapper 48 `$C000-$FFFF` IRQ/mirroring 写窗口和 HBlank IRQ hook |
| 48 | `taito.rs:1-142,379-412` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/33.c` | 23-116 | FCEUmm mapper 48 cross-check；同样记录 `A & 0xF003` 写译码、IRQ latch/counter/enable 和 `$E000` mirroring |
| 48 | `taito.rs:1-142,379-412` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp` | 529 | Nestopia 数据库/board 类型确认 `TAITO_TC0190FMC_PAL16R4` 对应 mapper 48 |
| 49 | `mmc3.rs:15-26,165-178,382-674,1095-1110` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 604-647 | Mapper 49 outer latch、PRG32/MMC3 mode、CHR bit extension、reset/power defaults |
| 49 | `mmc3.rs:15-26,165-178,382-674,1095-1110` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 647-705 | Mapper 49 submapper 1 `$41` default 与 partial write preserve 行为 |
| 51 | `multicart.rs:182-254; mapper.rs:228-240,253-286,371-423,580-612,873-876,1038-1041,1195-1198,1410-1439` | `/Users/sunmeng/workspace/fc/fceux/src/boards/51.cpp` | 23-83 | Mapper 51：bank/mode 两寄存器、`$6000-$7FFF` 低区 PRG-ROM window、`$6000-$7FFF` mode 写、`$8000-$FFFF` bank 写、reset/power defaults |
| 51 | `multicart.rs:182-254; mapper.rs:228-240,253-286,371-423,580-612,873-876,1038-1041,1195-1198,1410-1439` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/51.c` | 23-83 | FCEUmm mapper 51 cross-check；同源 PRG8/PRG16/mirroring/register window 行为 |
| 52 | `mmc3.rs:15-21,128-136,240-274,400-414,598-616` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 652-693 | Mapper 52 one-shot low latch、outer PRG/CHR mask |
| 52 | `mmc3.rs:15-21,128-136,240-274,400-414,598-616` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 710-769 | Mapper 52 submapper 14/CHR-RAM alternate path 记录；当前只实现基础路径 |
| 53 | `multicart.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/supervision.cpp` | 31-70,73-86 | BMC SuperVision 16-in-1：`cmd0/cmd1` 状态、`$6000-$7FFF` 低区写锁存、低区 PRG8 ROM window、高区 32KB/16KB 模式、mirroring 与 reset |
| 53 | `multicart.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/supervision.c` | 31-59,62-75 | FCEUmm mapper 53 cross-check；同源 iNES SuperVision16in1 PRG/CHR/mirroring 逻辑 |
| 53 | `multicart.rs` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardBmcSuperVision16in1.cpp` | 41-60,96-128 | Nestopia BMC SuperVision16in1 cross-check；EPROM-first CRC 分支记录为后续精修，当前第一版按 FCEUX/FCEUmm iNES 53 路径 |
| 56/142 | `basic/irq.rs` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Kaiser/Kaiser202.h` | 17-30,43-55,57-115 | Kaiser202/KS202 与 KS7032 共享模型：8KB PRG、1KB CHR、CPU-cycle IRQ、selected register、Mapper56 `$F000/$F800/$FC00` PRG high-bit/mirroring/CHR writes 与 low PRG-ROM/WRAM 切换 |
| 56/142 | `basic/irq.rs` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardKaiser.cpp` | 42-70,423-510,512-523 | Nestopia Kaiser KS202 cross-check；IRQ latch/control、`ctrl` selected register、`$F000` 地址子译码、mirroring 和 CHR1K bank 行为 |
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
| 165 | `mmc3.rs:15,298-306,457-491,591-592,1078-1085,1412-1426,1457-1459,1799-1853; factory.rs:145; tests.rs:107,283,464` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 918-973 | Mapper 165：MMC3 基础、`M165CW` 4KB CHR-ROM/CHR-RAM page 0、FD/FE latch、PPU hook、4KB CHR-RAM |
| 165 | `mmc3.rs:15,298-306,457-491,591-592,1078-1085,1412-1426,1457-1459,1799-1853; factory.rs:145; tests.rs:107,283,464` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 969-1024 | FCEUmm mapper 165 cross-check；同样使用 DRegBuf 0/1 与 2/4、`V == 0` 映射 4KB CHR-RAM |
| 165 | `mmc3.rs:15,298-306,457-491,591-592,1078-1085,1412-1426,1457-1459,1799-1853; factory.rs:145; tests.rs:107,283,464` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_165.h` | 5-54 | Mesen2 mapper 165：两个 4KB latch、CHR page size/CHR-RAM size、VRAM address hook 与 latch 地址掩码 cross-check |
| 165 | `mmc3.rs:15,298-306,457-491,591-592,1078-1085,1412-1426,1457-1459,1799-1853; factory.rs:145; tests.rs:107,283,464` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp` | 589-590 | Nestopia WAIXING_SH2 mapper 165 metadata：512KB PRG、256KB CHR、可选 WRAM、4KB CHR-RAM |
| 165 | `mmc3.rs:15,298-306,457-491,591-592,1078-1085,1412-1426,1457-1459,1799-1853; factory.rs:145; tests.rs:107,283,464` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.cpp` | 2799-2810 | Nestopia mapper 165 board name/WRAM variant selection cross-check |
| 168 | `special.rs:273-382; mapper.rs:270,361; factory.rs:156; dispatch.rs:74; tests.rs:121,310,505,1065-1089` | `/Users/sunmeng/workspace/fc/fceux/src/boards/168.cpp` | 33-42,48-56,68-77 | Mapper 168 / Racermate Challenge II：64KB CHR-RAM、低 4KB CHR 固定 bank 0、高 4KB CHR 由 `reg & 0x0F` 选择、低 16KB PRG 由 `reg >> 6` 选择、高 16KB 固定末 bank；FCEUX 仅 `$B000` 写入 reg |
| 168 | `special.rs:273-382; mapper.rs:270,361; factory.rs:156; dispatch.rs:74; tests.rs:121,310,505,1065-1089` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/168.c` | 36-45,51-59,71-80 | FCEUmm mapper 168 cross-check；同源 PRG16/CHR4/64KB CHR-RAM，同样记录 `$B000` 写窗口 |
| 168 | `special.rs:273-382; mapper.rs:270,361; factory.rs:156; dispatch.rs:74; tests.rs:121,310,505,1065-1089` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Racermate.h` | 11-17,32-55 | Mesen2 Racermate cross-check；补充 CPU clock hook、1024 周期 IRQ、`$C000-$FFFF` IRQ reload/ack，以及更宽的 `$8000-$BFFF` bank register 窗口 |
| 64 | `rambo1.rs:1-309` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardTengenRambo1.cpp` | 75-96, 190-225, 233-344 | RAMBO-1 register map、IRQ unit、PRG/CHR update、write handlers |
| 64 | `rambo1.rs:1-309` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardTengenRambo1.hpp` | 82-106 | CPU M2 divisor、A12 filter、IRQ delay/source constants |
| 158 | `rambo1.rs:17-20,70-73,129-183,209-273,371-408` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/tengen.c` | 21-23,101-123,126-174,198-220 | Mapper 158 复用 RAMBO-1，CHR write wrapper 将 bank bit7 缓存为 per-nametable page，并忽略普通 mirroring 写 |
| 158 | `rambo1.rs:17-20,70-73,129-183,209-273,371-408` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Tengen/Rambo1_158.h` | 5-37 | Mapper 158 写 `$8001` 时按当前 register 和 bit7 更新 nametable page，且不转发 `$A000` mirroring 写 |
| 158 | `rambo1.rs:17-20,70-73,129-183,209-273,371-408` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Tengen/Rambo1.h` | 96-167,170-177 | RAMBO-1 PRG/CHR update、register decode、IRQ source 与 A12 watcher 仍由基础 RAMBO-1 行为提供 |
| 158 | `rambo1.rs:17-20,70-73,129-183,209-273,371-408` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardTengen.cpp` | 49-55,69-80 | T800037 忽略 `$A000` mirroring poke，并由 CHR regs bit7 重建 nametable banks；Nestopia bit 取反差异记录为后续 ROM 证据项 |
| 158 | `rambo1.rs:17-20,70-73,129-183,209-273,371-408` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardTengen.hpp` | 65-75 | T800037 board 类继承 RAMBO-1 并覆盖 `UpdateChr()` |
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
| 81 | `latch/discrete.rs:153-198; mapper.rs:228-242,300,450,627,903,1070,1234,1454-1466` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/81.c` | 24-31 | Mapper 81 / NTDEC N715062：写入地址 latch 的 bit2-3 选择低 16KB PRG bank，高 16KB 固定末 bank，写入数据 bit0-1 选择 CHR8，固定垂直 mirroring |
| 82 | `taito.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/82.cpp` | 37-63, 66-96 | Taito X1-017 PRG/CHR swap/mirroring |
| 82 | `taito.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/82_552.c` | 38-71, 92-107 | mapper 82 与 552 PRG bit 译码差异 cross-check；当前只落地 mapper 82 |
| 90/209/211 | `basic/jy.rs:52-492; basic.rs:19; mapper.rs:271-352; dispatch.rs:54; factory.rs:99,220,222; tests.rs:91,252,445,2238-2301` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/jyasic.c` | 79-189, 191-253, 255-380, 393-512 | JY ASIC 共用实现：PRG/CHR/nametable sync、ALU/DIP read、register decode、CPU/HBlank/PPU IRQ source、mapper 90/209/211 init 差异 |
| 90/209/211 | `basic/jy.rs:52-492; basic.rs:19; mapper.rs:271-352; dispatch.rs:54; factory.rs:99,220,222; tests.rs:91,252,445,2238-2301` | `/Users/sunmeng/workspace/fc/fceux/src/boards/90.cpp` | 25-27, 74-183, 185-242, 244-379, 381-418, 438-507 | Mapper 90/209/211 差异说明、PRG/CHR/NT/ALU register 行为、IRQ clocking、209 CHR latch、211 forced nametable control |
| 90/209/211 | `basic/jy.rs:52-492; basic.rs:19; mapper.rs:271-352; dispatch.rs:54; factory.rs:99,220,222; tests.rs:91,252,445,2238-2301` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/JyCompany/JyCompany.h` | 11-54, 64-108, 167-271, 274-367, 369-452 | JY ASIC 状态字段、PRG bit reverse、CHR block/mirror、advanced nametable control、ALU/read/write、CPU clock/PPU hook IRQ counter |
| 209 | `basic/jy.rs:52-492; factory.rs:220` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.cpp` | 3081-3085 | Nestopia mapper 209 归类为 `JYCOMPANY_TYPE_B` cross-check |
| 91 | `jy.rs:1-133` | `/Users/sunmeng/workspace/fc/fceux/src/boards/91.cpp` | 35-48, 51-57, 67-74, 80-83 | JY PRG/CHR low-register decode、HBlank IRQ hook |
| 91 | `jy.rs:1-133` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/91.c` | 48-61, 63-87, 97-104, 110-118 | submapper 1 outer bank/mirroring latch 与 HBlank IRQ |
| 91 | `jy.rs:1-133` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/JyCompany/Mapper91.h` | 5-18, 25-42 | Mapper 91 PRG/CHR register decode cross-check；Mesen2 用 MMC3 IRQ 复用 |
| 92 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/72.cpp` | 35-80 | Jaleco 2-in-1 mapper 92 PRG fixed-low/CHR latch 写位 |
| 92 | `latch/discrete.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/addrlatch.c` | 247-263 | address-latch 形式 cross-check；保留后续按地址高位差异精修入口 |
| 95 | `namco.rs:34-203` | `/Users/sunmeng/workspace/fc/fceux/src/boards/80.cpp` | 106-122, 124-134, 153-184 | Mapper 95 写寄存器、CHR bit5 到 mirroring cache、PPU hook 模型 cross-check |
| 95 | `namco.rs:34-203` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/80.c` | 106-122, 124-134, 153-184 | FCEUmm Mapper 95 与 FCEUX 同源行为 cross-check |
| 95 | `namco.rs:34-203` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Namco/Namco108.h` | 5-23 | Namco108 固定 PRG/CHR mode 与 hardwired mirroring 行为 |
| 95 | `namco.rs:34-203` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Namco/Namco108_95.h` | 5-18 | Mapper 95 reg0/reg1 bit5 到四个 nametable 页映射；当前实现采用此模型 |
| 96 | `latch/discrete.rs:259-316; mapper.rs:235-237,297,445,612,881,1040,1201,2200-2221` | `/Users/sunmeng/workspace/fc/fceux/src/boards/96.cpp` | 25-68 | Oeka Kids board：reg/ppulatch 状态、PRG32=`reg&3`、CHR4 low=`reg&4|ppulatch`、CHR4 high=`reg&4|3`、固定单屏 0 mirroring、PPU nametable hook |
| 96 | `latch/discrete.rs:259-316; mapper.rs:235-237,297,445,612,881,1040,1201,2200-2221` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/96.c` | 32-75 | FCEUmm mapper 96 cross-check；同样记录 Oeka Kids PPU hook 和 attribute-table 精度 TODO 注释 |
| 99 | `latch/discrete.rs:238-292; mapper.rs:176-180,282,351; bus.rs:532-535,671-693; cartridge.rs:407-409; factory.rs:114; dispatch.rs:58; tests.rs:101,291,484,2011-2028` | `/Users/sunmeng/workspace/fc/fceux/src/boards/99.cpp` | 34-44,47-55,68-78 | Mapper 99 / VS UniSystem：`$4016` 写 latch 选择 CHR8 bank `(value >> 2) & 1`，并为 VS Gumshoe 选择 `$8000-$9FFF` PRG8 bank `value & 4`，写后继续调用原 controller strobe handler |
| 99 | `latch/discrete.rs:238-292; mapper.rs:176-180,282,351; bus.rs:532-535,671-693; cartridge.rs:407-409; factory.rs:114; dispatch.rs:58; tests.rs:101,291,484,2011-2028` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/99.c` | 34-44,47-55,67-78 | FCEUmm mapper 99 cross-check；同源 PRG/CHR latch、8KB WRAM、`$4016` handler wrapping |
| 99 | `latch/discrete.rs:238-292; mapper.rs:176-180,282,351; bus.rs:532-535,671-693; cartridge.rs:407-409; factory.rs:114; dispatch.rs:58; tests.rs:101,291,484,2011-2028` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardVsSystem.cpp` | 38-61 | Nestopia VS System cross-check；`Poke_4016` 更新 CHR8/PRG8 后转发原 `$4016` poke，peek 也代理原控制器路径 |
| 99 | `latch/discrete.rs:238-292; mapper.rs:176-180,282,351; bus.rs:532-535,671-693; cartridge.rs:407-409; factory.rs:114; dispatch.rs:58; tests.rs:101,291,484,2011-2028` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/VsSystem/VsSystem.h` | 16-19,82-97 | Mesen2 VS System cross-check；PRG/CHR page size、Work RAM size、控制管理器 bit 变化后更新 CHR8 与 VS Gumshoe PRG8；当前本项目先以 `$4016` 写通知表达同一 latch |
| 108 | `special.rs:113-165,187-208` | `/Users/sunmeng/workspace/fc/fceux/src/boards/108.cpp` | 31-48,54-58 | Mapper 108 低区 `setprg8(0x6000, reg)`、高区 `setprg32(0x8000, ~0)`、固定 `setchr8(0)` 与写窗口 |
| 108 | `special.rs:113-165,187-208` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/108.c` | 37-53,60-64 | FCEUmm Mapper 108 cross-check；同样覆盖 `$8000-$8FFF` 与 `$F000-$FFFF` 写处理 |
| 111 | `special.rs:218-342,608-647; cartridge.rs:708-713,823-838; mapper.rs:275,359; dispatch.rs:64; factory.rs:122; tests.rs:95,302,512` | `/Users/sunmeng/workspace/fc/fceux/src/boards/cheapocabra.cpp` | 35-85,210-244 | Cheapocabra/GTROM：寄存器 bit0-3 选择 PRG32、bit4 选择 8KB pattern CHR-RAM、bit5 选择 8KB nametable page，`$5000-$5FFF` 与 `$7000-$7FFF` 写窗口，iNES CHR-RAM 32KB |
| 111 | `special.rs:218-342,608-647; cartridge.rs:708-713,823-838; mapper.rs:275,359; dispatch.rs:64; factory.rs:122; tests.rs:95,302,512` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/cheapocabra.c` | 35-80,204-243 | FCEUmm mapper 111 cross-check；同样记录 32KB CHR-RAM、power-on `reg=0xFF`、普通写窗口与 battery-backed flash 分支 |
| 111 | `special.rs:218-342,608-647; cartridge.rs:708-713,823-838; mapper.rs:275,359; dispatch.rs:64; factory.rs:122; tests.rs:95,302,512` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Homebrew/Cheapocabra.h` | 17-35,68-99 | Mesen2 Cheapocabra：PRG32/CHR8/nametable page 选择、低区读用 open bus 更新 register、`$8000-$FFFF` flash command 路径；Mesen2 flash save 与 `FlashSST39SF040.h` 留作后续精修 |
| 206 | `namco.rs:86-154,275-301` | `/Users/sunmeng/workspace/fc/fceux/src/boards/206.cpp` | 33-78 | Mapper 206 Namco108 subset：2KB/1KB CHR、8KB PRG、cmd/data 写译码、power defaults |
| 206 | `namco.rs:86-154,275-301` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/206_486.c` | 33-78 | FCEUmm Mapper 206 cross-check；同文件 80-98 记录相关 mapper 486 直写变体 |
| 206 | `namco.rs:86-154,275-301` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Namco/Namco108.h` | 6-28 | Namco108 写地址 mask、固定最后两个 PRG bank、hardwired mirroring 行为 |
| 206 | `namco.rs:86-154,275-301` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 479 | mapper 206 归类到 Namco108 |
| 106 | `unlicensed.rs:339-431` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Mapper106.h` | 14-16, 18-26, 36-73 | PRG/CHR register decode、CPU-cycle IRQ |
| 106 | `unlicensed.rs:339-431` | `/Users/sunmeng/workspace/fc/fceux/src/boards/106.cpp` | 36-59, 81-87 | FCEUX PRG/CHR sync 和 IRQ overflow cross-check |
| 104 | `special.rs:113-162; mapper.rs:228-242,307,475,637,915,1082,1251,1468-1489` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/104.c` | 35-69,77-91 | Mapper 104 / Pegasus 5-in-1：`preg[0]/preg[1]` 双 16KB PRG register、`$8000-$9FFF` outer bank 写、`$C000-$FFFF` inner bank 写、固定 CHR8、固定垂直 mirroring、8KB WRAM |
| 105 | `mmc1.rs:23-318; factory.rs:111-116; tests.rs:86-88,258-260,432-434` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Nintendo/MMC1_105.h` | 6-70 | Mapper 105 / NWC：`chrReg0` bit4 初始化与 IRQ enable/reset、CPU-clock timer、初始化完成前固定 PRG32 bank 0、bit3 选择 MMC1/32KB PRG 模式、PRG bank bit3 强制 |
| 105 | `mmc1.rs:23-318; factory.rs:111-116; tests.rs:86-88,258-260,432-434` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc1.cpp` | 230-258 | NWC CHR/PRG hook cross-check：`NWCCHRHook` bit4 清 IRQ/计数、bit3 选择 MMC1 PRG 或 32KB PRG、`NWCPRGHook` 强制 PRG bit3 |
| 105 | `mmc1.rs:23-318; factory.rs:111-116; tests.rs:86-88,258-260,432-434` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc1.c` | 252-284 | FCEUmm NWC hook 与 reset/power cross-check；记录 DIP 扩展计时值为后续输入/比赛模式精修项 |
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
| 14 | `mmc3.rs:22-27,214-230,769-784,833-839,1056-1064,1467-1519,1589-1591,1976-1985,2000-2015,2248-2280; factory.rs:31; tests.rs:21,237,450` | `/Users/sunmeng/workspace/fc/fceux/src/boards/sl1632.cpp` | 22-104 | Mapper 14 / Rex Soft SL-1632：`$A131` mode latch、direct 8KB PRG、direct 1KB CHR nibble registers、mirroring latch、切入 MMC3 后复用 MMC3 PRG/CHR/IRQ |
| 14 | `mmc3.rs:22-27,214-230,769-784,833-839,1056-1064,1467-1519,1589-1591,1976-1985,2000-2015,2248-2280; factory.rs:31; tests.rs:21,237,450` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardRexSoftSl1632.cpp` | 38-196 | Nestopia SL-1632 cross-check；确认 direct mode 和 MMC3 mode 切换、CHR high bit shift table、mirroring gate |
| 14 | `mmc3.rs:22-27,214-230,769-784,833-839,1056-1064,1467-1519,1589-1591,1976-1985,2000-2015,2248-2280; factory.rs:31; tests.rs:21,237,450` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/14.c` | 21-84,94-101 | FCEUmm 新实现 cross-check；其 VRC2/MMC3 双 ASIC 激活路径需要后续复合 mapper 架构精修，本轮第一版按 FCEUX/Nestopia 的 MMC3-compatible SL-1632 path 落地 |
| 126 | `mmc3.rs:32,310-320,691-733,802,943-995,1119-1136,1314-1354,1381-1390,1549-1552,1611,1859-1864,2069-2100; factory.rs:135; tests.rs:105,320,541` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/126_422_534.c` | 21-36,46-189,191-204 | Mapper 126 / PowerJoy 84-in-1：TEC9719 MMC3 clone、outer PRG bit 反相、PRG mode 0/1/2/3、CHR A18/A19 交换、CNROM mode、extended mirroring、`$6000-$7FFF` extra regs、SL0 reset DIP 和 low WRAM fall-through |
| 126 | `mmc3.rs:32,310-320,691-733,802,943-995,1119-1136,1314-1354,1381-1390,1549-1552,1611,1859-1864,2069-2100; factory.rs:135; tests.rs:105,320,541` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_126.h` | 5-88 | Mesen2 MMC3_126 cross-check；记录 PRG/CHR outer bank、CNROM CHR update、低区 register write gate |
| 126 | `mmc3.rs:32,310-320,691-733,802,943-995,1119-1136,1314-1354,1381-1390,1549-1552,1611,1859-1864,2069-2100; factory.rs:135; tests.rs:105,320,541` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp` | 298 | Nestopia BMC_POWERJOY_84IN1 metadata cross-check |
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
| 134 | `mmc3.rs:19-41,236-244,629-642,751-760,1108-1119,1171-1178,1213-1219,1435-1439,1618-1675; factory.rs:130; tests.rs:99,274` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/134.c` | 21-23,30-71,73-95 | Mapper 134 WX-KB4K/T4A54A/BS-5652：PRG/CHR outer mask/OR、NROM-128/256、CNROM 模式、DIP 读、低区写锁定与 WRAM fall-through |
| 134 | `mmc3.rs:19-41,236-244,629-642,751-760,1108-1119,1171-1178,1213-1219,1435-1439,1618-1675; factory.rs:130; tests.rs:99,274` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 862-915 | FCEUX mapper 134 cross-check；记录较旧 GenMMC3 512KB mask 公式、`EXPREGS[0]` lock 和 `$6002.0-1` 例外 |
| 134 | `mmc3.rs:19-41,236-244,629-642,751-760,1108-1119,1171-1178,1213-1219,1435-1439,1618-1675; factory.rs:130; tests.rs:99,274` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_134.h` | 6-52 | Mesen2 mapper 134 最小 exReg 模型 cross-check；确认 `0x6001` 写入可扩展 PRG bit5/CHR bit8，当前实现采用 FCEUmm 完整 multicart 寄存器集 |
| 142 | `basic/irq.rs:745-860; factory.rs:129; mapper.rs:234-239,318; dispatch.rs:68; tests.rs:97,273,460,683-740` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/KS7032.c` | 56-63,78-129,131-158 | KS7032 PRG8/低区 PRG-ROM/固定 CHR8、IRQ reload/control/ack、CPU-cycle IRQ，以及 `$E000/$F000` selected-register 写协议；当前按此版本保持 IRQ 命中后 reload 并继续使能 |
| 142 | `basic/irq.rs:745-860; factory.rs:129; mapper.rs:234-239,318; dispatch.rs:68; tests.rs:97,273,460,683-740` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Kaiser/Kaiser202.h` | 17-30,43-55,57-115 | Kaiser202 mapper 142/56 shared model cross-check；确认 PRG page size 8KB、CPU clock hook、nibble reload、selected register 和可选 low PRG-ROM/WRAM 模型 |
| 142 | `basic/irq.rs:745-860; factory.rs:129; mapper.rs:234-239,318; dispatch.rs:68; tests.rs:97,273,460,683-740` | `/Users/sunmeng/workspace/fc/fceux/src/boards/ks7032.cpp` | 35-67,72-87 | 旧 KS7032 实现 cross-check；记录早期版本 IRQ 命中后关闭使能、PRG low window register 编号差异，作为后续具体 ROM 精修参考 |
| 149 | `latch/sachen.rs:64-135` | `/Users/sunmeng/workspace/fc/fceux/src/boards/sachen.cpp` | 306-310 | SA0036 复用 SA72007 CHR=`value>>7` |
| 149 | `latch/sachen.rs:64-135` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/sachen.c` | 213-215 | FCEUmm SA0036 cross-check |
| 149 | `latch/sachen.rs:64-135` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Sachen/Sachen_149.h` | 10-19 | mapper 149 固定 PRG、CHR bit7 latch |
| 137,141 | `latch/sachen.rs:175-330,551-607` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Sachen/Sachen8259.h` | 4-114 | Sachen 8259 A/D 变体：`$4100/$4101` register/data 写协议、PRG32、A=2KB CHR、D=1KB CHR + 高 4KB 固定末页、simple mode 与 mirroring |
| 137,141 | `latch/sachen.rs:175-330,551-607` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/sachen.c` | 31-131 | FCEUmm S8259A/S8259D sync cross-check；记录 type 0/3 的 CHR bank OR/shift、mapper137 镜像反转和 `setmirrorw(0,1,1,1)` |
| 137,141 | `latch/sachen.rs:175-330,551-607` | `/Users/sunmeng/workspace/fc/fceux/src/boards/sachen.cpp` | 134-222 | FCEUX S8259 cross-check；写窗口 `$4100-$7FFF`，`A &= 0x4101`，A/D init 分别对应 mapper141/137 |
| 137,141 | `latch/sachen.rs:175-330,551-607` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardSachenS8259.cpp` | 40-152 | Nestopia S8259 bank swap 与 mirroring LUT cross-check；`SetNametables(0,1,1,1)` 当前暂用 `Mirroring::FourScreen` 近似，后续扩 per-nametable page hook 可精修 |
| 154 | `namco.rs:94-104,220-243,339-355` | `/Users/sunmeng/workspace/fc/fceux/src/boards/88.cpp` | 34-55,78-82 | Mapper 154 复用 Namco108/mapper88 bank layout，并在 command write bit6 上选择单屏 mirroring |
| 154 | `namco.rs:94-104,220-243,339-355` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/88.c` | 34-55,79-84 | FCEUmm Mapper 154 cross-check；同样复用 mapper 88 sync 并启用 `is154` mirroring |
| 154 | `namco.rs:94-104,220-243,339-355` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Namco/Namco108_154.h` | 6-13 | Mapper 154 command write bit6 映射 ScreenA/ScreenB 单屏 mirroring |
| 155 | `mmc1.rs:10-43` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc1.cpp` | 40-48,332-335 | Mapper 155 与 MMC1 相同，但 WRAM disable bit 被忽略，WRAM 始终可用 |
| 155 | `mmc1.rs:10-43` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc1.c` | 55-67,362-365 | FCEUmm Mapper 155 cross-check；`is155` 绕过 WRAM disable 读写门控 |
| 155 | `mmc1.rs:10-43` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Nintendo/MMC1_155.h` | 4-13 | Mapper 155 在 `UpdateState()` 中强制 `_wramDisable = false` |
| 156 | `opencorp.rs:1-121` | `/Users/sunmeng/workspace/fc/fceux/src/boards/156.cpp` | 36-89,102-113 | OpenCorp/Daou306 PRG16、固定末尾 PRG16、8 个 CHR1 low/high register、mirroring register、WRAM 低区 |
| 156 | `opencorp.rs:1-121` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/156.c` | 42-95,108-119 | FCEUmm Mapper 156 cross-check；同样记录 DIS23C01 DAOU ROM controller 行为 |
| 156 | `opencorp.rs:1-121` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardOpenCorp.cpp` | 42-75,78-114 | Nestopia Daou306 CHR1 register layout、mirroring fallback 与 `$C010` PRG swap |
| 166 | `subor.rs:1-124` | `/Users/sunmeng/workspace/fc/fceux/src/boards/subor.cpp` | 31-69,76-87 | Subor mapper 166 PRG bank formula、mode bits 与 reset defaults |
| 166 | `subor.rs:1-124` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/subor.c` | 31-70,77-88 | FCEUmm Subor cross-check，并采用其 `regs[0].bit0` mirroring 行为 |
| 166 | `subor.rs:1-124` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Subor166.h` | 27-54 | Mesen2 Subor166 outer/inner PRG bank XOR、mode select 与 mapper 167 alternate mode |
| 166 | `subor.rs:1-124` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardSubor.cpp` | 83-124 | Nestopia Type1/Type0 mode formula cross-check；`NstBoard.hpp:507-508` 记录 166/167 type 映射 |
| 167 | `subor.rs:1-124` | `/Users/sunmeng/workspace/fc/fceux/src/boards/subor.cpp` | 31-69,83-87 | Subor mapper 167 NROM-like bank order 和 fixed bank 差异 |
| 167 | `subor.rs:1-124` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/subor.c` | 31-70,84-88 | FCEUmm Subor mapper 167 cross-check |
| 167 | `subor.rs:1-124` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Subor166.h` | 38-53 | mapper ID 167 的 `altMode` PRG order/fixed bank 行为 |
| 167 | `subor.rs:1-124` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardSubor.cpp` | 83-124 | Nestopia Subor Type0/Type1 PRG bank mode cross-check |
| 175 | `special.rs:300-366; mapper.rs:228-242,320-321,510-511,652-653,939-940,1108-1109,1281-1282,2249-2277` | `/Users/sunmeng/workspace/fc/fceux/src/boards/175.cpp` | 32-67,70-78 | Mapper 175：`$8000` mirroring latch、`$A000` bank latch、写后延迟 PRG16/PRG8 提交、读 `$FFFC` 后同步、CHR8 和 mirroring bit |
| 175 | `special.rs:300-366; mapper.rs:228-242,320-321,510-511,652-653,939-940,1108-1109,1281-1282,2249-2277` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/175.c` | 32-67,70-78 | FCEUmm mapper 175 cross-check；同样记录 read side-effect 和延迟 PRG commit |
| 177 | `special.rs:368-417; mapper.rs:228-242,320-321,510-511,652-653,939-940,1108-1109,1281-1282,2249-2277` | `/Users/sunmeng/workspace/fc/fceux/src/boards/177.cpp` | 34-53,62-79 | Mapper 177 / Henggedianzi XH-32A：固定 CHR8、`$6000-$7FFF` WRAM、PRG32=`reg&0x1f`、mirroring bit5、power/reset 默认 |
| 177 | `special.rs:368-417; mapper.rs:228-242,320-321,510-511,652-653,939-940,1108-1109,1281-1282,2249-2277` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/177.c` | 34-53,62-87 | FCEUmm mapper 177 cross-check；新增 reset hook 语义也纳入本项目第一版 |
| 178 | `basic/waixing.rs:5-159,345-411; mapper.rs; factory.rs; tests.rs` | `/Users/sunmeng/workspace/fc/fceux/src/boards/178.cpp` | 85-149,151-171,183-199 | Waixing FS305/NJ0430：`$4800-$4FFF` 四寄存器、NROM/UNROM PRG16/PRG32 模式、CHR8=0、32KB WRAM bank、mirroring bit、`$5800` PCM enable/status 和 sensor IRQ 注释 |
| 178 | `basic/waixing.rs:5-159,345-411; mapper.rs; factory.rs; tests.rs` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/178.c` | 88-110,140-196,204-244 | FCEUmm mapper 178 cross-check；记录 submapper 3 pad/address-bit behavior、soft reset pad increment、ADPCM decode/`$4011` 输出为后续精修项 |
| 178 | `basic/waixing.rs:5-159,345-411; mapper.rs; factory.rs; tests.rs` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Waixing/Waixing178.h` | 4-61 | Mesen2 Waixing178：PRG page size 16KB、CHR page size 8KB、32KB work RAM、`$4800-$4FFF` register window 与 mirroring 语义 |
| 252 | `basic/waixing.rs:162-366,552-628; mapper.rs:182-185,284,409; dispatch.rs:109,251-252; factory.rs:271; ppu.rs:1192-1195,1311-1358; tests.rs:209,433` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Waixing/Waixing252.h` | 5-68 | Mapper 252：8KB PRG page、1KB CHR page、`$8000/$A000/$9000` PRG/mirroring、`$B000-$EFFF` CHR nibble register formula、`$F000-$F00C` VRC4-style IRQ control 和 CPU clock hook |
| 252 | `basic/waixing.rs:162-366,552-628; mapper.rs:182-185,284,409; dispatch.rs:109,251-252; factory.rs:271; ppu.rs:1192-1195,1311-1358; tests.rs:209,433` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/252_253.c` | 20-71 | FCEUmm mapper 252/253 later CHR-RAM mask/compare path：默认 `mask=0xFE/compare=0x06`，PPUDATA 写时按当前 CHR bank `0x88/0xC2/0xC8` 切换 CHR-RAM window |
| 252 | `basic/waixing.rs:162-366,552-628; mapper.rs:182-185,284,409; dispatch.rs:109,251-252; factory.rs:271; ppu.rs:1192-1195,1311-1358; tests.rs:209,433` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.cpp` | 3366-3373 | mapper 252 归类到 Waixing SGZ |
| 252 | `basic/waixing.rs:162-366,552-628; mapper.rs:182-185,284,409; dispatch.rs:109,251-252; factory.rs:271; ppu.rs:1192-1195,1311-1358; tests.rs:209,433` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp` | 594 | WAIXING_SGZ 元数据：mapper 252、512KB PRG、256KB CHR、8KB CHR-RAM |
| 185 | `basic/discrete.rs:9-63,191-207` | `/Users/sunmeng/workspace/fc/fceux/src/boards/185.cpp` | 24-50,59-70,83-93 | CNROM copy-protection：CHR enable 判定、dummy `0xFF` CHR mapping、固定 PRG16 首尾 bank |
| 185 | `basic/discrete.rs:9-63,191-207` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/185.c` | 24-50,60-71,84-94 | FCEUmm mapper 185 cross-check；同样记录 `datareg != 0x13` 例外与 dummy CHR |
| 181 | `basic/discrete.rs:9-82,285-303` | `/Users/sunmeng/workspace/fc/fceux/src/boards/185.cpp` | 52-57,59-70,96-106 | Mapper 181 / Seicross-style CNROM protection：复用 Mapper185 dummy CHR 机制，`datareg bit0=0` 选择 CHR-ROM，bit0=1 选择 dummy `0xFF` CHR page |
| 181 | `basic/discrete.rs:9-82,285-303` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/185.c` | 53-58,60-71,97-108 | FCEUmm mapper 181 cross-check；确认固定首/尾 PRG16 与 `Sync181` gating 条件 |
| 186 | `basic/discrete.rs:217-321,475-504; mapper.rs:278,382; dispatch.rs:87; factory.rs:191; tests.rs:143,349,561` | `/Users/sunmeng/workspace/fc/fceux/src/boards/186.cpp` | 52-120 | Family Study Box：32KB WRAM banked 到 `$6000-$7FFF`、3KB SWRAM `$4400-$4FFF`、`$4200-$43FF` 状态/寄存器窗口、PROM 低 16KB 可切换且高 16KB 固定 bank 0 |
| 186 | `basic/discrete.rs:217-321,475-504; mapper.rs:278,382; dispatch.rs:87; factory.rs:191; tests.rs:143,349,561` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/186.c` | 25-94 | FCEUmm mapper 186 cross-check；同样记录 `regs[0] >> 6` 选择 PRAM bank、`regs[1]` 选择 PROM bank、状态读 `$4202=0x40` 以及未列状态读 `0xFF` |
| 176 | `mmc3.rs:15-84,320-345,883-1033,1136-1138,1242-1244,1744-1821,1860-1861,2012-2029,2182-2200,2861-2898; factory.rs:177; tests.rs:148,367,590` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/fk23c.c` | 20-43,52-94,116-169,171-232,234-260,267-365,367-437,473- | Mapper 176 FK23C：low `$5000-$5FFF` 外层寄存器、PRG mode 0-5、CHR 8K/CNROM/extended MMC3、A12 IRQ、reset 默认；DIP reset heuristic、FS005 extended WRAM/mixed CHR-RAM、submapper 3/4/5 深水区与 mapper 523 留待后续精修 |
| 176 | `mmc3.rs:15-84,320-345,883-1033,1136-1138,1242-1244,1744-1821,1860-1861,2012-2029,2182-2200,2861-2898; factory.rs:177; tests.rs:148,367,590` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Waixing/Fk23C.h` | 7-112,147-183,185-230,232-258,260-359,362-390 | Mesen2 FK23C cross-check：UpdatePrg/UpdateChr、low/high register decode、A12 IRQ watcher、WRAM config flags |
| 176 | `mmc3.rs:15-84,320-345,883-1033,1136-1138,1242-1244,1744-1821,1860-1861,2012-2029,2182-2200,2861-2898; factory.rs:177; tests.rs:148,367,590` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardBmcFk23c.cpp` | 170-242,254-297,299-337 | Nestopia BMC FK23C reset/update/poke cross-check |
| 176 | `mmc3.rs:15-84,320-345,883-1033,1136-1138,1242-1244,1744-1821,1860-1861,2012-2029,2182-2200,2861-2898; factory.rs:177; tests.rs:148,367,590` | `/Users/sunmeng/workspace/fc/fceux/src/boards/fk23c.cpp` | 24-90,92-153,155-205,209-245 | FCEUX older FK23C implementation cross-check |
| 187 | `mmc3.rs:17-54,222-228,520-535,599-605,840-875,940-945,1009-1015,1040-1057,1167-1169,1371-1408` | `/Users/sunmeng/workspace/fc/fceux/src/boards/187.cpp` | 24-83 | Mapper 187 A98402：CHR bit8 扩展、PRG forced 16/32KB modes、`$8000/$8001` 门控、`$5000/$6000` latch 与 security read |
| 187 | `mmc3.rs:17-54,222-228,520-535,599-605,840-875,940-945,1009-1015,1040-1057,1167-1169,1371-1408` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/187.c` | 28-87 | FCEUmm mapper 187 cross-check；同样记录 A98402/A9711/A9746 board 注释和 protection data |
| 187 | `mmc3.rs:17-54,222-228,520-535,599-605,840-875,940-945,1009-1015,1040-1057,1167-1169,1371-1408` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_187.h` | 6-95 | Mesen2 MMC3_187：register range、SelectPrgPage/SelectChrPage override、security read 与 serialize 字段 cross-check |
| 188 | `basic/discrete.rs:69-127,209-224` | `/Users/sunmeng/workspace/fc/fceux/src/boards/karaoke.cpp` | 23-63 | Karaoke Studio expansion cartridge：PRG16 latch、`$6000-$7FFF` 固定设备读值 3、固定 horizontal mirroring |
| 188 | `basic/discrete.rs:69-127,209-224` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/karaoke.c` | 23-63 | FCEUmm mapper 188 cross-check；同样记录 latch 为 0 时的特殊 bank 与 bit4 选择低/高 PRG block |
| 189 | `mmc3.rs:18-29,198-203,439-440,760-762,918-920,1064-1086` | `/Users/sunmeng/workspace/fc/fceux/src/boards/189.cpp` | 24-43 | MMC3 PRG32 outer latch、`V | (V >> 4)` 低区写、power reset |
| 189 | `mmc3.rs:18-29,198-203,439-440,760-762,918-920,1064-1086` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/189.c` | 24-43 | FCEUmm mapper 189 cross-check；同样记录 `$4120-$7FFF` 写窗口与 `setprg32` |
| 190 | `special.rs:219-272,316-345; mapper.rs:229-242,253-367; factory.rs:163; tests.rs:127,304,484` | `/Users/sunmeng/workspace/fc/fceux/src/boards/190.cpp` | 24-85 | Magic Kid GooGoo：8KB WRAM at `$6000-$7FFF`、`$8000/$C000` PRG16 latch、`$A000-$BFFF` 四个 CHR2 latch、固定垂直 mirroring |
| 190 | `special.rs:219-272,316-345; mapper.rs:229-242,253-367; factory.rs:163; tests.rs:127,304,484` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/190.c` | 24-90 | FCEUmm mapper 190 cross-check；power/reset 清零 PRG/CHR latch，CHR register 保存完整 byte |
| 191 | `mmc3.rs:240-244,1109-1125` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 975-988 | Mapper 191 CHR bank bit7 选择 2KB CHR-RAM；当前复用通用 `ChrRamWindow` |
| 191 | `mmc3.rs:240-244,1109-1125` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_ChrRam.h` | 5-38 | `MMC3_ChrRam(0x80, 0xFF, 2)` 参数模型 cross-check |
| 191 | `mmc3.rs:240-244,1109-1125` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/191.c` | 30-39,42-63,66-72 | 新 FCEUmm mapper 191 submapper/PRG low-register 行为记录；当前先落地 FCEUX/Mesen2 CHR-RAM first pass |
| 192 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 990-1008 | Mapper 192 CHR banks 8..B 路由到 4KB CHR-RAM |
| 192 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1026-1045 | FCEUmm Mapper 192 cross-check |
| 192 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_ChrRam.h` | 5-38 | 通用 MMC3 CHR-RAM window 机制 |
| 192 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 465 | mapper 192 使用 `MMC3_ChrRam(0x08, 0x0B, 4)` |
| 193 | `basic/discrete.rs:133-185,226-247` | `/Users/sunmeng/workspace/fc/fceux/src/boards/193.cpp` | 23-49,51-70 | MEGA-SOFT War in the Gulf：低区 4 个 register、固定尾部 PRG8、CHR4/CHR2/CHR2 译码 |
| 193 | `basic/discrete.rs:133-185,226-247` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/193.c` | 26-50,54-73 | FCEUmm mapper 193 cross-check；同样记录 `$6000-$6003` 写窗口与 power sync |
| 195 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1028-1045 | Mapper 195 基础 CHR banks 0..3 路由到 4KB CHR-RAM；FCEUX 还记录 `$5000` 4KB PRG-RAM |
| 195 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/195.c` | 30-97 | 新 FCEUmm Mapper 195 CHR-RAM + PPU write intercept 保护逻辑；当前仅落地 Mesen/FCEUX 基础 CHR-RAM window |
| 195 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_ChrRam.h` | 5-38 | 通用 MMC3 CHR-RAM window 机制 |
| 195 | `mmc3.rs:187-220,842-888` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 468 | mapper 195 使用 `MMC3_ChrRam(0x00, 0x03, 4)` |
| 196 | `mmc3.rs:18-31,262-277,457-466,502-503,678-686,714-718,806-814,943-948,1172-1198` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1053-1093 | Mapper 196 高区地址线重映射、低区 PRG32 latch、普通 MMC3 CHR/IRQ |
| 196 | `mmc3.rs:18-31,262-277,457-466,502-503,678-686,714-718,806-814,943-948,1172-1198` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1065-1104 | FCEUmm mapper 196 cross-check；同样记录 `addr & 0xF000 | (!!(addr&0xE) ^ (addr&1))` 等价重映射 |
| 196 | `mmc3.rs:18-31,262-277,457-466,502-503,678-686,714-718,806-814,943-948,1172-1198` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_196.h` | 10-48 | Mesen2 Mapper 196 PRG32 latch 与 high-register remap cross-check |
| 197 | `mmc3.rs:14-15,306-320,378-428,713-724,896-899,1567-1581` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1130-1144 | Mapper 197 MMC3 clone：board-specific CHR 2KB cwrap，PRG/IRQ 复用 MMC3 |
| 197 | `mmc3.rs:14-15,306-320,378-428,713-724,896-899,1567-1581` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1142-1217 | FCEUmm mapper 197 submapper 0/1/2 CHR cwrap cross-check；submapper 3 的低寄存器 outer PRG/CHR 留作精修 |
| 198 | `mmc3.rs:33,322-327,610-617,1266,1584-1598` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1146-1159 | Mapper 198 大 PRG MMC3 clone：PRG pwrap 高 bank mask 与 `$5000-$5FFF` 低区 WRAM |
| 198 | `mmc3.rs:33,322-327,610-617,1266,1584-1598` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1219-1239 | FCEUmm mapper 198 cross-check；同样记录 CHR-RAM board 路径和 4KB low WRAM |
| 198 | `mmc3.rs:33,322-327,610-617,1266,1584-1598` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_198.h` | 5-63 | Mesen2 MMC3_198 pwrap/read/write behavior cross-check；其注释提示部分 ROM 可能实际应使用其他 mapper |
| 199 | `mmc3.rs:37,376-384,594-644,800-804,944-947,1042-1055,1263-1289,1305-1310,1740-1742,2395-2427; factory.rs:208; tests.rs:162,378,592` | `/Users/sunmeng/workspace/fc/fceux/src/boards/199.cpp` | 32-97 | Mapper 199 / Waixing Type G：fixed PRG `$C000/$E000` extra regs、four fixed low CHR slots、CHR bank `<8` selects 8KB CHR-RAM、2-bit mirroring、extra register write protocol |
| 199 | `mmc3.rs:37,376-384,594-644,800-804,944-947,1042-1055,1263-1289,1305-1310,1740-1742,2395-2427; factory.rs:208; tests.rs:162,378,592` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_199.h` | 6-67 | Mesen2 MMC3_199 主参考：CHR-RAM 1KB page model、extra regs reset defaults、WriteRegister override、mirroring override、SelectPrg/SelectChr override |
| 199 | `mmc3.rs:37,376-384,594-644,800-804,944-947,1042-1055,1263-1289,1305-1310,1740-1742,2395-2427; factory.rs:208; tests.rs:162,378,592` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.cpp` | 2994-3006 | Nestopia WAIXING_TYPE_G install metadata cross-check |
| 199 | `mmc3.rs:37,376-384,594-644,800-804,944-947,1042-1055,1263-1289,1305-1310,1740-1742,2395-2427; factory.rs:208; tests.rs:162,378,592` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp` | 578-586 | Nestopia WAIXING_TYPE_G board type / mapper 199 metadata cross-check |
| 199 | `mmc3.rs:37,376-384,594-644,800-804,944-947,1042-1055,1263-1289,1305-1310,1740-1742,2395-2427; factory.rs:208; tests.rs:162,378,592` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/199.c` | 23-38 | 差异参考：新版 FCEUmm 使用 unbanked CHR-RAM 和 `$5000` 4KB PRG-RAM window，本次未作为主行为 |
| 208 | `mmc3.rs:17-54,304-316,546-553,864-875,946-955,978-984,1016-1019,1058-1068,1177-1179,1410-1435` | `/Users/sunmeng/workspace/fc/fceux/src/boards/208.cpp` | 24-77 | Mapper 208 Gouder 37017：256-byte protection LUT、PRG32 latch、`$4800/$6800/$5000-$5FFF` 写窗口与 protection read |
| 208 | `mmc3.rs:17-54,304-316,546-553,864-875,946-955,978-984,1016-1019,1058-1068,1177-1179,1410-1435` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/208.c` | 30-97 | FCEUmm mapper 208 cross-check；包含 submapper 1 的 PRG/mirroring 变体记录 |
| 208 | `mmc3.rs:17-54,304-316,546-553,864-875,946-955,978-984,1016-1019,1058-1068,1177-1179,1410-1435` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_208.h` | 5-75 | Mesen2 MMC3_208：register ranges、protection LUT、PRG4x update 与 read/write register cross-check |
| 215 | `mmc3.rs:19-45,396-402,789-819,917-928,1143-1178,1275-1276,1558-1568,1718-1721,2134-2165; factory.rs:216; tests.rs:168,375,581` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_215.h` | 6-114 | Mapper 215 / UNL-8237：`$5000/$5001/$5007` exRegs、PRG forced 16KB/32KB modes、CHR high bits、high-register address LUT 和 command register LUT remap |
| 215 | `mmc3.rs:19-45,396-402,789-819,917-928,1143-1178,1275-1276,1558-1568,1718-1721,2134-2165; factory.rs:216; tests.rs:168,375,581` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/Unl8237A.h` | 5-50 | 8237A 子类 cross-check；记录其额外 PRG/CHR high-bit 规则暂未混入 mapper215，后续若遇到对应 UNIF/NES2 board 再单独加变体 |
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
| 221 | `multicart.rs:137-199,412-436,470-492,959-981; mapper.rs:529,932,1091,1254` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Ntdec/Mapper221.h` | 5-55 | Mapper 221 700-in-1/400-in-1：mode latch、PRG latch、UNROM/NROM-256/NROM-128、mirroring |
| 221 | `multicart.rs:137-199,412-436,470-492,959-981; mapper.rs:529,932,1091,1254` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/n625092.c` | 23-75 | FCEUmm mapper 221 cross-check；记录 submapper 1 outer bit、CHR-RAM/CHR-ROM writeable flag 与未焊接 PRG bank open-bus read |
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
| 205 | `mmc3.rs:18-38,339-343,643-645,714-717,1110-1121,1345-1346,1682-1705; mapper.rs:539,969,1141,1317` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_205.h` | 5-50 | Mapper 205：低区 register 选择 `_selectedBlock`，PRG bank 按 block mask/OR，CHR bank 按 block 扩展 bit7/8 |
| 205 | `mmc3.rs:18-38,339-343,643-645,714-717,1110-1121,1345-1346,1682-1705; mapper.rs:539,969,1141,1317` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 478 | mapper 205 归类到 `MMC3_205` |
| 205 | `mmc3.rs:18-38,339-343,643-645,714-717,1110-1121,1345-1346,1682-1705; mapper.rs:539,969,1141,1317` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1161-1214 | FCEUX mapper 205 cross-check；记录 low write 后 `CartBW` fall-through、solder pad 与 split-ROM 差异，后两者留作后续精修 |
| 205 | `mmc3.rs:18-38,339-343,643-645,714-717,1110-1121,1345-1346,1682-1705; mapper.rs:539,969,1141,1317` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1241-1290 | FCEUmm mapper 205 cross-check；与当前第一版一致使用 single-ROM outer block wrap |
| 205 | `mmc3.rs:18-38,339-343,643-645,714-717,1110-1121,1345-1346,1682-1705; mapper.rs:539,969,1141,1317` | `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp` | 266 | Nestopia BMC_15IN1 board 元数据：mapper 205、512KB PRG、512KB CHR |
| 249 | `mmc3.rs:18-40,369-373,671-682,746-753,959-971,1268-1270,1393-1394,1752-1773; mapper.rs:568,993,1166,1347` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_249.h` | 5-57 | Mapper 249 / Waixing security：`$5000` exReg、bit1 选择 PRG/CHR bit permutation，其余继续复用 MMC3 |
| 249 | `mmc3.rs:18-40,369-373,671-682,746-753,959-971,1268-1270,1393-1394,1752-1773; mapper.rs:568,993,1166,1347` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1292-1330 | FCEUX mapper 249 cross-check；PRG 小/大 bank 两条 permutation 分支与 CHR permutation |
| 249 | `mmc3.rs:18-40,369-373,671-682,746-753,959-971,1268-1270,1393-1394,1752-1773; mapper.rs:568,993,1166,1347` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1381-1419 | FCEUmm mapper 249 cross-check；同样使用 `$5000` exReg 与普通 MMC3 power |
| 249 | `mmc3.rs:18-40,369-373,671-682,746-753,959-971,1268-1270,1393-1394,1752-1773; mapper.rs:568,993,1166,1347` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 516 | mapper 249 归类到 `MMC3_249` |
| 245 | `mmc3.rs:18-30,262-268,457-460,496-497,1127-1142` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1266-1289 | Mapper 245 CHR bank 低 3 位、CHR reg0 bit1 扩展 PRG outer bit、power reset |
| 245 | `mmc3.rs:18-30,262-268,457-460,496-497,1127-1142` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1355-1379 | FCEUmm mapper 245 cross-check；同样记录 `M245CW/M245PW` 行为 |
| 245 | `mmc3.rs:18-30,262-268,457-460,496-497,1127-1142` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_245.h` | 5-42 | Mesen2 Mapper 245 PRG outer bit 与 CHR-RAM mode cross-check |
| 250 | `mmc3.rs:18-38,360-365,651,704,914-916,973-974,1326,1626-1654; mapper.rs:567,990,1161,1340` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1332-1350 | Mapper 250：`(addr & 0xE000) | ((addr & 0x400) >> 10)` 重映射 MMC3 register address，并用 `addr & 0xff` 作为写入值 |
| 250 | `mmc3.rs:18-38,360-365,651,704,914-916,973-974,1326,1626-1654; mapper.rs:567,990,1161,1340` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1421-1440 | FCEUmm mapper 250 cross-check；同样拆分 command/IRQ 写 handler 后回到 GenMMC3 |
| 250 | `mmc3.rs:18-38,360-365,651,704,914-916,973-974,1326,1626-1654; mapper.rs:567,990,1161,1340` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_250.h` | 5-11 | Mesen2 MMC3_250：override `WriteRegister` 后调用普通 MMC3 register writer |
| 250 | `mmc3.rs:18-38,360-365,651,704,914-916,973-974,1326,1626-1654; mapper.rs:567,990,1161,1340` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 517 | mapper 250 归类到 `MMC3_250` |
| 253 | `waixing.rs:1-295` | `/Users/sunmeng/workspace/fc/fceux/src/boards/253.cpp` | 44-89, 110-145 | Mapper 253 PRG/CHR/mirroring、IRQ、2KB CHR-RAM 与 8KB WRAM |
| 253 | `waixing.rs:1-295` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Waixing/Mapper253.h` | 17-21, 54-80, 83-130 | Mapper 253 page size、CHR-RAM window、114-cycle IRQ scaler、register decode |
| 253 | `waixing.rs:1-804` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/252_253.c` | 37-104 | 252/253 later VRC4-style CHR-RAM mask path；Mapper252 已落地 PPUDATA hook，Mapper253 仍保留当前 FCEUX/Mesen2 第一版 |
| 254 | `mmc3.rs:18-32,279-289,509-510,724-730,814-831,948-954,1200-1230` | `/Users/sunmeng/workspace/fc/fceux/src/boards/mmc3.cpp` | 1353-1380 | Mapper 254 protected WRAM reads、`$8000` unlock、`$A001` XOR mask、MMC3 command write |
| 254 | `mmc3.rs:18-32,279-289,509-510,724-730,814-831,948-954,1200-1230` | `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/mmc3.c` | 1442-1469 | FCEUmm mapper 254 cross-check；同样记录 WRAM read XOR 保护 |
| 254 | `mmc3.rs:18-32,279-289,509-510,724-730,814-831,948-954,1200-1230` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_254.h` | 5-45 | Mesen2 Mapper 254 `ReadRegister` / `WriteRegister` cross-check |
| 255 | `multicart.rs:146-166,396-407,882-896` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Bmc255.h` | 5-25 | BMC255 地址 latch PRG/CHR/mirroring 公式；当前按 Mesen2 独立实现 |
| 255 | `multicart.rs:146-166,396-407,882-896` | `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/MapperFactory.cpp` | 521 | mapper 255 归类到 `Bmc255` |

## 当前实现和来源的对应关系

- `Mapper43::write_any_register()` 对应 Mesen2 `Mapper43.h:69-82` 与 FCEUX `43.cpp:51-58`。
- `Mapper43::expansion_prg_index()` / `low_prg_index()` 对应 Mesen2 `Mapper43.h:29-33,62-67` 与 FCEUX `43.cpp:38-48`。
- `Mapper36::write_register()` / `$4100` 读回对应 FCEUX `36.cpp:35-53`。
- `BandaiFcg::write_register_inner()` / `cpu_clock()` 对应 Mesen2 `BandaiFcg.h:127-239` 与 FCEUX/FCEUmm `bandai.cpp`/`bandai.c:268-297`；submapper 4 采用 FCG-1/2 直接写 IRQ counter 且只开放低区写，submapper 5 采用 LZ93D50 reload latch 且只开放高区写；Mapper153 `$800D` 通过 `MapperOps::low_prg_ram_read_enabled()` / `low_prg_ram_write_enabled()` gate 到 Cartridge 默认低区 SRAM。
- `Eeprom24C0x::write()` / `BandaiFcg::read_low_register_with_open_bus()` 对应 Mesen2 `Eeprom24C01.h:42-127`、`Eeprom24C02.h:42-144`、`BaseEeprom24C0X.h:55-68` 和 `BandaiFcg.h:142-156`；读值只驱动 bit4，其余位保留 CPU open bus；无 EEPROM 的 Mapper153 低区读回落到 Cartridge PRG-RAM。
- `MapperOps::read_low_register_with_open_bus()` / `peek_low_register_with_open_bus()` 是为 Bandai 这类低区串行设备读增加的 open-bus aware 钩子；默认仍回落到 `read_low_register_with_prg_ram()` / `peek_low_register_with_prg_ram()`，现有 mapper 行为不变。
- `Mapper35::write_register()` / `notify_a12()` 对应 Mesen2 `Mapper35.h:28-63`，并复用本项目 `A12EdgeFilter` 表达 MMC3-style A12 rising edge IRQ；FCEUmm `jyasic.c:455-488` 用于确认 mapper 35 属于 JYASIC single-cart 且 iNES mapper 35 默认有 8KB WRAM。
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
- `Mmc3OuterBank::Mapper14` / `Mmc3::new_14()` / `mapper14_write()` 对应 FCEUX `sl1632.cpp:22-104` 与 Nestopia `NstBoardRexSoftSl1632.cpp:38-196`；本项目第一版实现 direct mode 的 8KB PRG latch、1KB CHR nibble latch、mirroring latch，以及 `mode bit1` 切入 MMC3 后的标准 MMC3 PRG/CHR/IRQ 复用和 CHR bit8 shift table。FCEUmm `14.c:21-84,94-101` 的 VRC2/MMC3 双 ASIC path 需要后续复合 mapper 架构支持，当前未混入第一版。
- `Mmc3OuterBank::Mapper126` / `Mmc3::new_126()` 对应 FCEUmm `126_422_534.c:21-36,46-189,191-204` 与 Mesen2 `MMC3_126.h:5-88`；本项目按 FCEUmm 的 mapper 126/TEC9719 主线实现 outer PRG bit 反相、PRG 16K/32K/UNROM/ANROM-style modes、Mapper126 CHR A18/A19 交换、CNROM mode、extended mirroring、`$6000-$7FFF` extra regs、SL0 reset DIP 和 low WRAM fall-through。FCEUmm submapper 1-4 与旧 dump ROM bit 重排记录为后续具体 ROM 精修项，当前未混入第一版。
- `Mmc3OuterBank::Mapper45` 的 PRG/CHR AND/OR wrapper、低区 serial register 和 reset defaults 对应 FCEUX `mmc3.cpp:502-565`、FCEUmm `mmc3.c:505-589`、Mesen2 `MMC3_45.h:40-80` 与 Nestopia `NstBoardBmcHero.cpp:97-129`；FCEUX/FCEUmm 的可选 open-bus read 侧效应暂未落地，留作需要具体 ROM 证据时精修。
- `Mmc3OuterBank::Mapper114` / `Mapper115` / `Mapper121` 目前先按参考项目的第一版实现，后续若遇到特定 ROM 证据，可继续补齐更细的 reset/protection/read side-effect 行为。
- `Mapper252::write_chr_nibble()` / `notify_ppudata_write()` / `cpu_clock()` 对应 Mesen2 `Waixing252.h:5-68` 与 FCEUmm `252_253.c:20-71`；PPUDATA 写通知通过 `MapperOps::notify_ppudata_write()` 在 PPU `$2007` 写入真实 CHR/nametable 前触发。
- `Mapper253::write_chr_register()` / `chr_ram_index()` / `cpu_clock()` 对应 FCEUX `253.cpp:44-89,110-145` 与 Mesen2 `Mapper253.h:54-130`。
- `Namco163::new_210()` / `NamcoVariant::{Namco175,Namco340}` 对应 FCEUmm `n106.c:51,127-132,164-212,404-470`、FCEUX `n106.cpp:51,128-134,166-213,379-469`、Mesen2 `Namco163.h:8-14,61-88,141-155,178-282` 与 Nestopia `NstBoard.cpp:3087-3095` / `NstBoard.hpp:465-466`；本项目将 mapper 210 收进 Namco163 变体，保留 PRG/CHR 与 Namco340 mirroring，禁用 N163 expansion audio/readback/cpu-clock，Namco175 WRAM write-protect 细节留作后续 battery/WRAM 精修。
- `Mapper92::write_register()` 对应 FCEUX `72.cpp:35-80` 的 mapper 92 变体。
- `Mapper122::write_register()` 对应 FCEUmm `122.c:25-33` 的 A0 选择两个 4KB CHR latch。
- `ColorDreams::write_register()` / `apply_bus_conflict()` 对应 FCEUX/FCEUmm `datalatch.cpp`/`datalatch.c:222-233,157-167` 与 Mesen2 `ColorDreams.h:5-29`；mapper 144 只接受奇地址写并使用 bit0-only conflict。
- `MapperOps::apply_bus_conflict()` 是为 mapper 144 增加的 bus-conflict 细分钩子；默认仍保持原有 AND 语义。
- `Sachen133::write_low_register()` / `write_register()` 对应 Mesen2 `Sachen_133.h:19-25` 与 FCEUX/FCEUmm `sachen.cpp`/`sachen.c:273-296,180-203`。
- `SachenSa0161m::write_expansion()` / `write_register()` 对应 FCEUX/FCEUmm `sachen.cpp`/`sachen.c:258-284,306-316,165-190,213-222` 与 Mesen2 `Nina03_06.h:5-35`。
- `Sachen149::write_register()` 对应 Mesen2 `Sachen_149.h:16-19` 与 FCEUX/FCEUmm SA0036 的 SA72007 sync 路径。
- `Mapper108::low_prg_index()` / `write_register()` 对应 FCEUX `108.cpp:31-48` 与 FCEUmm `108.c:37-53`；高区固定最后 32KB，低区 `$6000-$7FFF` 按写入值映射 PRG-ROM。
- `Mapper111::write_expansion()` / `write_low_register()` / `nametable_chr_index()` 对应 FCEUX/FCEUmm `cheapocabra.cpp`/`cheapocabra.c:58-80` 与 Mesen2 `Cheapocabra.h:68-83,93-99`；本项目按 32KB CHR-RAM 表达 GTROM 的 2 个 pattern page 与 2 个 nametable page，并用 open-bus aware read hook 表达低区读副作用。battery-backed SST39SF040 flash command/ID/erase/write 需要 Cartridge/Mapper factory 传入 battery/flash backing，留作后续精修。
- `Namco108Mapper154::write_register()` 对应 FCEUX/FCEUmm `88.cpp`/`88.c:47-55` 与 Mesen2 `Namco108_154.h:9-12`；banking 复用 Namco118，command write bit6 只改变单屏 mirroring。
- `Mmc1::new_155()` 对应 FCEUX/FCEUmm `mmc1.cpp`/`mmc1.c:332-335,362-365` 与 Mesen2 `MMC1_155.h:7-12`；当前先保存 variant intent，等待普通 MMC1 PRG-RAM disable gating 落地后体现差异。
- `Mapper156::write_register()` / `chr_index()` 对应 FCEUX/FCEUmm `156.cpp`/`156.c:49-68,36-47` 与 Nestopia `NstBoardOpenCorp.cpp:42-60,78-114`；当前 8KB WRAM 通过 Cartridge 默认 iNES PRG-RAM 路径提供。
- `NanjingMapper` 的 mapper 162/163/164 PRG 译码、`$5000-$57FF` 扩展寄存器读写和 HBlank CHR split 对应 FCEUmm `162.c:36-78,81-107`、`163.c:36-83,86-118` 与 `164.c:36-86`；mapper 162 也 cross-check Mesen2 `Waixing162.h:15-46`、FCEUX `164.cpp:180-229` 的 FS304 分支与 Nestopia `NstBoard.cpp:2772-2781` 的板卡映射，mapper 163 cross-check Mesen2 `Nanjing.h:38-119` 与 FCEUX `164.cpp:133-178`，mapper 164 cross-check Mesen2 `Waixing164.h:4-32` 与 Nestopia `NstBoardWaixingFfv.cpp:35-99`。本项目当前按 FCEUmm/Nestopia 的 162/163/164 分离语义实现，163 的 older protection read `$5100/$5500` 与 164 的 EEPROM/1bpp 变体留给后续精修批次。
- `Subor166::selected_banks()` 对应 FCEUX/FCEUmm `subor.cpp`/`subor.c:31-58`、Mesen2 `Subor166.h:36-53` 与 Nestopia `NstBoardSubor.cpp:93-124`；mapper 166/167 只用 variant 切换 NROM-like bank order 和 fixed bank。
- `Mapper181` / `Mapper185::chr_enabled()` / `chr_read()` 对应 FCEUX/FCEUmm `185.cpp`/`185.c:45-58`；当前用 mapper-owned read override 返回 `0xFF` 来表达 dummy CHR page，181 与 185 共享固定 PRG16 和 dummy CHR 架构，仅 gating 条件不同。
- `Mapper188::write_register()` / `read_low_register()` 对应 FCEUX/FCEUmm `karaoke.cpp`/`karaoke.c:26-52`；PRG latch 为 0 时选择 `7 + prg16/16`，bit4 控制低 8 个或高 8 个 PRG16 bank。
- `Mmc3OuterBank::Mapper189` 对应 FCEUX/FCEUmm `189.cpp`/`189.c:24-43`；低区写保存 `value | (value >> 4)`，PRG 走 32KB outer bank，CHR 和 IRQ 继续复用普通 MMC3。
- `Mapper190::write_register()` / `chr_index()` / `prg_index()` 对应 FCEUX/FCEUmm `190.cpp`/`190.c:24-90`；本项目复用 cartridge PRG-RAM 路径提供 `$6000-$7FFF` WRAM，高区实现 PRG16 和四个 CHR2 latch。
- `Mmc3::new_191()` 复用 `ChrRamWindow`，对应 FCEUX `mmc3.cpp:977-987` 与 Mesen2 `MapperFactory.cpp:464` 的 `MMC3_ChrRam(0x80,0xFF,2)`；FCEUmm `191.c` 新增的 submapper/PRG low-register 细节留作后续精修。
- `Mmc3ChrLayout::Mapper165` 对应 FCEUX/FCEUmm `mmc3.cpp`/`mmc3.c:918-973,969-1024` 与 Mesen2 `MMC3_165.h:5-54`；本项目用现有 PPU bus watcher 同时驱动 MMC3 IRQ 和 MMC2-style 4KB CHR latch，寄存器值 0 时路由到 mapper-owned 4KB CHR-RAM。
- `Mapper193::write_low_register()` / `chr_index()` / `prg_index()` 对应 FCEUX/FCEUmm `193.cpp`/`193.c:38-59`；低区四个寄存器直接控制 PRG8 与 CHR4/2/2。
- `Mapper186::write_expansion()` / `read_expansion()` / mapper-owned low WRAM 对应 FCEUX/FCEUmm `186.cpp`/`186.c:36-74`；本项目用 mapper 内 32KB WRAM 表达 `setprg8r(0x10,0x6000,regs[0]>>6)`，用 3KB SWRAM 表达 `$4400-$4FFF`，并通过关闭 Cartridge 默认 low PRG-RAM 避免双写。
- `Mmc3OuterBank::Mapper187` 对应 FCEUX/FCEUmm `187.cpp`/`187.c:24-67,28-70` 与 Mesen2 `MMC3_187.h:24-87`；本项目用 MMC3 outer bank 分支表达 PRG forced 16/32KB、CHR bit8 扩展、`$8000/$8001` 门控和 `$5000-$5FFF` security read。
- `Mmc3OuterBank::Mapper196` 与 `mapper196_remap_addr()` 对应 FCEUX/FCEUmm `mmc3.cpp`/`mmc3.c:1059-1079,1072-1090` 与 Mesen2 `MMC3_196.h:24-47`；低区写启用 PRG32 latch，高区写先重排地址线再进入普通 MMC3 helper。
- `Mmc3ChrLayout::Mapper197` / `mapper197_chr_write()` 对应 FCEUX/FCEUmm `mmc3.cpp:1132-1143` 与 `mmc3.c:1144-1217`；当前落地 submapper 0/1/2 的 2KB CHR cwrap，submapper 3 的低寄存器 outer PRG/CHR 留作后续精修。
- `Mmc3OuterBank::Mapper198` / `Mmc3::new_198()` 对应 FCEUX/FCEUmm `mmc3.cpp:1148-1158` 与 `mmc3.c:1221-1238`；本项目复用低区 WRAM helper，并按 pwrap 规则将 PRG bank `>= 0x50` mask 到 `bank & 0x4F`。
- `Mmc3OuterBank::Mapper199` / `Mmc3::new_199()` 对应 FCEUX `199.cpp:32-97` 与 Mesen2 `MMC3_199.h:6-67`；本项目按 FCEUX/Mesen2 的 Waixing Type G 行为实现 `$C000/$E000` fixed PRG extra regs、`$0000/$0400/$0800/$0C00` fixed CHR slots、CHR bank `<8` 选择 8KB CHR-RAM 和 2-bit mirroring。FCEUmm `199.c:23-38` 的 unbanked CHR-RAM / `$5000` PRG-RAM window 记录为参考差异，未作为当前主实现。
- `Mmc3OuterBank::Mapper208` 对应 FCEUX/FCEUmm `208.cpp`/`208.c:24-61,32-79` 与 Mesen2 `MMC3_208.h:8-75`；本项目将 protection LUT、`$4800/$6800` PRG32 latch、`$5000-$5FFF` protection registers 和 submapper 1 PRG source 合并在同一 MMC3 变体中。
- `Mmc3::new_215()` / `Mmc3OuterBank::Mapper215` / `mapper215_write()` 对应 Mesen2 `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_215.h:6-114`；本项目复用 MMC3 标准寄存器 helper 表达 high-register address LUT、command register LUT、forced PRG modes 和 CHR high-bit 扩展。
- `Mmc3OuterBank::Mapper245` 的 PRG outer bit 与 CHR bank mask 对应 FCEUX/FCEUmm `mmc3.cpp`/`mmc3.c:1268-1277,1357-1365`，并用 Mesen2 `MMC3_245.h:23-40` cross-check PRG bank order。
- `Mmc3OuterBank::Mapper254` 的 low read XOR 保护对应 FCEUX/FCEUmm `mmc3.cpp`/`mmc3.c:1355-1367,1444-1456` 与 Mesen2 `MMC3_254.h:27-43`；本项目通过 `read_low_register_with_prg_ram()` 保留底层 WRAM byte 后再组合保护值。
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
- `AddrLatchVariant::Mapper221` / `AddrLatch16k::new_221()` 对应 Mesen2 `Mapper221.h:19-50` 与 FCEUmm `n625092.c:36-62`；本项目保存 mode/prg 两个地址 latch，支持 UNROM/NROM-256/NROM-128 与 submapper 1 outer bit，并通过 open-bus high-read hook 表达未焊接 PRG bank。
- `Mapper212::select_from_addr()` / low read OR bit7 对应 Mesen2 `Mapper212.h:18-38` 和 FCEUX `addrlatch.cpp:274-289`。
- `Mapper222::notify_a12()` / register decode 对应 Mesen2 `Mapper222.h:33-64`。
- `Mapper235::read_register_with_open_bus()` / reset UNROM mode 对应 FCEUX `235.cpp:42-95`。
- `TaitoX1005::write_low_register()` / gated WRAM 对应 FCEUX/FCEUmm `80.cpp`/`80.c:75-103,136-142`。
- `Namco108Mapper206::write_register()` / `chr_index()` / `prg_index()` 对应 FCEUX/FCEUmm `206.cpp`/`206_486.c:33-78`；Mesen2 `Namco108.h:15-27` 用于确认地址 mask 与固定 PRG/CHR mode。
- `TaitoX1005::new_207()` / `set_chr_2k()` alternate mirroring 对应 FCEUX/FCEUmm `80.cpp`/`80.c:87-103,145-190` 与 Mesen2 `TaitoX1005.h:50-97,110-113`。
- `TaitoX1017::chr_index()` 的 pattern half swap 对应 FCEUX `82.cpp:37-50` 与 FCEUmm `82_552.c:45-58`。
- `Mapper81::write_register()` / `prg_index()` / `chr_index()` 对应 FCEUmm `81.c:24-31`；本项目显式保存地址 latch 和数据 latch，以等价表达 FCEUmm `Latch_Init` 提供的 `latch.addr` / `latch.data`。
- `Mapper99::write_controller_strobe()` 对应 FCEUX/FCEUmm `99.cpp`/`99.c:34-44,47-55`、Nestopia `NstBoardVsSystem.cpp:38-61` 与 Mesen2 `VsSystem.h:82-97`；本项目新增 `MapperOps::write_controller_strobe()`，由 Bus 在普通 controller strobe 后调用，避免 mapper 复用 `$4016` 时破坏手柄写行为。
- `Mapper104::write_register()` / `prg_index()` 对应 FCEUmm `104.c:35-69,77-91`；`setprg8r(0x10,0x6000,0)` 由本项目 Cartridge 的普通低区 PRG-RAM fallback 提供，mapper 内只保存两个 PRG16 register 并固定 vertical mirroring。
- `Mapper168::write_register()` / `chr_read()` / `chr_write()` / `cpu_clock()` 对应 FCEUX/FCEUmm `168.cpp`/`168.c:33-42,48-56,68-77` 与 Mesen2 `Racermate.h:11-17,32-55`；本项目按 FCEUX/FCEUmm 的 PRG/CHR/CHR-RAM 行为，并补入 Mesen2 的 CPU-cycle IRQ 与 `$C000-$FFFF` reload/ack。
- `Mapper171::write_register()` / `chr_index()` / `prg_index()` 对应 Mesen2 `Core/NES/Mappers/Kaiser/Kaiser7058.h:5-24` 与 Nestopia `source/core/board/NstBoardKaiser.cpp:181-188`；本项目按 KAISER KS-7058 的固定 PRG32 与 `$F000/$F080` 两个 CHR4 register 实现。FCEUX `src/boards/mmc1.cpp:338-343` 把 iNES 171 作为“MMC1 with fixed mirroring”处理，作为参考差异记录，未作为本次主实现。
- `Mapper175::read_register()` 对应 FCEUX/FCEUmm `175.cpp`/`175.c:54-59`；本项目用高区 read side-effect hook 在读 `$FFFC` 时提交 `committed_prg`，peek 路径保持无副作用。
- `Mapper177::write_register()` / `prg_index()` 对应 FCEUX/FCEUmm `177.cpp`/`177.c:34-53`；参考中的 `$6000-$7FFF` WRAM mapping 继续由 Cartridge 默认低区 PRG-RAM fallback 提供。
- `Mapper177::new_179()` / `HenggedianziVariant::Mapper179` 对应 Mesen2 `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Unlicensed/Henggedianzi179.h:10-27` 与 Nestopia `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardHenggedianzi.cpp:43-51,57-60`；本项目复用 Mapper177 结构表达 Henggedianzi XJZB：`$5000-$5FFF` 写 PRG32=`value>>1`，高区写只控制 vertical/horizontal mirroring，CHR8 固定 0。
- `Mapper178::write_expansion()` / mapper-owned low WRAM / `prg_index()` 对应 FCEUX/FCEUmm `178.cpp`/`178.c:85-149` 与 Mesen2 `Waixing178.h:32-60`；本项目使用 mapper 内 32KB WRAM 表达 `setprg8r(0x10,0x6000,reg[3]&3)`，并支持 FCEUmm submapper 3 的 pad/address-bit 读变体。FCEUmm 的 ADPCM decode 写 `$4011` 与 FCEUX sensor IRQ 留作后续音频/输入精修项。
- `MapperOps::read_expansion_with_open_bus()` / `peek_expansion_with_open_bus()` 是为 TXC/Sachen 低寄存器读增加的 open-bus aware 钩子；默认仍回落到旧 `read_expansion()` / `peek_expansion()`，现有 mapper 行为不变。
- `TxcChip` 对应 FCEUmm `src/boards/txcchip.c:58-146` 与 Mesen2 `Core/NES/Mappers/Txc/TxcChip.h:6-96`；本项目保留 accumulator/inverter/staging/output/increase/Y/invert/mask，并按参考实现的 `$4100-$4103` 低寄存器和高区 output commit 语义更新。
- `TxcMapper::Mapper132` 对应 FCEUmm `src/boards/txcchip.c:200-229` 与 Mesen2 `Core/NES/Mappers/Txc/Txc22211A.h:6-54`；PRG32 使用 `(output >> 2) & 1`，CHR8 使用 `output & 3`，`$4100` 读返回 `(open_bus & 0xF0) | txc_read_low4`。
- `TxcMapper::Mapper173` 对应 FCEUmm `src/boards/txcchip.c:231-244` 与 Mesen2 `Core/NES/Mappers/Txc/Txc22211C.h:5-20`；PRG32 固定 0，CHR-ROM 大于 8KB 时按 `output bit0 + Y + output bit1` 组合 CHR8，小 CHR dump 先固定 CHR8 0。
- `TxcMapper::Mapper136` 对应 FCEUmm `src/boards/txcchip.c:248-276` 与 Mesen2 `Core/NES/Mappers/Sachen/Sachen_136.h:8-58`；本项目采用 Mesen2 的 Sachen_136 PRG32 固定 0、CHR8=`output`，读值保留 open-bus bit6-7。
- `TxcMapper::Mapper147` 对应 FCEUmm `src/boards/txcchip.c:278-308` 与 Mesen2 `Core/NES/Mappers/Sachen/Sachen_147.h:8-61`；写值使用 `((value & 0xFC) >> 2) | ((value & 0x03) << 6)`，读值反向展开，PRG/CHR 分别取 `output` 的 board-specific bit fields。
- `TxcMapper::Mapper172` 对应 FCEUmm `src/boards/txcchip.c:310-344` 与 Mesen2 `Core/NES/Mappers/Txc/Txc22211B.h:6-62`；写读值使用 6-bit reverse permutation，CHR8=`output`，mirroring 由 TXC invert flag 选择 vertical/horizontal。
- `Sachen74Ls374N::new()` / Mapper150/243 对应 Mesen2 `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Sachen/Sachen74LS374N.h:14-79`、FCEUmm `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/sachen.c:250-331`、FCEUX `/Users/sunmeng/workspace/fc/fceux/src/boards/sachen.cpp:26-93` 与 Nestopia `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp:502-503`；本项目按 Mesen2 的 8-register 74LS374N 模型实现，Mapper150 支持 DIP 影响写值 bit2 与读回 D2 open-bus，Mapper243 禁用读回。FCEUmm/FCEUX 的 Mapper150 PRG/CHR bit packing 与 Mesen2 不完全一致，当前按 Mesen2/Nestopia 元数据落第一版；`SetNametables(0,0,0,1)` 暂用 `Mirroring::FourScreen` 近似，后续若扩 per-nametable page hook 可精修。
- `FfeMapper` / `FfeMode::{Mapper6,Mapper17}` 对应 FCEUX `src/boards/ffe.cpp:26-153`；Mapper6 使用 latch 模式的 PRG16/CHR8，Mapper17 使用 `$4504-$4507` PRG8 与 `$4510-$4517` CHR1 寄存器，`$42FE-$42FF` mirroring 和 `$4501-$4503` CPU-cycle IRQ 按同文件 `65-75,108-117` 实现。本项目继续通过 Cartridge 默认 PRG-RAM 路径提供 `$6000-$7FFF` WRAM。
- `Mapper218` 对应 FCEUX `src/boards/datalatch.cpp:462-493`、FCEUmm `src/boards/218.c:23-34` 与 Mesen2 `Core/NES/Mappers/Homebrew/MagicFloor218.h:4-31`；本项目用 mapper-owned 2KB pattern RAM 表达 `$0000-$1FFF` pattern table 到 NTARAM page A/B 的 1KB 页映射，PRG32 固定 0。Four-screen header 的 bit0 细分目前由 `Mirroring::FourScreen -> SingleScreenLow` 近似，后续若扩 factory/header bit 透传可精修到 FCEUX `mirrorAs2Bits` 语义。
- `TaitoTc0190::new_48()` / `hblank_clock()` 对应 FCEUX/FCEUmm `33.cpp`/`33.c:66-97,110-115`；普通 TC0190 bank writes 复用同文件 `52-63`。
- `Rambo1::new_158()` / `set_mapper158_nametable()` / mapper-owned nametable read/write 对应 FCEUmm `tengen.c:198-220` 与 Mesen2 `Rambo1_158.h:5-37`；RAMBO-1 基础 PRG/CHR/IRQ 仍对应 Mesen2 `Rambo1.h:96-177`。
- `JyAsic` / `JyAsicVariant::{Mapper90,Mapper209,Mapper211}` 对应 FCEUmm `jyasic.c:79-380,393-512`、FCEUX `90.cpp:25-27,74-418,438-507` 与 Mesen2 `JyCompany.h:11-452`；当前实现覆盖 PRG/CHR/NT/ALU/CPU-clock IRQ/HBlank 近似/PPU-read IRQ/CHR latch，CPU-write IRQ source 留待 MapperOps 全局 CPU write hook。
- `Mapper29::write_register()` / `prg_index()` / `chr_index()` 对应 FCEUX `datalatch.cpp:248-256`、FCEUmm `datalatch.c:186-194` 与 Mesen2 `SealieComputing.h:8-31`；当前第一版按 FCEUX/Mesen2 的高区 register 窗口实现，并在 iNES CHR-RAM 默认容量中补 32KB。
- `Mapper51::write_low_register()` / `low_prg_index()` / `write_register()` 对应 FCEUX/FCEUmm `51.cpp`/`51.c:31-72`；本项目用现有 low-register + low-PRG-ROM hook 表达 `$6000-$7FFF` 既是 mode 写窗口又是 PRG-ROM 读窗口的行为。
- `Mapper128::write_register()` / `prg_index()` 对应 FCEUmm `src/boards/128.c:24-38,40-55`；本项目保存 outer address latch 与 data latch，表达低 16KB inner bank、高 16KB fixed `outer|7`、固定 CHR8 0 和 outer bit1 mirroring。
- `Mapper236::write_register()` / `prg_index()` / reset DIP 对应 FCEUmm `src/boards/236.c:33-78,80-110`；本项目按 CHR-ROM/CHR-RAM 两种变体译码 PRG/CHR，soft reset 递增 DIP，并在 mode 1 中把高区 PRG read 地址低 4 位替换为 DIP。
- `Mapper237::write_register()` / `read_register()` / reset DIP 对应 FCEUmm `src/boards/237.c:42-78,84-95`；本项目保存两个 register、锁定位、DIP 读回和 PRG16/mirroring 译码。
- `AddrLatchVariant::Mapper239` 对应 FCEUmm `src/boards/239.c:24-37`；本项目复用 AddrLatch16k 架构表达 address latch PRG32/PRG16、CHR8 和 bit4 mirroring。
- `Mmc3::new_250()` / `mapper250_remap_addr()` 对应 FCEUX `mmc3.cpp:1332-1350`、FCEUmm `mmc3.c:1421-1440` 与 Mesen2 `MMC3_250.h:5-11`；本项目把地址线写协议折回 `write_standard_register()`，继续复用普通 MMC3 PRG/CHR/mirroring 和 A12 IRQ。
- `Mmc3::new_205()` / `Mmc3OuterBank::Mapper205` 对应 Mesen2 `MMC3_205.h:20-49`、FCEUX `mmc3.cpp:1167-1199` 与 FCEUmm `mmc3.c:1245-1270`；本项目按 single-ROM outer block 语义实现，低区写后通过 `low_register_write_falls_through()` 继续写 PRG-RAM 以覆盖 FCEUX `CartBW` 行为。
- `Mmc3::new_249()` / `mapper249_permute_large_bank()` 对应 Mesen2 `MMC3_249.h:24-55` 与 FCEUX/FCEUmm `mmc3.cpp`/`mmc3.c:1294-1315,1383-1404`；本项目用 `$5000` expansion write 保存 exReg，并在 PRG/CHR wrapper 层复刻 security permutation。
- `Mmc3::new_182()` / `Mmc3OuterBank::Mapper182` / `mapper182_write()` 对应 FCEUmm `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/182.c:28-75` 与 Mesen2 `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_182.h:9-37`；本项目复用 MMC3 标准寄存器 helper 表达 remapped high writes、AX5202P outer PRG/CHR bank mask/OR、低区 `$6000-$7FFF` register write，并通过 `low_register_write_falls_through()` 保留普通 PRG-RAM 写入。
- `Mmc3::new_223()` / `Mmc3OuterBank::Mapper223` 对应 Nestopia `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardWaixing.cpp:96-105,159-169`、`NstBoardWaixing.hpp:48-61`、`NstBoard.hpp:585` 与 `NstBoard.cpp:3183-3186,3954`；本项目按 Waixing Type I 落地为标准 MMC3 + mapper-owned `$5000-$5FFF` WRAM/security 窗口，PRG/CHR/IRQ 继续复用 MMC3 现有路径。
- `Mmc3::new_224()` / `Mmc3OuterBank::Mapper224` 对应 Mesen2 `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_224.h:14-54`，并用 Nestopia `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp:586` 确认 mapper 224 / Waixing Type J 元数据；本项目复用 MMC3 PRG/CHR/IRQ，只在 `$5000` 保存 PRG outer bit 并把 PRG8 bank 限到 6-bit inner bank。
- `Mmc3::new_238()` / `Mmc3OuterBank::Mapper238` 对应 Mesen2 `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Mmc3Variants/MMC3_238.h:12-40`，并用 Nestopia `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoard.hpp:316` 与 FCEUX `/Users/sunmeng/workspace/fc/fceux/src/boards/ax5705.cpp:47` 的 mapper 238 线索确认覆盖范围；本项目通过 expansion + low-register hooks 表达 `$4020-$7FFF` security register 读写，高区 `$8000-$FFFF` 保持普通 PRG-ROM read。

## 以后替换时的删除边界

- 先替换 `fc-core/src/mapper/basic/unlicensed.rs:1-884`。
- 同批替换 `fc-core/src/mapper/basic/latch/discrete.rs` 里 Mapper 29 / 36 / 72 / 79 / 92 / 99 / 122 的新增段、`fc-core/src/mapper/basic/latch/sachen.rs` 里 Sachen 133 / 137 / 141 / 146 / 148 / 149 / 150 / 243 的新增段、`fc-core/src/mapper/basic/core.rs` 里 ColorDreams/Mapper144 的扩展段、`fc-core/src/mapper/basic/taito.rs` 里 Mapper 80 / 207 / 82 的新增段，以及 `fc-core/src/mapper/basic/multicart.rs` 里 Mapper 51 / 59 / 63 / 128 / 201 / 217 / 221 / 228 / 236 / 237 / 239 / 255 的新增段。
- 同批替换 `fc-core/src/mapper/basic/latch/sunsoft.rs` 里 Mapper 68 的新增段，并同步检查 `MapperOps::nametable_chr_index` 与 `Cartridge::mapper_has_nametable_chr_mapping` 是否仍有其他使用者。
- 同批替换 `fc-core/src/mapper/basic/core.rs` 里 Mapper 232 的新增段。
- 同批替换 `fc-core/src/mapper/basic/bandai.rs` 里 Mapper16 / 153 / 159、Bandai FCG、24C01/24C02 EEPROM 新增段，并同步检查 `MapperOps::read_low_register_with_open_bus` / `peek_low_register_with_open_bus` / `low_prg_ram_read_enabled` / `low_prg_ram_write_enabled` 是否仍有使用者。
- 同批替换 `fc-core/src/mapper/basic/txc.rs` 里 TXC chip/helper 与 Mapper132/136/147/172/173 新增段，并同步检查 `MapperOps::read_expansion_with_open_bus` / `peek_expansion_with_open_bus` 是否仍有使用者。
- 同批替换 `fc-core/src/mapper/basic/discrete.rs` 里 Mapper 181 / 185 / 186 / 188 / 193 / 218 的新增段，以及 `fc-core/src/mapper/mmc3.rs` 里 Mapper14 / Mapper126 / Mapper176 / Mapper182 / Mapper187 / Mapper189 / Mapper191 / Mapper196 / Mapper197 / Mapper198 / Mapper199 / Mapper208 / Mapper215 / Mapper223 / Mapper224 / Mapper238 / Mapper245 / Mapper254 的 `Mmc3OuterBank` / `Mmc3ChrLayout` 分支、构造、低区写、扩展区读写、低区读、reset 和测试段。
- 同批替换 `fc-core/src/mapper/basic/irq.rs` 里 FFE Mapper6/17 新增段。
- 同批替换 `fc-core/src/mapper/basic/konami.rs` 的 VRC1 段、`fc-core/src/mapper/basic/jy.rs` 的 Mapper35/90/91/209/211 段、`fc-core/src/mapper/basic/sl12.rs` 的 Mapper116 段、`fc-core/src/mapper/basic/waixing.rs` 的 Mapper178 / Mapper252 / Mapper253 段、`fc-core/src/mapper/vrc4.rs` 的 VRC2/VRC4 段、`fc-core/src/mapper/rambo1.rs` 的 Mapper64/158 段，`fc-core/src/mapper/basic/taito.rs` 的 Mapper48 段，以及 `fc-core/src/mapper/mmc3.rs` 的 Mapper37/44/45/47/52/76/119 变体段。
- 同批替换 `fc-core/src/mapper/basic/special.rs` 里 Mapper111 / Mapper168 / Mapper171 / Mapper177 / Mapper179 的新增段，并同步检查 `MapperOps::has_chr_read` / `chr_read` / `chr_write` / `cpu_clock` / `nametable_chr_index` 是否仍有使用者。
- 若替换 Mapper99，请同步检查 `MapperOps::write_controller_strobe`、`Cartridge::cpu_write_controller_strobe` 与 `Bus::write($4016)` 是否仍有使用者。
- 再处理 `fc-core/src/mapper.rs` 里 Mapper 43/60/75/76/83/91/99/106/168/183/212/222/235 的导出、枚举、构造和 dispatch 分支。
- 若替换 Mapper91，请同步检查 `MapperOps::hblank_clock`、`Cartridge::mapper_clocks_hblank` 与 `Bus::clock_ppu_dot()` 的 HBlank hook 是否仍有使用者。
- 最后检查 `fc-core/src/cartridge.rs` 的 open-bus aware 读钩子是否仍被其他 mapper 使用；如果无使用者，可收窄接口。
