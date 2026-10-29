# 桌面可行性验证

日期：2026-10-28；对象：工程文档V0.1.0与独立原生探针，非发布应用。用户在本轮授权直接验证，Tauri选型仍Proposed。

## 当前结论

Windows原生与真实Tauri/WebView探针均已运行；用户分别确认显示桌面后仍可见，勾选、拖动、打开详情正常。自动化另验证了Tauri详情中文正文修改/回读、背景色切换及修正后的背景透明。Windows基础路线可行，但生产级多屏/DPI/恢复、安装与macOS尚未验证。

| 项目 | 证据与状态 |
| --- | --- |
| 本机环境 | OS注册表DisplayVersion=26H2，Build=26300.9711；ProductName返回Windows 10 Pro是注册表字段原值，不据此判定实际Win10/11品牌。Node v24.19.0，npm 11.17.0。 |
| Tauri构建前提 | vswhere确认VS2022 BuildTools存在VC x86/x64组件；WebView2目录155.0.4283.62存在。PATH和默认用户cargo位置未找到Rust，完整构建仍未执行；安装目录存在也不等于实际链接/渲染已通过。 |
| 原生探针 | 系统自带.NET Framework csc编译成功，执行DesktopProbe.exe并经Computer Use点击Attach to desktop；日志记录attached=True，host_class=Progman，父窗口句柄一致。 |
| 可见与交互 | 用户实机验收通过：对Win+D后可见、勾选、拖动、打开详情的询问回复“可见，勾选/拖动/打开详情都正常”。来源为用户确认，非自动化视觉断言；详情文字编辑未单独确认。 |
| 透明度/背景 | 原生探针未验证；Tauri普通窗口下已自动确认黄色→蓝色及alpha 0.8→0.4外观更新；发现根背景不透明并修复后，捕获到面板下层窗口内容透出。挂桌面后的透明度/颜色切换未单独视觉复测，设置持久化尚未实现。 |
| Tauri/WebView | 构建运行通过；自动化验证详情正文修改/回读；桌面挂载API成功，用户确认Win+D后可见、勾选/拖动/打开详情正常，详见最新记录。 |
| macOS | 仅查证官方API与插件资料，没有Mac实机结果。 |

## 已识别的方案限制

