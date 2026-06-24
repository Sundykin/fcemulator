use super::*;
use crate::cartridge::Cartridge;

/// Locks the `watches_ppu_bus` table to exactly the mappers that override
/// `notify_a12`. If a new mapper hooks the PPU bus, add it here AND set its
/// flag, or the PPU fast path will silently drop its A12/CHR-latch events.
#[test]
fn watches_ppu_bus_matches_notify_a12_overrides() {
    let mir = Mirroring::Horizontal;
    let cases = [
        (0u16, false), // NROM
        (1, false),    // MMC1
        (2, false),    // UNROM
        (3, false),    // CNROM
        (6, false),    // FFE F4xxx
        (7, false),    // AxROM
        (8, false),    // Mapper 8
        (11, false),   // ColorDreams
        (12, true),    // Mapper 12 MMC3 A12 IRQ
        (13, false),   // CPROM
        (14, true),    // Mapper 14 SL-1632 uses MMC3 A12 IRQ in MMC3 mode
        (15, false),   // 100-in-1 multicart
        (16, false),   // Bandai FCG/LZ93D50
        (17, false),   // FFE F4xxx full mode
        (18, false),   // Jaleco SS88006
        (19, false),   // Namco 163
        (21, false),   // VRC4 IRQ is CPU-clocked, not PPU-bus-clocked
        (22, false),   // VRC2a
        (23, false),   // VRC2/VRC4
        (24, false),   // VRC6a
        (26, false),   // VRC6b
        (28, false),   // Action 53
        (29, false),   // Sealie Computing
        (31, false),   // Mapper 31
        (32, false),   // Irem G-101
        (33, false),   // Taito TC0190
        (34, false),   // BNROM
        (35, true),    // Mapper 35 A12 IRQ
        (36, false),   // TXC/Micro Genius simplified mapper
        (37, true),    // Mapper 37 MMC3 A12 IRQ
        (39, false),   // Mapper 39
        (40, false),   // Mapper 40
        (42, false),   // Mapper 42
        (43, false),   // Mapper 43 IRQ is CPU-clocked, not PPU-bus-clocked
        (44, true),    // Mapper 44 MMC3 A12 IRQ
        (45, true),    // Mapper 45 MMC3 A12 IRQ
        (47, true),    // Mapper 47 MMC3 A12 IRQ
        (48, false),   // Mapper 48 IRQ is HBlank-clocked
        (49, true),    // Mapper 49 MMC3 A12 IRQ
        (25, false),   // VRC4 IRQ is CPU-clocked, not PPU-bus-clocked
        (52, true),    // Mapper 52 MMC3 A12 IRQ
        (53, false),   // BMC SuperVision 16-in-1
        (56, false),   // Kaiser KS202 IRQ is CPU-clocked
        (66, false),   // GxROM
        (67, false),   // Sunsoft-3
        (68, false),   // Sunsoft-4 nametable CHR mapping does not need A12 notify
        (69, false),   // FME-7 / Sunsoft 5B
        (41, false),   // Caltron 6-in-1
        (46, false),   // Color Dreams 46
        (50, false),   // Mapper 50
        (51, false),   // Mapper 51
        (57, false),   // Mapper 57
        (58, false),   // Mapper 58
        (59, false),   // Mapper 59
        (60, false),   // Mapper 60
        (61, false),   // Mapper 61
        (62, false),   // Mapper 62
        (63, false),   // Mapper 63
        (64, true),    // Tengen RAMBO-1 can use PPU A12 IRQ mode
        (65, false),   // Irem H3001
        (70, false),   // Bandai 74161/7432
        (71, false),   // Codemasters
        (72, false),   // Mapper 72
        (73, false),   // VRC3
        (75, false),   // VRC1
        (76, true),    // Mapper 76 MMC3 A12 IRQ
        (77, false),   // Irem LROG017
        (78, false),   // Jaleco JF-16
        (79, false),   // Mapper 79
        (80, false),   // Taito X1-005
        (81, false),   // Mapper 81
        (82, false),   // Taito X1-017
        (83, false),   // Mapper 83 IRQ is CPU-clocked, not PPU-bus-clocked
        (85, false),   // VRC7
        (87, false),   // Jaleco JF-xx
        (88, false),   // Namco 118
        (90, true),    // JY ASIC watches PPU bus for PPU-read IRQ / CHR latch
        (91, false),   // Mapper 91 IRQ is HBlank-clocked
        (92, false),   // Mapper 92
        (96, true),    // Mapper 96 PPU nametable latch
        (95, false),   // Namco 108 mapper 95
        (99, false),   // VS UniSystem controller-strobe latch
        (101, false),  // Jaleco JF-xx ordered bits
        (103, false),  // Mapper 103
        (104, false),  // Mapper 104
        (105, false),  // Mapper 105 IRQ is CPU-clocked, not PPU-bus-clocked
        (106, false),  // Mapper 106 IRQ is CPU-clocked, not PPU-bus-clocked
        (108, false),  // Mapper 108
        (111, false),  // Cheapocabra / GTROM
        (114, true),   // Mapper 114 MMC3 A12 IRQ
        (115, true),   // Mapper 115 MMC3 A12 IRQ
        (117, true),   // Mapper 117 A12 IRQ
        (118, true),   // Mapper 118 MMC3 A12 IRQ
        (119, true),   // Mapper 119 MMC3 A12 IRQ
        (120, false),  // Mapper 120
        (121, true),   // Mapper 121 MMC3 A12 IRQ
        (122, false),  // Mapper 122
        (126, true),   // Mapper 126 MMC3 A12 IRQ
        (128, false),  // Mapper 128
        (132, false),  // TXC 22211A
        (133, false),  // Sachen SA72008
        (134, true),   // Mapper 134 MMC3 A12 IRQ
        (136, false),  // Sachen JV001
        (137, false),  // Sachen 8259D
        (142, false),  // Mapper 142 IRQ is CPU-clocked, not PPU-bus-clocked
        (144, false),  // Mapper 144 ColorDreams variant
        (141, false),  // Sachen 8259A
        (147, false),  // Sachen JV001
        (146, false),  // Sachen SA016-1M
        (148, false),  // Sachen SA0037
        (149, false),  // Sachen SA0036
        (150, false),  // Sachen 74LS374N
        (243, false),  // Sachen 74LS374N
        (156, false),  // Mapper 156
        (162, false),  // Waixing FS304
        (163, false),  // Nanjing FC-001
        (158, true),   // Mapper 158 RAMBO-1 can use PPU A12 IRQ mode
        (159, false),  // Bandai LZ93D50 with 24C01
        (165, true),   // Mapper 165 MMC3 A12 IRQ and CHR latch
        (166, false),  // Subor 166
        (167, false),  // Subor 167
        (112, false),  // NTDEC ASDER
        (116, true),   // Mapper 116 can switch into MMC3 A12 IRQ mode
        (151, false),  // Mapper 151
        (153, false),  // Bandai FCG with SRAM
        (154, false),  // Namco 108 mapper 154
        (155, false),  // MMC1 mapper 155
        (168, false),  // Racermate CPU IRQ; no PPU-bus hook
        (170, false),  // Mapper 170
        (171, false),  // Kaiser KS-7058
        (172, false),  // TXC 22211B
        (173, false),  // TXC 22211C
        (152, false),  // Bandai 74161/7432
        (174, false),  // Mapper 174
        (175, false),  // Mapper 175
        (177, false),  // Mapper 177
        (178, false),  // Waixing FS305/NJ0430
        (181, false),  // Mapper 181 CNROM protection
        (182, true),   // Mapper 182 MMC3 A12 IRQ
        (183, false),  // Mapper 183 IRQ is CPU-clocked, not PPU-bus-clocked
        (186, false),  // Mapper 186 Family Study Box
        (188, false),  // Mapper 188
        (230, false),  // Mapper 230
        (233, false),  // Mapper 233
        (234, false),  // Mapper 234
        (236, false),  // Mapper 236
        (237, false),  // Mapper 237
        (187, true),   // Mapper 187 MMC3 A12 IRQ
        (191, true),   // Mapper 191 MMC3 A12 IRQ
        (190, false),  // Mapper 190
        (192, true),   // Mapper 192 MMC3 A12 IRQ
        (195, true),   // Mapper 195 MMC3 A12 IRQ
        (196, true),   // Mapper 196 MMC3 A12 IRQ
        (197, true),   // Mapper 197 MMC3 A12 IRQ
        (198, true),   // Mapper 198 MMC3 A12 IRQ
        (199, true),   // Mapper 199 MMC3 A12 IRQ
        (200, false),  // Mapper 200
        (201, false),  // Mapper 201
        (202, false),  // Mapper 202
        (204, false),  // Mapper 204
        (205, true),   // Mapper 205 MMC3 A12 IRQ
        (206, false),  // Namco 108 mapper 206
        (207, false),  // Taito X1-005 mapper 207
        (208, true),   // Mapper 208 MMC3 A12 IRQ
        (209, true),   // JY ASIC watches PPU bus for PPU-read IRQ / CHR latch
        (210, false),  // Namcot 175/340
        (211, true),   // JY ASIC watches PPU bus for PPU-read IRQ / CHR latch
        (212, false),  // Mapper 212
        (213, false),  // Mapper 213
        (214, false),  // Mapper 214
        (215, true),   // Mapper 215 MMC3 A12 IRQ
        (216, false),  // Mapper 216
        (217, false),  // Mapper 217
        (218, false),  // Magic Floor
        (221, false),  // Mapper 221
        (222, true),   // Mapper 222 A12 IRQ
        (224, true),   // Mapper 224 MMC3 A12 IRQ
        (226, false),  // Mapper 226
        (227, false),  // Mapper 227
        (228, false),  // Mapper 228
        (225, false),  // Mapper 225
        (229, false),  // Mapper 229
        (231, false),  // Mapper 231
        (232, false),  // Mapper 232
        (238, true),   // Mapper 238 MMC3 A12 IRQ
        (239, false),  // Mapper 239
        (240, false),  // Mapper 240
        (241, false),  // Mapper 241
        (242, false),  // Mapper 242
        (244, false),  // Mapper 244
        (245, true),   // Mapper 245 MMC3 A12 IRQ
        (249, true),   // Mapper 249 MMC3 A12 IRQ
        (250, true),   // Mapper 250 MMC3 A12 IRQ
        (246, false),  // Mapper 246
        (253, false),  // Mapper 253 IRQ is CPU-clocked, not PPU-bus-clocked
        (254, true),   // Mapper 254 MMC3 A12 IRQ
        (255, false),  // Mapper 255
        (184, false),  // Sunsoft 184
        (4, true),     // MMC3
        (5, true),     // MMC5
        (9, true),     // MMC2
        (10, true),    // MMC4
    ];
    for (num, expected) in cases {
        let submapper = if num == 34 { 2 } else { 0 };
        let m = Mapper::new(num, 2, 1, mir, submapper).expect("construct mapper");
        assert_eq!(m.watches_ppu_bus(), expected, "mapper {num}");
    }
}

