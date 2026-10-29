# OS / 桌面 / WebView 合同

状态：待验证；来源为用户需求、Tauri官方资料索引 https://v2.tauri.app/ 、https://v2.tauri.app/plugin/global-shortcut/ 、https://v2.tauri.app/plugin/notification/ 、https://v2.tauri.app/learn/system-tray/ 。本次未联网核实，未锁版本。

更新2026-10-28：已读取Tauri prerequisites、插件作者FAQ、Microsoft SetParent和Apple窗口层级文档，结果见[桌面可行性报告](../DESKTOP_FEASIBILITY.md)。上列快捷键/通知/托盘资料仍未逐个核实，框架版本未锁。Windows原生父级挂载通过，不等于交互或Tauri验证通过。

应用内拟定adapter：registerShortcut、showQuickEntry、attachDesktopList、setAppearance、showDetail、notify、openHttpUrl、quit。实际参数/返回值/错误码在框架确认后补齐；目前仅约定应用错误类别：shortcut_conflict、notification_denied、desktop_unsupported、platform_failure。

限制：
- 普通非置顶不保证显示桌面后仍可见；真正桌面层实现需单独评估，不能凭Tauri窗口API承诺。
- 列表贴桌面不意味着详情也贴桌面；前台输入焦点、键盘和拖动需分别验证。
- 通知权限、勿扰、系统设置决定能否展示，notify返回不等于用户已看到。
- 进程退出/设备关机无法依赖应用内计时器提醒，当前不创建系统后台任务。
- Windows系统WebView2、macOS WebKit及各CPU架构是兼容性矩阵的一部分；没有“零运行时依赖”保证。
- 不抢占系统保留快捷键；冲突可更改，退出释放。
- 桌面层内部API如存在系统更新不稳定性，需记录证据并让用户决定，禁止默默忽略。
验证：两系统显示桌面/全屏/Spaces/多屏/DPI/休眠、快捷键焦点、通知权限拒绝、托盘退出与安装后身份，记录EVIDENCE。具体OS API文档和版本待能力验证时补。
