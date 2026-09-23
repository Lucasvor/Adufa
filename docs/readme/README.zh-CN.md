<div align="center">
  <img src="../assets/adufa-icon.svg" width="112" alt="Adufa 图标：将音频通道重定向至所选输出设备">
  <h1>Adufa</h1>
  <p><strong>让每个应用都使用正确的音频设备。</strong></p>
  <p>快速、本地优先的按应用音频输出切换与音量控制工具。</p>
</div>

<p align="center">
  <a href="../../README.md">English</a> ·
  <a href="../../README.pt-BR.md">Português (Brasil)</a> ·
  <a href="README.es.md">Español</a> ·
  <a href="README.fr.md">Français</a> ·
  <a href="README.de.md">Deutsch</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.ja.md">日本語</a> ·
  <strong>简体中文</strong>
</p>

<p align="center">
  <code>Windows 测试版</code> · <code>计划支持 macOS</code> · <code>计划支持 Linux</code> · <code>GPL-3.0-or-later</code>
</p>

![Adufa — 让每个应用都使用正确的音频设备](../assets/adufa-hero.svg)

## 为什么选择 Adufa？

视频通话应使用耳机，音乐应从扬声器播放，浏览器有时需要显示器或虚拟音频线。
Adufa 能显示正在发声的应用，并将每个应用发送到合适的输出设备，无需每次都在
Windows 音量混合器中翻找。

Adufa 刻意保持小巧：它驻留在通知区域，在工作位置附近打开，记住应用路由，
使用完毕后不打扰你。

## 查看实际效果

![Adufa 简短演示：托盘弹窗、任务栏辅助面板、音量控制和输出选择](../assets/adufa-demo.gif)

此动画是面向文档、以确定方式渲染的原生界面，不包含桌面截图或个人数据。

## 当前可用功能

当前公开测试版支持 **Windows 10 22H2 和 Windows 11**。

- 自动发现具有活动音频会话的应用。
- 更改单个应用的输出设备，不影响系统默认设备。
- 在 Adufa 或应用重启后保留应用路由。
- 一次选择即可让应用恢复为`系统默认`（`System default`）。
- 调整每个应用当前的音量和静音状态。
- 在受支持的 Windows 11 版本中，右键单击正在运行的任务栏应用，即使尚未建立音频会话，
  也可在原生菜单旁打开实验性的输出、音量和静音控制。
- 提供 **查找声音**（`Find sound`）临时实时视图，突出显示声音最大的应用。
- 使用 `Ctrl + Alt + A` 在光标附近打开快速选择器。
- 可在登录时启动；此选项在你主动启用前保持关闭。
- 支持英语、巴西葡萄牙语、西班牙语、法语、德语、意大利语、日语和简体中文。
- 检测 Windows 显示语言，也允许手动选择语言。
- 完全本地运行，无账户、分析、遥测或音频上传。

### 实验性 Windows 集成

在受支持的 Windows 11 版本中，右键单击正在运行应用的任务栏图标，即使应用尚未播放
音频，也可以在原生任务栏菜单旁打开 Adufa 的紧凑辅助面板。原生菜单仍然可用；Adufa
只是补充音量和输出控制。

此集成会优先使用任务栏按钮的 AppID，并以可访问名称和可执行文件路径作为后备方案。它仍是
测试版功能；当 Windows 无法提供可靠匹配时，可以改用全局快捷键或托盘弹窗。

## 安装测试版

### 下载便携版本

带标签的版本由 GitHub Actions 构建。打开 [Releases 页面](../../../releases)，下载并运行
`Adufa-Windows-x64.exe`。

初始测试版可执行文件为便携版且未签名。在签名安装包可用之前，Windows 可能显示
SmartScreen 警告。允许运行前，请确认文件来自本仓库的 release。

### 从源代码构建

要求：

