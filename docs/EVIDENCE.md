# LittleTodo 证据记录

## 2026-10-28 工程初始化

- 对象：V0.1.0，未提交；初始目标目录为空。
- 依据：用户需求与后续答复、本机project-harness-init skill、平台Harness与版本约定。
- 预演：init --level L3 --dry-run，列出13个基础文件，无跳过。
- 生成：基础13文件，补充README、3份ADR、DATA_MODEL及3份系统合同。
- 验证：harness.py check --target D:\airdroid\src-jm\littletodo 返回 L3 PASS；本地Markdown链接检查PASS；无残留初始化标记。仅骨架检查，不覆盖任何产品C1–C13。
- 状态：技术栈ADR为Proposed；数据库结构和实现路径为提议；跨日、贴桌面、工作日和退出语义已由用户确认。
- 未验证：所有业务代码/测试、SQL、桌面层、通知、性能、包体、安装、签名/公证和macOS。无依赖安装、脚手架、联网合同验证或系统提醒注册。

## 2026-10-28 用户授权的直接可行性验证

- 对象：V0.1.0文档与隔离的Windows原生probe，不是发布构建；无Git提交可绑定。
- 详情：[桌面可行性报告](DESKTOP_FEASIBILITY.md)。实际读取Tauri、插件作者、Microsoft及Apple官方资料，发现desktop-underlay交互限制。
- 使用[Computer Use skill](C:/Users/Administrator/.codex/plugins/cache/openai-bundled/computer-use/26.1023.50603/skills/computer-use/SKILL.md)观察并点击本地探针控制器；没有代发消息或上传数据。
- 系统.NET csc编译成功，probe运行后挂载桌面成功，日志attached=True/host_class=Progman；桌面可见、拖动、点击与详情仍待实机确认。
- 本机有VC BuildTools与WebView2目录，未发现Rust；未安装依赖，未构建Tauri，无macOS实测。
- 原生probe只有父子关系证据，不覆盖产品C3/C4全部验收；其他业务标准仍UNPROVEN。

## 2026-10-28 Windows原生探针人工验收

- 用户答复：“可见，勾选/拖动/打开详情都正常”，确认Win+D后可见及这三项交互；这是一条人工实机证据。
- 日志最后记录probe_closed，时间为2026-10-28 14:25:48（Asia/Shanghai）；探针已退出。
- 限定：仅本机原生WinForms探针，不是Tauri应用或跨平台验收。详情文字编辑、透明度/背景、多屏/DPI、Explorer重启与macOS尚未确认；产品C3/C4仍非完整PROVEN。

## 2026-10-28 Tauri/WebView探针编译与部分交互

- 对象：V0.1.0隔离探针，无提交；debug exe hash与依赖版本见DESKTOP_FEASIBILITY。
- 通过：官方Rust安装器哈希校验，minimal工具链安装；npm ci；Tauri debug --no-bundle构建（tsc、vite及Rust链接）；cargo check --locked --offline。
- 自动化通过：WebView合成面板渲染、长行省略、勾选事件及视觉状态；日志包含main:ready、panel:ready、panel:checkbox_on。
- 修复：同步命令创建详情导致空白阻塞；依据官方已知问题改为async并重新构建。修复后的运行回归尚未完成。
- 阻碍：恢复窗口失败；重新选择重试后GetCursorPos拒绝访问。停止GUI操作，等待用户解锁确认；没有绕过桌面限制。
- 未验证：修复后详情、桌面挂载、外观透明度与背景、多屏/IME/DPI、生产功能与安装、macOS。产品标准仍未完整达成。

## 2026-10-28 解锁后Tauri回归与桌面人工验收

