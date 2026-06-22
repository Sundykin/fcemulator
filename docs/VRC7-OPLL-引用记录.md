# VRC7 / OPLL 引用记录

这份记录只用于当前 `fc-core` 的 VRC7/OPLL 对齐与回溯。
如果将来要做闭源分支，直接替换下面标出的代码块即可。

## 当前代码范围

- `fc-core/src/mapper/expansion_audio.rs:426-888`
  - `Vrc7Audio` wrapper
  - OPLL / VRC7 内部状态快照
  - write / mute / reset / clock / output / replay
- `fc-core/src/mapper/expansion_audio.rs:922-1012`
  - VRC7 回归测试
- `fc-core/src/mapper.rs:1444-1545`
  - mapper 级 VRC7/VRC6/FME7/N163 音频测试
- `fc-core/Cargo.toml:10-17`
  - 当前使用的 `oxideav-nsf` 依赖

## 对照来源

| 参考来源 | 行号 | 主要用途 |
|---|---:|---|
| `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/vrc7.c` | 133-176, 199-253 | VRC7 寄存器选择/写入、音频 reset、初始化、状态保存、NSF 端口映射 |
| `/Users/sunmeng/workspace/fc/libretro-fceumm/src/boards/emu2413.c` | 51-69, 71-219 | VRC7 预设 patch、OPLL 表/参数参考 |
| `/Users/sunmeng/workspace/fc/Mesen2/Core/NES/Mappers/Audio/Vrc7Audio.h` | 10-90 | 音频 clock、寄存器写入、mute、序列化骨架 |
| `/Users/sunmeng/workspace/fc/Mesen2/Core/Shared/Utilities/emu2413.h` | 83-181, 191-227 | OPLL 结构、API、chip type / patch 相关定义 |
| `/Users/sunmeng/workspace/fc/Mesen2/Core/Shared/Utilities/emu2413.cpp` | 42-104, 211-218 | VRC7 preset、LFO / table 构建的实现参考 |
| `/Users/sunmeng/workspace/fc/Mesen2/Core/Shared/Utilities/Emu2413Serializer.h` | 5-90 | OPLL 完整状态序列化字段清单 |
| `/Users/sunmeng/workspace/fc/nestopia/source/core/board/NstBoardKonamiVrc7.cpp` | 36-43, 53-219, 238-470, 476-1024 | VRC7 patch table、寄存器映射、save/load、采样更新、状态恢复 |

## 当前实现和来源的对应关系

- `Vrc7Audio::write()` 对应 `vrc7.c:141-169` 和 `Vrc7Audio.h:76-90`
- `Vrc7Audio::reset()` / `set_muted()` 对应 `Vrc7Audio.h:66-90` 与 `vrc7.c:199-253`
- `Vrc7Audio` 的完整 state roundtrip 对应 `vrc7.c:223-246`、`Emu2413Serializer.h:8-90`、`NstBoardKonamiVrc7.cpp:343-470`
- 当前的 OPLL 引擎行为对齐参考 `emu2413.c/.h`、`emu2413.cpp/.h`、`NstBoardKonamiVrc7.cpp`

## 以后替换时的删除边界

- 先删/替换 `fc-core/src/mapper/expansion_audio.rs:426-888`
- 再处理 `fc-core/Cargo.toml` 里的 `oxideav-nsf`
- 最后清理这份记录和相关测试