1. [desktop-underlay作者FAQ](https://github.com/Charlie-XIAO/tauri-plugin-desktop-underlay/blob/main/FAQ.md)明确该插件底层窗口不接收鼠标/键盘。它的Windows/macOS支持声明不能证明可交互待办适用，不能直接采用。
2. [wallpaper插件作者说明](https://github.com/meslzy/tauri-plugin-wallpaper)区分壁纸模式与pin模式；pin保持置顶，不满足用户“不全局置顶”。本次只读资料，没有安装或执行该插件。
3. [SetParent官方说明](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent)要求正确处理窗口style；跨进程DPI感知模式差异可能产生重置/异常。当前探针仅将自己的面板挂到已有Progman/WorkerW，不触发未公开shell消息；宿主层次本身仍不是稳定公开的桌面小组件合同。资源管理器重启、桌面图标遮挡、多屏/DPI仍需验证。
4. [Apple SwiftUI desktop层级](https://developer.apple.com/documentation/swiftui/windowlevel/desktop)描述壁纸上方、其他窗口下方，官方Markdown标明macOS 15起；不是Tauri已提供同样接口的证明，也不证明显示桌面、点击与Spaces行为。较早系统的[AppKit NSWindow.level](https://developer.apple.com/documentation/appkit/nswindow/level-swift.property)可作为适配研究入口，仍需Mac实测。
5. [Tauri官方前提](https://v2.tauri.app/start/prerequisites/)需要Rust与平台开发依赖；本机缺少Rust，因此本轮没有把选型文档当成完整运行证据。

以上线上资料在本轮实际读取；未下载执行第三方插件或软件。

## 原生探针的范围与使用

源码与编译步骤见[probe说明](../probes/windows-desktop/README.md)。只显示合成任务，无SQL、快捷键、网络、提醒或自启；不修改桌面图标样式，不重启Explorer。
控制器保持普通窗口，面板挂桌面，Exit probe释放探针窗口。日志记录2026-10-28 14:25:48（Asia/Shanghai）probe_closed，探针已退出；可重新启动exe并点击Attach to desktop。

用户检查：Win+D后观察右上角黄色面板；勾选合成任务；拖动标题；打开详情并编辑；回控制器改变透明度/背景，再看桌面；普通应用应可遮住面板。分别报告可见与交互，避免仅看父窗口日志就断言通过。

自动化边界：Computer Use skill的guidance禁止发送Windows键组合，且选择API只返回顶层窗口；因此该次Win+D及桌面子窗口操作需用户确认，不另写键盘注入或UI Automation脚本绕过。

## 下一验证阶段

Windows原生/Tauri基础交互已获用户确认。建议下一阶段采用Tauri候选开展Windows首版实现；完整跨平台选型仍受macOS验证约束。需要继续验证中文IME组合输入、多屏/DPI、Explorer重启恢复、快捷键/托盘/通知、发布包性能与安装。Mac需Mac执行器与对应实机；目前没有可用Mac执行环境的证据。产品C3/C4尚未完整PROVEN。

## 后续Tauri/WebView实验

- 用户要求“接下来做什么呢，你继续”，授权推进隔离探针。官方create-tauri-app生成vanilla-ts工程，路径probes/tauri-desktop；不接业务数据库。
- 官方rustup安装器SHA256与同源sha256文件一致：00F139DA614DCED87D3C3D4121C43EEBDAC3C1BF8A5EE86B9D80DFD71C8D70FD。minimal profile，未修改全局PATH，Rust 1.99.1 / x86_64-pc-windows-msvc。
- Rust直连超时，构建进程继承本机已配置的代理后下载成功；未修改系统代理。前端原生依赖加载失败后按锁文件npm ci重装，构建通过。最初两次npm安装曾短暂重叠并被停止，依赖完整性已通过重装及真实构建重新确认。
- 版本：tauri Rust 2.12.5，tauri API/CLI 2.12.4，TypeScript 6.0.3，Vite 8.3.5；完整传递依赖见Cargo.lock和package-lock.json。
- 已执行通过：npm run tauri build -- --debug --no-bundle（含tsc与vite build）；cargo check --locked --offline。成功生成并运行debug exe，无安装包。
- 自动化看到合成任务面板完整渲染、长行省略，点击勾选后截图为勾选状态，日志panel:checkbox_on。不是Markdown完整渲染或产品功能通过。
- 失败与修复：同步命令创建详情时出现空白并阻塞；[Tauri官方已知问题](https://docs.rs/tauri/2.12.5/tauri/webview/struct.WebviewWindowBuilder.html)说明Windows同步创建WebView可能死锁。已将open_detail改async并重新构建，编译通过；修复后实际打开/编辑尚待验证。
- 修复后GUI自动化窗口恢复失败，重新选择窗口重试后GetCursorPos返回拒绝访问0x80070005。已停止UI输入并请求用户确认Windows解锁；不把受限桌面诊断为应用缺陷，也不声称锁屏已确证。
- 本阶段中间debug exe SHA256：BA5455DDA41E0575ACE83B6BE31E29BA7BC4E6DA045896DDFD45CEF0C713CD5E；下方修复根背景后的构建已替代此产物。

## 解锁后回归及Tauri人工桌面验收

- 用户确认解锁后，自动化成功恢复控制器；通过打开详情、修改为合成中文正文、回读并截图确认。日志有detail_opened、detail:ready、detail:detail_edited。此为Unicode文本编辑，不等于IME候选/组合输入完整验证。
- 控制器蓝色背景及alpha 0.8/0.4设置分别产生appearance_requested和panel:appearance_applied，自动化看到颜色/浅深变化。
- 随后发现HTML根背景仍不透明，仅颜色变浅不能证明系统透明。已对panel的documentElement设置透明背景并重建；置前截图实际看到下层窗口内容透出。修复属于显示层缺陷；不再以应用事件确认代替视觉透明证据。
- 修复后构建SHA256：0EDB4FB34DE920717AF6AD37435FCF2E9B972A8BC8E57589B6A7FCC4DEF80C76。Tauri桌面挂载日志attach=Ok，parent=65912。
- 对“此次是真实Tauri/WebView面板”的Win+D、勾选、拖动、笔图标详情测试，用户回复“可见，勾选/拖动/详情均正常”。记录为人工验收；这一轮日志未补录到对应勾选/拖动事件，不声称自动日志已独立佐证。
- 限定：贴桌面后改变透明度/背景未单独视觉复测；设置持久化、任务存储、Markdown渲染、IME、多屏/DPI、Explorer重启、托盘/快捷键/通知、release性能、安装及macOS未验证。探针仍运行，不是正式应用。