- 详情async修复已自动化回归：打开、修改合成中文正文、回读一致，日志与截图可观察；未将Unicode赋值当成完整IME验证。
- 普通窗口背景黄色→蓝色、alpha 0.8/0.4应用与颜色变化已观察。发现HTML根背景阻止系统透明，修复后重新构建并看到下层内容透出。
- 最终debug exe SHA256：0EDB4FB34DE920717AF6AD37435FCF2E9B972A8BC8E57589B6A7FCC4DEF80C76；无提交、无安装包。
- Tauri桌面挂载API成功；用户对本轮Tauri版本确认“可见，勾选/拖动/详情均正常”。来源是人工实机验收；未声称挂载后交互均有日志佐证。
- Windows基础方案可行；设置持久化、挂桌面后的外观切换、IME、多屏/DPI、Explorer恢复、业务功能、性能、安装与macOS仍待验证。

## M1：快速输入与SQLite自动预验收

- 对象：V0.1.0 Windows debug应用，独立合成数据库 `.tools/qa-m1`，未提交；该记录对应阶段版本，后续改动需重新构建与回归。
- 8个离线Rust测试通过，覆盖空输入、重复提交、事务失败回滚和草稿保留、陈旧版本拒绝、完成/重新打开事件、排序原子性、重启持久化及只读分析、拒绝未来数据库版本。
- UI冒烟：点击入口打开快速记录，合成中文/Markdown/URL以P2保存，输入窗关闭；SQLite查询有1条open/P2任务、1条created事件；进程重启后列表显示相同内容；Ctrl+Alt+N唤起输入窗口。
- 用户数据未用于测试。P1/P3选择、快捷键冲突设置、安装版与跨平台仍需在后续阶段复验。M1预验收通过不等于产品已发布。

## M2–M3：局部自动预验收

- Windows debug构建，独立合成数据库 `.tools/qa-m1`；使用debug专用LITTLETODO_QA_UNPIN=1保持列表为普通窗口以便自动化，禁止将此结果描述为产品贴桌面已复验。LITTLETODO_QA_NO_NOTIFICATIONS=1避免调试时发送定时提醒。
- M2：详情显示合成中文内容，Markdown预览显示格式/链接；修改内容保存后列表刷新；完成后待办数归零并进入已完成折叠区。三项Markdown安全测试通过，覆盖脚本/图片、危险链接协议、摘要；SQLite测试覆盖排序原子性。实际GUI拖动排序、位置/外观重启恢复、实际产品挂桌面、多屏仍待复验。
- M3：列表收起后进程仍在；从独立合成窗口按Ctrl+Alt+N可打开快速输入；有未保存合成草稿时点击退出被拦截并提示，关闭输入窗后草稿进入SQLite，再次退出进程结束。
- 11项Rust离线测试通过，覆盖工作日09–18点/周末、同一时段去重与空待办不提醒。系统托盘右键菜单、实际系统通知显示、休眠恢复、安装版及macOS尚未验证。

## M4：首个安装候选的预验收（已被内存优化候选替代）

- Windows x64 V0.2.0，NSIS currentUser安装包1894027字节，SHA256 `5C1403FC04068BC36073521DB2820B1514B252F767D94D22AB5E0B99AF75A147`，未签名。离线检查3项Markdown与11项Rust测试通过。
- 该候选安装到 `.tools/qa-install`，安装进程返回0，生成程序和卸载器，注册表显示V0.2.0，桌面和开始菜单快捷方式指向隔离安装目录。首次启动使用 `.tools/qa-installed-data`，不使用真实事项库。
- 用户确认安装版显示桌面后可见，添加、勾选及笔形详情操作正常；隔离库中可见1条已完成任务。用户随后确认托盘展示与菜单能力正常；系统通知因暂时不方便检查，仍待实机确认。
- 内存一次快照：应用进程和其7个WebView2子进程，Private总计226.2 MiB、工作集总计518.8 MiB；两者口径不同，进程工作集相加可能包含共享页，不能视作独占物理内存。没有同机系统便签对照。
- 为减少后台WebView，后续改为按需创建/关闭辅助窗口和按页面拆分前端；当前包已重建，需对新候选重新做回归，不把上一候选的人工操作直接当成新版本通过。
- 可执行文件与构建目录原始exe只有3字节bundle标识不同（安装版NSS，构建目录UNK）；记录各自哈希而不是误称文件字节完全相同。

## M4：按需窗口候选构建（2026-10-28）

