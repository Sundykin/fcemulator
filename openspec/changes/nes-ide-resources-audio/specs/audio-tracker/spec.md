## ADDED Requirements

### Requirement: 乐曲工程模型
系统 SHALL 提供 2A03 乐曲模型:乐曲含速度(tempo)、乐器表、Pattern 表与播放顺序(order);通道覆盖 Pulse1/Pulse2/Triangle/Noise/DPCM 五路。乐曲 MUST 可保存到工程并重新加载且内容一致。

#### Scenario: 保存并重载乐曲
- **WHEN** 用户保存一首乐曲后重新打开
- **THEN** 速度、乐器、Pattern、order 与五通道内容与保存前一致

### Requirement: 驱动自研 APU 的播放引擎
系统 SHALL 用 `fc-core` APU 内核做预览播放:逐帧把音符 + 乐器包络 + 效果解算为 APU 寄存器写入并取样输出。预览 MUST 经 `ControlDeck` 暴露的接口驱动,不修改内核锁步时钟。

#### Scenario: 试听乐曲
- **WHEN** 用户点击播放
- **THEN** 系统逐帧驱动 APU 内核发声,通过音频输出听到乐曲

#### Scenario: 停止与定位
- **WHEN** 用户停止或跳到某行
- **THEN** 播放停止或从该位置继续,APU 状态相应复位/前进

### Requirement: 乐器与 DPCM
系统 SHALL 提供乐器系统:音量、琶音、音高、占空比包络;并支持 DPCM 采样映射到音符。

#### Scenario: 应用音量包络
- **WHEN** 一个带音量包络的乐器演奏一个音符
- **THEN** 该音符的输出音量按包络逐帧变化

#### Scenario: DPCM 触发
- **WHEN** DPCM 通道演奏映射了采样的音符
- **THEN** 系统经 APU DMC 播放该采样

### Requirement: 音序与 Pattern 编辑
系统 SHALL 提供音序(order)与 Pattern 编辑视图,可输入/编辑每通道每行的音符、乐器、音量与效果列。

#### Scenario: 输入音符
- **WHEN** 用户在 Pattern 某通道某行输入一个音符
- **THEN** 该行记录音符与当前乐器,播放时按 order 演奏到该 Pattern 时发声

### Requirement: 钢琴卷帘编辑(够用版)
系统 SHALL 提供钢琴卷帘视图编辑音符的音高与时长(M2 仅"够用"水平,不追求专业级人机工程)。

#### Scenario: 卷帘放置音符
- **WHEN** 用户在卷帘上拖出一个音符
- **THEN** 对应通道在该时间/音高记录该音符,与 Pattern 视图数据一致

### Requirement: 汇编导出
系统 SHALL 把乐曲导出为 ca65 汇编(对齐 FamiTone2/FamiStudio 声音引擎格式),并可捆绑声音引擎 asm 作为工程模板,使乐曲经现有 build-pipeline 链接进 `.nes`。

#### Scenario: 导出并构建
- **WHEN** 用户导出乐曲为 ca65 并触发构建
- **THEN** 导出的乐曲数据 + 引擎被链接进 `.nes`,可在内嵌运行中播放

### Requirement: FTM 文本导入
系统 SHALL 支持导入 FamiTracker 的文本导出(FTM text),建立对应乐曲模型(基础保真,不做二进制 FTM/NSF 导入)。

#### Scenario: 导入 FTM 文本
- **WHEN** 用户导入一个 FamiTracker 文本导出
- **THEN** 系统解析出乐曲的乐器/Pattern/order 并可在 tracker 中编辑与试听
