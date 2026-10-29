# ADR-0001: 桌面技术栈与分层

- Status: Accepted for Windows V0.1.0; macOS validation pending
- Date: 2026-10-28

## Context

用户要求纯客户端、Windows/macOS、安装包较小、依赖少、主流成熟，同时必须贴桌面且显示桌面后仍可见。现无代码、无构建配置；未进行平台能力或包体实测。

## Options

| 方案 | 适合点 | 代价与待验证 |
| --- | --- | --- |
| Tauri 2 + Rust + TypeScript + Vite，前端暂建议原生DOM/CSS | 复用系统WebView；Rust侧统一SQLite/OS；小界面可不引入完整UI框架 | Rust工具链；Windows WebView2与macOS WebKit差异；桌面层可能需要原生扩展；插件能力/维护状态尚未核实 |
| Electron + TypeScript | 桌面生态成熟，窗口、托盘与通知常见实践多 | 内置Chromium/Node通常增加分发体积和基础内存；真正桌面层仍非跨平台普通配置 |
| Avalonia + .NET | 跨平台桌面UI、类型化核心与SQLite | 运行时/自包含包体，Markdown渲染与桌面层仍需验证；不是天然零依赖 |

前端候选原生DOM/CSS与Svelte均可；前者依赖更少，后者组件状态维护便利。界面简单时优先前者，待用户认可，不预设必须上React等框架。
SQLite候选Rust库为rusqlite；是否静态捆绑SQLite及具体版本待依赖审查。Markdown候选需维护良好且可禁HTML/远程资源，不为省依赖手写不可靠解析器。

## Decision

用户在确认4个Windows milestone与M5 macOS交付后要求“开始按阶段实现预验收”，据此采用Tauri/Rust/TypeScript并推进M1–M4。产品工程位于仓库根src/与src-tauri/，隔离probe保留作为证据。选择原生DOM/CSS，Markdown用维护中的解析器并净化；数据库使用rusqlite bundled，应用服务位于Rust侧。以下早期Proposed记录是过程证据，以本段当前决定为准。

2026-10-28后续：用户在原生探针验收后明确要求“你继续”，据此进入隔离的Tauri/WebView探针阶段，允许获取构建依赖与最小工程；不表示正式产品选型已冻结。Rust使用官方安装器minimal profile，暂不修改全局PATH；前端探针使用原生HTML/CSS/JS，无UI框架或业务数据库。

实验结果：Windows原生/Tauri桌面基础交互已分别获用户实机确认；Tauri详情中文编辑、普通窗口外观与透明度已自动化验证。建议进入Tauri Windows首版实现，保留macOS验证门槛；本ADR仍为Proposed，详情见../DESKTOP_FEASIBILITY.md。实际锁定Rust Tauri 2.12.5、API/CLI 2.12.4、TypeScript 6.0.3、Vite 8.3.5。

2026-10-28：用户明确要求直接验证，授权最小可行性探针；这不等于接受Tauri或全面实施。先用已有Windows/.NET工具验证桌面宿主与交互的基础行为，不安装工具链。原生探针结果不能替代Tauri/WebView验证。

建议优先评估Tauri 2，未选定。用户确认后先搭最小验证工程，验证桌面层、快捷键/焦点、托盘、通知和安装依赖，再冻结选型，不直接全面实现。
版本锁定时查官方文档并记录精确版本，不写“最新版”作为复现依据。

## Consequences

先设两平台能力门槛：显示桌面后仍可见、普通应用可遮挡、可拖动、点击编辑可正常获得焦点、透明度/背景可用、多屏可恢复。Windows桌面宿主技术及macOS窗口层级行为需要分别验证；未经验证不可承诺框架开箱即用。
若桌面层不可靠或需要脆弱的系统内部接口，先呈现证据和替代方案供用户决定，不默默降级需求。
包体和内存预算必须实测，框架名不能证明比便签轻。工具链依赖与最终用户依赖分别列清。