- 快速输入、详情、设置改为按需创建及安全关闭后销毁；列表按页面拆包，避免加载详情的 Markdown 解析器。设置原生关闭按钮已接入关闭事件；该修复尚未在最终安装版实机交互验证。
- `npm run check` 通过：前端构建、版本镜像一致、3 项 Markdown 测试与 11 项 Rust 测试。独立 release target 离线生成 Windows x64 NSIS 安装包，未覆盖正在运行的测试程序。
- 最终候选 `LittleTodo_0.2.0_x64-setup.exe`，1898496 字节，SHA256 `7C0D09BB278726965C53B04EDD250E73F804C2D2C850040A634F9C2A7A01FB56`；Authenticode 状态 NotSigned。该候选已隔离安装并启动，功能验收仍在进行。
- 前一候选的桌面交互与托盘菜单经用户确认；系统通知展示、最终候选交互、卸载保留、Explorer 重启、多屏/DPI、GUI 排序和性能对照仍待验证。macOS 需独立环境。

- 用户确认退出后，在原隔离目录静默安装最终候选，安装器返回 0，注册表版本 0.2.0，启动进程路径为 `.tools/qa-install/littletodo.exe`，使用原 `.tools/qa-installed-data`。安装后的 exe SHA256 `1FE845D72FA17D1D2C0CCAEC23C0B2247BA6C1C1768B70543A9EF1BB35E34F55`；最终候选桌面、通知及设置关闭行为仍待验证。

- 用户反馈最终候选的测试通知未见弹窗。Windows PushNotification-Platform 操作日志在 16:28:54 记录该应用的本地 toast 已递送到会话（事件 2416、2418、3052、3153）；全局 ToastEnabled 和 NOC_GLOBAL_SETTING_TOASTS_ENABLED 均为 1。日志不证明横幅已显示，原因尚待排查；通知未通过验收。通知插件后台丢弃系统调用错误，插件的请求返回成功也不证明显示。

- 用户进一步确认通知中心也没有测试通知。开始菜单快捷方式的 `System.AppUserModel.ID` 为 `com.littletodo.desktop`、目标为隔离安装 exe；与发送标识一致，但 `Get-StartApps` 未列出此标识。`SHQueryUserNotificationState` 返回 5（接受通知），不能排除其他系统通知设置或应用注册问题；根因未确定。

- 用户确认 Windows 通知设置里找不到 LittleTodo。Shell-Core 事件 28115/28117 显示快捷方式已加入并更新应用解析缓存；WinRT ToastNotifier 对该标识返回 Enabled。使用独立 PowerShell WinRT 诊断发送合成 toast，Show 成功，系统该应用通知历史中可找到该诊断条目；这些证据均不证明用户看到了横幅或通知中心条目。诊断脚本首次因 PowerShell 5 编码导致 XML 错误未发送，改为 ASCII 文本后成功提交。

## V0.2.1 Windows 通知标识修复（2026-10-28）

- 对象：V0.2.1 源码与 release `littletodo.exe`；未生成新 NSIS 安装包（Tauri 拉取 NSIS 3.13 超时）。隔离目录 `.tools/qa-install` 已替换为 release 构建产物并启动，数据目录仍为 `.tools/qa-installed-data`。
- 根因：从安装目录直接启动 exe 时，进程默认 AppUserModelID 与 toast 发送标识 `com.littletodo.desktop` 不一致；`tauri-plugin-notification` 的 `show()` 异步调用且对调用方恒返回 `Ok`，掩盖 WinRT 失败，导致 `finish_reminder` 误记为已递送。
- 修复：启动时调用 `SetCurrentProcessExplicitAppUserModelID` 并写入 `HKCU\Software\Classes\AppUserModelId\com.littletodo.desktop` 的 DisplayName；提醒与设置内测试改为同步 `tauri-winrt-notification` 发送并向上返回错误。
- 验证：`npm run check` 通过（3 项 Markdown、11 项 Rust 离线测试；新增 1 项 ignored 实机 toast 测试未纳入 check）。ignored 测试 `live_toast_uses_app_id` 在本机执行通过；启动 V0.2.1 后注册表键已出现。
- 用户确认（2026-10-28）：V0.2.1 在「设置 → 测试系统提醒」后**收到横幅通知**，**通知中心有 LittleTodo 条目**。本条为 C9 中「系统能展示提醒」的人工实机证据；整点自动提醒、勿扰/权限拒绝、休眠恢复与安装版路径仍待单独验证。NSIS 安装包、卸载保留数据与其余 M4 项未复验。

