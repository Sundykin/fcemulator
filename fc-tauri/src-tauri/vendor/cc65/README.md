# Vendored cc65 (ca65 + ld65)

M1 创作最小闭环把 cc65 的 `ca65`(汇编器)与 `ld65`(链接器)作为 sidecar 捆绑,
由 `src-tauri/src/build_pipeline.rs` 调用,完成 `ca65 → ld65 → .nes` 一键构建。

- **来源**: https://github.com/cc65/cc65
- **版本**: V2.19,git `cc3c40c54e51b2d9a22b63c85c418a2b11763377`
- **许可证**: zlib(见 `LICENSE`,允许商业闭源打包)

## 目录布局

预编译二进制按 Rust target-triple 子目录存放:

```
vendor/cc65/
  LICENSE
  README.md
  build-cc65.sh              # 复现/补充其它平台二进制的脚本
  aarch64-apple-darwin/      # macOS Apple Silicon(已 vendor)
    ca65  ld65
  x86_64-apple-darwin/       # 其它平台:用 build-cc65.sh 在对应机器上生成
  x86_64-unknown-linux-gnu/
  x86_64-pc-windows-msvc/    # Windows 下二进制为 ca65.exe / ld65.exe
```

运行时解析顺序(见 `build_pipeline.rs`):
1. 环境变量 `FC_CC65_DIR`(显式覆盖,优先)
2. `vendor/cc65/<host-triple>/`(本捆绑目录)
3. `PATH` 上的 `ca65` / `ld65`(兜底)

> M1 仅保证开发主机 target;跨平台二进制矩阵与 macOS 签名/公证留到 M4。
