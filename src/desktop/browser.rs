/// Open `url` in the system default browser.
///
/// Spawns the platform opener (`open`, `start`, `xdg-open`) and returns immediately.
pub fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "windows")]
    let cmd = "explorer";
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let cmd = "xdg-open";

    std::process::Command::new(cmd)
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("open browser: {e}"))
}