## V0.2.1 NSIS 安装包与卸载数据保留（2026-10-28）

- 对象：`src-tauri/target/release/bundle/nsis/LittleTodo_0.2.1_x64-setup.exe`，1900364 字节，SHA256 `E279D83E00AD6E1FA1864AFDD2675E661336E40EBCA6A688DBC9A54A2ECCE45D`，Authenticode 未签名。构建前本机手动下载 NSIS 3.13 至 `%LOCALAPPDATA%\tauri\NSIS`（Tauri 内置下载曾超时）。
- 静默安装到 `.tools/qa-install` 返回 0；注册表 `DisplayVersion` 0.2.1，`InstallLocation` 指向隔离目录。安装目录内 `littletodo.exe` SHA256 `7AF7D0D7D2DA6B55094480B699019876CE33CC51775A88F2E63D304707753D6B`（与构建目录 exe 哈希不同，符合 NSIS 补丁 bundle 标识的预期差异，见 V0.2.0 记录）。
- 卸载保留数据：在无 `LITTLETODO_DATA_DIR` 下启动安装版生成 `%LOCALAPPDATA%\com.littletodo.desktop\littletodo.sqlite3`（65536 字节）后，执行 `uninstall.exe /S`（未勾选删除数据）；卸载后安装目录已移除，上述 SQLite 文件仍存在。未验证交互式卸载 UI 与「清除数据」勾选项。
- 安装版已重新静默安装并启动，合成库仍为 `.tools/qa-installed-data`。GUI 拖动排序、多屏/DPI、整点自动提醒实机、性能对照与 macOS 仍待验证。

## V0.2.2 浮动列表与双快捷键（2026-10-29）

- 依据：用户 2026-10-29 需求与 `docs/adr/ADR-0004-list-window-and-shortcuts.md`；`OBJECTIVE` 已重基线 C1/C3 与已确认行为。
- 实现：列表不再默认贴桌面底层；启动 `prepare_panel` 解除旧挂载并按持久化坐标定位。默认 **Ctrl+Alt+L** 显示待办（置顶+聚焦）、**Ctrl+Alt+N** 快速记录；设置页可改且不得相同。收起列表时取消置顶。
- 验证：`npm run check` 通过（2026-10-29，V0.2.2）。安装包与实机快捷键/置顶行为待用户复验。

## V0.3.0 发布构建（2026-10-29）

- 对象：`LittleTodo_0.3.0_x64-setup.exe`，1912212 字节，SHA256 `0F2841EA09C8F146FB3BA1DDB806A02CF1A590774ADCCC527A91EAF4DB06761D`；构建目录 `littletodo.exe` SHA256 `406070A520378382916095E6B9D8BB6B9CFFF573368587D756064C734F5F744C`。Authenticode 未签名。
- 相对 V0.2.2 功能：按日浏览（‹›）、复制到今日、托盘双击置顶、双快捷键、浮动列表、V0.2.1 通知修复等（见 ADR-0004/0005）。
- 验证：`npm run check` 通过（3 项 Markdown、12 项 Rust 离线测试 + 1 ignored）。安装包路径：`src-tauri/target/release/bundle/nsis/`。实机全路径验收仍待用户确认。

## V0.4.0 发布构建（2026-10-29）