#[test]
fn clocks_cpu_matches_cpu_clock_overrides() {
    let mir = Mirroring::Horizontal;
    let cases = [
        (0u16, false), // NROM
        (1, false),    // MMC1
        (2, false),    // UNROM
        (3, false),    // CNROM
        (4, false),    // MMC3 uses PPU A12 edges
        (5, false),    // MMC5 currently clocks from PPU nametable fetches
        (6, true),     // FFE IRQ counter clocks per CPU cycle
        (7, false),    // AxROM
        (8, false),    // Mapper 8
        (9, false),    // MMC2 CHR latch watches PPU bus
        (10, false),   // MMC4 CHR latch watches PPU bus
        (11, false),   // ColorDreams
        (12, false),   // Mapper 12 uses PPU A12 edges
        (13, false),   // CPROM
        (14, false),   // Mapper 14 uses PPU A12 edges in MMC3 mode
        (15, false),   // 100-in-1 multicart
        (16, true),    // Bandai FCG/LZ93D50 IRQ counter clocks per CPU cycle
        (17, true),    // FFE IRQ counter clocks per CPU cycle
        (18, true),    // Jaleco SS88006 IRQ counter clocks per CPU cycle
        (19, true),    // Namco 163 IRQ + expansion audio clock per CPU cycle
        (21, true),    // VRC4 IRQ counter clocks per CPU cycle
        (22, false),   // VRC2a has no IRQ
        (23, true),    // Ambiguous VRC2/VRC4 mapper defaults to VRC4-compatible IRQs
        (24, true),    // VRC6 IRQ + expansion audio clock per CPU cycle
        (26, true),    // VRC6 IRQ + expansion audio clock per CPU cycle
        (28, false),   // Action 53
        (29, false),   // Sealie Computing
        (31, false),   // Mapper 31
        (32, false),   // Irem G-101
        (33, false),   // Taito TC0190
        (34, false),   // BNROM
        (35, false),   // Mapper 35 uses PPU A12 edges
        (36, false),   // TXC/Micro Genius simplified mapper
        (37, false),   // Mapper 37 uses PPU A12 edges
        (39, false),   // Mapper 39
        (40, true),    // Mapper 40 IRQ counter clocks per CPU cycle
        (42, true),    // Mapper 42 IRQ counter clocks per CPU cycle
        (43, true),    // Mapper 43 IRQ counter clocks per CPU cycle
        (44, false),   // Mapper 44 uses PPU A12 edges
        (45, false),   // Mapper 45 uses PPU A12 edges
        (47, false),   // Mapper 47 uses PPU A12 edges
        (48, false),   // Mapper 48 IRQ is HBlank-clocked
        (49, false),   // Mapper 49 uses PPU A12 edges
        (25, true),    // VRC4 IRQ counter clocks per CPU cycle
        (52, false),   // Mapper 52 uses PPU A12 edges
        (53, false),   // BMC SuperVision 16-in-1
        (56, true),    // Kaiser KS202 IRQ counter clocks per CPU cycle
        (66, false),   // GxROM
        (67, true),    // Sunsoft-3 IRQ counter clocks per CPU cycle
        (68, false),   // Sunsoft-4 has no CPU-cycle IRQ hook
        (69, true),    // FME-7 IRQ + expansion audio clock per CPU cycle
        (41, false),   // Caltron 6-in-1
        (46, false),   // Color Dreams 46
        (50, true),    // Mapper 50 IRQ counter clocks per CPU cycle
        (51, false),   // Mapper 51
        (57, false),   // Mapper 57
        (58, false),   // Mapper 58
        (59, false),   // Mapper 59
        (60, false),   // Mapper 60
        (61, false),   // Mapper 61
        (62, false),   // Mapper 62
        (63, false),   // Mapper 63
        (64, true),    // Tengen RAMBO-1 can use CPU-cycle IRQ mode
        (65, true),    // Irem H3001 IRQ counter clocks per CPU cycle
        (70, false),   // Bandai 74161/7432
        (71, false),   // Codemasters
        (72, false),   // Mapper 72
        (73, true),    // VRC3 IRQ counter clocks per CPU cycle
        (75, false),   // VRC1
        (76, false),   // Mapper 76 uses PPU A12 edges
        (77, false),   // Irem LROG017
        (78, false),   // Jaleco JF-16
        (79, false),   // Mapper 79
        (80, false),   // Taito X1-005
        (81, false),   // Mapper 81
        (82, false),   // Taito X1-017
        (83, true),    // Mapper 83 IRQ counter clocks per CPU cycle
        (85, true),    // VRC7 IRQ + expansion audio clock per CPU cycle
        (87, false),   // Jaleco JF-xx
        (88, false),   // Namco 118
        (90, true),    // JY ASIC can clock IRQs from CPU cycles
        (91, false),   // Mapper 91 IRQ is HBlank-clocked
        (92, false),   // Mapper 92
        (96, false),   // Mapper 96 PPU nametable latch has no CPU clock
        (95, false),   // Namco 108 mapper 95
        (99, false),   // VS UniSystem updates on $4016 writes, not CPU clock
        (101, false),  // Jaleco JF-xx ordered bits
        (103, false),  // Mapper 103
        (104, false),  // Mapper 104
        (105, true),   // Mapper 105 NWC timer clocks per CPU cycle
        (106, true),   // Mapper 106 IRQ counter clocks per CPU cycle
        (108, false),  // Mapper 108
        (111, false),  // Cheapocabra / GTROM
        (114, false),  // Mapper 114 uses PPU A12 edges
        (115, false),  // Mapper 115 uses PPU A12 edges
        (117, false),  // Mapper 117 uses PPU A12 edges
        (118, false),  // Mapper 118 uses PPU A12 edges
        (119, false),  // Mapper 119 uses PPU A12 edges
        (120, false),  // Mapper 120
        (121, false),  // Mapper 121 uses PPU A12 edges
        (122, false),  // Mapper 122
        (126, false),  // Mapper 126 uses PPU A12 edges
        (128, false),  // Mapper 128
        (132, false),  // TXC 22211A
        (133, false),  // Sachen SA72008
        (134, false),  // Mapper 134 uses PPU A12 edges
        (136, false),  // Sachen JV001
        (137, false),  // Sachen 8259D
        (142, true),   // Mapper 142 IRQ counter clocks per CPU cycle
        (144, false),  // Mapper 144 ColorDreams variant
        (141, false),  // Sachen 8259A
        (147, false),  // Sachen JV001
        (146, false),  // Sachen SA016-1M
        (148, false),  // Sachen SA0037
        (149, false),  // Sachen SA0036
        (150, false),  // Sachen 74LS374N
        (243, false),  // Sachen 74LS374N
        (156, false),  // Mapper 156
        (162, false),  // Waixing FS304
        (163, false),  // Nanjing FC-001
        (158, true),   // Mapper 158 RAMBO-1 can use CPU-cycle IRQ mode
        (159, true),   // Bandai LZ93D50 IRQ counter clocks per CPU cycle
        (165, false),  // Mapper 165 uses PPU A12/CHR latch
        (166, false),  // Subor 166
        (167, false),  // Subor 167
        (112, false),  // NTDEC ASDER
        (116, false),  // Mapper 116 uses PPU A12 edges only in MMC3 mode
        (151, false),  // Mapper 151
        (153, true),   // Bandai FCG IRQ counter clocks per CPU cycle
        (154, false),  // Namco 108 mapper 154
        (155, false),  // MMC1 mapper 155
        (168, true),   // Racermate IRQ counter clocks per CPU cycle
        (170, false),  // Mapper 170
        (171, false),  // Kaiser KS-7058
        (172, false),  // TXC 22211B
        (173, false),  // TXC 22211C
        (152, false),  // Bandai 74161/7432
        (174, false),  // Mapper 174
        (175, false),  // Mapper 175
        (177, false),  // Mapper 177
        (178, false),  // Waixing FS305/NJ0430
        (181, false),  // Mapper 181 CNROM protection
        (182, false),  // Mapper 182 uses PPU A12 edges
        (183, true),   // Mapper 183 IRQ counter clocks per CPU cycle
        (186, false),  // Mapper 186 Family Study Box
        (188, false),  // Mapper 188
        (230, false),  // Mapper 230
        (233, false),  // Mapper 233
        (234, false),  // Mapper 234
        (235, false),  // Mapper 235
        (236, false),  // Mapper 236
        (237, false),  // Mapper 237
        (238, false),  // Mapper 238 uses PPU A12 edges
        (187, false),  // Mapper 187 uses PPU A12 edges
        (190, false),  // Mapper 190
        (191, false),  // Mapper 191 uses PPU A12 edges
        (192, false),  // Mapper 192 uses PPU A12 edges
        (195, false),  // Mapper 195 uses PPU A12 edges
        (196, false),  // Mapper 196 uses PPU A12 edges
        (197, false),  // Mapper 197 uses PPU A12 edges
        (198, false),  // Mapper 198 uses PPU A12 edges
        (199, false),  // Mapper 199 uses PPU A12 edges
        (200, false),  // Mapper 200
        (201, false),  // Mapper 201
        (202, false),  // Mapper 202
        (204, false),  // Mapper 204
        (205, false),  // Mapper 205 uses PPU A12 edges
        (206, false),  // Namco 108 mapper 206
        (207, false),  // Taito X1-005 mapper 207
        (208, false),  // Mapper 208 uses PPU A12 edges
        (209, true),   // JY ASIC can clock IRQs from CPU cycles
        (210, false),  // Namcot 175/340
        (211, true),   // JY ASIC can clock IRQs from CPU cycles
        (212, false),  // Mapper 212
        (213, false),  // Mapper 213
        (214, false),  // Mapper 214
        (215, false),  // Mapper 215 uses PPU A12 edges
        (216, false),  // Mapper 216
        (217, false),  // Mapper 217
        (218, false),  // Magic Floor
        (221, false),  // Mapper 221
        (222, false),  // Mapper 222 uses PPU A12 edges
        (224, false),  // Mapper 224 uses PPU A12 edges
        (226, false),  // Mapper 226
        (227, false),  // Mapper 227
        (228, false),  // Mapper 228
        (225, false),  // Mapper 225
        (229, false),  // Mapper 229
        (231, false),  // Mapper 231
        (232, false),  // Mapper 232
        (239, false),  // Mapper 239
        (240, false),  // Mapper 240
        (241, false),  // Mapper 241
        (242, false),  // Mapper 242
        (244, false),  // Mapper 244
        (245, false),  // Mapper 245 uses PPU A12 edges
        (249, false),  // Mapper 249 uses PPU A12 edges
        (250, false),  // Mapper 250 uses PPU A12 edges
        (246, false),  // Mapper 246
        (253, true),   // Mapper 253 IRQ counter clocks per CPU cycle
        (254, false),  // Mapper 254 uses PPU A12 edges
        (255, false),  // Mapper 255
        (184, false),  // Sunsoft 184
    ];
    for (num, expected) in cases {
        let submapper = if num == 34 { 2 } else { 0 };
        let m = Mapper::new(num, 2, 1, mir, submapper).expect("construct mapper");
        assert_eq!(m.clocks_cpu(), expected, "mapper {num}");
    }
}

#[test]
fn clocks_hblank_matches_hblank_clock_overrides() {
    let mir = Mirroring::Horizontal;
    let cases = [
        (0u16, false),
        (1, false),
        (2, false),
        (3, false),
        (4, false),
        (5, false),
        (6, false),
        (7, false),
        (8, false),
        (9, false),
        (10, false),
        (11, false),
        (12, false),
        (13, false),
        (14, false),
        (15, false),
        (16, false),
        (17, false),
        (18, false),
        (19, false),
        (21, false),
        (22, false),
        (23, false),
        (24, false),
        (26, false),
        (28, false),
        (29, false),
        (31, false),
        (32, false),
        (33, false),
        (34, false),
        (35, false),
        (36, false),
        (38, false),
        (37, false),
        (39, false),
        (40, false),
        (41, false),
        (42, false),
        (43, false),
        (44, false),
        (45, false),
        (46, false),
        (47, false),
        (48, true),
        (49, false),
        (50, false),
        (51, false),
        (52, false),
        (53, false),
        (56, false),
        (57, false),
        (58, false),
        (59, false),
        (60, false),
        (61, false),
        (62, false),
        (63, false),
        (64, false),
        (65, false),
        (66, false),
        (67, false),
        (68, false),
        (69, false),
        (70, false),
        (71, false),
        (72, false),
        (73, false),
        (75, false),
        (76, false),
        (77, false),
        (78, false),
        (79, false),
        (80, false),
        (81, false),
        (82, false),
        (83, false),
        (85, false),
        (86, false),
        (87, false),
        (88, false),
        (89, false),
        (90, true),
        (91, true),
        (92, false),
        (93, false),
        (94, false),
        (95, false),
        (96, false),
        (97, false),
        (99, false),
        (101, false),
        (103, false),
        (104, false),
        (105, false),
        (106, false),
        (107, false),
        (108, false),
        (111, false),
        (112, false),
        (113, false),
        (114, false),
        (115, false),
        (116, false),
        (117, false),
        (118, false),
        (119, false),
        (120, false),
        (121, false),
        (122, false),
        (126, false),
        (128, false),
        (132, false),
        (133, false),
        (134, false),
        (136, false),
        (137, false),
        (144, false),
        (141, false),
        (147, false),
        (146, false),
        (148, false),
        (149, false),
        (150, false),
        (156, false),
        (162, true),
        (163, true),
        (158, false),
        (159, false),
        (165, false),
        (140, false),
        (142, false),
        (151, false),
        (152, false),
        (153, false),
        (154, false),
        (155, false),
        (166, false),
        (167, false),
        (168, false),
        (170, false),
        (171, false),
        (172, false),
        (173, false),
        (174, false),
        (175, false),
        (177, false),
        (178, false),
        (180, false),
        (181, false),
        (182, false),
        (183, false),
        (184, false),
        (186, false),
        (187, false),
        (188, false),
        (190, false),
        (191, false),
        (192, false),
        (194, false),
        (195, false),
        (196, false),
        (197, false),
        (198, false),
        (199, false),
        (200, false),
        (201, false),
        (202, false),
        (203, false),
        (204, false),
        (205, false),
        (206, false),
        (207, false),
        (208, false),
        (209, true),
        (210, false),
        (211, true),
        (212, false),
        (213, false),
        (214, false),
        (215, false),
        (216, false),
        (217, false),
        (218, false),
        (221, false),
        (222, false),
        (224, false),
        (225, false),
        (226, false),
        (227, false),
        (228, false),
        (229, false),
        (230, false),
        (231, false),
        (232, false),
        (233, false),
        (234, false),
        (235, false),
        (236, false),
        (237, false),
        (238, false),
        (239, false),
        (240, false),
        (241, false),
        (242, false),
        (243, false),
        (244, false),
        (245, false),
        (249, false),
        (250, false),
        (246, false),
        (253, false),
        (254, false),
        (255, false),
    ];
    for (num, expected) in cases {
        let submapper = if num == 34 { 2 } else { 0 };
        let m = Mapper::new(num, 2, 1, mir, submapper).expect("construct mapper");
        assert_eq!(m.clocks_hblank(), expected, "mapper {num}");
    }
}

#[test]
fn mapper34_bnrom_switches_32k_prg_bank() {
    let mut m = Mapper::new(34, 8, 0, Mirroring::Horizontal, 2).expect("bnrom");
    assert_eq!(m.prg_index(0x8000), 0);
    assert_eq!(m.prg_index(0xFFFF), 0x7FFF);
    m.write_register(0x8000, 2);
    assert_eq!(m.prg_index(0x8000), 2 * 0x8000);
    assert_eq!(m.prg_index(0xC123), 2 * 0x8000 + 0x4123);
    m.write_register(0x8000, 9);
    assert_eq!(m.prg_index(0x8000), 0x8000);
}

#[test]
fn mapper34_nina01_switches_prg_and_4k_chr_banks() {
    let mut m = Mapper::new(34, 4, 2, Mirroring::Horizontal, 0).expect("nina-001");
    assert_eq!(m.prg_index(0x8000), 0);
    assert_eq!(m.chr_index(0x0000), 0);
    assert_eq!(m.chr_index(0x1000), 0x1000);

    assert!(m.write_low_register(0x7FFD, 0x03));
    assert!(m.write_low_register(0x7FFE, 0x04));
    assert!(m.write_low_register(0x7FFF, 0x15));
    assert!(!m.write_low_register(0x7FFC, 0x02));

    assert_eq!(m.prg_index(0x8000), 0x8000);
    assert_eq!(m.prg_index(0xC123), 0xC123);
    assert_eq!(m.chr_index(0x0007), 4 * 0x1000 + 7);
    assert_eq!(m.chr_index(0x1007), 5 * 0x1000 + 7);
}

#[test]
fn mapper8_switches_low_prg16_and_chr8_from_latch() {
    let mut m8 = Mapper::new(8, 8, 4, Mirroring::Vertical, 0).expect("mapper 8");
    assert_eq!(m8.prg_index(0x8004), 0x0004);
    assert_eq!(m8.prg_index(0xC004), 0x4000 + 4);

    m8.write_register(0x8000, 0x1B);
    assert_eq!(m8.prg_index(0x8004), 3 * 0x4000 + 4);
    assert_eq!(m8.prg_index(0xC004), 0x4000 + 4);
    assert_eq!(m8.chr_index(0x0010), 3 * 0x2000 + 0x10);
}

#[test]
fn mapper28_action53_selects_prg_modes_chr_and_mirroring() {
    let mut m28 = Mapper::new(28, 128, 4, Mirroring::Vertical, 0).expect("mapper 28");
    assert_eq!(m28.prg_index(0x8004), 126 * 0x4000 + 4);
    assert_eq!(m28.prg_index(0xC004), 127 * 0x4000 + 4);
    assert_eq!(m28.mirroring(), Mirroring::SingleScreenLow);

    m28.write_expansion(0x5000, 0x81);
    m28.write_register(0x8000, 0x02);
    assert_eq!(m28.prg_index(0x8004), 4 * 0x4000 + 4);
    assert_eq!(m28.prg_index(0xC004), 5 * 0x4000 + 4);

    m28.write_expansion(0x5000, 0x00);
    m28.write_register(0x8000, 0x13);
    assert_eq!(m28.chr_index(0x0010), 3 * 0x2000 + 0x10);
    assert_eq!(m28.mirroring(), Mirroring::SingleScreenHigh);

    m28.write_expansion(0x5000, 0x80);
    m28.write_register(0x8000, 0x1A);
    assert_eq!(m28.mirroring(), Mirroring::Vertical);
    m28.write_expansion(0x5000, 0x01);
    m28.write_register(0x8000, 0x17);
    assert_eq!(m28.prg_index(0x8004), 4 * 0x4000 + 4);
    assert_eq!(m28.prg_index(0xC004), 7 * 0x4000 + 4);

    m28.reset(true);
    assert_eq!(m28.prg_index(0x8004), 126 * 0x4000 + 4);
    assert_eq!(m28.prg_index(0xC004), 127 * 0x4000 + 4);
}

#[test]
fn mapper29_sealie_computing_switches_prg16_and_chr8() {
    let mut m29 = Mapper::new(29, 8, 0, Mirroring::Vertical, 0).expect("mapper 29");
    assert_eq!(m29.prg_index(0x8004), 0x0004);
    assert_eq!(m29.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m29.chr_index(0x1010), 0x1010);
    assert_eq!(m29.mirroring(), Mirroring::Vertical);

    m29.write_register(0x8000, 0x17);
    assert_eq!(m29.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m29.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m29.chr_index(0x1010), 3 * 0x2000 + 0x1010);
}

