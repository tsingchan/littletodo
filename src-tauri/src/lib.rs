mod db;
mod desktop;
mod reminder;
mod win_notify;
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_opener::OpenerExt;
use db::{Db, Task, SaveTask, CompleteTask, ReorderTasks, CopyTaskToToday};

#[derive(Clone,serde::Serialize,serde::Deserialize)]
#[serde(default)]
struct Preferences {
    color:String, opacity:f64, x:Option<i32>, y:Option<i32>, shortcut:String,
    shortcut_show_list:String,
    show_completed:bool, notifications_preview:bool,
}
fn default_show_list_shortcut()->String{"Ctrl+Alt+L".into()}
impl Default for Preferences {
    fn default()->Self{Self{color:"cream".into(),opacity:0.9,x:None,y:None,shortcut:"Ctrl+Alt+N".into(),shortcut_show_list:default_show_list_shortcut(),show_completed:false,notifications_preview:true}}
}

struct State {
    db: Mutex<Db>,
    data_path: String,
    data_dir: String,
    detail_task_id: Mutex<Option<String>>,
    shortcut_error: Mutex<Option<String>>,
    shortcut_quick: Mutex<String>,
    shortcut_show_list: Mutex<String>,
    dirty_windows: Mutex<std::collections::HashSet<String>>,
    tray_error: Mutex<Option<String>>,
    quick_opening: std::sync::atomic::AtomicBool,
}
static TOAST_APP: OnceLock<tauri::AppHandle> = OnceLock::new();
fn toast_open_list_from_notify(){
    if let Some(app)=TOAST_APP.get(){
        let show=app.clone();
        let run=app.clone();
        let _=run.run_on_main_thread(move ||show_panel(&show,true));
    }
}
fn validate_shortcut(value:&str)->db::Result<()>{
    if value.len()>50 || value.trim().is_empty(){return Err("快捷键格式无效".into());}
    Ok(())
}
fn register_shortcuts(app:&tauri::AppHandle,quick:&str,show_list:&str)->Result<(),String>{
    if quick.eq_ignore_ascii_case(show_list){return Err("两个快捷键不能相同".into());}
    validate_shortcut(quick).map_err(|e|e)?;
    validate_shortcut(show_list).map_err(|e|e)?;
    let _=app.global_shortcut().unregister_all();
    app.global_shortcut().register(quick).map_err(|e|format!("快速记录快捷键不可用或已被占用：{e}"))?;
    app.global_shortcut().register(show_list).map_err(|e|{
        let _=app.global_shortcut().unregister(quick);
        format!("显示待办快捷键不可用或已被占用：{e}")
    })
}
fn unregister_shortcuts(app:&tauri::AppHandle,quick:&str,show_list:&str){
    let _=app.global_shortcut().unregister(quick);
    let _=app.global_shortcut().unregister(show_list);
}
fn shortcut_matches(stored:&str,fired:&Shortcut)->bool{
    Shortcut::from_str(stored).map(|parsed|parsed.id()==fired.id()).unwrap_or(false)
}
fn with_db<T>(state:&State, action:impl FnOnce(&mut Db)->db::Result<T>)->db::Result<T>{
    let mut db=state.db.lock().map_err(|_|"数据库忙，请重新启动应用".to_string())?; action(&mut db)
}
fn show_quick(app:&tauri::AppHandle) {
    if let Some(window)=app.get_webview_window("quick") {
        let _=window.unminimize();let _=window.show();let _=window.set_focus();
        let _=window.emit("focus-input",());
        return;
    }
    let Some(state)=app.try_state::<State>() else {return;};
    if state.quick_opening.swap(true,std::sync::atomic::Ordering::SeqCst){return;}
    let handle=app.clone();
    tauri::async_runtime::spawn(async move {
        let result=tauri::WebviewWindowBuilder::new(&handle,"quick",tauri::WebviewUrl::App("index.html".into()))
            .title("LittleTodo · 快速记录").inner_size(560.0,420.0).min_inner_size(420.0,320.0).center().build();
        if let Some(state)=handle.try_state::<State>(){state.quick_opening.store(false,std::sync::atomic::Ordering::SeqCst);}
        match result {
            Ok(window)=>{let _=window.set_focus();},
            Err(error)=>{let _=handle.emit_to("main","desktop-error",format!("无法打开快速记录：{error}"));show_panel(&handle,true);}
        }
    });
}
#[tauri::command]
fn list_tasks(state:tauri::State<State>,local_date:String)->db::Result<Vec<Task>> {
    with_db(&state,|db|db.list_for_date(&local_date))
}
#[tauri::command]
fn get_task(state:tauri::State<State>,id:String)->db::Result<Task>{with_db(&state,|db|db.get(&id))}
#[tauri::command]
fn save_task(app:tauri::AppHandle,state:tauri::State<State>,input:SaveTask)->db::Result<Task>{
    let result=with_db(&state,|db|db.save(input))?;
    let _=app.emit("tasks-changed",());Ok(result)
}
#[tauri::command]
fn complete_task(app:tauri::AppHandle,state:tauri::State<State>,input:CompleteTask)->db::Result<Task>{
    let result=with_db(&state,|db|db.complete(input))?;let _=app.emit("tasks-changed",());Ok(result)
}
#[tauri::command]
fn reorder_tasks(app:tauri::AppHandle,state:tauri::State<State>,input:ReorderTasks)->db::Result<()>{
    with_db(&state,|db|db.reorder(input))?;let _=app.emit("tasks-changed",());Ok(())
}
#[tauri::command]
fn copy_task_to_today(app:tauri::AppHandle,state:tauri::State<State>,input:CopyTaskToToday)->db::Result<Task>{
    let result=with_db(&state,|db|db.copy_to_today(input))?;
    let _=app.emit("tasks-changed",());Ok(result)
}
#[tauri::command]
fn new_task(app:tauri::AppHandle){show_quick(&app);}
#[tauri::command]
fn get_preferences(state:tauri::State<State>)->db::Result<Preferences>{
    with_db(&state,|db|Ok(db.setting("preferences")?.unwrap_or_default()))
}
#[tauri::command]
async fn save_preferences(app:tauri::AppHandle,state:tauri::State<'_,State>,prefs:Preferences)->db::Result<()> {
    if !["cream","mint","blue","pink"].contains(&prefs.color.as_str()) || !prefs.opacity.is_finite() || !(0.4..=1.0).contains(&prefs.opacity){return Err("外观设置无效".into());}
    validate_shortcut(&prefs.shortcut)?;
    validate_shortcut(&prefs.shortcut_show_list)?;
    if prefs.shortcut.eq_ignore_ascii_case(&prefs.shortcut_show_list){return Err("两个快捷键不能相同".into());}
    let mut quick=state.shortcut_quick.lock().map_err(|_|"快捷键设置不可用")?;
    let mut show_list=state.shortcut_show_list.lock().map_err(|_|"快捷键设置不可用")?;
    let previous_quick=quick.clone();
    let previous_show=show_list.clone();
    let prior_error=state.shortcut_error.lock().map_err(|_|"快捷键设置不可用")?.is_some();
    let changed=previous_quick!=prefs.shortcut || previous_show!=prefs.shortcut_show_list || prior_error;
    if changed {
        unregister_shortcuts(&app,&previous_quick,&previous_show);
        register_shortcuts(&app,&prefs.shortcut,&prefs.shortcut_show_list)?;
    }
    if let Err(err)=with_db(&state,|db|db.set_setting("preferences",&prefs)) {
        if changed {
            unregister_shortcuts(&app,&prefs.shortcut,&prefs.shortcut_show_list);
            let _=register_shortcuts(&app,&previous_quick,&previous_show);
        }
        return Err(err);
    }
    if changed {
        *quick=prefs.shortcut.clone();
        *show_list=prefs.shortcut_show_list.clone();
        if let Ok(mut error)=state.shortcut_error.lock(){*error=None;}
    }
    let _=app.emit("preferences-changed",&prefs);Ok(())
}
fn safe_position(window:&tauri::WebviewWindow,requested:Option<(i32,i32)>)->db::Result<(i32,i32)> {
    let monitors=window.available_monitors().map_err(|e|e.to_string())?;
    if monitors.is_empty(){return Err("无法获取显示器信息".into());}
    if let Some((x,y))=requested {
        if monitors.iter().any(|m|{let p=m.position();let s=m.size();x>=p.x && y>=p.y && x+160<(p.x+s.width as i32) && y+100<(p.y+s.height as i32)}){return Ok((x,y));}
    }
    let primary=window.primary_monitor().map_err(|e|e.to_string())?.unwrap_or_else(||monitors[0].clone());
    let p=primary.position();let s=primary.size();
    Ok((p.x+(s.width as i32-400).max(0),p.y+30))
}
fn ensure_main_panel(window:&tauri::WebviewWindow)->db::Result<()> {
    const MIN_W:f64=300.0;
    const MIN_H:f64=240.0;
    const DEF_W:f64=380.0;
    const DEF_H:f64=460.0;
    #[cfg(target_os="windows")]
    if let Ok(handle)=window.hwnd(){
        let _=desktop::detach(handle.0 as isize);
    }
    let scale=window.scale_factor().map_err(|e|e.to_string())?;
    let size=window.inner_size().map_err(|e|e.to_string())?;
    let min_w=(MIN_W*scale).round() as u32;
    let min_h=(MIN_H*scale).round() as u32;
    if size.width<min_w || size.height<min_h {
        window.set_size(tauri::PhysicalSize::new((DEF_W*scale).round() as u32,(DEF_H*scale).round() as u32)).map_err(|e|e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
fn prepare_panel(window:tauri::WebviewWindow,state:tauri::State<State>)->db::Result<()> {
    if window.label()!="main"{return Err("无效窗口".into());}
    #[cfg(debug_assertions)]
    if std::env::var("LITTLETODO_QA_UNPIN").as_deref()==Ok("1") {
        let prefs:Preferences=with_db(&state,|db|Ok(db.setting("preferences")?.unwrap_or_default()))?;
        let (x,y)=safe_position(&window,prefs.x.zip(prefs.y))?;
        window.set_position(tauri::PhysicalPosition::new(x,y)).map_err(|e|e.to_string())?;
        let _=window.set_skip_taskbar(false);
        let _=window.set_always_on_top(false);
        return Ok(());
    }
    let prefs:Preferences=with_db(&state,|db|Ok(db.setting("preferences")?.unwrap_or_default()))?;
    let (x,y)=safe_position(&window,prefs.x.zip(prefs.y))?;
    #[cfg(target_os="windows")]
    if let Ok(handle)=window.hwnd(){
        let _=desktop::detach(handle.0 as isize);
    }
    ensure_main_panel(&window)?;
    window.set_position(tauri::PhysicalPosition::new(x,y)).map_err(|e|e.to_string())?;
    window.set_skip_taskbar(false).map_err(|e|e.to_string())?;
    window.set_always_on_top(false).map_err(|e|e.to_string())?;
    Ok(())
}
#[tauri::command]
fn attach_panel(window:tauri::WebviewWindow,state:tauri::State<State>)->db::Result<()> {
    prepare_panel(window,state)
}
#[tauri::command]
fn move_panel(window:tauri::WebviewWindow,x:i32,y:i32)->db::Result<()> {
    if window.label()!="main"{return Err("无效窗口".into());}
    let allowed=safe_position(&window,Some((x,y)))?;
    if allowed!=(x,y){return Err("已到达屏幕边缘".into());}
    #[cfg(target_os="windows")]
    if let Ok(handle)=window.hwnd(){
        if desktop::attached(handle.0 as isize){
            return desktop::move_to(handle.0 as isize,x,y);
        }
    }
    window.set_position(tauri::PhysicalPosition::new(x,y)).map_err(|e|e.to_string())
}
#[tauri::command]
fn save_position(state:tauri::State<State>,x:i32,y:i32)->db::Result<()> {
    with_db(&state,|db|{let mut prefs:Preferences=db.setting("preferences")?.unwrap_or_default();prefs.x=Some(x);prefs.y=Some(y);db.set_setting("preferences",&prefs)})
}
#[tauri::command]
async fn open_detail(app:tauri::AppHandle,state:tauri::State<'_,State>,id:String)->db::Result<()> {
    uuid::Uuid::parse_str(&id).map_err(|_|"事项标识无效")?;
    with_db(&state,|db|db.get(&id))?;
    const LABEL:&str="detail";
    if let Some(window)=app.get_webview_window(LABEL) {
        window.show().map_err(|e|e.to_string())?;window.set_focus().map_err(|e|e.to_string())?;
        let dirty=state.dirty_windows.lock().map_err(|_|"窗口状态不可用")?.contains(LABEL);
        if dirty {
            let _=window.emit("detail-unsaved-block",());
            return Ok(());
        }
        let same=state.detail_task_id.lock().map_err(|_|"窗口状态不可用")?.as_deref()==Some(id.as_str());
        if same {
            let _=window.emit("detail-reopen",());
        } else {
            *state.detail_task_id.lock().map_err(|_|"窗口状态不可用")?=Some(id.clone());
            let _=window.emit("detail-open",serde_json::json!({"id":id}));
        }
    } else {
        *state.detail_task_id.lock().map_err(|_|"窗口状态不可用")?=Some(id.clone());
        let window=tauri::WebviewWindowBuilder::new(&app,LABEL,tauri::WebviewUrl::App("index.html".into()))
            .title("LittleTodo · 事项详情").inner_size(650.0,560.0).min_inner_size(480.0,420.0)
            .build().map_err(|e|e.to_string())?;
        let _=window.emit("detail-open",serde_json::json!({"id":id}));
    } Ok(())
}
#[tauri::command]
fn current_detail_id(state:tauri::State<State>)->db::Result<Option<String>>{
    Ok(state.detail_task_id.lock().map_err(|_|"窗口状态不可用")?.clone())
}
#[tauri::command]
async fn open_settings(app:tauri::AppHandle)->db::Result<()> {
    if let Some(window)=app.get_webview_window("settings") {window.show().map_err(|e|e.to_string())?;window.set_focus().map_err(|e|e.to_string())?;}
    else {tauri::WebviewWindowBuilder::new(&app,"settings",tauri::WebviewUrl::App("index.html".into())).title("LittleTodo · 设置").inner_size(500.0,560.0).build().map_err(|e|e.to_string())?;}
    Ok(())
}
#[tauri::command]
fn open_url(app:tauri::AppHandle,url:String)->db::Result<()> {
    let parsed=tauri::Url::parse(&url).map_err(|_|"链接格式无效")?;
    if !["http","https"].contains(&parsed.scheme()) || parsed.host_str().is_none(){return Err("只允许打开http或https链接".into());}
    app.opener().open_url(parsed.as_str(),None::<&str>).map_err(|e|e.to_string())
}
#[tauri::command]
fn open_data_dir(app:tauri::AppHandle,state:tauri::State<State>)->db::Result<()> {
    let dir=std::path::Path::new(&state.data_dir);
    if !dir.is_dir(){return Err("数据目录不存在".into());}
    app.opener().open_path(&state.data_dir,None::<&str>).map_err(|e|e.to_string())
}
#[tauri::command]
fn app_info(state:tauri::State<State>)->serde_json::Value{
    serde_json::json!({"data_path":state.data_path,"data_dir":state.data_dir,"shortcut_error":state.shortcut_error.lock().ok().and_then(|x|x.clone()),"tray_error":state.tray_error.lock().ok().and_then(|x|x.clone()),"version":env!("CARGO_PKG_VERSION")})
}
#[tauri::command]
fn get_draft(state:tauri::State<State>)->db::Result<Option<serde_json::Value>>{with_db(&state,|db|db.setting("quick_draft"))}
#[tauri::command]
fn set_draft(state:tauri::State<State>,value:serde_json::Value)->db::Result<()>{
    if serde_json::to_string(&value).map_err(|e|e.to_string())?.len()>110_000{return Err("草稿过长".into());}
    with_db(&state,|db|db.set_setting("quick_draft",&value))
}
#[tauri::command]
fn set_dirty(window:tauri::WebviewWindow,state:tauri::State<State>,dirty:bool)->db::Result<()>{
    let label=window.label().to_owned();
    if label!="quick" && label!="detail"{return Err("无效窗口".into());}
    let mut pending=state.dirty_windows.lock().map_err(|_|"窗口状态不可用")?;
    if dirty{pending.insert(label);}else{pending.remove(&label);}Ok(())
}
#[tauri::command]
async fn close_aux(window:tauri::WebviewWindow)->db::Result<()>{
    if window.label()!="quick" && window.label()!="settings" && window.label()!="detail"{return Err("无效窗口".into());}
    window.destroy().map_err(|e|e.to_string())
}
fn request_exit(app:&tauri::AppHandle){
    let Some(state)=app.try_state::<State>() else {return;};
    let dirty=state.dirty_windows.lock().ok().and_then(|items|items.iter().next().cloned());
    if let Some(label)=dirty {
        if let Some(window)=app.get_webview_window(&label){
            let _=window.show();let _=window.set_focus();let _=window.emit("quit-blocked",());return;
        }
        if let Ok(mut pending)=state.dirty_windows.lock(){pending.remove(&label);}
    }
    app.exit(0);
}
#[tauri::command]
fn quit_app(app:tauri::AppHandle){request_exit(&app)}
fn show_panel(app:&tauri::AppHandle,raise:bool){
    if let Some(window)=app.get_webview_window("main"){
        let _=ensure_main_panel(&window);
        if raise {
            let _=window.set_always_on_top(true);
        }
        let _=window.unminimize();
        let _=window.show();
        let _=window.set_focus();
        let _=window.emit("refresh-tasks",());
    }
}
#[tauri::command]
fn hide_panel(window:tauri::WebviewWindow,state:tauri::State<State>)->db::Result<()>{
    if window.label()!="main"{return Err("无效窗口".into());}
    if let Some(error)=state.tray_error.lock().map_err(|_|"托盘状态不可用")?.as_ref(){return Err(format!("托盘不可用，不能收起：{error}"));}
    window.set_always_on_top(false).map_err(|e|e.to_string())?;
    window.hide().map_err(|e|e.to_string())
}
#[tauri::command]
fn show_list(app:tauri::AppHandle){show_panel(&app,true);}
#[tauri::command]
fn next_reminder_info(state:tauri::State<State>)->db::Result<serde_json::Value>{
    let next=reminder::next_reminder_after(chrono::Local::now());
    let open=with_db(&state,|db|db.count_open_tasks())?;
    let when=reminder::format_next_reminder_local(next);
    let label=if open>0{format!("下次提醒：{when}（工作日 9–18 整点）")}else{format!("下次可提醒：{when}（当前无未完成，到时不会弹通知）")};
    Ok(serde_json::json!({"label":label,"at_local":next.format("%Y-%m-%dT%H:%M:%S").to_string(),"open_count":open}))
}
#[tauri::command]
fn test_notification(app:tauri::AppHandle)->db::Result<()>{
    let id=app.config().identifier.clone();
    win_notify::show_toast(&id,"LittleTodo 提醒测试","这是一条测试提醒。工作日有未完成事项时，LittleTodo 会在整点提醒你。",win_notify::ToastKind::Test)
}
fn create_tray(app:&tauri::App)->tauri::Result<()> {
    use tauri::menu::{Menu,MenuItem};
    use tauri::tray::{TrayIconBuilder,TrayIconEvent,MouseButton};
    let show=MenuItem::with_id(app,"show","显示待办",true,None::<&str>)?;
    let quick=MenuItem::with_id(app,"quick","快速记录",true,None::<&str>)?;
    let settings=MenuItem::with_id(app,"settings","设置",true,None::<&str>)?;
    let quit=MenuItem::with_id(app,"quit","退出 LittleTodo",true,None::<&str>)?;
    let menu=Menu::with_items(app,&[&show,&quick,&settings,&quit])?;
    let icon=app.default_window_icon().ok_or_else(||tauri::Error::AssetNotFound("icon".into()))?.clone();
    TrayIconBuilder::with_id("littletodo").icon(icon).tooltip("LittleTodo").menu(&menu).show_menu_on_left_click(false)
        .on_menu_event(|app,event|match event.id().as_ref(){
            "show"=>show_panel(app,true),"quick"=>show_quick(app),
            "settings"=>{let app=app.clone();tauri::async_runtime::spawn(async move {let _=open_settings(app).await;});},
            "quit"=>request_exit(app),_=>{}
        })
        .on_tray_icon_event(|tray,event|{
            if let TrayIconEvent::DoubleClick{button:MouseButton::Left,..}=event{
                show_panel(tray.app_handle(),true);
            }
        })
        .build(app)?;
    Ok(())
}
pub fn run(){
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app,_,_|show_quick(app)))
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app,shortcut,event|{
            if event.state!=ShortcutState::Pressed {return;}
            let Some(state)=app.try_state::<State>() else {return;};
            let quick=state.shortcut_quick.lock().ok().map(|v|v.clone());
            let show_list=state.shortcut_show_list.lock().ok().map(|v|v.clone());
            if let (Some(quick),Some(show_list))=(quick,show_list){
                if shortcut_matches(&quick,shortcut){show_quick(app);}
                else if shortcut_matches(&show_list,shortcut){show_panel(app,true);}
            }
        }).build())
        .plugin(tauri_plugin_opener::Builder::new().open_js_links_on_click(false).build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app|{
            let path=if let Some(dir)=std::env::var_os("LITTLETODO_DATA_DIR"){std::path::PathBuf::from(dir)}
                else {app.path().app_local_data_dir()?};
            std::fs::create_dir_all(&path)?;
            let database=path.join("littletodo.sqlite3");
            let data_dir=path.display().to_string();
            let db=Db::open(&database).map_err(std::io::Error::other)?;
            let config=app.config();
            if let Err(error)=win_notify::ensure_app_identity(&config.identifier,config.product_name.as_deref().unwrap_or("LittleTodo")){
                eprintln!("LittleTodo 通知标识注册失败：{error}");
            }
            let _=TOAST_APP.set(app.handle().clone());
            win_notify::bind_show_list_action(toast_open_list_from_notify);
            let prefs=db.setting::<Preferences>("preferences").ok().flatten().unwrap_or_default();
            let error=register_shortcuts(app.handle(),&prefs.shortcut,&prefs.shortcut_show_list).err();
            app.manage(State{db:Mutex::new(db),data_path:database.display().to_string(),data_dir,detail_task_id:Mutex::new(None),shortcut_error:Mutex::new(error),shortcut_quick:Mutex::new(prefs.shortcut),shortcut_show_list:Mutex::new(prefs.shortcut_show_list),dirty_windows:Mutex::new(std::collections::HashSet::new()),tray_error:Mutex::new(None),quick_opening:std::sync::atomic::AtomicBool::new(false)});
            if let Err(error)=create_tray(app){if let Ok(mut value)=app.state::<State>().tray_error.lock(){*value=Some(error.to_string());}}
            #[cfg(debug_assertions)]
            let start_reminders=std::env::var("LITTLETODO_QA_NO_NOTIFICATIONS").as_deref()!=Ok("1");
            #[cfg(not(debug_assertions))]
            let start_reminders=true;
            if start_reminders{reminder::start(app.handle().clone());}
            Ok(())
        })
        .on_window_event(|window,event|{
            if let tauri::WindowEvent::Destroyed=event {
                if let Some(state)=window.try_state::<State>(){
                    if let Ok(mut dirty)=state.dirty_windows.lock(){dirty.remove(window.label());}
                    if window.label()=="detail"{if let Ok(mut id)=state.detail_task_id.lock(){*id=None;}}
                }
            }
            if let tauri::WindowEvent::CloseRequested{api,..}=event {
                api.prevent_close();
                if window.label()=="quick" || window.label()=="settings" || window.label()=="detail" {
                    let _=window.emit("attempt-close",());
                } else if window.label()=="main" {
                    let state=window.state::<State>();
                    if state.tray_error.lock().map(|e|e.is_none()).unwrap_or(false){let _=window.hide();}
                    else {let _=window.emit("desktop-error","托盘不可用，不能收起窗口");}
                } else {let _=window.hide();}
            }
        })
        .invoke_handler(tauri::generate_handler![list_tasks,get_task,save_task,complete_task,reorder_tasks,copy_task_to_today,new_task,app_info,get_draft,set_draft,close_aux,quit_app,set_dirty,hide_panel,show_list,test_notification,next_reminder_info,get_preferences,save_preferences,prepare_panel,attach_panel,move_panel,save_position,open_detail,current_detail_id,open_settings,open_url,open_data_dir])
        .run(tauri::generate_context!()).expect("LittleTodo启动失败");
}

#[cfg(test)]
mod shortcut_tests {
    use super::shortcut_matches;
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::Shortcut;
    #[test]
    fn stored_ctrl_alt_format_matches_fired_hotkey() {
        let stored = "Ctrl+Alt+N";
        let parsed = Shortcut::from_str(stored).unwrap();
        assert!(shortcut_matches(stored, &parsed));
        assert!(shortcut_matches("ctrl+alt+l", &Shortcut::from_str("Ctrl+Alt+L").unwrap()));
    }
}
