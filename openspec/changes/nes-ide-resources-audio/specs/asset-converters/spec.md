## ADDED Requirements

### Requirement: 图片转 CHR
系统 SHALL 把已量化为 4 色索引的 PNG 图片转换为 NES 2bpp planar CHR 字节(按 8×8 图块切分)。输入颜色超过 4 种或尺寸非 8 的倍数时,系统 MUST 报告明确错误而非产出错误数据。

#### Scenario: 转换合法图片
- **WHEN** 用户对一张 4 色、宽高均为 8 的倍数的 PNG 触发转换
- **THEN** 系统输出对应的 `.chr` 字节(图块数 = (宽/8)×(高/8),每图块 16 字节)

#### Scenario: 拒绝非法图片
- **WHEN** 输入 PNG 含多于 4 种颜色或边长非 8 的倍数
- **THEN** 系统 SHALL 报告具体错误(颜色数/尺寸),不产出 CHR

### Requirement: Tiled 地图转字节数组
系统 SHALL 把 Tiled 导出的地图(CSV/JSON 图层)转换为命名表/属性字节数组,落入 `map/` 供构建引用。

#### Scenario: 转换 Tiled 地图
- **WHEN** 用户导入一个 Tiled 导出的地图图层
- **THEN** 系统输出对应的地图字节文件并可登记进工程