#[test]
fn mapper51_switches_multicart_bank_mode_and_low_prg_rom() {
    let mut m51 = Mapper::new(51, 128, 0, Mirroring::Vertical, 0).expect("mapper 51");
    assert_eq!(m51.prg_index(0x8004), 0x0004);
    assert_eq!(m51.prg_index(0xC004), 0x4000 + 4);
    assert_eq!(m51.low_prg_index(0x6004), Some(0x23 * 0x2000 + 4));
    assert_eq!(m51.mirroring(), Mirroring::Vertical);

    m51.write_register(0x8000, 0x05);
    assert_eq!(m51.prg_index(0x8004), 10 * 0x4000 + 4);
    assert_eq!(m51.prg_index(0xC004), 11 * 0x4000 + 4);
    assert_eq!(m51.low_prg_index(0x6004), Some(0x37 * 0x2000 + 4));

    assert!(m51.write_low_register(0x6000, 0x10));
    assert_eq!(m51.prg_index(0x8004), 11 * 0x4000 + 4);
    assert_eq!(m51.prg_index(0xC004), 15 * 0x4000 + 4);
    assert_eq!(m51.low_prg_index(0x6004), Some(0x3F * 0x2000 + 4));
    assert_eq!(m51.mirroring(), Mirroring::Vertical);

    m51.write_register(0xC000, 0x1D);
    assert_eq!(m51.prg_index(0x8004), 27 * 0x4000 + 4);
    assert_eq!(m51.prg_index(0xC004), 31 * 0x4000 + 4);
    assert!(m51.write_low_register(0x7000, 0x12));
    assert_eq!(m51.mirroring(), Mirroring::Horizontal);

    m51.reset(true);
    assert_eq!(m51.prg_index(0x8004), 0x0004);
    assert_eq!(m51.prg_index(0xC004), 0x4000 + 4);
    assert_eq!(m51.mirroring(), Mirroring::Vertical);
}

#[test]
fn mapper81_uses_address_latch_for_prg16_and_data_latch_for_chr8() {
    let mut m81 = Mapper::new(81, 8, 4, Mirroring::Vertical, 0).expect("mapper 81");
    assert_eq!(m81.prg_index(0x8004), 0x0004);
    assert_eq!(m81.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m81.chr_index(0x1010), 0x1010);

    m81.write_register(0x800C, 0x03);
    assert_eq!(m81.prg_index(0x8004), 3 * 0x4000 + 4);
    assert_eq!(m81.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m81.chr_index(0x1010), 3 * 0x2000 + 0x1010);
    assert_eq!(m81.mirroring(), Mirroring::Vertical);
}

#[test]
fn mapper104_switches_inner_and_outer_prg16_banks() {
    let mut m104 = Mapper::new(104, 128, 0, Mirroring::Vertical, 0).expect("mapper 104");
    assert_eq!(m104.prg_index(0x8004), 0x0004);
    assert_eq!(m104.prg_index(0xC004), 0x0F * 0x4000 + 4);
    assert_eq!(m104.chr_index(0x1010), 0x1010);
    assert_eq!(m104.mirroring(), Mirroring::Vertical);

    m104.write_register(0xC000, 0x06);
    assert_eq!(m104.prg_index(0x8004), 0x06 * 0x4000 + 4);
    assert_eq!(m104.prg_index(0xC004), 0x0F * 0x4000 + 4);

    m104.write_register(0x8000, 0x0B);
    assert_eq!(m104.prg_index(0x8004), 0x36 * 0x4000 + 4);
    assert_eq!(m104.prg_index(0xC004), 0x3F * 0x4000 + 4);
    m104.write_register(0x9000, 0x04);
    assert_eq!(m104.prg_index(0x8004), 0x36 * 0x4000 + 4);

    m104.reset(true);
    assert_eq!(m104.prg_index(0x8004), 0x0004);
    assert_eq!(m104.prg_index(0xC004), 0x0F * 0x4000 + 4);
}

#[test]
fn txc_mapper132_uses_txc_accumulator_for_prg_chr_and_open_bus_reads() {
    let mut m = Mapper::new(132, 8, 4, Mirroring::Horizontal, 0).expect("mapper 132");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.chr_index(0x1004), 0x1004);
    assert_eq!(m.peek_expansion_with_open_bus(0x4100, 0xA0), Some(0xA0));

    m.write_expansion(0x4102, 0x0D);
    m.write_expansion(0x4100, 0);
    assert_eq!(m.read_expansion_with_open_bus(0x4100, 0xA0), Some(0xAD));
    m.write_register(0x8000, 0);
    assert_eq!(m.prg_index(0x8004), 1 * 0x8000 + 4);
    assert_eq!(m.chr_index(0x1004), 1 * 0x2000 + 0x1004);
    assert_eq!(m.peek_expansion_with_open_bus(0x4101, 0xA5), None);
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.reset(true);
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.chr_index(0x1004), 0x1004);
}

#[test]
fn txc_sachen_variants_apply_board_specific_bit_permutations() {
    let mut m136 = Mapper::new(136, 8, 64, Mirroring::Horizontal, 0).expect("mapper 136");
    m136.write_expansion(0x4101, 0);
    m136.write_expansion(0x4102, 0x25);
    m136.write_expansion(0x4100, 0);
    m136.write_register(0x8000, 0);
    assert_eq!(m136.prg_index(0x8004), 0x0004);
    assert_eq!(m136.chr_index(0x1004), 0x25 * 0x2000 + 0x1004);
    assert_eq!(m136.read_expansion_with_open_bus(0x4100, 0xC0), Some(0xE5));

    let mut m147 = Mapper::new(147, 8, 16, Mirroring::Horizontal, 0).expect("mapper 147");
    m147.write_expansion(0x4101, 0);
    m147.write_expansion(0x4102, 0x8F);
    m147.write_expansion(0x4100, 0);
    m147.write_register(0x8000, 0);
    assert_eq!(m147.prg_index(0x8004), 3 * 0x8000 + 4);
    assert_eq!(m147.chr_index(0x1004), 1 * 0x2000 + 0x1004);
    assert_eq!(m147.read_expansion_with_open_bus(0x4100, 0xA0), Some(0x8F));
}

#[test]
fn txc_mapper172_controls_chr_and_mirroring_from_jv001_invert_flag() {
    let mut m = Mapper::new(172, 4, 64, Mirroring::Horizontal, 0).expect("mapper 172");
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.write_expansion(0x4101, 0);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    m.write_expansion(0x4102, 0x28);
    m.write_expansion(0x4100, 0);
    m.write_register(0x8000, 0);
    assert_eq!(m.prg_index(0xC004), 0x4004);
    assert_eq!(m.chr_index(0x1004), 0x05 * 0x2000 + 0x1004);
    assert_eq!(m.read_expansion_with_open_bus(0x4100, 0xC0), Some(0xE8));
}

#[test]
fn txc_mapper173_uses_y_flag_in_chr_selection() {
    let mut m = Mapper::new(173, 4, 8, Mirroring::Horizontal, 0).expect("mapper 173");
    m.write_expansion(0x4102, 0x0D);
    m.write_expansion(0x4100, 0);
    m.write_register(0x8000, 0);
    assert_eq!(m.chr_index(0x1004), 3 * 0x2000 + 0x1004);
    assert_eq!(m.read_expansion_with_open_bus(0x4100, 0xA0), Some(0xAD));
    assert_eq!(m.chr_index(0x1004), 3 * 0x2000 + 0x1004);

    let mut small_chr = Mapper::new(173, 4, 1, Mirroring::Horizontal, 0).expect("mapper 173");
    small_chr.write_expansion(0x4102, 0x0D);
    small_chr.write_expansion(0x4100, 0);
    small_chr.write_register(0x8000, 0);
    assert_eq!(small_chr.chr_index(0x1004), 0x1004);
}

#[test]
fn mapper142_switches_prg8_and_low_prg_rom_window() {
    let mut m = Mapper::new(142, 16, 0, Mirroring::Vertical, 0).expect("mapper 142");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.prg_index(0xA004), 0x0004);
    assert_eq!(m.prg_index(0xC004), 0x0004);
    assert_eq!(m.prg_index(0xE004), 31 * 0x2000 + 4);
    assert_eq!(m.low_prg_index(0x6004), Some(0x0004));
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.write_register(0xE000, 0x01);
    m.write_register(0xF000, 0x05);
    assert_eq!(m.prg_index(0x8004), 5 * 0x2000 + 4);

    m.write_register(0xE000, 0x02);
    m.write_register(0xF001, 0x16);
    assert_eq!(m.prg_index(0xA004), 22 * 0x2000 + 4);

    m.write_register(0xE000, 0x03);
    m.write_register(0xF002, 0x17);
    assert_eq!(m.prg_index(0xC004), 23 * 0x2000 + 4);

    m.write_register(0xE000, 0x04);
    m.write_register(0xF800, 0x09);
    assert_eq!(m.low_prg_index(0x6004), Some(9 * 0x2000 + 4));

    m.write_register(0xE000, 0x00);
    m.write_register(0xF000, 0x13);
    assert_eq!(m.prg_index(0x8004), 0x15 * 0x2000 + 4);
}

#[test]
fn mapper142_cpu_irq_reloads_and_acks() {
    let mut m = Mapper::new(142, 16, 0, Mirroring::Horizontal, 0).expect("mapper 142");
    m.write_register(0x8000, 0x0D);
    m.write_register(0x9000, 0x0C);
    m.write_register(0xA000, 0x0B);
    m.write_register(0xB000, 0x0A);
    m.write_register(0xC000, 0x02);

    for _ in 0..(0xFFFF - 0xABCD - 1) {
        m.cpu_clock();
    }
    assert!(!m.irq());
    m.cpu_clock();
    assert!(m.irq());

    m.write_register(0xD000, 0x00);
    assert!(!m.irq());
    m.cpu_clock();
    assert!(!m.irq());

    m.write_register(0xC000, 0x00);
    for _ in 0..4 {
        m.cpu_clock();
    }
    assert!(!m.irq());
}

#[test]
fn mapper31_pages_eight_4k_prg_windows_from_expansion_regs() {
    let mut m31 = Mapper::new(31, 64, 0, Mirroring::Vertical, 0).expect("mapper 31");
    assert_eq!(m31.prg_index(0x8004), 0x0004);
    assert_eq!(m31.prg_index(0xF004), 0xFF * 0x1000 + 4);

    m31.write_expansion(0x5000, 0x12);
    m31.write_expansion(0x5007, 0x34);
    assert_eq!(m31.prg_index(0x8004), 0x12 * 0x1000 + 4);
    assert_eq!(m31.prg_index(0xF004), 0x34 * 0x1000 + 4);
    assert_eq!(m31.chr_index(0x1010), 0x1010);
}

#[test]
fn cprom_switches_only_high_4k_chr_ram() {
    let mut m = Mapper::new(13, 2, 0, Mirroring::Horizontal, 0).expect("cprom");
    assert_eq!(m.prg_index(0xBEEF), 0x3EEF);
    assert_eq!(m.chr_index(0x0008), 0x0008);
    assert_eq!(m.chr_index(0x1008), 0x0008);
    m.write_register(0x8000, 3);
    assert_eq!(m.chr_index(0x0008), 0x0008);
    assert_eq!(m.chr_index(0x1008), 3 * 0x1000 + 8);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
}

#[test]
fn ffe_mapper6_latch_mode_switches_prg16_chr8_mirroring_and_irq() {
    let mut m = Mapper::new(6, 16, 4, Mirroring::Horizontal, 0).expect("mapper 6");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x1004);
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.write_register(0x8000, 0x2F);
    assert_eq!(m.prg_index(0x8004), 11 * 0x4000 + 4);
    assert_eq!(m.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 3 * 0x2000 + 0x1004);

    m.write_expansion(0x42FE, 0x10);
    assert_eq!(m.mirroring(), Mirroring::SingleScreenHigh);
    m.write_expansion(0x42FF, 0x00);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
    m.write_expansion(0x42FF, 0x10);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);

    m.write_expansion(0x4502, 0xFE);
    m.write_expansion(0x4503, 0xFF);
    assert!(!m.irq());
    m.cpu_clock();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(m.irq());
    m.clear_irq();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(!m.irq());
}

#[test]
fn ffe_mapper17_full_mode_switches_8k_prg_and_1k_chr_registers() {
    let mut m = Mapper::new(17, 16, 16, Mirroring::Horizontal, 0).expect("mapper 17");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.prg_index(0xA004), 0x0004);
    assert_eq!(m.prg_index(0xC004), 0x0004);
    assert_eq!(m.prg_index(0xE004), 31 * 0x2000 + 4);

    m.write_expansion(0x4504, 3);
    m.write_expansion(0x4505, 4);
    m.write_expansion(0x4506, 5);
    m.write_expansion(0x4507, 6);
    assert_eq!(m.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m.prg_index(0xE004), 6 * 0x2000 + 4);

    m.write_expansion(0x4510, 0x12);
    m.write_expansion(0x4517, 0x1F);
    assert_eq!(m.chr_index(0x0004), 0x12 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1C04), 0x1F * 0x0400 + 4);

    m.write_register(0x8000, 0x2F);
    assert_eq!(m.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m.chr_index(0x0004), 0x12 * 0x0400 + 4);
}

#[test]
fn bandai_mapper16_switches_banks_mirroring_irq_and_eeprom_bit() {
    let mut m = Mapper::new(16, 32, 16, Mirroring::Horizontal, 0).expect("mapper 16");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.prg_index(0xC004), 0x0F * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x0004);
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    assert!(m.write_low_register(0x6000, 0x12));
    assert!(m.write_low_register(0x6007, 0x1F));
    assert!(m.write_low_register(0x6008, 0x0B));
    assert_eq!(m.chr_index(0x0004), 0x12 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1C04), 0x1F * 0x0400 + 4);
    assert_eq!(m.prg_index(0x8004), 0x1B * 0x4000 + 4);
    assert_eq!(m.prg_index(0xC004), 31 * 0x4000 + 4);

    assert!(m.write_low_register(0x6009, 0x03));
    assert_eq!(m.mirroring(), Mirroring::SingleScreenHigh);
    assert!(m.write_low_register(0x6009, 0x01));
    assert_eq!(m.mirroring(), Mirroring::Horizontal);

    assert_eq!(
        m.peek_low_register_with_open_bus(0x6000, 0x55, 0xFF),
        Some(0xEF)
    );
    m.write_low_register(0x600D, 0x60);
    m.write_low_register(0x600D, 0x20);
    m.write_low_register(0x600D, 0x60);
    assert_eq!(
        m.read_low_register_with_open_bus(0x6000, 0x55, 0x00),
        Some(0x10)
    );

    m.write_low_register(0x600B, 0x02);
    m.write_low_register(0x600C, 0x00);
    m.write_low_register(0x600A, 0x01);
    m.cpu_clock();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(m.irq());
}

#[test]
fn bandai_mapper16_submapper_write_windows_and_fcg_irq_counter() {
    let mut fcg = Mapper::new(16, 32, 16, Mirroring::Horizontal, 4).expect("mapper 16 sub4");
    fcg.write_register(0x8008, 0x03);
    assert_eq!(fcg.prg_index(0x8004), 0x0004);
    assert!(fcg.write_low_register(0x6008, 0x03));
    assert_eq!(fcg.prg_index(0x8004), 3 * 0x4000 + 4);

    fcg.write_low_register(0x600B, 0x01);
    fcg.write_low_register(0x600C, 0x00);
    fcg.write_low_register(0x600A, 0x01);
    fcg.cpu_clock();
    assert!(!fcg.irq());
    fcg.cpu_clock();
    assert!(fcg.irq());

    let mut lz93d50 = Mapper::new(16, 32, 16, Mirroring::Horizontal, 5).expect("mapper 16 sub5");
    assert!(!lz93d50.write_low_register(0x6008, 0x04));
    assert_eq!(lz93d50.prg_index(0x8004), 0x0004);
    lz93d50.write_register(0x8008, 0x04);
    assert_eq!(lz93d50.prg_index(0x8004), 4 * 0x4000 + 4);
}

