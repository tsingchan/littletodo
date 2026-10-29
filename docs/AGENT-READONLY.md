# 让 Agent 只读使用 LittleTodo 数据

LittleTodo 不会自动把数据交给任何 Agent。若你**主动**让 Cursor、Codex 等工具帮你整理日报或分析待办，请先把范围说清楚，并只读打开本地数据库。

## 数据在哪

- **目录**：设置页「数据存储目录」所示路径（Windows 通常为 `%LOCALAPPDATA%\com.littletodo.desktop\`）。
- **主库文件**：该目录下的 `littletodo.sqlite3`。
- **WAL 模式**：运行中可能还有 `littletodo.sqlite3-wal`、`.sqlite3-shm`。不要只复制单个 `.sqlite3` 当作完整备份；分析时优先在**应用已退出**后打开，或使用 SQLite 官方备份机制做一致性快照（见 `docs/DATA_MODEL.md`）。

开发/测试可通过环境变量 `LITTLETODO_DATA_DIR` 指向其它目录；Agent 应读取用户机器上设置页显示的实际路径，而不是仓库内路径。

## 推荐打开方式

1. **只读连接**（应用运行中也可，但可能遇到 busy）：
   ```text
   file:/C:/Users/你/AppData/Local/com.littletodo.desktop/littletodo.sqlite3?mode=ro
   ```
   使用 SQLite 客户端或脚本时开启 `query_only`（若支持），**不要**写入、迁移或 `DELETE`。

2. **在 Cursor 等 IDE 中**：让用户在设置里点「打开目录」，把该文件夹加入工作区，或把 `littletodo.sqlite3` 的**只读副本**放到 Agent 可访问的位置；在对话里写明「仅分析此库，禁止修改」。

3. **字段与查询口径**：以 `src-tauri/migrations/001.sql` 为准；可读视图 `v_tasks_readable`、`v_task_events_readable` 及示例 SQL 见 [DATA_MODEL.md](DATA_MODEL.md)。

## 给 Agent 的提示词示例（可复制）

```text
请只读分析我 LittleTodo 数据库（路径：___设置页显示的 littletodo.sqlite3 完整路径___）。
使用 SQLite 只读模式，不要写入或删除任何数据。
按 DATA_MODEL.md 的口径统计：今日创建事项、今日完成数、当前未完成按优先级分组。
正文和链接可能含敏感信息，仅用于我的本地日报，不要上传或发送到外部服务。
```

## 禁止事项

- 不要把真实库、WAL/SHM 或含工作内容的导出提交到 git 或公开仓库。
- Agent 默认**不得**改 schema、写 tasks 表或代替用户完成/删除事项（产品当前也无删除入口）。
- 日志、截图、对话里避免粘贴完整事项正文，除非用户明确授权。
