# ADR-0006: 复制确认、单一详情窗与数据目录入口

- Status: Accepted
- Date: 2026-10-29

## Context

用户要求：复制到今日时若今日已有相同正文与优先级的事项需确认；详情编辑同时只保留一个窗口，未保存时阻止切换并提示；设置页展示数据目录、提供打开目录，并说明 Agent 只读使用方式。

## Decision

- **复制确认**：在 UI 层于执行 `copy_task_to_today` 前查询今日列表；若已有与源任务相同 `content_markdown` 与 `priority` 的条目（含源任务本身已在今日的情况），经用户确认后再提交；取消则不调用命令。
- **单一详情窗**：详情窗口固定 label `detail`；`State.detail_task_id` 记录当前编辑 id。未保存（`dirty_windows` 含 `detail`）时再次 `open_detail` 仅聚焦并向前端发送 `detail-unsaved-block`；已保存且 id 不同则 `detail-open` 切换任务。
- **数据目录**：`app_info` 增加 `data_dir`；新增 `open_data_dir` 用系统文件管理器打开该目录（Rust `opener`，非任意路径 IPC）。Agent 说明见 `docs/AGENT-READONLY.md`，设置页摘要并链到用户指南。

## Consequences

- 能力配置中 `detail-*` 改为 `detail`。
- C7 语义补充：同一时刻最多一个详情编辑窗。