#[test]
fn bandai_mapper153_uses_chr_low_bits_as_prg_outer_and_gates_sram() {
    let mut mapper = Mapper::new(153, 32, 0, Mirroring::Horizontal, 0).expect("mapper 153");
    mapper.write_register(0x8000, 0x01);
    mapper.write_register(0x8008, 0x02);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x4000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x1F * 0x4000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x1004);

    let mut rom = vec![0u8; 16 + 32 * 0x4000];
    rom[0..4].copy_from_slice(b"NES\x1A");
    rom[4] = 32;
    rom[5] = 0;
    rom[6] = 0x90;
    rom[7] = 0x98;
    rom[10] = 0x07;
    for (i, byte) in rom[16..].iter_mut().enumerate() {
        *byte = (i as u8).wrapping_mul(3).wrapping_add(1);
    }
    let mut cart = Cartridge::from_bytes(&rom).expect("mapper 153 rom");
    assert!(!cart.cpu_write(0x6000, 0x5A));
    assert_eq!(cart.cpu_read_with_open_bus(0x6000, 0xA5), 0x5A);
    assert!(cart.cpu_write(0x800D, 0x00));
    assert!(!cart.cpu_write(0x6000, 0xC3));
    assert_eq!(cart.cpu_read_with_open_bus(0x6000, 0xA5), 0xA5);
    assert!(cart.cpu_write(0x800D, 0x20));
    assert!(!cart.cpu_write(0x6000, 0xC3));
    assert_eq!(cart.cpu_read_with_open_bus(0x6000, 0xA5), 0xC3);
}

#[test]
fn bandai_mapper159_uses_24c01_eeprom_and_high_register_window() {
    let mut mapper = Mapper::new(159, 32, 16, Mirroring::Horizontal, 0).expect("mapper 159");
    assert!(!mapper.write_low_register(0x6008, 0x04));
    assert_eq!(mapper.prg_index(0x8004), 0x0004);
    mapper.write_register(0x8008, 0x04);
    assert_eq!(mapper.prg_index(0x8004), 4 * 0x4000 + 4);
    assert_eq!(
        mapper.peek_low_register_with_open_bus(0x6000, 0x55, 0xFF),
        Some(0xEF)
    );

    mapper.write_register(0x800D, 0x60);
    mapper.write_register(0x800D, 0x20);
    mapper.write_register(0x800D, 0x60);
    assert_eq!(
        mapper.read_low_register_with_open_bus(0x6000, 0x55, 0x00),
        Some(0x10)
    );
}

#[test]
fn mapper168_racermate_switches_banks_and_clocks_irq() {
    let mut mapper = Mapper::new(168, 8, 0, Mirroring::Horizontal, 0).expect("mapper 168");
    assert!(mapper.clocks_cpu());
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    assert_eq!(mapper.prg_index(0x8004), 0x0004);
    assert_eq!(mapper.prg_index(0xC004), 7 * 0x4000 + 4);

    assert!(mapper.chr_write(0x1004, 0x34));
    assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x34));
    mapper.write_register(0xB000, 0xC5);
    assert_eq!(mapper.prg_index(0x8004), 3 * 0x4000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 5 * 0x1000 + 4);
    assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x00));

    mapper.write_register(0xC000, 0);
    for _ in 0..1023 {
        mapper.cpu_clock();
    }
    assert!(!mapper.irq());
    mapper.cpu_clock();
    assert!(mapper.irq());
    mapper.write_register(0xFFFF, 0);
    assert!(!mapper.irq());
}

#[test]
fn mapper15_selects_8k_multicart_modes() {
    let mut m = Mapper::new(15, 32, 0, Mirroring::Vertical, 0).expect("mapper 15");
    assert_eq!(m.prg_index(0x8000), 0);
    assert_eq!(m.prg_index(0xA000), 0x2000);
    assert_eq!(m.prg_index(0xC000), 0x4000);
    assert_eq!(m.prg_index(0xE000), 0x6000);

    m.write_register(0x8001, 0x43);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    assert_eq!(m.prg_index(0x8000), 0x86 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 0x87 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 0x8E * 0x2000);
    assert_eq!(m.prg_index(0xE000), 0x8F * 0x2000);

    m.write_register(0x8002, 0x81);
    assert_eq!(m.prg_index(0x8000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xE000), 3 * 0x2000);
}

#[test]
fn irem_g101_switches_prg_mode_and_1k_chr_pages() {
    let mut m = Mapper::new(32, 8, 8, Mirroring::Vertical, 0).expect("mapper 32");
    m.write_register(0x8000, 3);
    m.write_register(0xA000, 4);
    assert_eq!(m.prg_index(0x8000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 4 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
    assert_eq!(m.prg_index(0xE000), 15 * 0x2000);

    m.write_register(0x9000, 0x03);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    assert_eq!(m.prg_index(0x8000), 14 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 4 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 3 * 0x2000);

    m.write_register(0xB006, 9);
    assert_eq!(m.chr_index(0x1804), 9 * 0x0400 + 4);
}

#[test]
fn taito_tc0190_switches_8k_prg_and_mixed_chr_pages() {
    let mut m = Mapper::new(33, 8, 8, Mirroring::Vertical, 0).expect("mapper 33");
    m.write_register(0x8000, 0x45);
    m.write_register(0x8001, 6);
    m.write_register(0x8002, 7);
    m.write_register(0x8003, 8);
    m.write_register(0xA002, 9);

    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    assert_eq!(m.prg_index(0x8000), 5 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 6 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
    assert_eq!(m.chr_index(0x0004), 14 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x0404), 15 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x0804), 16 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1804), 9 * 0x0400 + 4);
}

#[test]
fn low_register_multicarts_follow_reference_windows() {
    let mut caltron = Mapper::new(41, 16, 16, Mirroring::Vertical, 0).expect("mapper 41");
    assert!(caltron.write_low_register(0x603C, 0));
    assert_eq!(caltron.mirroring(), Mirroring::Horizontal);
    assert_eq!(caltron.prg_index(0x8000), 4 * 0x8000);
    assert_eq!(caltron.chr_index(0x0004), 12 * 0x2000 + 4);
    caltron.write_register(0x8000, 2);
    assert_eq!(caltron.chr_index(0x0004), 14 * 0x2000 + 4);
    assert!(!caltron.write_low_register(0x6800, 0));

    let mut color46 = Mapper::new(46, 64, 128, Mirroring::Vertical, 0).expect("mapper 46");
    assert!(color46.write_low_register(0x6000, 0xA5));
    color46.write_register(0x8000, 0x71);
    assert_eq!(color46.prg_index(0x8000), 11 * 0x8000);
    assert_eq!(color46.chr_index(0x0004), 87 * 0x2000 + 4);
    assert_eq!(color46.mirroring(), Mirroring::Vertical);
}

#[test]
fn irq_mappers_follow_reference_clock_sources() {
    let mut m50 = Mapper::new(50, 16, 0, Mirroring::Horizontal, 0).expect("mapper 50");
    assert_eq!(m50.low_prg_index(0x6004), Some(0x0F * 0x2000 + 4));
    assert_eq!(m50.prg_index(0x8004), 0x08 * 0x2000 + 4);
    assert_eq!(m50.prg_index(0xA004), 0x09 * 0x2000 + 4);
    assert_eq!(m50.prg_index(0xE004), 0x0B * 0x2000 + 4);
    assert_eq!(m50.mirroring(), Mirroring::Horizontal);
    m50.write_expansion(0x4020, 0x0F);
    assert_eq!(m50.prg_index(0xC004), 0x0F * 0x2000 + 4);
    m50.write_expansion(0x4120, 0x01);
    for _ in 0..0x0FFF {
        m50.cpu_clock();
    }
    assert!(!m50.irq());
    m50.cpu_clock();
    assert!(m50.irq());
    m50.clear_irq();
    assert!(!m50.irq());
    m50.write_expansion(0x4120, 0x00);
    for _ in 0..0x1000 {
        m50.cpu_clock();
    }
    assert!(!m50.irq());

    let mut m117 = Mapper::new(117, 8, 8, Mirroring::Horizontal, 0).expect("mapper 117");
    assert!(m117.watches_ppu_bus());
    assert_eq!(m117.prg_index(0x8004), 12 * 0x2000 + 4);
    assert_eq!(m117.prg_index(0xE004), 15 * 0x2000 + 4);
    m117.write_register(0x8000, 3);
    m117.write_register(0xA004, 9);
    assert_eq!(m117.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m117.chr_index(0x1004), 9 * 0x0400 + 4);
    m117.write_register(0xD000, 0);
    assert_eq!(m117.mirroring(), Mirroring::Vertical);

    m117.write_register(0xC001, 2);
    m117.write_register(0xC003, 0);
    m117.write_register(0xE000, 1);
    m117.notify_a12(0x0000, 0);
    m117.notify_a12(0x1000, 12);
    assert!(!m117.irq());
    m117.notify_a12(0x0000, 15);
    m117.notify_a12(0x1000, 18);
    assert!(!m117.irq());
    m117.notify_a12(0x0000, 19);
    m117.notify_a12(0x1000, 31);
    assert!(m117.irq());
    m117.write_register(0xC002, 0);
    assert!(!m117.irq());
}

#[test]
fn additional_cpu_irq_mappers_follow_reference_bank_and_irq_rules() {
    let mut m40 = Mapper::new(40, 8, 0, Mirroring::Horizontal, 0).expect("mapper 40");
    assert_eq!(m40.low_prg_index(0x6004), Some(6 * 0x2000 + 4));
    assert_eq!(m40.prg_index(0x8004), 4 * 0x2000 + 4);
    assert_eq!(m40.prg_index(0xA004), 5 * 0x2000 + 4);
    assert_eq!(m40.prg_index(0xE004), 7 * 0x2000 + 4);
    m40.write_register(0xE000, 3);
    assert_eq!(m40.prg_index(0xC004), 3 * 0x2000 + 4);
    m40.write_register(0xA000, 0);
    for _ in 0..0x0FFF {
        m40.cpu_clock();
    }
    assert!(!m40.irq());
    m40.cpu_clock();
    assert!(m40.irq());
    m40.write_register(0x8000, 0);
    assert!(!m40.irq());

    let mut m42 = Mapper::new(42, 8, 8, Mirroring::Vertical, 0).expect("mapper 42");
    m42.write_register(0x8000, 4);
    m42.write_register(0xE000, 7);
    m42.write_register(0xE001, 0x08);
    assert_eq!(m42.low_prg_index(0x6004), Some(7 * 0x2000 + 4));
    assert_eq!(m42.prg_index(0x8004), 12 * 0x2000 + 4);
    assert_eq!(m42.prg_index(0xE004), 15 * 0x2000 + 4);
    assert_eq!(m42.chr_index(0x0004), 4 * 0x2000 + 4);
    assert_eq!(m42.mirroring(), Mirroring::Horizontal);
    m42.write_register(0xE002, 0x02);
    for _ in 0..0x5FFF {
        m42.cpu_clock();
    }
    assert!(!m42.irq());
    m42.cpu_clock();
    assert!(m42.irq());
    m42.write_register(0xE002, 0x00);
    assert!(!m42.irq());

    let mut m67 = Mapper::new(67, 8, 8, Mirroring::Horizontal, 0).expect("mapper 67");
    m67.write_register(0x8800, 2);
    m67.write_register(0x9800, 3);
    m67.write_register(0xF800, 5);
    m67.write_register(0xE800, 2);
    assert_eq!(m67.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m67.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m67.chr_index(0x0004), 2 * 0x0800 + 4);
    assert_eq!(m67.chr_index(0x0804), 3 * 0x0800 + 4);
    assert_eq!(m67.mirroring(), Mirroring::SingleScreenLow);
    m67.write_register(0xC000, 0x00);
    m67.write_register(0xC800, 0x01);
    m67.write_register(0xD800, 0x10);
    m67.cpu_clock();
    assert!(!m67.irq());
    m67.cpu_clock();
    assert!(m67.irq());

    let mut m73 = Mapper::new(73, 8, 0, Mirroring::Vertical, 0).expect("mapper 73");
    m73.write_register(0xF000, 6);
    assert_eq!(m73.prg_index(0x8004), 6 * 0x4000 + 4);
    assert_eq!(m73.prg_index(0xC004), 7 * 0x4000 + 4);
    m73.write_register(0x8000, 0x0E);
    m73.write_register(0x9000, 0x0F);
    m73.write_register(0xA000, 0x0F);
    m73.write_register(0xB000, 0x0F);
    m73.write_register(0xC000, 0x02);
    m73.cpu_clock();
    assert!(!m73.irq());
    m73.cpu_clock();
    assert!(m73.irq());
    m73.write_register(0xD000, 0);
    assert!(!m73.irq());
}

#[test]
fn jaleco_and_irem_irq_mappers_decode_nibbles_and_count_cpu_cycles() {
    let mut m18 = Mapper::new(18, 16, 16, Mirroring::Vertical, 0).expect("mapper 18");
    m18.write_register(0x8000, 0x03);
    m18.write_register(0x8001, 0x01);
    m18.write_register(0x8002, 0x04);
    m18.write_register(0x8003, 0x01);
    m18.write_register(0x9000, 0x05);
    m18.write_register(0x9001, 0x01);
    assert_eq!(m18.prg_index(0x8004), 0x13 * 0x2000 + 4);
    assert_eq!(m18.prg_index(0xA004), 0x14 * 0x2000 + 4);
    assert_eq!(m18.prg_index(0xC004), 0x15 * 0x2000 + 4);
    assert_eq!(m18.prg_index(0xE004), 31 * 0x2000 + 4);
    m18.write_register(0xA000, 0x06);
    m18.write_register(0xA001, 0x01);
    assert_eq!(m18.chr_index(0x0004), 0x16 * 0x0400 + 4);
    m18.write_register(0xF002, 3);
    assert_eq!(m18.mirroring(), Mirroring::SingleScreenHigh);
    m18.write_register(0xE000, 2);
    m18.write_register(0xE001, 0);
    m18.write_register(0xE002, 0);
    m18.write_register(0xE003, 0);
    m18.write_register(0xF000, 0);
    m18.write_register(0xF001, 0x01);
    m18.cpu_clock();
    assert!(!m18.irq());
    m18.cpu_clock();
    assert!(m18.irq());

    let mut m65 = Mapper::new(65, 16, 16, Mirroring::Horizontal, 0).expect("mapper 65");
    m65.write_register(0x8000, 3);
    m65.write_register(0xA000, 4);
    m65.write_register(0xC000, 5);
    assert_eq!(m65.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m65.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m65.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m65.prg_index(0xE004), 31 * 0x2000 + 4);
    m65.write_register(0xB004, 9);
    assert_eq!(m65.chr_index(0x1004), 9 * 0x0400 + 4);
    m65.write_register(0x9001, 0x80);
    assert_eq!(m65.mirroring(), Mirroring::Horizontal);
    m65.write_register(0x9005, 0x00);
    m65.write_register(0x9006, 0x02);
    m65.write_register(0x9004, 0);
    m65.write_register(0x9003, 0x80);
    m65.cpu_clock();
    assert!(!m65.irq());
    m65.cpu_clock();
    assert!(m65.irq());
    m65.clear_irq();
    assert!(!m65.irq());
}

