## ADDED Requirements

### Requirement: 命名表/地图编辑
系统 SHALL 提供内置地图编辑器,在网格上用工程 CHR 图块拼接命名表(nametable),支持设置每 16×16 区块的属性(调色板)与一层碰撞标记。编辑器 MUST 以当前工程的 CHR 资源作为图块调色盘来源。

#### Scenario: 放置图块
- **WHEN** 用户从图块调色盘选一个图块并在地图网格点击
- **THEN** 该网格单元显示并记录所选图块索引

#### Scenario: 设置属性与碰撞
- **WHEN** 用户切换到属性/碰撞层并标记某区块
- **THEN** 系统记录该区块的调色板属性或碰撞标记,可视化叠加显示

### Requirement: 地图导出与工程登记
系统 SHALL 把地图导出为字节数组——命名表字节 + 属性表字节(+ 可选碰撞层)——为 `.bin`(供 `.incbin`)或 `.inc`(供 `.include`),落盘到 `map/` 并在 `project.toml` 登记。导出布局 MUST 文档化以便 asm 端解读。

#### Scenario: 导出地图字节
- **WHEN** 用户导出当前地图
- **THEN** 系统写出 `map/<name>.bin`(或 `.inc`),内容含命名表 + 属性(+ 碰撞)字节,并登记进工程

#### Scenario: 导出可进构建
- **WHEN** 工程 asm 以 `.incbin`/`.include` 引用导出的地图并构建
- **THEN** 构建成功,地图字节进入 `.nes`
