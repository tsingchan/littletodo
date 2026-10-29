use chrono::{DateTime,Datelike,TimeZone,Timelike,Weekday};
use tauri::Manager;
use crate::{State,Preferences,with_db,win_notify};

pub fn slot_for<T:TimeZone>(local:&DateTime<T>,zone:&str)->Option<String> where T::Offset:std::fmt::Display {
    if matches!(local.weekday(),Weekday::Sat|Weekday::Sun)|| !(9..=18).contains(&local.hour()) || local.minute()!=0{return None;}
    Some(format!("{}-{:02}-{}-{}",local.format("%Y-%m-%d"),local.hour(),zone,local.offset()))
}
pub fn next_reminder_after(from:DateTime<chrono::Local>)->DateTime<chrono::Local>{
    use chrono::Duration;
    let mut t=from.with_second(0).unwrap().with_nanosecond(0).unwrap();
    if from.minute()!=0 || from.second()>0 || from.nanosecond()>0{t+=Duration::hours(1);}
    loop{
        while matches!(t.weekday(),Weekday::Sat|Weekday::Sun){
            t=(t+Duration::days(1)).with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
        }
        let h=t.hour();
        if h<9{t=t.with_hour(9).unwrap();continue;}
        if h>18{t=(t+Duration::days(1)).with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap();continue;}
        return t;
    }
}
pub fn format_next_reminder_local(dt:DateTime<chrono::Local>)->String{
    format!("{} {:02}:{:02}",dt.format("%Y年%m月%d日"),dt.hour(),dt.minute())
}
pub fn start(app:tauri::AppHandle){
    std::thread::spawn(move || loop {
        let local=chrono::Local::now();
        let zone=iana_time_zone::get_timezone().unwrap_or_else(|_|local.offset().to_string());
        if let Some(slot)=slot_for(&local,&zone) {
            if let Some(state)=app.try_state::<State>() {
                let scheduled=local.with_minute(0).and_then(|v|v.with_second(0)).and_then(|v|v.with_nanosecond(0)).unwrap_or(local);
                let result=with_db(&state,|db|db.claim_reminder(&slot,&scheduled.with_timezone(&chrono::Utc).to_rfc3339()));
                if let Ok(Some(tasks))=result {
                    let preview=with_db(&state,|db|Ok(db.setting::<Preferences>("preferences")?.unwrap_or_default().notifications_preview)).unwrap_or(false);
                    let body=if preview {
                        let names=tasks.iter().take(3).map(|task|{
                            let text=task.content_markdown.lines().find(|l|!l.trim().is_empty()).unwrap_or("").trim();
                            let clean:String=text.chars().filter(|c|!c.is_control()).take(45).collect();
                            format!("{} {}",task.priority,clean)
                        }).collect::<Vec<_>>().join("；");
                        format!("还有{}项未完成：{}",tasks.len(),names)
                    }else{format!("还有{}项未完成",tasks.len())};
                    let id=app.config().identifier.clone();
                    let delivered=win_notify::show_toast(&id,"LittleTodo 待办提醒",&body,win_notify::ToastKind::TaskReminder).is_ok();
                    let _=with_db(&state,|db|db.finish_reminder(&slot,delivered));
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    });
}
#[cfg(test)]
mod tests{
    use super::*;
    use chrono::{FixedOffset,TimeZone};
    fn at(y:i32,m:u32,d:u32,h:u32,min:u32,sec:u32)->DateTime<FixedOffset>{FixedOffset::east_opt(8*3600).unwrap().with_ymd_and_hms(y,m,d,h,min,sec).unwrap()}
    #[test] fn working_hours_and_weekends(){assert!(slot_for(&at(2026,10,28,9,0,0),"Asia/Shanghai").is_some());assert!(slot_for(&at(2026,10,28,18,0,59),"Asia/Shanghai").is_some());for d in [at(2026,10,28,8,0,0),at(2026,10,28,19,0,0),at(2026,10,28,9,1,0),at(2026,10,31,9,0,0),at(2026,11,1,9,0,0)]{assert!(slot_for(&d,"Asia/Shanghai").is_none());}}
    #[test] fn different_hours_have_different_keys(){assert_ne!(slot_for(&at(2026,10,28,9,0,0),"Asia/Shanghai"),slot_for(&at(2026,10,28,10,0,0),"Asia/Shanghai"));}
    #[test] fn next_reminder_skips_nights_and_weekends(){
        use chrono::Local;
        let l=|d:DateTime<FixedOffset>|d.with_timezone(&Local);
        assert_eq!(next_reminder_after(l(at(2026,10,28,10,30,0))).hour(),11);
        assert_eq!(next_reminder_after(l(at(2026,10,28,18,5,0))).format("%Y-%m-%d %H:%M").to_string(),"2026-10-29 09:00");
        assert_eq!(next_reminder_after(l(at(2026,10,31,12,0,0))).format("%Y-%m-%d %H:%M").to_string(),"2026-11-02 09:00");
    }
}