#[test]
fn address_latch_multicarts_decode_prg_chr_and_mirroring_bits() {
    let mut m57 = Mapper::new(57, 16, 16, Mirroring::Vertical, 0).expect("mapper 57");
    m57.write_register(0x8000, 0x47);
    m57.write_register(0x8800, 0xB8);
    assert_eq!(m57.mirroring(), Mirroring::Horizontal);
    assert_eq!(m57.prg_index(0x8000), 4 * 0x4000);
    assert_eq!(m57.prg_index(0xC000), 5 * 0x4000);
    assert_eq!(m57.chr_index(0x0004), 15 * 0x2000 + 4);

    let mut m58 = Mapper::new(58, 16, 16, Mirroring::Vertical, 0).expect("mapper 58");
    m58.write_register(0x80CB, 0);
    assert_eq!(m58.mirroring(), Mirroring::Horizontal);
    assert_eq!(m58.prg_index(0x8000), 3 * 0x4000);
    assert_eq!(m58.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m58.chr_index(0x0004), 1 * 0x2000 + 4);

    let mut m61 = Mapper::new(61, 32, 0, Mirroring::Vertical, 0).expect("mapper 61");
    m61.write_register(0x80B2, 0);
    assert_eq!(m61.mirroring(), Mirroring::Horizontal);
    assert_eq!(m61.prg_index(0x8000), 5 * 0x4000);
    assert_eq!(m61.prg_index(0xC000), 5 * 0x4000);

    let mut m62 = Mapper::new(62, 128, 128, Mirroring::Vertical, 0).expect("mapper 62");
    m62.write_register(0xA2E5, 3);
    assert_eq!(m62.mirroring(), Mirroring::Horizontal);
    assert_eq!(m62.prg_index(0x8000), 98 * 0x4000);
    assert_eq!(m62.prg_index(0xC000), 98 * 0x4000);
    assert_eq!(m62.chr_index(0x0004), 23 * 0x2000 + 4);
}

#[test]
fn mapper53_supervision_switches_low_and_high_prg_windows() {
    let mut m = Mapper::new(53, 256, 0, Mirroring::Vertical, 0).expect("mapper 53");

    assert_eq!(m.low_prg_index(0x6004), Some((0x0F + 4) * 0x2000 + 4));
    assert_eq!(m.prg_index(0x8004), 4);
    assert_eq!(m.prg_index(0xC004), 1 * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x1004);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
    assert!(!m.low_prg_ram_read_enabled(0x6000));
    assert!(!m.low_prg_ram_write_enabled(0x6000));

    assert!(m.write_low_register(0x6000, 0x3B));
    assert_eq!(m.low_prg_index(0x6004), Some((0xBF + 4) * 0x2000 + 4));
    assert_eq!(m.mirroring(), Mirroring::Horizontal);

    assert!(m.write_low_register(0x6000, 0x33));
    m.write_register(0x8000, 0x05);
    assert_eq!(m.low_prg_index(0x6004), Some((0xBF + 4) * 0x2000 + 4));
    assert_eq!(m.prg_index(0x8004), (((0x0B << 3) | 0x05) + 2) * 0x4000 + 4);
    assert_eq!(m.prg_index(0xC004), (((0x0B << 3) | 0x07) + 2) * 0x4000 + 4);

    m.reset(true);
    assert_eq!(m.low_prg_index(0x6004), Some((0x0F + 4) * 0x2000 + 4));
    assert_eq!(m.prg_index(0xC004), 1 * 0x4000 + 4);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
}

#[test]
fn mapper56_kaiser202_extends_mapper142_with_chr_and_mirroring_writes() {
    let mut m = Mapper::new(56, 16, 16, Mirroring::Vertical, 0).expect("mapper 56");

    assert_eq!(m.prg_index(0xE004), 31 * 0x2000 + 4);
    assert_eq!(m.low_prg_index(0x6004), None);
    assert!(m.low_prg_ram_read_enabled(0x6000));
    assert!(m.low_prg_ram_write_enabled(0x6000));

    m.write_register(0xE000, 0x01);
    m.write_register(0xF000, 0x13);
    assert_eq!(m.prg_index(0x8004), 0x13 * 0x2000 + 4);

    m.write_register(0xF001, 0x2F);
    assert_eq!(m.prg_index(0x8004), 0x1F * 0x2000 + 4);

    m.write_register(0xE000, 0x05);
    m.write_register(0xF000, 0x04);
    assert_eq!(m.low_prg_index(0x6004), Some(4));
    assert!(!m.low_prg_ram_read_enabled(0x6000));
    assert!(!m.low_prg_ram_write_enabled(0x6000));

    m.write_register(0xF800, 0x00);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    m.write_register(0xF800, 0x01);
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.write_register(0xFC03, 0x12);
    assert_eq!(m.chr_index(0x0C04), 0x12 * 0x0400 + 4);

    m.write_register(0x8000, 0x0D);
    m.write_register(0x9000, 0x0F);
    m.write_register(0xA000, 0x0F);
    m.write_register(0xB000, 0x0F);
    m.write_register(0xC000, 0x02);
    m.cpu_clock();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(m.irq());
    m.write_register(0xD000, 0);
    assert!(!m.irq());
}

#[test]
fn irem_lrog017_routes_low_chr_to_rom_and_upper_6k_to_ram() {
    let mut m = Mapper::new(77, 16, 16, Mirroring::Horizontal, 0).expect("mapper 77");
    m.write_register(0x8000, 0x53);
    assert_eq!(m.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m.chr_index(0x0004), 5 * 0x0800 + 4);
    assert!(m.has_chr_read());

    assert!(!m.chr_write(0x0004, 0xAA));
    assert!(m.chr_write(0x0804, 0x55));
    assert!(m.chr_write(0x1004, 0x66));
    assert_eq!(m.chr_read(0x0804, ChrAccess::Default), Some(0x55));
    assert_eq!(m.chr_read(0x1004, ChrAccess::Default), Some(0x66));
    assert_eq!(m.chr_read(0x0004, ChrAccess::Default), None);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
}

#[test]
fn namco118_switches_mmc3_style_banks_without_irq() {
    let mut m = Mapper::new(88, 8, 16, Mirroring::Vertical, 0).expect("mapper 88");
    m.write_register(0x8000, 0);
    m.write_register(0x8001, 7);
    m.write_register(0x8000, 2);
    m.write_register(0x8001, 3);
    m.write_register(0x8000, 6);
    m.write_register(0x8001, 4);
    m.write_register(0x8000, 7);
    m.write_register(0x8001, 5);

    assert_eq!(m.prg_index(0x8000), 4 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 5 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
    assert_eq!(m.prg_index(0xE000), 15 * 0x2000);
    assert_eq!(m.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1004), 0x43 * 0x0400 + 4);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
}

#[test]
fn ntdec112_uses_command_register_outer_chr_and_mirroring() {
    let mut m = Mapper::new(112, 8, 64, Mirroring::Vertical, 0).expect("mapper 112");
    m.write_register(0x8000, 0);
    m.write_register(0xA000, 4);
    m.write_register(0x8000, 1);
    m.write_register(0xA000, 5);
    m.write_register(0x8000, 2);
    m.write_register(0xA000, 6);
    m.write_register(0x8000, 4);
    m.write_register(0xA000, 7);
    m.write_register(0xC000, 0x10);
    m.write_register(0xE000, 1);

    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    assert_eq!(m.prg_index(0x8000), 4 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 5 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
    assert_eq!(m.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1004), 0x107 * 0x0400 + 4);
}

#[test]
fn mapper151_selects_three_8k_prg_and_two_4k_chr_pages() {
    let mut m = Mapper::new(151, 8, 16, Mirroring::Horizontal, 0).expect("mapper 151");
    m.write_register(0x8000, 1);
    m.write_register(0xA000, 2);
    m.write_register(0xC000, 3);
    m.write_register(0xE000, 4);
    m.write_register(0xF000, 5);

    assert_eq!(m.prg_index(0x8000), 1 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 2 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xE000), 15 * 0x2000);
    assert_eq!(m.chr_index(0x0004), 4 * 0x1000 + 4);
    assert_eq!(m.chr_index(0x1004), 5 * 0x1000 + 4);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
}

#[test]
fn late_address_latch_multicarts_decode_reference_bits() {
    let mut m200 = Mapper::new(200, 16, 16, Mirroring::Vertical, 0).expect("mapper 200");
    m200.write_register(0x800B, 0);
    assert_eq!(m200.prg_index(0x8000), 3 * 0x4000);
    assert_eq!(m200.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m200.chr_index(0x0004), 3 * 0x2000 + 4);
    assert_eq!(m200.mirroring(), Mirroring::Vertical);

    let mut m202 = Mapper::new(202, 16, 16, Mirroring::Vertical, 0).expect("mapper 202");
    m202.write_register(0x8009, 0);
    assert_eq!(m202.prg_index(0x8000), 4 * 0x4000);
    assert_eq!(m202.prg_index(0xC000), 5 * 0x4000);
    assert_eq!(m202.chr_index(0x0004), 4 * 0x2000 + 4);
    assert_eq!(m202.mirroring(), Mirroring::Horizontal);

    let mut m204 = Mapper::new(204, 16, 16, Mirroring::Vertical, 0).expect("mapper 204");
    m204.write_register(0x8015, 0);
    assert_eq!(m204.prg_index(0x8000), 5 * 0x4000);
    assert_eq!(m204.prg_index(0xC000), 5 * 0x4000);
    assert_eq!(m204.chr_index(0x0004), 5 * 0x2000 + 4);
    assert_eq!(m204.mirroring(), Mirroring::Horizontal);

    let mut m213 = Mapper::new(213, 16, 16, Mirroring::Vertical, 0).expect("mapper 213");
    m213.write_register(0x800A, 0);
    assert_eq!(m213.prg_index(0x8000), 2 * 0x4000);
    assert_eq!(m213.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m213.chr_index(0x0004), 1 * 0x2000 + 4);

    let mut m214 = Mapper::new(214, 16, 16, Mirroring::Vertical, 0).expect("mapper 214");
    m214.write_register(0x800D, 0);
    assert_eq!(m214.prg_index(0x8000), 3 * 0x4000);
    assert_eq!(m214.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m214.chr_index(0x0004), 1 * 0x2000 + 4);

    let mut m225 = Mapper::new(225, 128, 128, Mirroring::Vertical, 0).expect("mapper 225");
    m225.write_register(0xFA3C, 0);
    assert_eq!(m225.prg_index(0x8000), 104 * 0x4000);
    assert_eq!(m225.prg_index(0xC000), 104 * 0x4000);
    assert_eq!(m225.chr_index(0x0004), 124 * 0x2000 + 4);
    assert_eq!(m225.mirroring(), Mirroring::Horizontal);

    let mut m229 = Mapper::new(229, 64, 256, Mirroring::Vertical, 0).expect("mapper 229");
    m229.write_register(0x8031, 0);
    assert_eq!(m229.prg_index(0x8000), 17 * 0x4000);
    assert_eq!(m229.prg_index(0xC000), 17 * 0x4000);
    assert_eq!(m229.chr_index(0x0004), 0x31 * 0x2000 + 4);
    assert_eq!(m229.mirroring(), Mirroring::Horizontal);
}

#[test]
fn additional_address_latch_multicarts_decode_reference_bits() {
    let mut m174 = Mapper::new(174, 16, 16, Mirroring::Vertical, 0).expect("mapper 174");
    m174.write_register(0x80F5, 0);
    assert_eq!(m174.prg_index(0x8000), 6 * 0x4000);
    assert_eq!(m174.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m174.chr_index(0x0004), 2 * 0x2000 + 4);
    assert_eq!(m174.mirroring(), Mirroring::Horizontal);

    let mut m216 = Mapper::new(216, 4, 4, Mirroring::Horizontal, 0).expect("mapper 216");
    m216.write_register(0x800D, 0);
    assert_eq!(m216.prg_index(0x8000), 1 * 0x8000);
    assert_eq!(m216.chr_index(0x0004), 6 * 0x2000 + 4);
    assert_eq!(m216.peek_expansion(0x5000), Some(0));
    m216.write_expansion(0x5000, 0xFF);
    assert_eq!(m216.prg_index(0x8000), 1 * 0x8000);
    assert_eq!(m216.mirroring(), Mirroring::Horizontal);

    let mut m227 = Mapper::new(227, 64, 0, Mirroring::Vertical, 0).expect("mapper 227");
    m227.write_register(0x8206, 0);
    assert_eq!(m227.prg_index(0x8000), 1 * 0x4000);
    assert_eq!(m227.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m227.chr_index(0x0004), 4);
    assert_eq!(m227.mirroring(), Mirroring::Horizontal);

    let mut m231 = Mapper::new(231, 32, 0, Mirroring::Vertical, 0).expect("mapper 231");
    m231.write_register(0x80A2, 0);
    assert_eq!(m231.prg_index(0x8000), 2 * 0x4000);
    assert_eq!(m231.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m231.mirroring(), Mirroring::Horizontal);

    let mut m242 = Mapper::new(242, 32, 0, Mirroring::Vertical, 0).expect("mapper 242");
    m242.write_register(0x807A, 0);
    assert_eq!(m242.prg_index(0x8000), 30 * 0x4000);
    assert_eq!(m242.prg_index(0xC000), 31 * 0x4000);
    assert_eq!(m242.mirroring(), Mirroring::Horizontal);
}

#[test]
fn address_latch_compatibility_batch_decodes_reference_bits() {
    let mut m59 = Mapper::new(59, 16, 16, Mirroring::Vertical, 0).expect("mapper 59");
    m59.write_register(0x81BF, 0);
    assert_eq!(m59.prg_index(0x8000), 6 * 0x4000);
    assert_eq!(m59.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m59.chr_index(0x0004), 7 * 0x2000 + 4);
    assert_eq!(m59.read_register(0x8000, 0xAA), Some(0));
    assert_eq!(m59.mirroring(), Mirroring::Vertical);

    let mut m63 = Mapper::new(63, 4, 0, Mirroring::Vertical, 0).expect("mapper 63");
    m63.write_register(0x803F, 0);
    assert_eq!(
        m63.read_register_with_open_bus(0x8000, 0xAA, 0x5C),
        Some(0x5C)
    );
    m63.write_register(0x800B, 0);
    assert_eq!(m63.read_register_with_open_bus(0x8000, 0xAA, 0x5C), None);
    assert_eq!(m63.prg_index(0x8000), 2 * 0x4000);
    assert_eq!(m63.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m63.mirroring(), Mirroring::Horizontal);

    let mut m201 = Mapper::new(201, 8, 8, Mirroring::Horizontal, 0).expect("mapper 201");
    m201.write_register(0x8003, 0);
    assert_eq!(m201.prg_index(0x8000), 6 * 0x4000);
    assert_eq!(m201.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m201.chr_index(0x0004), 3 * 0x2000 + 4);
    assert_eq!(m201.mirroring(), Mirroring::Horizontal);

    let mut m217 = Mapper::new(217, 8, 16, Mirroring::Vertical, 0).expect("mapper 217");
    m217.write_register(0x801F, 0);
    assert_eq!(m217.prg_index(0x8000), 6 * 0x4000);
    assert_eq!(m217.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m217.chr_index(0x0004), 15 * 0x2000 + 4);
    assert_eq!(m217.mirroring(), Mirroring::Vertical);
}

