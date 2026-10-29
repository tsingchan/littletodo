# 架构决策记录（ADR）

记录有分歧、会影响后续实现的决策。没有分歧的事不写 ADR。

- 文件名：`ADR-0001-<kebab-case-标题>.md`，编号递增，不复用。
- 按 `ADR-0000-template.md` 编写。
- 决策被推翻时，不删除旧 ADR：把旧 ADR 的 Status 改为 `Superseded by ADR-XXXX`，再写新 ADR。
- 改变行为语义、数据模型、外部合同或职责边界之前，先写或更新 ADR。

## 索引

| 编号 | 标题 | 状态 |
| --- | --- | --- |
| [ADR-0001](ADR-0001-stack.md) | 桌面技术栈与分层 | Proposed |
| [ADR-0002](ADR-0002-data-history.md) | SQLite单库与可追溯任务历史 | Proposed |
| [ADR-0003](ADR-0003-desktop-reminders.md) | 桌面窗口、后台生命周期与提醒 | Proposed |
