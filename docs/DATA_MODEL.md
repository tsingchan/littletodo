# SQLite 数据模型与Agent读取（设计提议）

Windows首版已按 `src-tauri/migrations/001.sql` 实现V1 schema；以该迁移为字段真相源，Agent不应根据推测写库。以下查询需先以合成数据库验证，且只读执行。版本迁移校验失败或数据库版本更新时停止写入。

## 存储与时间

一份应用管理数据库，建议Windows用户LocalAppData下LittleTodo目录、macOS用户Application Support下LittleTodo目录，最终由选定框架解析并提供“打开数据位置”，不硬编码用户名。用户数据、WAL/SHM和备份不入仓库。
不每天建库、不每天复制事项；任务ID跨日不变。时间使用UTC ISO8601固定毫秒格式，另存业务发生的本地日期、IANA时区与UTC偏移。历史日期不随当前机器时区重算。
明文SQLite适合本机可查询，但不等于加密；依赖用户目录权限，敏感正文/URL禁止写普通日志，Agent只能在用户授权范围访问。

## 表与约束

| 表 | 拟定字段与语义 |
| --- | --- |
| schema_migrations | version INTEGER PRIMARY KEY、applied_at_utc、checksum；迁移校验和与事务升级 |
| tasks | id TEXT PRIMARY KEY（UUID）、content_markdown TEXT NOT NULL、priority TEXT CHECK IN P1/P2/P3、status TEXT CHECK IN open/completed、sort_order INTEGER、created_at_utc、created_local_date、created_timezone、created_offset_minutes、updated_at_utc、completed_at_utc nullable、version INTEGER |
| task_events | event_id INTEGER PRIMARY KEY、task_id FK、command_id TEXT UNIQUE、event_type、occurred_at_utc、local_date、timezone、utc_offset_minutes、task_version、snapshot_json；保存变更后完整状态快照，方便还原历史 |
| reminder_slots | slot_key TEXT PRIMARY KEY（本地日期/小时/时区/偏移）、scheduled_at_utc、claimed_at_utc、status（claimed/submitted/failed/skipped_empty）、task_count；不存正文 |
| app_settings | key TEXT PRIMARY KEY、value_json、updated_at_utc；快捷键、位置、尺寸、背景/透明度、通知预览等；配置schema也需版本化 |
| commands | command_id PRIMARY KEY、fingerprint、response_json；操作去重。fingerprint为请求摘要，response_json是结果快照，属于敏感用户数据的一部分。 |

tasks约束：正文非空白（应用层处理Unicode空白）；completed必须有completed_at，open时必须为NULL；version正整数；priority仅三值；排序稳定以sort_order、id作次序。
建议索引：tasks(status, sort_order, id)、tasks(created_local_date)、task_events(task_id, occurred_at_utc, event_id)、task_events(local_date, event_type)。
不加独立“负责人”或“URL”列，避免假定正文格式。未来需精确负责人统计时再做显式结构字段迁移，不能仅靠正则冒充准确解析。

事件类型：created、updated、completed、reopened、reordered。正文和优先级修改统一为updated。事件除了occurred_at_utc还有recorded_at_utc：前者在系统时钟回拨时按前一事件递增至少1毫秒，后者保留真实系统时钟。时钟回拨跨日导致effective时间跨边界时，历史“日末状态”需要额外解释，不能无条件声称准确。
任务更新与事件写入在同一事务；command_id幂等，版本条件更新防陈旧覆盖；排序批量事务内完成，不以优先级自动重排。
首版不做删除入口，后续如增加删除，需定义软删除/清除与历史保留语义后迁移。
用户有意退出不写成任务数据。设置不进入任务事件。

## 只读分析合同

已提供v_tasks_readable当前视图及v_task_events_readable事件视图，字段使用稳定可读名称，不藏在不可解释二进制中。2026-10-28在单条合成事项（当天创建并完成）上验证下方四个查询：当天创建1、完成1、日末未完成0、当前无未完成；只读写入被拒绝。该样本不证明时区切换、跨日、时钟回拨或完整日报功能。
Agent用SQLite URI mode=ro并开启query_only；不使用immutable读取正在写的库；允许读取WAL相关文件，短读事务及时结束，busy_timeout有界；分析较大时先创建一致性快照。禁止直接拷贝活动主库当备份。

```sql
-- :day 是用户希望统计的历史本地日期 YYYY-MM-DD。
SELECT priority, count(*) AS open_count
FROM tasks WHERE status = 'open' GROUP BY priority;

SELECT id, content_markdown, priority, created_at_utc
FROM tasks WHERE created_local_date = :day ORDER BY created_at_utc, id;

-- 当日发生过完成的去重任务数；取消完成不抹去完成事件。
SELECT count(DISTINCT task_id) AS tasks_completed_during_day
FROM task_events WHERE event_type = 'completed' AND local_date = :day;

-- :cutoff_utc 为目标业务时区“次日00:00”换算UTC；必须按时区规则转换，不能恒加24小时。
-- 需要窗口函数的SQLite最低版本在依赖锁定时确认。
WITH ranked AS (
  SELECT task_id, snapshot_json,
    row_number() OVER (
      PARTITION BY task_id ORDER BY occurred_at_utc DESC, event_id DESC
    ) AS rn
  FROM task_events WHERE occurred_at_utc < :cutoff_utc
)
SELECT task_id, snapshot_json
FROM ranked WHERE rn = 1
  AND json_extract(snapshot_json, '$.status') = 'open';
```

当日完成数（完成事件）与日末已完成数不同；同日完成再取消，仍算发生一次完成，日报需明确此口径。
历史日末清单按事件快照还原，不能用当前tasks.status假装历史状态；机器时钟回拨下按时间排序可能失真，需定义单调业务时间/事件序边界并测试后再开放“准确历史”声明。
日报只包含用户授权的信息，URL/正文可能敏感，不自动上传、发送或调用在线模型。

## 迁移与恢复

foreign_keys启用；WAL、busy_timeout与写串行策略须测试后启用；不要为性能降低提交耐久性而不说明风险。
当前只实现新库V1原子初始化与迁移校验，未来V2升级前必须实现并验证SQLite一致性备份与恢复；新schema不能默认被旧应用理解，当前遇到高版本会停止写入。回退应用前检查兼容性，不允许静默降级写库。保留备份不得自动覆盖用户新数据。