#[test]
fn txc_and_jaleco_mapper_batch_follow_reference_registers() {
    let mut m36 = Mapper::new(36, 32, 16, Mirroring::Vertical, 0).expect("mapper 36");
    m36.write_register(0xC000, 0x5A);
    assert_eq!(m36.prg_index(0x8004), 5 * 0x8000 + 4);
    assert_eq!(m36.chr_index(0x0004), 10 * 0x2000 + 4);
    assert_eq!(m36.read_expansion(0x4100), Some(0x5A));
    assert!(m36.has_bus_conflicts());
    assert_eq!(m36.mirroring(), Mirroring::Horizontal);
    m36.write_register(0x8000, 0x21);
    assert_eq!(m36.prg_index(0x8004), 2 * 0x8000 + 4);
    assert_eq!(m36.mirroring(), Mirroring::Vertical);

    let mut m92 = Mapper::new(92, 16, 16, Mirroring::Horizontal, 0).expect("mapper 92");
    m92.write_register(0x8000, 0x85);
    m92.write_register(0x9000, 0x43);
    assert_eq!(m92.prg_index(0x8004), 4);
    assert_eq!(m92.prg_index(0xC004), 5 * 0x4000 + 4);
    assert_eq!(m92.chr_index(0x0004), 3 * 0x2000 + 4);
    assert_eq!(m92.mirroring(), Mirroring::Horizontal);

    let mut m72 = Mapper::new(72, 16, 16, Mirroring::Vertical, 0).expect("mapper 72");
    m72.write_low_register(0x6000, 0x83);
    m72.write_register(0x8000, 0x45);
    assert_eq!(m72.prg_index(0x8004), 3 * 0x4000 + 4);
    assert_eq!(m72.prg_index(0xC004), 15 * 0x4000 + 4);
    assert_eq!(m72.chr_index(0x0004), 5 * 0x2000 + 4);
    assert_eq!(m72.mirroring(), Mirroring::Vertical);
}

#[test]
fn data_latch_multicarts_decode_reference_bits() {
    let mut m39 = Mapper::new(39, 8, 0, Mirroring::Horizontal, 0).expect("mapper 39");
    m39.write_register(0x8000, 3);
    assert_eq!(m39.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m39.chr_index(0x0004), 4);
    assert_eq!(m39.mirroring(), Mirroring::Horizontal);

    let mut m128 = Mapper::new(128, 64, 0, Mirroring::Vertical, 0).expect("mapper 128");
    m128.write_register(0x8126, 0x05);
    assert_eq!(m128.prg_index(0x8004), 0x204D * 0x4000 + 4);
    assert_eq!(m128.prg_index(0xC004), 0x204F * 0x4000 + 4);
    assert_eq!(m128.chr_index(0x1004), 0x1004);
    assert_eq!(m128.mirroring(), Mirroring::Vertical);
    m128.write_register(0xF123, 0x02);
    assert_eq!(m128.prg_index(0x8004), 0x3C4A * 0x4000 + 4);
    m128.reset(true);
    assert_eq!(m128.prg_index(0xC004), 7 * 0x4000 + 4);

    let mut m226 = Mapper::new(226, 128, 0, Mirroring::Vertical, 0).expect("mapper 226");
    m226.write_register(0x8000, 0xE3);
    m226.write_register(0x8001, 0x01);
    assert_eq!(m226.prg_index(0x8000), 99 * 0x4000);
    assert_eq!(m226.prg_index(0xC000), 99 * 0x4000);
    assert_eq!(m226.mirroring(), Mirroring::Vertical);

    let mut m236 = Mapper::new(236, 128, 16, Mirroring::Vertical, 0).expect("mapper 236");
    m236.write_register(0xA0A1, 0);
    m236.write_register(0xD015, 0);
    assert_eq!(m236.prg_index(0x800F), 5 * 0x4000);
    assert_eq!(m236.prg_index(0xC00F), (5 | 7) * 0x4000);
    assert_eq!(m236.chr_index(0x1004), 1 * 0x2000 + 0x1004);
    assert_eq!(m236.mirroring(), Mirroring::Vertical);
    m236.reset(true);
    m236.write_register(0x8000, 0);
    m236.write_register(0xD015, 0);
    assert_eq!(m236.prg_index(0x800F), 5 * 0x4000 + 0x0001);

    let mut m236_chr_ram = Mapper::new(236, 128, 0, Mirroring::Vertical, 0).expect("mapper 236");
    m236_chr_ram.write_register(0xC012, 0);
    m236_chr_ram.write_register(0x8F23, 0);
    assert_eq!(m236_chr_ram.prg_index(0x8004), 0x11A * 0x4000);
    assert_eq!(m236_chr_ram.chr_index(0x1004), 0x1004);

    let mut m237 = Mapper::new(237, 128, 0, Mirroring::Vertical, 0).expect("mapper 237");
    m237.write_register(0x8001, 0xA5);
    assert_eq!(m237.peek_register(0x8000, 0xFF), Some(0));
    assert_eq!(m237.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m237.prg_index(0xC004), 5 * 0x4000 + 4);
    assert_eq!(m237.mirroring(), Mirroring::Vertical);
    m237.reset(true);
    m237.write_register(0x8001, 0x00);
    assert_eq!(m237.peek_register(0x8000, 0xFF), Some(1));
    m237.write_register(0x8002, 0xFF);
    assert_eq!(m237.peek_register(0x8000, 0xFF), None);

    let mut m239 = Mapper::new(239, 64, 64, Mirroring::Vertical, 0).expect("mapper 239");
    m239.write_register(0x8015, 0);
    assert_eq!(m239.prg_index(0x8004), 0x400A * 0x4000 + 4);
    assert_eq!(m239.prg_index(0xC004), 0x400B * 0x4000 + 4);
    assert_eq!(m239.chr_index(0x1004), 0x8015 * 0x2000 + 0x1004);
    assert_eq!(m239.mirroring(), Mirroring::Horizontal);
    m239.write_register(0x8003, 0);
    assert_eq!(m239.prg_index(0x8004), 0x8003 * 0x4000 + 4);
    assert_eq!(m239.prg_index(0xC004), 0x8003 * 0x4000 + 4);

    let mut m240 = Mapper::new(240, 32, 16, Mirroring::Horizontal, 0).expect("mapper 240");
    m240.write_expansion(0x4020, 0xA5);
    assert_eq!(m240.prg_index(0x8000), 10 * 0x8000);
    assert_eq!(m240.chr_index(0x0004), 5 * 0x2000 + 4);
    assert_eq!(m240.mirroring(), Mirroring::Horizontal);

    let mut m241 = Mapper::new(241, 32, 0, Mirroring::Vertical, 0).expect("mapper 241");
    m241.write_register(0x8000, 7);
    assert_eq!(m241.prg_index(0x8000), 7 * 0x8000);
    assert_eq!(m241.chr_index(0x0004), 4);

    let mut m244 = Mapper::new(244, 8, 8, Mirroring::Horizontal, 0).expect("mapper 244");
    m244.write_register(0x8000, 0x31);
    assert_eq!(m244.prg_index(0x8000), 1 * 0x8000);
    m244.write_register(0x8000, 0x5B);
    assert_eq!(m244.chr_index(0x0004), 6 * 0x2000 + 4);

    let mut m246 = Mapper::new(246, 8, 8, Mirroring::Vertical, 0).expect("mapper 246");
    assert_eq!(m246.prg_index(0xE000), 15 * 0x2000);
    assert!(m246.write_low_register(0x6001, 3));
    assert!(m246.write_low_register(0x6004, 5));
    assert_eq!(m246.prg_index(0xA000), 3 * 0x2000);
    assert_eq!(m246.chr_index(0x0004), 5 * 0x0800 + 4);
    assert!(!m246.write_low_register(0x6800, 1));
}

#[test]
fn special_mapper_interfaces_cover_low_prg_reads_and_reset_hooks() {
    let mut m103 = Mapper::new(103, 8, 0, Mirroring::Vertical, 0).expect("mapper 103");
    m103.write_register(0x8000, 6);
    assert_eq!(m103.low_prg_index(0x6004), None);
    m103.write_register(0xF000, 0x10);
    assert_eq!(m103.low_prg_index(0x6004), Some(6 * 0x2000 + 4));
    assert_eq!(m103.prg_index(0x8000), 12 * 0x2000);
    assert_eq!(m103.prg_index(0xE000), 15 * 0x2000);
    m103.write_register(0xE000, 0x08);
    assert_eq!(m103.mirroring(), Mirroring::Horizontal);

    let mut m120 = Mapper::new(120, 8, 0, Mirroring::Horizontal, 0).expect("mapper 120");
    m120.write_expansion(0x41FF, 7);
    assert_eq!(m120.low_prg_index(0x6004), Some(7 * 0x2000 + 4));
    assert_eq!(m120.prg_index(0x8000), 2 * 0x8000);
    assert_eq!(m120.mirroring(), Mirroring::Horizontal);

    let mut m170 = Mapper::new(170, 2, 1, Mirroring::Vertical, 0).expect("mapper 170");
    assert!(m170.write_low_register(0x6502, 0x40));
    assert_eq!(m170.peek_low_register(0x7777), Some(0xF7));
    assert_eq!(m170.read_low_register(0x7001), Some(0xF0));
    m170.reset(true);
    assert_eq!(m170.peek_low_register(0x7777), Some(0x77));
}

#[test]
fn unlicensed_mapper_batch_matches_reference_bank_and_irq_rules() {
    let mut m43 = Mapper::new(43, 16, 0, Mirroring::Vertical, 0).expect("mapper 43");
    assert_eq!(m43.expansion_prg_index(0x5004), Some(16 * 0x1000 + 4));
    assert_eq!(m43.low_prg_index(0x6004), Some(2 * 0x2000 + 4));
    m43.write_expansion(0x4022, 0x02);
    m43.write_expansion(0x4120, 0x01);
    assert_eq!(m43.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m43.prg_index(0xE004), 8 * 0x2000 + 4);
    assert_eq!(m43.low_prg_index(0x6004), Some(4));
    m43.write_expansion(0x4122, 0x01);
    for _ in 0..4095 {
        m43.cpu_clock();
    }
    assert!(!m43.irq());
    m43.cpu_clock();
    assert!(m43.irq());

    let mut m60 = Mapper::new(60, 8, 4, Mirroring::Horizontal, 0).expect("mapper 60");
    assert_eq!(m60.prg_index(0x8004), 4);
    m60.reset(true);
    assert_eq!(m60.prg_index(0x8004), 0x4000 + 4);
    assert_eq!(m60.chr_index(0x0004), 0x2000 + 4);
    assert_eq!(m60.mirroring(), Mirroring::Horizontal);

    let mut m83 = Mapper::new(83, 64, 64, Mirroring::Vertical, 0).expect("mapper 83");
    m83.write_expansion(0x5102, 0xA5);
    assert_eq!(m83.read_expansion(0x5102), Some(0xA5));
    m83.write_register(0x8100, 0x81);
    m83.write_register(0x8000, 0x12);
    assert_eq!(m83.mirroring(), Mirroring::Horizontal);
    assert_eq!(m83.prg_index(0x8004), 0x24 * 0x2000 + 4);
    m83.write_register(0x8310, 3);
    assert_eq!(m83.chr_index(0x0004), 6 * 0x0400 + 4);
    m83.write_register(0x8200, 1);
    m83.write_register(0x8201, 0);
    m83.cpu_clock();
    assert!(m83.irq());

    let mut m106 = Mapper::new(106, 32, 32, Mirroring::Vertical, 0).expect("mapper 106");
    m106.write_register(0x8008, 3);
    m106.write_register(0x8009, 4);
    m106.write_register(0x800A, 5);
    assert_eq!(m106.prg_index(0x8004), 0x13 * 0x2000 + 4);
    assert_eq!(m106.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m106.prg_index(0xC004), 5 * 0x2000 + 4);
    m106.write_register(0x8000, 7);
    m106.write_register(0x8001, 8);
    assert_eq!(m106.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m106.chr_index(0x0404), 9 * 0x0400 + 4);
    m106.write_register(0x800E, 0xFE);
    m106.write_register(0x800F, 0xFF);
    m106.cpu_clock();
    assert!(!m106.irq());
    m106.cpu_clock();
    assert!(m106.irq());
}

