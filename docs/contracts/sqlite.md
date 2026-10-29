# SQLite 合同

状态：设计待落地；来源：用户指定SQLite，官方索引 https://www.sqlite.org/lang_transaction.html 、https://www.sqlite.org/wal.html 、https://www.sqlite.org/backup.html 、https://www.sqlite.org/uri.html 。本次未联网验证。
字段/状态/约束/查询口径见 ../DATA_MODEL.md；客户端库及SQLite精确版本未知，在ADR-0001确认与迁移落地时锁定。
任务和事件同事务；提交失败不关窗；迁移失败不破坏原库；Agent只读，不能提供任意SQL写入IPC。
需区分busy/locked、磁盘满、损坏、迁移不兼容与版本冲突，实际库错误码映射待确认。
WAL模式下主库单文件复制可能遗漏已提交内容，一致性备份须使用合适的SQLite备份机制；分析读事务不得长期阻塞维护。
验证用合成临时库执行提交/失败/迁移/恢复/并发/查询案例；尚无已通过SQL或数据库。
