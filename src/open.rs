use error::{Error, Result};
use std::env;
use std::process::Command;

#[cfg(target_os = "macos")]
static OPEN_COMMANDS: &[&[&str]] = &[&["open"]];

#[cfg(target_os = "linux")]
static OPEN_COMMANDS: &[&[&str]] = &[&["xdg-open"], &["gvfs-open"], &["gnome-open"]];

#[cfg(target_os = "windows")]
static OPEN_COMMANDS: &[&[&str]] = &[&["cmd", "/C", "start"]];

#[cfg(
    not(
        any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux"
        )
    )
)]
static OPEN_COMMANDS: &[&[&str]] = &[];

pub fn open_in_browser(url: &str) -> Result<()> {
    if let Ok(cmd) = env::var("EXTRACT_REPO_URL_OPEN_CMD") {
        let status = Command::new(cmd).arg(url).status()?;
        return if status.success() {
            Ok(())
        } else {
            Err(Error::CannotOpenUrl(url.to_string()))
        };
    }

    if OPEN_COMMANDS.is_empty() {
        return Err(Error::OpenNotSupported);
    }

    for cmdline in OPEN_COMMANDS {
        let (cmd, args) = cmdline.split_first().unwrap();
        if Command::new(cmd).args(args).arg(url).status()?.success() {
            return Ok(());
        }
    }

    Err(Error::CannotOpenUrl(url.to_string()))
}
