use super::super::*;

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
        (157, false),  // Datach Barcode Battler
        (162, false),  // Waixing FS304
        (163, false),  // Nanjing FC-001
        (164, false),  // Waixing FFV / PEC-9588
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
        (176, true),   // Mapper 176 FK23C MMC3 A12 IRQ
        (177, false),  // Mapper 177
        (178, false),  // Waixing FS305/NJ0430
        (179, false),  // Henggedianzi XJZB
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
        (223, true),   // Mapper 223 MMC3 A12 IRQ
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
        (252, false),  // Mapper 252 IRQ is CPU-clocked
        (253, false),  // Mapper 253 IRQ is CPU-clocked, not PPU-bus-clocked
        (254, true),   // Mapper 254 MMC3 A12 IRQ
        (255, false),  // Mapper 255
        (258, true),   // Mapper 258 MMC3 A12 IRQ
        (264, false),  // Mapper 264 Yoko-derived board has no PPU bus latch
        (265, false),  // Mapper 265
        (266, true),   // Mapper 266 MMC3 A12 IRQ
        (267, true),   // Mapper 267 MMC3 A12 IRQ
        (271, false),  // Mapper 271
        (272, true),   // Mapper 272 watches PA13 falling edges
        (273, false),  // Mapper 273 IRQ is CPU-clocked, not PPU-bus-clocked
        (277, false),  // Mapper 277
        (280, false),  // Mapper 280
        (281, true),   // JY ASIC watches PPU bus for PPU-read IRQ / CHR latch
        (282, true),   // JY ASIC watches PPU bus for PPU-read IRQ / CHR latch
        (283, false),  // Mapper 283
        (285, false),  // Mapper 285
        (288, false),  // Mapper 288
        (291, true),   // Mapper 291 MMC3 A12 IRQ
        (293, false),  // Mapper 293
        (294, false),  // Mapper 294
        (295, true),   // JY ASIC watches PPU bus for PPU-read IRQ / CHR latch
        (297, false),  // Mapper 297 switches between Mapper70 latch and MMC1
        (298, false),  // TF1201 IRQ is CPU-prescaled, not PPU-bus-clocked
        (301, false),  // Mapper 301
        (308, false),  // Mapper 308 IRQ is CPU-clocked, not PPU-bus-clocked
        (310, false),  // Mapper 310
        (319, false),  // Mapper 319
        (321, true),   // Mapper 321 MMC3 A12 IRQ
        (326, false),  // Mapper 326
        (330, false),  // Mapper 330 IRQ is CPU-clocked
        (334, true),   // Mapper 334 MMC3 A12 IRQ
        (340, false),  // Mapper 340
        (341, false),  // Mapper 341
        (343, false),  // Mapper 343
        (352, false),  // Mapper 352
        (354, false),  // Mapper 354
        (357, false),  // Mapper 357
        (358, true),   // JY ASIC watches PPU bus for PPU-read IRQ / CHR latch
        (360, false),  // Mapper 360
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
        (157, true),   // Datach IRQ/barcode reader clocks per CPU cycle
        (162, false),  // Waixing FS304
        (163, false),  // Nanjing FC-001
        (164, false),  // Waixing FFV / PEC-9588
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
        (176, false),  // Mapper 176 uses PPU A12 edges
        (177, false),  // Mapper 177
        (178, false),  // Waixing FS305/NJ0430
        (179, false),  // Henggedianzi XJZB
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
        (223, false),  // Mapper 223 uses PPU A12 edges
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
        (252, true),   // Mapper 252 VRC4-style IRQ clocks per CPU cycle
        (253, true),   // Mapper 253 IRQ counter clocks per CPU cycle
        (254, false),  // Mapper 254 uses PPU A12 edges
        (255, false),  // Mapper 255
        (264, true),   // Mapper 264 can use CPU-cycle IRQ mode
        (265, false),  // Mapper 265
        (272, false),  // Mapper 272 uses PPU PA13 edges
        (273, true),   // Mapper 273 IRQ counter clocks per CPU cycle
        (271, false),  // Mapper 271
        (277, false),  // Mapper 277
        (280, false),  // Mapper 280
        (281, true),   // JY ASIC can clock IRQs from CPU cycles
        (282, true),   // JY ASIC can clock IRQs from CPU cycles
        (283, false),  // Mapper 283
        (285, false),  // Mapper 285
        (288, false),  // Mapper 288
        (293, false),  // Mapper 293
        (294, false),  // Mapper 294
        (295, true),   // JY ASIC can clock IRQs from CPU cycles
        (297, false),  // Mapper 297 has no IRQ clock
        (298, true),   // TF1201 IRQ counter clocks through CPU-cycle prescaler
        (301, false),  // Mapper 301
        (308, true),   // Mapper 308 IRQ counter clocks per CPU cycle
        (310, false),  // Mapper 310
        (319, false),  // Mapper 319
        (321, false),  // Mapper 321 uses PPU A12 edges
        (326, false),  // Mapper 326
        (330, true),   // Mapper 330 IRQ counter clocks per CPU cycle
        (334, false),  // Mapper 334 uses PPU A12 edges
        (340, false),  // Mapper 340
        (341, false),  // Mapper 341
        (343, false),  // Mapper 343
        (352, false),  // Mapper 352
        (354, false),  // Mapper 354
        (357, true),   // Mapper 357 IRQ counter clocks per CPU cycle
        (358, true),   // JY ASIC can clock IRQs from CPU cycles
        (360, false),  // Mapper 360
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
        (157, false),
        (162, true),
        (163, true),
        (164, false),
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
        (176, false),
        (177, false),
        (178, false),
        (179, false),
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
        (223, false),
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
        (252, false),
        (253, false),
        (254, false),
        (255, false),
        (264, true),
        (265, false),
        (272, false),
        (273, false),
        (271, false),
        (277, false),
        (280, false),
        (281, true),
        (282, true),
        (283, false),
        (285, false),
        (288, false),
        (293, false),
        (294, false),
        (295, true),
        (297, false),
        (298, false),
        (301, false),
        (308, false),
        (310, false),
        (319, false),
        (321, false),
        (326, false),
        (330, false),
        (334, false),
        (340, false),
        (341, false),
        (343, false),
        (352, false),
        (354, false),
        (358, true),
        (360, false),
    ];
    for (num, expected) in cases {
        let submapper = if num == 34 { 2 } else { 0 };
        let m = Mapper::new(num, 2, 1, mir, submapper).expect("construct mapper");
        assert_eq!(m.clocks_hblank(), expected, "mapper {num}");
    }
}
