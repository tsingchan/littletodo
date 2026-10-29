CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at_utc TEXT NOT NULL, checksum TEXT NOT NULL);
CREATE TABLE tasks(
 id TEXT PRIMARY KEY, content_markdown TEXT NOT NULL CHECK(length(trim(content_markdown))>0),
 priority TEXT NOT NULL CHECK(priority IN ('P1','P2','P3')),
 status TEXT NOT NULL CHECK(status IN ('open','completed')),
 sort_order INTEGER NOT NULL, created_at_utc TEXT NOT NULL, created_local_date TEXT NOT NULL,
 created_timezone TEXT NOT NULL, created_offset_minutes INTEGER NOT NULL,
 updated_at_utc TEXT NOT NULL, completed_at_utc TEXT, version INTEGER NOT NULL CHECK(version>0),
 CHECK((status='open' AND completed_at_utc IS NULL) OR (status='completed' AND completed_at_utc IS NOT NULL))
);
CREATE TABLE task_events(
 event_id INTEGER PRIMARY KEY, task_id TEXT NOT NULL REFERENCES tasks(id),
 command_id TEXT NOT NULL UNIQUE, event_type TEXT NOT NULL CHECK(event_type IN ('created','updated','completed','reopened','reordered')),
 occurred_at_utc TEXT NOT NULL, recorded_at_utc TEXT NOT NULL, local_date TEXT NOT NULL,
 timezone TEXT NOT NULL, utc_offset_minutes INTEGER NOT NULL, task_version INTEGER NOT NULL,
 snapshot_json TEXT NOT NULL CHECK(json_valid(snapshot_json))
);
CREATE TABLE commands(command_id TEXT PRIMARY KEY, fingerprint TEXT NOT NULL, response_json TEXT NOT NULL);
CREATE TABLE app_settings(key TEXT PRIMARY KEY, value_json TEXT NOT NULL CHECK(json_valid(value_json)), updated_at_utc TEXT NOT NULL);
CREATE TABLE reminder_slots(slot_key TEXT PRIMARY KEY, scheduled_at_utc TEXT NOT NULL, claimed_at_utc TEXT NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('claimed','submitted','failed','skipped_empty')), task_count INTEGER NOT NULL);
CREATE INDEX tasks_by_status_order ON tasks(status,sort_order,id);
CREATE INDEX tasks_by_date ON tasks(created_local_date);
CREATE INDEX events_by_task_time ON task_events(task_id,occurred_at_utc,event_id);
CREATE INDEX events_by_date ON task_events(local_date,event_type);
CREATE VIEW v_tasks_readable AS SELECT * FROM tasks;
CREATE VIEW v_task_events_readable AS SELECT * FROM task_events;
