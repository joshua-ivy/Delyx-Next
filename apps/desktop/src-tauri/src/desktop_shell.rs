use tauri::Manager;

pub const APP_NAME: &str = "Delyx Next";
pub const APP_IDENTIFIER: &str = "com.geaux.delyxnext";
pub const MAIN_WINDOW_LABEL: &str = "main";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopShellInfo {
    pub name: &'static str,
    pub identifier: &'static str,
    pub milestone: &'static str,
    pub main_window_label: &'static str,
    pub native_menu_policy: &'static str,
    pub startup_behavior: &'static str,
    pub reopen_behavior: &'static str,
    pub signing_policy: &'static str,
}

pub fn desktop_shell_info() -> DesktopShellInfo {
    DesktopShellInfo {
        identifier: APP_IDENTIFIER,
        main_window_label: MAIN_WINDOW_LABEL,
        milestone: "D12 refined Windows desktop shell",
        name: APP_NAME,
        native_menu_policy: "renderer_command_ui",
        reopen_behavior: "single_instance_focus_main_window",
        signing_policy: "unsigned_dev_build",
        startup_behavior: "focus_main_window",
    }
}

pub fn setup_desktop_shell(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    focus_existing_main_window(app.handle());
    Ok(())
}

pub fn focus_existing_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_windows_desktop_shell_policy() {
        let info = desktop_shell_info();

        assert_eq!(info.name, "Delyx Next");
        assert_eq!(info.identifier, "com.geaux.delyxnext");
        assert_eq!(info.main_window_label, "main");
        assert_eq!(info.native_menu_policy, "renderer_command_ui");
        assert_eq!(info.reopen_behavior, "single_instance_focus_main_window");
        assert_eq!(info.signing_policy, "unsigned_dev_build");
    }
}
