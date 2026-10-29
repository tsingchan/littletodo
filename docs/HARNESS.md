# LittleTodo 分层验证

## 当前命令

PowerShell，在仓库根执行（依赖本机安装的skill和Python）：

```powershell
python "$env:USERPROFILE\.agents\skills\project-harness-init\scripts\harness.py" check --target .
```

它只检查骨架文件、模板标记、ADR、版本和密钥忽略规则，不验证应用功能。

## Windows业务工程离线检查

在仓库根运行 `npm run check`：TypeScript/Vite构建、VERSION镜像一致性检查、Markdown安全测试及 `cargo test --locked --offline`。首次需安装Rust、Node与依赖；后续离线依赖缓存应完整。2026-10-28通过3项Markdown测试及11项Rust测试。`VERSION` 是唯一版本真相源，package.json、Cargo.toml和Tauri配置是构建镜像；harness模板脚本仍会将这些必要的镜像警告为“multiple version sources”，以本项目一致性检查结果为准。

## 隔离Tauri探针的已验证命令

在probes/tauri-desktop目录，已有node_modules与Rust依赖缓存后执行：

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
$env:CARGO_NET_OFFLINE = 'true'
npm run tauri build -- --debug --no-bundle
```

类型检查、前端打包及Rust debug链接已实际通过，无安装包。Rust独立检查可在src-tauri目录运行cargo check --locked --offline，也已通过。配置、依赖安装和运行步骤见../probes/tauri-desktop/PROBE.md；这些命令只验证隔离探针，不替代下方计划中的业务L0/L1测试。

## 实现后需建立的验证

当前 `src/` 为UI、`src-tauri/src/db.rs` 为事务存储、`src-tauri/src/reminder.rs` 为提醒规则。表格中的更多案例仍是待补充的验证，不代表已执行。首次下载依赖需要网络，缓存后日常检查应能离线。

| 层 | 解决什么不确定性 | 计划位置 | 命令 / 离线 |
| --- | --- | --- | --- |
| L0 合同 | Markdown协议/HTML过滤、通知拒绝、快捷键冲突、平台适配错误 | tests/contracts | 待 ADR-0001 落地后补；合成fixture，离线 |
| L1 核心 | 空输入、P1/P2/P3、Unicode摘要、排序、跨日、周末、整点、夏令时、时区切换和无任务不提醒 | core、tests/core | 待 ADR-0001 落地后补；可注入时钟，离线 |
| L2 持久化 | 临时SQLite事务回滚、磁盘满、迁移失败恢复、事件一致、去重、陈旧版本、只读并发查询 | storage、tests/storage | 待 ADR-0001 落地后补；临时库，离线 |
| L3 集成 | 提交后关窗/广播、失败保留输入、关闭与退出区别、通知去重 | app、tests/integration | 待 ADR-0001 落地后补；fake OS adapter，离线 |
| L4 平台冒烟 | C1–C12关键路径，显示桌面、桌面层点击/编辑、托盘恢复、睡眠唤醒、多屏/DPI、全屏/Spaces | docs/EVIDENCE.md | 分别Windows/macOS实机；系统权限人工控制，无云服务 |

优先验证桌面层可行性及数据不丢失。少量UI冒烟不能代替数据库断言；骨架检查不能代替L0/L1。

## 打包与性能

分别在 Windows 与 macOS 构建、安装与验证，记录系统/架构、工具和依赖版本、WebView版本、产物格式、大小、SHA256及签名/公证。确切构建命令待选型。
Windows不能证明macOS安装成功。Tauri候选的WebView2缺失/离线安装需单独验证，不把使用系统WebView写成零依赖。
按C13在同机同条件测量多次启动、内存和包体，结果写EVIDENCE。
所有测试、样例、截图仅合成数据，真实库和签名材料不入仓库。
