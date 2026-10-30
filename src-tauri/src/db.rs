use chrono::{DateTime, Local, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, String>;
fn sql(error: rusqlite::Error) -> String { format!("数据库操作失败：{error}") }
const MIGRATION: &str = include_str!("../migrations/001.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String, pub content_markdown: String, pub priority: String, pub status: String,
    pub sort_order: i64, pub created_at_utc: String, pub created_local_date: String,
    pub created_timezone: String, pub created_offset_minutes: i32, pub updated_at_utc: String,
    pub completed_at_utc: Option<String>, pub version: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveTask {
    pub command_id: String, pub id: Option<String>, pub expected_version: Option<i64>,
    pub content_markdown: String, pub priority: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTask { pub command_id: String, pub id: String, pub expected_version: i64, pub completed: bool }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskVersion { pub id: String, pub version: i64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderTasks { pub command_id: String, pub local_date: String, pub tasks: Vec<TaskVersion> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTaskToToday { pub command_id: String, pub source_id: String }

fn validate_local_date(date: &str) -> Result<()> {
    if chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() { return Err("日期格式无效".into()); }
    Ok(())
}

pub struct Db { pub conn: Connection }
struct Stamp { utc: String, date: String, timezone: String, offset: i32 }
fn stamp() -> Stamp {
    let now = Local::now();
    Stamp { utc: now.with_timezone(&Utc).to_rfc3339_opts(SecondsFormat::Millis, true),
        date: now.format("%Y-%m-%d").to_string(),
        timezone: iana_time_zone::get_timezone().unwrap_or_else(|_| now.offset().to_string()),
        offset: now.offset().local_minus_utc()/60 }
}
fn command_id(id: &str) -> Result<()> { Uuid::parse_str(id).map(|_|()).map_err(|_| "无效的操作标识".into()) }
fn fingerprint<T:Serialize>(kind:&str,input:&T)->Result<String>{
    let bytes=serde_json::to_vec(input).map_err(|e|e.to_string())?;
    Ok(format!("{kind}:{:x}",Sha256::digest(bytes)))
}
fn content(body: &str, priority: &str) -> Result<()> {
    if body.trim().is_empty() { return Err("请填写事项内容".into()); }
    if body.len() > 100_000 { return Err("内容过长，请限制在100KB以内".into()); }
    if !["P1","P2","P3"].contains(&priority) { return Err("优先级必须是P1、P2或P3".into()); }
    Ok(())
}
fn read_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task { id:row.get(0)?,content_markdown:row.get(1)?,priority:row.get(2)?,status:row.get(3)?,
        sort_order:row.get(4)?,created_at_utc:row.get(5)?,created_local_date:row.get(6)?,
        created_timezone:row.get(7)?,created_offset_minutes:row.get(8)?,updated_at_utc:row.get(9)?,
        completed_at_utc:row.get(10)?,version:row.get(11)? })
}
fn get(conn: &Connection, id: &str) -> Result<Task> {
    conn.query_row("SELECT * FROM tasks WHERE id=?1", [id], read_task).optional().map_err(sql)?
        .ok_or_else(|| "事项不存在".into())
}
fn cached(tx: &Transaction<'_>, id: &str, fingerprint: &str) -> Result<Option<serde_json::Value>> {
    let entry: Option<(String,String)> = tx.query_row("SELECT fingerprint,response_json FROM commands WHERE command_id=?1",
        [id], |r| Ok((r.get(0)?,r.get(1)?))).optional().map_err(sql)?;
    match entry {
        Some((fp,json)) if fp==fingerprint => serde_json::from_str(&json).map(Some).map_err(|e|e.to_string()),
        Some(_) => Err("操作标识已用于其他内容，请重新打开编辑器".into()),
        None => Ok(None),
    }
}
fn remember<T: Serialize>(tx: &Transaction<'_>, id: &str, fingerprint: &str, response: &T) -> Result<()> {
    tx.execute("INSERT INTO commands VALUES(?1,?2,?3)",params![id,fingerprint,serde_json::to_string(response).map_err(|e|e.to_string())?]).map_err(sql)?;
    Ok(())
}
fn record(tx: &Transaction<'_>, task: &Task, id: &str, event: &str, at: &Stamp) -> Result<()> {
    let previous: Option<String> = tx.query_row("SELECT MAX(occurred_at_utc) FROM task_events",[],|r|r.get(0)).map_err(sql)?;
    let mut effective = DateTime::parse_from_rfc3339(&at.utc).map_err(|e| e.to_string())?.with_timezone(&Utc);
    if let Some(last) = previous {
        let last=DateTime::parse_from_rfc3339(&last).map_err(|e|e.to_string())?.with_timezone(&Utc);
        if effective <= last { effective=last+chrono::Duration::milliseconds(1); }
    }
    tx.execute("INSERT INTO task_events(task_id,command_id,event_type,occurred_at_utc,recorded_at_utc,local_date,timezone,utc_offset_minutes,task_version,snapshot_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![task.id,id,event,effective.to_rfc3339_opts(SecondsFormat::Millis,true),at.utc,at.date,at.timezone,at.offset,task.version,serde_json::to_string(task).map_err(|e|e.to_string())?]).map_err(sql)?;
    Ok(())
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn=Connection::open(path).map_err(sql)?;
        conn.busy_timeout(std::time::Duration::from_secs(3)).map_err(sql)?;
        conn.pragma_update(None,"foreign_keys",true).map_err(sql)?;
        let version: i64=conn.pragma_query_value(None,"user_version",|r|r.get(0)).map_err(sql)?;
        if version>1 { return Err("此数据库来自更新版本，请升级LittleTodo；已停止写入".into()); }
        let hash=format!("{:x}",Sha256::digest(MIGRATION.as_bytes()));
        if version==0 {
            conn.execute_batch("BEGIN IMMEDIATE").map_err(sql)?;
            let result=(|| {
                conn.execute_batch(MIGRATION).map_err(sql)?;
                conn.execute("INSERT INTO schema_migrations VALUES(1,?1,?2)",params![stamp().utc,hash]).map_err(sql)?;
                conn.pragma_update(None,"user_version",1).map_err(sql)?;
                conn.execute_batch("COMMIT").map_err(sql)
            })();
            if result.is_err() { let _=conn.execute_batch("ROLLBACK"); }
            result?;
        } else {
            let saved:String=conn.query_row("SELECT checksum FROM schema_migrations WHERE version=1",[],|r|r.get(0)).map_err(sql)?;
            if saved != hash { return Err("数据库迁移校验不一致，已停止写入".into()); }
        }
        conn.pragma_update(None,"journal_mode","WAL").map_err(sql)?;
        conn.pragma_update(None,"synchronous","FULL").map_err(sql)?;
        Ok(Self{conn})
    }
    pub fn list(&self) -> Result<Vec<Task>> { self.list_for_date(&stamp().date) }
    pub fn list_for_date(&self, date: &str) -> Result<Vec<Task>> {
        validate_local_date(date)?;
        let mut statement=self.conn.prepare("SELECT * FROM tasks WHERE created_local_date=?1 ORDER BY CASE status WHEN 'open' THEN 0 ELSE 1 END,sort_order,id").map_err(sql)?;
        statement.query_map([date],read_task).map_err(sql)?.collect::<std::result::Result<Vec<_>,_>>().map_err(sql)
    }
    pub fn count_open_tasks(&self)->Result<i64>{
        self.count_open_tasks_for_date(&stamp().date)
    }
    pub fn count_open_tasks_for_date(&self,date:&str)->Result<i64>{
        validate_local_date(date)?;
        self.conn.query_row("SELECT COUNT(*) FROM tasks WHERE status='open' AND created_local_date=?1",[date],|r|r.get(0)).map_err(sql)
    }
    pub fn get(&self,id:&str)->Result<Task>{get(&self.conn,id)}
    pub fn save(&mut self, input: SaveTask) -> Result<Task> {
        command_id(&input.command_id)?; content(&input.content_markdown,&input.priority)?;
        let fingerprint=fingerprint("save",&input)?;
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(sql)?;
        if let Some(value)=cached(&tx,&input.command_id,&fingerprint)? { return serde_json::from_value(value).map_err(|e|e.to_string()); }
        let at=stamp();
        let task = if let Some(id)=input.id.as_ref() {
            let old=get(&tx,id)?;
            if input.expected_version != Some(old.version) { return Err("事项已在其他窗口修改，请重新打开后再编辑。当前输入仍保留。".into()); }
            tx.execute("UPDATE tasks SET content_markdown=?1,priority=?2,updated_at_utc=?3,version=version+1 WHERE id=?4 AND version=?5",
                params![input.content_markdown,input.priority,at.utc,id,old.version]).map_err(sql)?;
            get(&tx,id)?
        } else {
            let id=Uuid::new_v4().to_string();
            let order:i64=tx.query_row("SELECT COALESCE(MAX(sort_order),0)+1024 FROM tasks",[],|r|r.get(0)).map_err(sql)?;
            tx.execute("INSERT INTO tasks VALUES(?1,?2,?3,'open',?4,?5,?6,?7,?8,?5,NULL,1)",
                params![id,input.content_markdown,input.priority,order,at.utc,at.date,at.timezone,at.offset]).map_err(sql)?;
            get(&tx,&id)?
        };
        record(&tx,&task,&input.command_id,if input.id.is_some() {"updated"} else {"created"},&at)?;
        if input.id.is_none() { tx.execute("DELETE FROM app_settings WHERE key='quick_draft'",[]).map_err(sql)?; }
        remember(&tx,&input.command_id,&fingerprint,&task)?;
        tx.commit().map_err(sql)?; Ok(task)
    }
    pub fn complete(&mut self,input:CompleteTask)->Result<Task>{
        command_id(&input.command_id)?;
        let fingerprint=fingerprint("complete",&input)?;
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(sql)?;
        if let Some(value)=cached(&tx,&input.command_id,&fingerprint)? {return serde_json::from_value(value).map_err(|e|e.to_string());}
        let old=get(&tx,&input.id)?;
        if old.version!=input.expected_version {return Err("事项已变化，请刷新列表后再操作".into());}
        let at=stamp(); let status=if input.completed {"completed"} else {"open"};
        tx.execute("UPDATE tasks SET status=?1,completed_at_utc=?2,updated_at_utc=?3,version=version+1 WHERE id=?4 AND version=?5",
            params![status,if input.completed {Some(at.utc.as_str())}else{None},at.utc,input.id,old.version]).map_err(sql)?;
        let task=get(&tx,&input.id)?;
        record(&tx,&task,&input.command_id,if input.completed {"completed"}else{"reopened"},&at)?;
        remember(&tx,&input.command_id,&fingerprint,&task)?;
        tx.commit().map_err(sql)?; Ok(task)
    }
    pub fn reorder(&mut self,input:ReorderTasks)->Result<()>{
        command_id(&input.command_id)?; validate_local_date(&input.local_date)?;
        let fingerprint=fingerprint("reorder",&input)?;
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(sql)?;
        if cached(&tx,&input.command_id,&fingerprint)?.is_some(){return Ok(());}
        let count:i64=tx.query_row("SELECT COUNT(*) FROM tasks WHERE status='open' AND created_local_date=?1",[&input.local_date],|r|r.get(0)).map_err(sql)?;
        let ids:std::collections::HashSet<_>=input.tasks.iter().map(|t|&t.id).collect();
        if count!=input.tasks.len() as i64 || ids.len()!=input.tasks.len(){return Err("列表已变化，请刷新后重新排序".into());}
        let at=stamp();
        for (index,item) in input.tasks.iter().enumerate(){
            let old=get(&tx,&item.id)?;
            if old.status!="open" || old.version!=item.version || old.created_local_date!=input.local_date {return Err("排序期间事项已变化，请重试".into());}
            tx.execute("UPDATE tasks SET sort_order=?1,version=version+1,updated_at_utc=?2 WHERE id=?3",params![(index as i64+1)*1024,at.utc,item.id]).map_err(sql)?;
            record(&tx,&get(&tx,&item.id)?,&format!("{}:{}",input.command_id,item.id),"reordered",&at)?;
        }
        remember(&tx,&input.command_id,&fingerprint,&serde_json::Value::Null)?;
        tx.commit().map_err(sql)?; Ok(())
    }
    pub fn copy_to_today(&mut self,input:CopyTaskToToday)->Result<Task>{
        command_id(&input.command_id)?;
        let fingerprint=fingerprint("copy_to_today",&input)?;
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(sql)?;
        if let Some(value)=cached(&tx,&input.command_id,&fingerprint)? {return serde_json::from_value(value).map_err(|e|e.to_string());}
        let source=get(&tx,&input.source_id)?;
        let at=stamp();
        let id=Uuid::new_v4().to_string();
        let order:i64=tx.query_row("SELECT COALESCE(MAX(sort_order),0)+1024 FROM tasks",[],|r|r.get(0)).map_err(sql)?;
        tx.execute("INSERT INTO tasks VALUES(?1,?2,?3,'open',?4,?5,?6,?7,?8,?5,NULL,1)",
            params![id,source.content_markdown,source.priority,order,at.utc,at.date,at.timezone,at.offset]).map_err(sql)?;
        let task=get(&tx,&id)?;
        record(&tx,&task,&input.command_id,"created",&at)?;
        remember(&tx,&input.command_id,&fingerprint,&task)?;
        tx.commit().map_err(sql)?; Ok(task)
    }
    pub fn setting<T:serde::de::DeserializeOwned>(&self,key:&str)->Result<Option<T>>{
        let value:Option<String>=self.conn.query_row("SELECT value_json FROM app_settings WHERE key=?1",[key],|r|r.get(0)).optional().map_err(sql)?;
        value.map(|s|serde_json::from_str(&s).map_err(|e|e.to_string())).transpose()
    }
    pub fn set_setting<T:Serialize>(&self,key:&str,value:&T)->Result<()>{
        self.conn.execute("INSERT INTO app_settings VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json,updated_at_utc=excluded.updated_at_utc",
            params![key,serde_json::to_string(value).map_err(|e|e.to_string())?,stamp().utc]).map_err(sql)?; Ok(())
    }
    pub fn claim_reminder(&mut self,slot:&str,scheduled_utc:&str)->Result<Option<Vec<Task>>>{
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(sql)?;
        let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM reminder_slots WHERE slot_key=?1)",[slot],|r|r.get(0)).map_err(sql)?;
        if exists{return Ok(None);}
        let today=stamp().date;
        let tasks={let mut query=tx.prepare("SELECT * FROM tasks WHERE status='open' AND created_local_date=?1 ORDER BY sort_order,id").map_err(sql)?;
            let rows=query.query_map([&today],read_task).map_err(sql)?;rows.collect::<std::result::Result<Vec<_>,_>>().map_err(sql)?};
        let status=if tasks.is_empty(){"skipped_empty"}else{"claimed"};
        tx.execute("INSERT INTO reminder_slots VALUES(?1,?2,?3,?4,?5)",params![slot,scheduled_utc,stamp().utc,status,tasks.len() as i64]).map_err(sql)?;
        tx.commit().map_err(sql)?;
        if tasks.is_empty(){Ok(None)}else{Ok(Some(tasks))}
    }
    pub fn finish_reminder(&mut self,slot:&str,ok:bool)->Result<()>{
        self.conn.execute("UPDATE reminder_slots SET status=?1 WHERE slot_key=?2 AND status='claimed'",params![if ok{"submitted"}else{"failed"},slot]).map_err(sql)?;Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn db()->Db { Db::open(Path::new(":memory:")).unwrap() }
    fn input(body:&str)->SaveTask{SaveTask{command_id:Uuid::new_v4().to_string(),id:None,expected_version:None,content_markdown:body.into(),priority:"P2".into()}}
    #[test] fn invalid_input_never_writes(){let mut db=db();assert!(db.save(input(" \n\u{3000}")).is_err());let mut bad=input("text");bad.priority="P0".into();assert!(db.save(bad).is_err());assert!(db.list().unwrap().is_empty());}
    #[test] fn duplicate_submission_and_conflicting_reuse(){let mut db=db();let request=input("负责人：示例");let first=db.save(request.clone()).unwrap();assert_eq!(db.save(request.clone()).unwrap(),first);let mut changed=request;changed.content_markdown="different".into();assert!(db.save(changed).is_err());assert_eq!(db.list().unwrap().len(),1);assert_eq!(db.conn.query_row("SELECT COUNT(*) FROM task_events",[],|r|r.get::<_,i64>(0)).unwrap(),1);}
    #[test] fn stale_editor_cannot_overwrite(){let mut db=db();let task=db.save(input("before")).unwrap();let mut edit=input("after");edit.id=Some(task.id.clone());edit.expected_version=Some(1);db.save(edit.clone()).unwrap();edit.command_id=Uuid::new_v4().to_string();edit.content_markdown="stale".into();assert!(db.save(edit).is_err());assert_eq!(db.get(&task.id).unwrap().content_markdown,"after");}
    #[test] fn transaction_failure_rolls_back_task(){let mut db=db();db.set_setting("quick_draft",&"preserve me").unwrap();db.conn.execute_batch("CREATE TRIGGER reject_event BEFORE INSERT ON task_events BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();assert!(db.save(input("should rollback")).is_err());assert!(db.list().unwrap().is_empty());assert_eq!(db.setting::<String>("quick_draft").unwrap().unwrap(),"preserve me");}
    #[test] fn complete_reopen_preserves_history(){let mut db=db();let task=db.save(input("pending")).unwrap();let done=db.complete(CompleteTask{command_id:Uuid::new_v4().to_string(),id:task.id.clone(),expected_version:1,completed:true}).unwrap();assert!(done.completed_at_utc.is_some());let open=db.complete(CompleteTask{command_id:Uuid::new_v4().to_string(),id:task.id,expected_version:2,completed:false}).unwrap();assert_eq!(open.status,"open");assert_eq!(open.created_local_date,done.created_local_date);assert!(open.completed_at_utc.is_none());assert_eq!(db.conn.query_row("SELECT COUNT(*) FROM task_events",[],|r|r.get::<_,i64>(0)).unwrap(),3);}
    #[test] fn reorder_is_atomic_and_requires_full_current_list(){let mut db=db();let a=db.save(input("a")).unwrap();let b=db.save(input("b")).unwrap();let day=a.created_local_date.clone();assert!(db.reorder(ReorderTasks{command_id:Uuid::new_v4().to_string(),local_date:day.clone(),tasks:vec![TaskVersion{id:a.id.clone(),version:1}]}).is_err());db.reorder(ReorderTasks{command_id:Uuid::new_v4().to_string(),local_date:day,tasks:vec![TaskVersion{id:b.id.clone(),version:1},TaskVersion{id:a.id.clone(),version:1}]}).unwrap();assert_eq!(db.list_for_date(&a.created_local_date).unwrap()[0].id,b.id);}
    #[test] fn list_for_date_and_copy_to_today(){let mut db=db();let task=db.save(input("history")).unwrap();assert_eq!(db.list_for_date(&task.created_local_date).unwrap().len(),1);let copy=db.copy_to_today(CopyTaskToToday{command_id:Uuid::new_v4().to_string(),source_id:task.id.clone()}).unwrap();assert_ne!(copy.id,task.id);assert_eq!(copy.content_markdown,task.content_markdown);assert_eq!(db.list().unwrap().len(),2);}
    #[test] fn reopen_file_and_read_only_analysis(){let path=std::env::temp_dir().join(format!("littletodo-test-{}.sqlite3",Uuid::new_v4()));{let mut db=Db::open(&path).unwrap();db.save(input("persist")).unwrap();} {let db=Db::open(&path).unwrap();assert_eq!(db.list().unwrap().len(),1);let ro=Connection::open_with_flags(&path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();assert_eq!(ro.query_row("SELECT COUNT(*) FROM v_tasks_readable",[],|r|r.get::<_,i64>(0)).unwrap(),1);assert!(ro.execute("DELETE FROM tasks",[]).is_err());} let _=std::fs::remove_file(path);}
    #[test] fn rejects_future_database(){let db=db();db.conn.pragma_update(None,"user_version",99).unwrap();let tmp=std::env::temp_dir().join(format!("littletodo-future-{}.db",Uuid::new_v4()));db.conn.backup(rusqlite::MAIN_DB,&tmp,None).unwrap();assert!(Db::open(&tmp).is_err());let _=std::fs::remove_file(tmp);}
    #[test] fn reminder_only_counts_today_open(){let mut db=db();db.save(input("yesterday")).unwrap();db.conn.execute("UPDATE tasks SET created_local_date='2000-01-01' WHERE content_markdown='yesterday'",[]).unwrap();assert!(db.claim_reminder("slot-today","2026-10-28T02:00:00Z").unwrap().is_none());db.save(input("today")).unwrap();assert_eq!(db.claim_reminder("slot-today2","2026-10-28T03:00:00Z").unwrap().unwrap().len(),1);}
    #[test] fn reminder_empty_and_duplicate_slots_are_not_sent(){let mut db=db();assert!(db.claim_reminder("empty","2026-10-28T01:00:00Z").unwrap().is_none());db.save(input("one")).unwrap();assert!(db.claim_reminder("empty","2026-10-28T01:00:00Z").unwrap().is_none());assert_eq!(db.claim_reminder("next","2026-10-28T02:00:00Z").unwrap().unwrap().len(),1);assert!(db.claim_reminder("next","2026-10-28T02:00:00Z").unwrap().is_none());db.finish_reminder("next",true).unwrap();assert_eq!(db.conn.query_row("SELECT status FROM reminder_slots WHERE slot_key='next'",[],|r|r.get::<_,String>(0)).unwrap(),"submitted");}
}