- 对象：`LittleTodo_0.4.0_x64-setup.exe`，1914496 字节，SHA256 `CE9760EEEA23FDF99AE1A39B8E829B5F01C7E8CF0713E21F2AA4E0F68FF76A90`；构建目录 `littletodo.exe`（4941312 字节）SHA256 `C8BD881902082080E274B2C1E9C9BAB834B3FED98F6660A2ADE0F81069FBC68D`。Authenticode 未签名。
- 相对 V0.3.0 功能：复制到今日重复确认、单一详情编辑窗与未保存拦截、设置页数据目录与「打开目录」、`docs/AGENT-READONLY.md`（见 ADR-0006）。
- 验证：`npm run check` 通过（2026-10-29，V0.4.0）。未做静默安装/卸载复验。

## V0.4.0 安装后快捷键与列表尺寸（2026-10-29）

- 现象：重装后全局快捷键无效；托盘「显示待办」列表窗体极小或不可见。
- 根因：快捷键 handler 用字符串比较 `Ctrl+Alt+N` 与系统触发的 `control+alt+…` 格式不一致；旧版贴桌面 `detach` 后保留子窗口尺寸。
- 修复已并入 V0.4.1（见下）。

## V0.4.1 发布构建（2026-10-29）

- 对象：`LittleTodo_0.4.1_x64-setup.exe`，1914448 字节，SHA256 `E10B63C40BADFAF48CF6E936133316AE81362C12A353A98A44497B0E70EE1A29`；构建目录 `littletodo.exe`（4941824 字节）SHA256 `AD1354ECEABA7D2F2A82CB223B39B993AA64EA02FA6B4D5C438E1846EFA341C5`。Authenticode 未签名。
- 相对 V0.4.0：修复全局快捷键匹配与列表窗体尺寸恢复；含 V0.4.0 全部功能。
- 验证：`npm run check` 通过（3 Markdown、13 Rust + 1 ignored）。安装包：`src-tauri/target/release/bundle/nsis/`。实机快捷键与显示待办待用户确认。

## V0.4.2 发布构建（2026-10-29）

- 对象：`LittleTodo_0.4.2_x64-setup.exe`，1921242 字节，SHA256 `CDCA80FAE47098B8CCA19B9BF0087571E8244B44F1DB5DDB525F5AD7E42E79F9`；构建目录 `littletodo.exe`（4971520 字节）SHA256 `09F542F995CB2EFD9CFB8DE2293484278EA5C5ADECC85D2F077F2F36C7566265`。Authenticode 未签名。
- 相对 V0.4.1：设置内 **Agent 使用指南** 独立窗（嵌入 `docs/AGENT-READONLY.md`）；整点提醒 `Scenario::Reminder` 与 **查看待办** / **知道了** 已随包发布。
- 验证：`npm run check` 通过（3 Markdown、14 Rust + 1 ignored）。安装包路径：`src-tauri/target/release/bundle/nsis/LittleTodo_0.4.2_x64-setup.exe`。实机 Agent 指南窗与整点提醒待用户确认。

## V0.4.5 发布构建（2026-10-29）

- 对象：`LittleTodo_0.4.5_x64-setup.exe`，1924345 字节，SHA256 `A14F17BD60E675EE4A90C68DEBA7F7DF14DCDE35107EC3D4134CC300D9C9F702`；构建目录 `littletodo.exe`（4979712 字节）SHA256 `B90088BE492B7C9B9D71AC691DE5CF36EF14A358FB8F67F2F4DC858CF5332548`。Authenticode 未签名。
- 相对 V0.4.2：修正列表「下次提醒」为工作日 9–18 **整点**；待办列表支持右/底/角拖动**放大**（380×460 为默认且最小，尺寸持久化）。
- 验证：`npm run check` 通过（3 Markdown、14 Rust + 1 ignored）。安装包：`src-tauri/target/release/bundle/nsis/LittleTodo_0.4.5_x64-setup.exe`。整点文案与列表缩放实机待用户确认。

## 整点提醒停留（2026-10-29，已含于 V0.4.2）

- 需求：人暂时离开时，短 Toast 易错过，希望回到电脑前仍能看到提醒。
- 实现：整点待办通知使用 `Scenario::Reminder` + **查看待办** / **知道了** 按钮；点击「查看待办」置顶显示列表。测试通知仍为 `Duration::Long`（约 25 秒）。最终展示受 Windows 通知策略与专注助手影响。
