#[cfg(windows)]
mod inner {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::OnceLock;
    use tauri_winrt_notification::{Duration, Scenario, Sound, Toast};

    static SHOW_LIST: OnceLock<fn()> = OnceLock::new();

    pub fn bind_show_list_action(action: fn()) {
        let _ = SHOW_LIST.set(action);
    }

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SetCurrentProcessExplicitAppUserModelID(app_id: *const u16) -> i32;
    }

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn write_reg_value(key: &str, value_name: Option<&str>, data: &str) -> Result<(), String> {
        use std::process::Command;
        #[cfg(windows)]
        use std::os::windows::process::CommandExt;
        let mut args = vec![
            "add".to_string(),
            format!(r"HKCU\{key}"),
            "/t".to_string(),
            "REG_SZ".to_string(),
            "/d".to_string(),
            data.to_string(),
            "/f".to_string(),
        ];
        if let Some(name) = value_name {
            args.insert(2, "/v".into());
            args.insert(3, name.into());
        } else {
            args.insert(2, "/ve".into());
        }
        let mut cmd = Command::new("reg");
        cmd.args(&args);
        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000);
        let status = cmd
            .status()
            .map_err(|e| format!("无法写入通知注册表：{e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("无法写入通知注册表".into())
        }
    }

    fn on_toast_action(action: Option<String>) -> Result<(), tauri_winrt_notification::Error> {
        if action.as_deref() == Some("show-list") {
            if let Some(f) = SHOW_LIST.get() {
                f();
            }
        }
        Ok(())
    }

    /// Windows 桌面 toast 需要进程 AppUserModelID 与发送标识一致；仅依赖开始菜单快捷方式不够。
    pub fn ensure_app_identity(app_id: &str, display_name: &str) -> Result<(), String> {
        let hr = unsafe { SetCurrentProcessExplicitAppUserModelID(wide(app_id).as_ptr()) };
        if hr < 0 {
            return Err(format!("无法注册应用通知标识（HRESULT {hr}）"));
        }
        let key = format!(r"Software\Classes\AppUserModelId\{app_id}");
        write_reg_value(&key, None, display_name)?;
        write_reg_value(&key, Some("DisplayName"), display_name)?;
        Ok(())
    }

    pub enum ToastKind {
        /// 设置页测试：较长横幅，仍会自动收起。
        Test,
        /// 整点待办：Reminder 场景，展开停留直至点击按钮；通知中心仍保留记录。
        TaskReminder,
    }

    pub fn show_toast(app_id: &str, title: &str, body: &str, kind: ToastKind) -> Result<(), String> {
        let mut toast = Toast::new(app_id).title(title).text2(body);
        match kind {
            ToastKind::Test => {
                toast = toast.duration(Duration::Long);
            }
            ToastKind::TaskReminder => {
                toast = toast
                    .duration(Duration::Long)
                    .scenario(Scenario::Reminder)
                    .sound(Some(Sound::Reminder))
                    .add_button("查看待办", "show-list")
                    .add_button("知道了", "ack")
                    .on_activated(on_toast_action);
            }
        }
        toast.show().map_err(|e| format!("无法提交系统提醒：{e:?}"))
    }
}

#[cfg(windows)]
pub use inner::{bind_show_list_action, ensure_app_identity, show_toast, ToastKind};

#[cfg(not(windows))]
pub fn bind_show_list_action(_action: fn()) {}

#[cfg(not(windows))]
pub fn ensure_app_identity(_app_id: &str, _display_name: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
pub enum ToastKind {
    Test,
    TaskReminder,
}

#[cfg(not(windows))]
pub fn show_toast(_app_id: &str, _title: &str, _body: &str, _kind: ToastKind) -> Result<(), String> {
    Err("此平台的系统提醒尚未实现".into())
}

#[cfg(all(windows, test))]
mod tests {
    use super::{ensure_app_identity, show_toast, ToastKind};

    #[test]
    #[ignore = "manual: confirms toast delivery on a Windows desktop session"]
    fn live_toast_uses_app_id() {
        ensure_app_identity("com.littletodo.desktop", "LittleTodo").unwrap();
        show_toast(
            "com.littletodo.desktop",
            "LittleTodo 提醒测试",
            "这是一条离线验证提醒。",
            ToastKind::Test,
        )
        .unwrap();
    }
}
