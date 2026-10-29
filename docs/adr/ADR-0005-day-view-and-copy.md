# ADR-0005: 按日浏览列表与复制到今日

- Status: Accepted
- Date: 2026-10-29

## Context

用户要求列表按「创建本地日期」逐日浏览（含已完成与未完成），并提供将历史事项复制为今日新事项。此前 OBJECTIVE 要求未完成跨日仍显示在同一列表。

## Decision

- 列表默认显示**当天** `created_local_date` 的任务；标题区提供 `<` / `>` 切换前后日期，不超过今天。
- 未完成事项**不再**自动出现在次日列表；仍保留原 `created_local_date`，可在历史日查看。
- **复制到今日**：新建一条 open 任务，正文与优先级与源任务相同，`created_local_date` 为操作当日；幂等 `command_id`。
- 拖动排序仅作用于当前浏览日内的 open 任务。工作日提醒仍针对**全部**未完成 open 任务（与 C9 一致）。

## Consequences

- C6/C8 验收语义调整：跨日「仍在列表」改为「仍可在创建日找到」。
- `list_tasks` 合同增加必填 `local_date`（`YYYY-MM-DD`）。