#[test]
fn more_unlicensed_mapper_batch_matches_reference_side_effects() {
    let mut m183 = Mapper::new(183, 32, 32, Mirroring::Vertical, 0).expect("mapper 183");
    assert!(m183.write_low_register(0x682A, 0));
    assert_eq!(m183.low_prg_index(0x6004), Some(0x2A * 0x2000 + 4));
    m183.write_register(0x8800, 3);
    m183.write_register(0xA800, 4);
    m183.write_register(0xA000, 5);
    assert_eq!(m183.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m183.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m183.prg_index(0xC004), 5 * 0x2000 + 4);
    m183.write_register(0xB000, 3);
    m183.write_register(0xB004, 2);
    assert_eq!(m183.chr_index(0x0004), 0x23 * 0x0400 + 4);
    m183.write_register(0x9800, 3);
    assert_eq!(m183.mirroring(), Mirroring::SingleScreenHigh);
    m183.write_register(0xF000, 0x0F);
    m183.write_register(0xF004, 0x0F);
    m183.write_register(0xF008, 0x01);
    for _ in 0..114 {
        m183.cpu_clock();
    }
    assert!(!m183.irq());
    m183.cpu_clock();
    assert!(m183.irq());

    let mut m212 = Mapper::new(212, 16, 16, Mirroring::Vertical, 0).expect("mapper 212");
    m212.write_register(0xC00B, 0);
    assert_eq!(m212.prg_index(0x8004), 2 * 0x4000 + 4);
    assert_eq!(m212.prg_index(0xC004), 3 * 0x4000 + 4);
    assert_eq!(m212.chr_index(0x0004), 3 * 0x2000 + 4);
    assert_eq!(m212.mirroring(), Mirroring::Horizontal);
    assert_eq!(
        m212.read_low_register_with_prg_ram(0x6000, 0x12),
        Some(0x92)
    );
    assert_eq!(m212.peek_low_register_with_prg_ram(0x6010, 0x12), None);

    let mut m222 = Mapper::new(222, 16, 16, Mirroring::Vertical, 0).expect("mapper 222");
    assert!(m222.watches_ppu_bus());
    m222.write_register(0x8000, 3);
    m222.write_register(0xA000, 4);
    m222.write_register(0xB002, 5);
    assert_eq!(m222.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m222.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m222.chr_index(0x0404), 5 * 0x0400 + 4);
    m222.write_register(0x9000, 1);
    assert_eq!(m222.mirroring(), Mirroring::Horizontal);
    m222.write_register(0xF000, 238);
    m222.notify_a12(0x0000, 0);
    m222.notify_a12(0x1000, 12);
    assert!(!m222.irq());
    m222.notify_a12(0x0000, 15);
    m222.notify_a12(0x1000, 24);
    assert!(m222.irq());

    let mut m235 = Mapper::new(235, 4, 0, Mirroring::Vertical, 0).expect("mapper 235");
    m235.write_register(0x803F, 0);
    assert_eq!(
        m235.read_register_with_open_bus(0x8000, 0xAA, 0x5C),
        Some(0x5C)
    );
    assert_eq!(m235.read_register_with_open_bus(0x8000, 0xAA, 0x5C), None);
    m235.write_register(0xA001, 0);
    assert_eq!(m235.mirroring(), Mirroring::Horizontal);

    let mut reset_235 = Mapper::new(235, 8, 0, Mirroring::Vertical, 0).expect("mapper 235");
    reset_235.reset(true);
    assert_eq!(reset_235.prg_index(0x8004), 4);
    assert_eq!(reset_235.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(reset_235.mirroring(), Mirroring::Vertical);
}

#[test]
fn reset_selected_and_read_side_effect_mappers_follow_reference_rules() {
    let mut m175 = Mapper::new(175, 32, 16, Mirroring::Vertical, 0).expect("mapper 175");
    assert_eq!(m175.prg_index(0x8004), 0x0004);
    assert_eq!(m175.prg_index(0xC004), 0x0004);
    assert_eq!(m175.prg_index(0xE004), 0x2000 + 4);
    m175.write_register(0xA000, 0x06);
    assert_eq!(m175.chr_index(0x1010), 6 * 0x2000 + 0x1010);
    assert_eq!(m175.prg_index(0x8004), 0x0004);
    assert_eq!(m175.prg_index(0xC004), 0x0004);
    assert_eq!(m175.prg_index(0xE004), 13 * 0x2000 + 4);
    assert_eq!(m175.read_register(0xFFFB, 0xAA), None);
    assert_eq!(m175.prg_index(0x8004), 0x0004);
    assert_eq!(m175.read_register(0xFFFC, 0xAA), None);
    assert_eq!(m175.prg_index(0x8004), 6 * 0x4000 + 4);
    assert_eq!(m175.prg_index(0xC004), 12 * 0x2000 + 4);
    m175.write_register(0x8000, 0x04);
    assert_eq!(m175.mirroring(), Mirroring::Horizontal);

    let mut m177 = Mapper::new(177, 64, 0, Mirroring::Vertical, 0).expect("mapper 177");
    assert_eq!(m177.prg_index(0x8004), 0x0004);
    assert_eq!(m177.chr_index(0x1010), 0x1010);
    assert_eq!(m177.mirroring(), Mirroring::Vertical);
    m177.write_register(0x9000, 0x25);
    assert_eq!(m177.prg_index(0x8004), 5 * 0x8000 + 4);
    assert_eq!(m177.prg_index(0xFFFF), 5 * 0x8000 + 0x7FFF);
    assert_eq!(m177.mirroring(), Mirroring::Horizontal);
    m177.reset(true);
    assert_eq!(m177.prg_index(0x8004), 0x0004);
    assert_eq!(m177.mirroring(), Mirroring::Vertical);

    let mut m230 = Mapper::new(230, 16, 0, Mirroring::Vertical, 0).expect("mapper 230");
    assert_eq!(m230.prg_index(0x8000), 0);
    assert_eq!(m230.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m230.mirroring(), Mirroring::Vertical);
    m230.write_register(0x8000, 5);
    assert_eq!(m230.prg_index(0x8000), 5 * 0x4000);
    m230.reset(true);
    assert_eq!(m230.prg_index(0x8000), 8 * 0x4000);
    assert_eq!(m230.prg_index(0xC000), 9 * 0x4000);
    assert_eq!(m230.mirroring(), Mirroring::Horizontal);

    let mut m233 = Mapper::new(233, 128, 0, Mirroring::Vertical, 0).expect("mapper 233");
    assert_eq!(m233.prg_index(0x8000), 0);
    m233.reset(true);
    assert_eq!(m233.prg_index(0x8000), 32 * 0x4000);
    m233.write_register(0x8000, 0xE3);
    m233.write_register(0x8001, 0x01);
    assert_eq!(m233.prg_index(0x8000), 99 * 0x4000);
    assert_eq!(m233.mirroring(), Mirroring::Vertical);

    let mut m234 = Mapper::new(234, 32, 64, Mirroring::Vertical, 0).expect("mapper 234");
    m234.write_register(0xFF80, 0xC2);
    m234.write_register(0xFFE8, 0x71);
    assert_eq!(m234.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m234.chr_index(0x0004), 15 * 0x2000 + 4);
    assert_eq!(m234.mirroring(), Mirroring::Horizontal);
    assert!(m234.has_bus_conflicts());

    let mut read_latch = Mapper::new(234, 32, 64, Mirroring::Vertical, 0).expect("mapper 234");
    assert_eq!(read_latch.read_register(0xFF80, 0x85), Some(0x85));
    assert_eq!(read_latch.prg_index(0x8000), 5 * 0x8000);
    assert_eq!(read_latch.chr_index(0x0004), 20 * 0x2000 + 4);
}

#[test]
fn bandai_74161_variants_switch_prg_chr_and_mirroring() {
    let mut m70 = Mapper::new(70, 8, 4, Mirroring::Horizontal, 0).expect("mapper 70");
    assert_eq!(m70.mirroring(), Mirroring::Vertical);
    m70.write_register(0x8000, 0x21);
    assert_eq!(m70.prg_index(0x8000), 2 * 0x4000);
    assert_eq!(m70.chr_index(0x0123), 0x2000 + 0x0123);
    assert_eq!(m70.mirroring(), Mirroring::Vertical);
    m70.write_register(0x8000, 0x80);
    assert_eq!(m70.mirroring(), Mirroring::SingleScreenHigh);
    m70.write_register(0x8000, 0x00);
    assert_eq!(m70.mirroring(), Mirroring::SingleScreenLow);

    let mut m152 = Mapper::new(152, 8, 4, Mirroring::Horizontal, 0).expect("mapper 152");
    m152.write_register(0x8000, 0x00);
    assert_eq!(m152.mirroring(), Mirroring::SingleScreenLow);
}

#[test]
fn jaleco_jf16_switches_banks_with_submapper_mirroring() {
    let mut m = Mapper::new(78, 8, 8, Mirroring::Horizontal, 0).expect("jf16");
    m.write_register(0x8000, 0x59);
    assert_eq!(m.prg_index(0x8000), 1 * 0x4000);
    assert_eq!(m.chr_index(0x0010), 5 * 0x2000 + 0x0010);
    assert_eq!(m.mirroring(), Mirroring::SingleScreenHigh);
    assert!(m.has_bus_conflicts());

    let mut holy_diver = Mapper::new(78, 8, 8, Mirroring::Horizontal, 3).expect("jf16 sub3");
    holy_diver.write_register(0x8000, 0x08);
    assert_eq!(holy_diver.mirroring(), Mirroring::Vertical);
    holy_diver.write_register(0x8000, 0x00);
    assert_eq!(holy_diver.mirroring(), Mirroring::Horizontal);
}

#[test]
fn jaleco_jfxx_and_sunsoft184_use_low_register_windows() {
    let mut m87 = Mapper::new(87, 2, 4, Mirroring::Vertical, 0).expect("mapper 87");
    assert!(m87.write_low_register(0x6000, 0x01));
    assert_eq!(m87.chr_index(0x0004), 2 * 0x2000 + 4);
    assert_eq!(m87.mirroring(), Mirroring::Vertical);

    let mut m101 = Mapper::new(101, 2, 4, Mirroring::Horizontal, 0).expect("mapper 101");
    assert!(m101.write_low_register(0x7FFF, 3));
    assert_eq!(m101.chr_index(0x0004), 3 * 0x2000 + 4);

    let mut m184 = Mapper::new(184, 2, 8, Mirroring::Vertical, 0).expect("mapper 184");
    assert!(m184.write_low_register(0x6000, 0x52));
    assert_eq!(m184.chr_index(0x0004), 2 * 0x1000 + 4);
    assert_eq!(m184.chr_index(0x1004), 0x85 * 0x1000 + 4);
    assert!(!m184.write_low_register(0x5FFF, 0x00));
}

#[test]
fn second_batch_latch_mappers_follow_reference_bank_bits() {
    let mut m38 = Mapper::new(38, 8, 4, Mirroring::Vertical, 0).expect("mapper 38");
    assert!(m38.write_low_register(0x7000, 0x0B));
    assert_eq!(m38.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m38.chr_index(0x0010), 2 * 0x2000 + 0x10);

    let mut m79 = Mapper::new(79, 2, 8, Mirroring::Horizontal, 0).expect("mapper 79");
    m79.write_expansion(0x4000, 0x0F);
    assert_eq!(m79.prg_index(0x8000), 0);
    assert_eq!(m79.chr_index(0x0010), 0x10);
    m79.write_expansion(0x4100, 0x0F);
    assert_eq!(m79.prg_index(0x8000), 0x8000);
    assert_eq!(m79.chr_index(0x0010), 7 * 0x2000 + 0x10);
    m79.write_register(0x8000, 0x02);
    assert_eq!(m79.prg_index(0x8000), 0);
    assert_eq!(m79.chr_index(0x0010), 2 * 0x2000 + 0x10);

    let mut m89 = Mapper::new(89, 8, 16, Mirroring::Horizontal, 0).expect("mapper 89");
    m89.write_register(0x8000, 0x98);
    assert_eq!(m89.prg_index(0x8000), 0x4000);
    assert_eq!(m89.chr_index(0x0010), 8 * 0x2000 + 0x10);
    assert_eq!(m89.mirroring(), Mirroring::SingleScreenHigh);

    let mut m107 = Mapper::new(107, 16, 16, Mirroring::Vertical, 0).expect("mapper 107");
    m107.write_register(0x8000, 0x0B);
    assert_eq!(m107.prg_index(0x8000), 5 * 0x8000);
    assert_eq!(m107.chr_index(0x0010), 11 * 0x2000 + 0x10);

    let mut m203 = Mapper::new(203, 8, 4, Mirroring::Horizontal, 0).expect("mapper 203");
    m203.write_register(0x8000, 0x0D);
    assert_eq!(m203.prg_index(0x8000), 3 * 0x4000);
    assert_eq!(m203.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m203.chr_index(0x0010), 0x2000 + 0x10);
}

#[test]
fn vrc1_mapper75_switches_prg_chr_and_mirroring() {
    let mut m75 = Mapper::new(75, 16, 32, Mirroring::Vertical, 0).expect("mapper 75");
    m75.write_register(0x8000, 3);
    m75.write_register(0xA000, 4);
    m75.write_register(0xC000, 5);
    assert_eq!(m75.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m75.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m75.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m75.prg_index(0xE004), 31 * 0x2000 + 4);

    m75.write_register(0xE000, 0x07);
    m75.write_register(0xF000, 0x09);
    assert_eq!(m75.chr_index(0x0004), 7 * 0x1000 + 4);
    assert_eq!(m75.chr_index(0x1004), 9 * 0x1000 + 4);
    assert_eq!(m75.mirroring(), Mirroring::Horizontal);

    m75.write_register(0x9000, 0x07);
    assert_eq!(m75.chr_index(0x0004), 0x17 * 0x1000 + 4);
    assert_eq!(m75.chr_index(0x1004), 0x19 * 0x1000 + 4);
    assert_eq!(m75.mirroring(), Mirroring::Vertical);
}

#[test]
fn mapper96_uses_ppu_nametable_latch_for_low_chr_bank() {
    let mut m96 = Mapper::new(96, 8, 8, Mirroring::Vertical, 0).expect("mapper 96");
    assert!(m96.watches_ppu_bus());
    assert_eq!(m96.mirroring(), Mirroring::SingleScreenLow);

    m96.write_register(0x8000, 0x06);
    assert_eq!(m96.prg_index(0x8000), 2 * 0x8000);
    assert_eq!(m96.prg_index(0xFFFF), 2 * 0x8000 + 0x7FFF);
    assert_eq!(m96.chr_index(0x0000), 0x04 * 0x1000);
    assert_eq!(m96.chr_index(0x1000), 0x07 * 0x1000);

    m96.notify_a12(0x2100, 0);
    assert_eq!(m96.chr_index(0x0000), 0x05 * 0x1000);
    m96.notify_a12(0x2300, 1);
    assert_eq!(m96.chr_index(0x0FFF), 0x07 * 0x1000 + 0x0FFF);

    m96.notify_a12(0x1000, 2);
    assert_eq!(m96.chr_index(0x0000), 0x07 * 0x1000);
}

#[test]
fn mapper99_vs_system_latches_prg_chr_from_controller_strobe() {
    let mut m99 = Mapper::new(99, 4, 2, Mirroring::FourScreen, 0).expect("mapper 99");
    assert_eq!(m99.mirroring(), Mirroring::FourScreen);
    assert_eq!(m99.prg_index(0x8004), 0x0004);
    assert_eq!(m99.prg_index(0xA004), 1 * 0x2000 + 4);
    assert_eq!(m99.prg_index(0xE004), 3 * 0x2000 + 4);
    assert_eq!(m99.chr_index(0x1004), 0x1004);

    assert!(m99.write_controller_strobe(0x04));
    assert_eq!(m99.prg_index(0x8004), 4 * 0x2000 + 4);
    assert_eq!(m99.prg_index(0xA004), 1 * 0x2000 + 4);
    assert_eq!(m99.chr_index(0x0004), 1 * 0x2000 + 4);

    m99.reset(true);
    assert_eq!(m99.prg_index(0x8004), 0x0004);
    assert_eq!(m99.chr_index(0x0004), 0x0004);
}

#[test]
fn mapper91_switches_jy_banks_and_hblank_irq() {
    let mut m91 = Mapper::new(91, 64, 128, Mirroring::Horizontal, 0).expect("mapper 91");
    assert!(m91.clocks_hblank());
    assert!(!m91.clocks_cpu());
    assert!(!m91.watches_ppu_bus());

    assert!(m91.write_low_register(0x6000, 3));
    assert!(m91.write_low_register(0x6001, 4));
    assert!(m91.write_low_register(0x6002, 5));
    assert!(m91.write_low_register(0x6003, 6));
    assert_eq!(m91.chr_index(0x0004), 3 * 0x0800 + 4);
    assert_eq!(m91.chr_index(0x0804), 4 * 0x0800 + 4);
    assert_eq!(m91.chr_index(0x1004), 5 * 0x0800 + 4);
    assert_eq!(m91.chr_index(0x1804), 6 * 0x0800 + 4);

    assert!(m91.write_low_register(0x7000, 7));
    assert!(m91.write_low_register(0x7001, 8));
    assert_eq!(m91.prg_index(0x8004), 7 * 0x2000 + 4);
    assert_eq!(m91.prg_index(0xA004), 8 * 0x2000 + 4);
    assert_eq!(m91.prg_index(0xC004), 0x0E * 0x2000 + 4);
    assert_eq!(m91.prg_index(0xE004), 0x0F * 0x2000 + 4);

    assert!(m91.write_low_register(0x7003, 0));
    for _ in 0..7 {
        m91.hblank_clock(0, 260);
        assert!(!m91.irq());
    }
    m91.hblank_clock(7, 260);
    assert!(m91.irq());

    assert!(m91.write_low_register(0x7002, 0));
    assert!(!m91.irq());
    m91.hblank_clock(8, 260);
    assert!(!m91.irq());
}

#[test]
fn mapper91_submapper1_selects_outer_bank_and_mirroring_latch() {
    let mut m91 = Mapper::new(91, 128, 512, Mirroring::Vertical, 1).expect("mapper 91 sub1");
    assert!(m91.write_low_register(0x6000, 2));
    assert!(m91.write_low_register(0x7000, 3));
    m91.write_register(0x8005, 0);
    assert_eq!(m91.prg_index(0x8004), (3 | 0x20) * 0x2000 + 4);
    assert_eq!(m91.chr_index(0x0004), (2 | 0x100) * 0x0800 + 4);

    assert!(m91.write_low_register(0x6004, 0));
    assert_eq!(m91.mirroring(), Mirroring::Vertical);
    assert!(m91.write_low_register(0x6005, 1));
    assert_eq!(m91.mirroring(), Mirroring::Horizontal);
}

#[test]
fn mapper35_switches_jy_banks_and_a12_irq() {
    let mut m35 = Mapper::new(35, 32, 16, Mirroring::Horizontal, 0).expect("mapper 35");
    assert!(m35.watches_ppu_bus());
    assert!(!m35.clocks_cpu());

    m35.write_register(0x8000, 3);
    m35.write_register(0x8001, 4);
    m35.write_register(0x8002, 5);
    m35.write_register(0x9006, 9);
    assert_eq!(m35.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m35.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m35.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m35.prg_index(0xE004), 63 * 0x2000 + 4);
    assert_eq!(m35.chr_index(0x1804), 9 * 0x0400 + 4);

    m35.write_register(0xD001, 0);
    assert_eq!(m35.mirroring(), Mirroring::Vertical);
    m35.write_register(0xD001, 1);
    assert_eq!(m35.mirroring(), Mirroring::Horizontal);

    m35.write_register(0xC005, 2);
    m35.write_register(0xC003, 0);
    m35.notify_a12(0x0000, 1);
    m35.notify_a12(0x1000, 12);
    assert!(!m35.irq());
    m35.notify_a12(0x0000, 13);
    m35.notify_a12(0x1000, 24);
    assert!(m35.irq());
    m35.clear_irq();
    assert!(!m35.irq());
    m35.notify_a12(0x0000, 25);
    m35.notify_a12(0x1000, 36);
    assert!(!m35.irq());
}

#[test]
fn jy_asic_mappers_switch_prg_chr_alu_nametable_and_irq() {
    let mut m90 = Mapper::new(90, 64, 128, Mirroring::Vertical, 0).expect("mapper 90");
    assert!(m90.watches_ppu_bus());
    assert!(m90.clocks_cpu());
    assert!(m90.clocks_hblank());
    assert_eq!(m90.prg_index(0x8004), 0x3C * 0x2000 + 4);
    assert_eq!(m90.prg_index(0xE004), 0x3F * 0x2000 + 4);

    m90.write_register(0x8000, 3);
    m90.write_register(0x8001, 4);
    m90.write_register(0x8002, 5);
    m90.write_register(0x8003, 6);
    m90.write_register(0xD000, 0x82);
    assert_eq!(m90.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m90.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m90.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m90.prg_index(0xE004), 0x3F * 0x2000 + 4);
    assert_eq!(m90.low_prg_index(0x6004), Some(6 * 0x2000 + 4));

    m90.write_register(0x9000, 2);
    m90.write_register(0xD000, 0x80);
    assert_eq!(m90.chr_index(0x0004), 2 * 0x2000 + 4);
    m90.write_register(0xD000, 0x98);
    m90.write_register(0x9007, 9);
    assert_eq!(m90.chr_index(0x1C04), 9 * 0x0400 + 4);

    m90.write_expansion(0x5800, 9);
    m90.write_expansion(0x5801, 7);
    m90.write_expansion(0x5802, 5);
    assert_eq!(m90.read_expansion_with_open_bus(0x5800, 0xAA), Some(63));
    assert_eq!(m90.read_expansion_with_open_bus(0x5801, 0xAA), Some(0));
    assert_eq!(m90.read_expansion_with_open_bus(0x5802, 0xAA), Some(5));
    assert_eq!(m90.read_expansion_with_open_bus(0x5000, 0xAA), Some(0x2A));

    m90.write_register(0xC001, 0x40);
    m90.write_register(0xC004, 0xFF);
    m90.write_register(0xC005, 0xFF);
    m90.write_register(0xC003, 0);
    m90.cpu_clock();
    assert!(m90.irq());

    let mut m209 = Mapper::new(209, 64, 128, Mirroring::Vertical, 0).expect("mapper 209");
    m209.write_register(0x9000, 1);
    m209.write_register(0x9002, 2);
    m209.write_register(0x9004, 3);
    m209.write_register(0x9006, 4);
    m209.write_register(0xD000, 0x08);
    assert_eq!(m209.chr_index(0x0004), 1 * 0x1000 + 4);
    m209.notify_a12(0x0FE8, 1);
    assert_eq!(m209.chr_index(0x0004), 2 * 0x1000 + 4);
    m209.notify_a12(0x1FE8, 2);
    assert_eq!(m209.chr_index(0x1004), 4 * 0x1000 + 4);

    let mut m211 = Mapper::new(211, 64, 128, Mirroring::Horizontal, 0).expect("mapper 211");
    assert!(m211.has_nametable_chr_mapping());
    m211.write_register(0xB000, 0x80);
    m211.write_register(0xD002, 0x00);
    assert_eq!(m211.nametable_chr_index(0x2004), Some(0x80 * 0x0400 + 4));
    m211.write_register(0xD000, 0x40);
    m211.write_register(0xB001, 0x03);
    assert_eq!(m211.nametable_chr_index(0x2404), Some(3 * 0x0400 + 4));
}

#[test]
fn taito_x1_mappers_follow_low_register_banking() {
    let mut m80 = Mapper::new(80, 16, 16, Mirroring::Vertical, 0).expect("mapper 80");
    assert!(m80.write_low_register(0x7EFA, 3));
    assert!(m80.write_low_register(0x7EFC, 4));
    assert!(m80.write_low_register(0x7EFE, 5));
    assert_eq!(m80.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m80.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m80.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m80.prg_index(0xE004), 31 * 0x2000 + 4);
    assert!(m80.write_low_register(0x7EF0, 0x07));
    assert!(m80.write_low_register(0x7EF2, 0x22));
    assert_eq!(m80.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m80.chr_index(0x0404), 7 * 0x0400 + 4);
    assert_eq!(m80.chr_index(0x1004), 0x22 * 0x0400 + 4);
    assert!(m80.write_low_register(0x7EF6, 1));
    assert_eq!(m80.mirroring(), Mirroring::Horizontal);
    assert_eq!(m80.read_low_register(0x7F42), Some(0xFF));
    assert!(m80.write_low_register(0x7EF8, 0xA3));
    assert!(m80.write_low_register(0x7F42, 0x5A));
    assert_eq!(m80.peek_low_register(0x7F42), Some(0x5A));

    let mut m82 = Mapper::new(82, 16, 16, Mirroring::Vertical, 0).expect("mapper 82");
    assert!(m82.write_low_register(0x7EFA, 0x0C));
    assert!(m82.write_low_register(0x7EFB, 0x10));
    assert!(m82.write_low_register(0x7EFC, 0x14));
    assert_eq!(m82.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m82.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m82.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m82.prg_index(0xE004), 31 * 0x2000 + 4);
    assert!(m82.write_low_register(0x7EF0, 0x06));
    assert!(m82.write_low_register(0x7EF2, 0x22));
    assert_eq!(m82.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m82.chr_index(0x0404), 7 * 0x0400 + 4);
    assert_eq!(m82.chr_index(0x1004), 0x22 * 0x0400 + 4);
    assert!(m82.write_low_register(0x7EF6, 0x03));
    assert_eq!(m82.mirroring(), Mirroring::Horizontal);
    assert_eq!(m82.chr_index(0x1004), 6 * 0x0400 + 4);
}

#[test]
fn unrom_variants_and_irem_tams1_map_fixed_banks_correctly() {
    let mut m93 = Mapper::new(93, 8, 0, Mirroring::Vertical, 0).expect("mapper 93");
    m93.write_register(0x8000, 0x70);
    assert_eq!(m93.prg_index(0x8000), 7 * 0x4000);
    assert_eq!(m93.prg_index(0xC000), 7 * 0x4000);

    let mut m94 = Mapper::new(94, 8, 0, Mirroring::Vertical, 0).expect("mapper 94");
    m94.write_register(0x8000, 0x1C);
    assert_eq!(m94.prg_index(0x8000), 7 * 0x4000);
    assert_eq!(m94.prg_index(0xC000), 7 * 0x4000);

    let mut m180 = Mapper::new(180, 8, 0, Mirroring::Horizontal, 0).expect("mapper 180");
    m180.write_register(0x8000, 7);
    assert_eq!(m180.prg_index(0x8000), 0);
    assert_eq!(m180.prg_index(0xC000), 7 * 0x4000);

    let mut m97 = Mapper::new(97, 8, 0, Mirroring::Horizontal, 0).expect("mapper 97");
    m97.write_register(0x8000, 0x8A);
    assert_eq!(m97.prg_index(0x8000), 7 * 0x4000);
    assert_eq!(m97.prg_index(0xC000), 10 * 0x4000);
    assert_eq!(m97.mirroring(), Mirroring::Vertical);
}

#[test]
fn expansion_and_low_latch_mappers_update_on_reference_windows() {
    let mut m86 = Mapper::new(86, 8, 8, Mirroring::Horizontal, 0).expect("mapper 86");
    assert!(m86.write_low_register(0x6000, 0x32));
    assert_eq!(m86.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m86.chr_index(0x0010), 2 * 0x2000 + 0x10);
    assert!(m86.write_low_register(0x7000, 0xFF)); // audio register window; no bank change
    assert_eq!(m86.chr_index(0x0010), 2 * 0x2000 + 0x10);

    let mut m113 = Mapper::new(113, 16, 16, Mirroring::Horizontal, 0).expect("mapper 113");
    m113.write_expansion(0x4100, 0xCF);
    assert_eq!(m113.prg_index(0x8000), 1 * 0x8000);
    assert_eq!(m113.chr_index(0x0010), 15 * 0x2000 + 0x10);
    assert_eq!(m113.mirroring(), Mirroring::Vertical);

    let mut m140 = Mapper::new(140, 8, 8, Mirroring::Vertical, 0).expect("mapper 140");
    assert!(m140.write_low_register(0x6000, 0x32));
    assert_eq!(m140.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m140.chr_index(0x0010), 2 * 0x2000 + 0x10);
}

#[test]
fn expansion_audio_mappers_expose_audible_outputs_and_reference_registers() {
    let mut fme7 = Mapper::new(69, 8, 8, Mirroring::Vertical, 0).expect("fme7");
    assert!(fme7.has_expansion_audio());
    assert!(fme7.clocks_cpu());
    fme7.write_register(0x8000, 9);
    fme7.write_register(0xA000, 2);
    assert_eq!(fme7.prg_index(0x8004), 2 * 0x2000 + 4);
    fme7.write_register(0x8000, 8);
    fme7.write_register(0xA000, 1);
    assert_eq!(fme7.low_prg_index(0x6004), Some(0x2000 + 4));
    fme7.write_register(0x8000, 0x0C);
    fme7.write_register(0xA000, 1);
    assert_eq!(fme7.mirroring(), Mirroring::Horizontal);
    fme7.write_register(0xC000, 0x18);
    fme7.write_register(0xE000, 0x0F);
    assert_eq!(fme7.expansion_audio(), 0.0);
    fme7.write_register(0xC000, 8);
    fme7.write_register(0xE000, 0x0F);
    assert!(fme7.expansion_audio() > 0.0);

    let mut n163 = Mapper::new(19, 8, 8, Mirroring::Vertical, 0).expect("namco 163");
    assert!(n163.has_expansion_audio());
    assert!(n163.clocks_cpu());
    n163.write_register(0xE000, 3);
    n163.write_register(0xE800, 4);
    n163.write_register(0xF000, 5);
    assert_eq!(n163.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(n163.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(n163.prg_index(0xC004), 5 * 0x2000 + 4);
    n163.write_register(0xF800, 0x80);
    n163.write_expansion(0x4800, 0xAB);
    n163.write_expansion(0x4800, 0xCD);
    n163.write_register(0xF800, 0x80);
    assert_eq!(n163.read_expansion(0x4800), Some(0xAB));
    assert_eq!(n163.read_expansion(0x4800), Some(0xCD));
    let mut ciram = [0u8; 0x1000];
    n163.write_register(0xC000, 0xE1);
    assert!(n163.nametable_write(0x2005, 0x42, &mut ciram));
    assert_eq!(n163.nametable_read(0x2005, &ciram), Some(0x42));
    n163.write_register(0xF800, 0x80);
    n163.write_expansion(0x4800, 0xFF);
    n163.write_register(0xF800, 0xF8);
    for value in [1, 0, 0, 0, 0, 0, 0, 0x0F] {
        n163.write_expansion(0x4800, value);
    }
    for _ in 0..15 {
        n163.cpu_clock();
    }
    assert!(n163.expansion_audio() > 0.0);

    let mut namcot340 = Mapper::new(210, 8, 8, Mirroring::Vertical, 2).expect("namcot 340");
    assert!(!namcot340.has_expansion_audio());
    assert!(!namcot340.clocks_cpu());
    namcot340.write_register(0xE000, 0x83);
    namcot340.write_register(0xE800, 4);
    namcot340.write_register(0xF000, 5);
    namcot340.write_register(0x9000, 7);
    assert_eq!(namcot340.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(namcot340.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(namcot340.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(namcot340.chr_index(0x0804), 7 * 0x0400 + 4);
    assert_eq!(namcot340.mirroring(), Mirroring::Horizontal);
    assert_eq!(namcot340.read_expansion(0x4800), None);
    namcot340.write_register(0xF800, 0x80);
    namcot340.write_expansion(0x4800, 0xFF);
    for _ in 0..15 {
        namcot340.cpu_clock();
    }
    assert_eq!(namcot340.expansion_audio(), 0.0);

    let mut vrc6 = Mapper::new(24, 8, 8, Mirroring::Vertical, 0).expect("vrc6a");
    assert!(vrc6.has_expansion_audio());
    assert!(vrc6.clocks_cpu());
    vrc6.write_register(0xD000, 3);
    vrc6.write_register(0xB003, 0x21);
    assert_eq!(vrc6.chr_index(0x0004), 2 * 0x0400 + 4);
    assert_eq!(vrc6.chr_index(0x0404), 3 * 0x0400 + 4);
    vrc6.write_register(0xB003, 0x23);
    assert_eq!(vrc6.mirroring(), Mirroring::Horizontal);
    vrc6.write_register(0x9000, 0x8F);
    vrc6.write_register(0x9001, 1);
    vrc6.write_register(0x9002, 0x80);
    assert!(vrc6.expansion_audio() > 0.0);
    vrc6.reset(true);
    assert_eq!(vrc6.expansion_audio(), 0.0);

    let mut vrc6b = Mapper::new(26, 8, 8, Mirroring::Vertical, 0).expect("vrc6b");
    vrc6b.write_register(0x9000, 0x8F);
    vrc6b.write_register(0x9001, 0x80);
    assert!(vrc6b.expansion_audio() > 0.0);

    let mut vrc7 = Mapper::new(85, 8, 8, Mirroring::Vertical, 0).expect("vrc7");
    assert!(vrc7.has_expansion_audio());
    assert!(vrc7.clocks_cpu());
    for (reg, value) in [
        (0x00, 0x21),
        (0x01, 0x21),
        (0x02, 0x00),
        (0x03, 0x00),
        (0x04, 0xF7),
        (0x05, 0xF7),
        (0x06, 0x10),
        (0x07, 0x10),
        (0x30, 0x00),
        (0x10, 0x00),
        (0x20, 0x19),
    ] {
        vrc7.write_register(0x9010, reg);
        vrc7.write_register(0x9030, value);
    }
    let mut peak = 0.0f32;
    for _ in 0..25_000 {
        vrc7.cpu_clock();
        peak = peak.max(vrc7.expansion_audio().abs());
    }
    assert!(peak > 0.0);
    vrc7.write_register(0xE000, 0x40);
    assert_eq!(vrc7.expansion_audio(), 0.0);
    vrc7.write_register(0xE000, 0x00);
    vrc7.reset(true);
    assert_eq!(vrc7.expansion_audio(), 0.0);
}