- Windows 10 22H2 或 Windows 11，x64；
- [Rust](https://www.rust-lang.org/tools/install) 1.85 或更高版本，以及 MSVC toolchain；
- 安装了 **使用 C++ 的桌面开发** 和 Windows SDK 的 Visual Studio Build Tools。

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --locked -p router-windows --target x86_64-pc-windows-msvc
```

运行：

```powershell
.\target\x86_64-pc-windows-msvc\release\router-windows.exe
```

日常使用请采用 release 构建。release 构建是 Windows GUI 应用，不会打开终端窗口。
debug 构建会有意保留控制台以便诊断。

## 使用方法

### 从托盘路由应用

1. 在要路由的应用中开始播放音频。
2. 打开通知区域的隐藏图标并选择 Adufa。
3. 选择应用所在的行。
4. 选择一个输出，或选择`系统默认`（`System default`）以删除已保存路由。
5. 单击弹窗外部或按 `Esc` 将其关闭。

路由关联的是稳定的应用身份，而不是临时进程 ID。应用重启后，只要平台能安全识别它，
Adufa 就会恢复已保存的选择。

### 查找哪个应用正在发声

1. 打开 Adufa。
2. 选择 **查找声音**（`Find sound`）。
3. 查看实时电平指示器；最强的可听音源会被突出显示。
4. 选择该应用以更改其输出。

查找声音不会录音。它只读取 Windows 已提供的会话峰值，并在紧凑弹窗关闭时停止。

### 使用任务栏辅助面板

1. 保持目标应用运行，然后右键单击其任务栏图标。此时不需要正在播放音频。
2. 在相邻的 Adufa 面板中静音、调整音量或选择输出。
3. 选择输出后，辅助面板和原生菜单都会关闭。应用创建音频会话后，Windows 会应用所选路由。

如果辅助面板没有出现，请在指向目标应用时按 `Ctrl + Alt + A`，或从通知区域打开 Adufa。

### 更改语言或启动行为

打开 **设置**（`Settings`）可以：

- 跟随 Windows 语言，或选择任一受支持语言；
- 启用或禁用登录时打开 Adufa；
- 打开 Windows 音量混合器进行系统级控制。

## 键盘和鼠标参考

| 输入 | 操作 |
| --- | --- |
| `Ctrl + Alt + A` | 为指针下正在发声的应用在光标附近打开快速选择器 |
| 右键单击任务栏中正在运行的应用 | 在原生菜单旁打开实验性 Adufa 辅助面板 |
| `Tab` 或 `↓` | 移至下一项 |
| `↑` | 移至上一项 |
| `Enter` 或 `Space` | 激活当前聚焦项 |
| `Esc` | 关闭当前选择器或从设置返回 |
| 单击外部 | 关闭 Adufa 临时窗口 |

若其他应用已占用该全局快捷键，快捷键可能不可用；Adufa 会继续运行，托盘操作仍可使用。

## 平台状态

| 平台 | 状态 | 计划使用的后端 |
| --- | --- | --- |
| Windows 10 22H2 / Windows 11 | **测试版可用** | Windows Core Audio / WASAPI 和原生 Win32 UI |
| macOS 14.2+ | 计划支持 | Core Audio 和原生菜单栏界面 |
| Linux、Wayland 和 X11 | 计划支持 | PipeWire 和原生桌面集成 |

跨平台是产品方向，并不表示当前已实现功能对等。每个后端都会明确报告能力，Adufa 不会
假装不受支持的路由操作已成功。图标和核心交互语言保持一致；行为和视觉材质遵循各平台原生方式。

## 隐私

Adufa 的设计原则是仅在本地运行。

- 无遥测或分析。
- 无用户账户。
- 不录制或上传音频。
- 音频路由不需要网络服务。
- 路由和偏好设置保存在你的电脑上。

在 Windows 上，配置保存在当前用户的本地应用数据目录中。删除便携可执行文件不会自动删除该偏好设置文件。

## 路线图

在相关平台实现得到验证前，不承诺具体日期。

### 开发中

- 提升 Windows 测试版在受支持 Windows 10 和 11 版本上的稳定性。
- 签名安装程序和便携 release 校验和。
- 无障碍标签、高对比度验证和改进的键盘操作流程。
- 由母语使用者审核翻译。
- 可靠、可选且保护隐私的更新检查。

### 下一步

- 应用和输出设备搜索。
- 收藏输出设备。
- 可配置的全局快捷键。
- 扩展迷你混音器。
- 配置文件和按应用自动规则。
- 更好的诊断和可恢复的路由重新关联。

### 计划支持的平台

- 使用 Core Audio 的 macOS 14.2+，面向 Apple Silicon 和 Intel 发布。
- 在 Wayland 和 X11 上使用 PipeWire 的 Linux；初始 Linux 版本不包含仅支持 PulseAudio 的后端。

### 探索中

- 在操作系统提供安全 API 时，实现更深入的原生菜单集成。
- 不上传音频或活动数据的可选偏好设置同步。
- 多组设备，以及适用于工作、游戏、通话和直播的可复用配置文件。

## 故障排除

### 找不到某个应用

开始播放并重新打开 Adufa。部分应用只有在发声时才创建音频会话。受保护或系统所有的会话
可能提供较少身份信息，并显示在某个分组应用下。

### 已保存的输出不可用

Adufa 会保留目标路由，而不会悄悄替换为名称相似的设备。重新连接完全相同的设备、选择另一输出，
或选择`系统默认`（`System default`）。

### 任务栏辅助面板没有打开

此集成仍属实验性功能，需要在任务栏图标和发声应用之间取得唯一匹配。请尝试全局快捷键或托盘弹窗。
对于只能在 Windows 11 上可靠工作的集成，Windows 10 会使用备用操作流程。

### `Ctrl + Alt + A` 没有反应

其他程序可能已注册该全局快捷键。请从托盘打开 Adufa；可配置快捷键已列入路线图。

### 出现终端窗口

你可能正在运行 debug 构建，或通过 `cargo run` 启动。请构建并启动
[从源代码构建](#从源代码构建)中所示的 release 可执行文件。

## 开发

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

默认会忽略一项针对真实 Windows 音频会话的往返测试，因为它会暂时修改真实会话。仅应在拥有
活动、可随时弃用音频会话的开发计算机上运行此测试。

仓库结构：

- `crates/router-engine`：与平台无关的身份、命令和状态；
- `platforms/windows`：Windows 音频、持久化、任务栏集成和原生 UI；
- `docs/adr`：已接受的架构决策；
- `docs/design`：交互与标识研究；
- `docs/assets`：生成的文档和品牌资源。

CI 工作流会验证格式、Clippy 和测试，然后生成 Windows x64 便携可执行文件。推送
`v0.1.0-beta.1` 等标签会创建 GitHub Release，并附加 `Adufa-Windows-x64.exe`。

## 参与贡献

错误报告应包含 Windows 版本、受影响的应用、预期输出、实际输出，以及所使用的是托盘、快捷键
还是任务栏操作流程。未经检查，请勿附加包含私有路径的录音、配置文件或日志。

贡献应遵守核心规则：稳定的应用身份、事件驱动观察、如实报告能力、平台原生界面，以及无遥测。

## 许可证和名称

源代码采用 **GPL-3.0-or-later** 许可证，与 Cargo workspace 中的声明一致。“Adufa”和项目
美术资源用于标识官方构建；开源许可证并不表示对修改后发行版的认可。

该名称目前只进行过初步的网页和仓库重名检查。这不属于法律上的商标核准；正式发行前应在计划
发布的地区完成正式检索。

---

<p align="center"><strong>Adufa</strong> — 用一个小巧的控制界面，管理桌面隐藏起来的音频路径。</p>
