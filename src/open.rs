use crate::error::{Error, Result};
use std::env;
use std::process::Command;

#[cfg(target_os = "macos")]
static OPEN_COMMANDS: &[&[&str]] = &[&["open"]];

#[cfg(target_os = "linux")]
static OPEN_COMMANDS: &[&[&str]] = &[&["xdg-open"], &["gvfs-open"], &["gnome-open"]];

#[cfg(target_os = "windows")]
static OPEN_COMMANDS: &[&[&str]] = &[&["cmd", "/C", "start"]];

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux"
)))]
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Guard<F: FnMut()>(F);

    impl<F: FnMut()> Drop for Guard<F> {
        fn drop(&mut self) {
            self.0();
        }
    }

    #[test]
    fn open_successfully() {
        let old = env::var("EXTRACT_REPO_URL_OPEN_CMD").unwrap_or("".to_string());

        Guard(|| {
            env::set_var("EXTRACT_REPO_URL_OPEN_CMD", &old);
        });

        env::set_var("EXTRACT_REPO_URL_OPEN_CMD", "true");

        assert_eq!(open_in_browser("https://example.com"), Ok(()));
    }

    #[test]
    fn open_failure() {
        let old = env::var("EXTRACT_REPO_URL_OPEN_CMD").unwrap_or("".to_string());

        Guard(|| {
            env::set_var("EXTRACT_REPO_URL_OPEN_CMD", &old);
        });

        env::set_var("EXTRACT_REPO_URL_OPEN_CMD", "false");

        let url = "https://example.com";
        assert_eq!(
            open_in_browser(url),
            Err(Error::CannotOpenUrl(url.to_string()))
        );
    }

    #[test]
    fn open_command_not_found() {
        let old = env::var("EXTRACT_REPO_URL_OPEN_CMD").unwrap_or("".to_string());

        Guard(|| {
            env::set_var("EXTRACT_REPO_URL_OPEN_CMD", &old);
        });

        env::set_var(
            "EXTRACT_REPO_URL_OPEN_CMD",
            "command-doesnt-exist-in-your-system",
        );

        match open_in_browser("https://example.com") {
            Ok(..) => panic!("unexpected success"),
            Err(Error::IoFailure(..)) => { /* ok */ }
            Err(e) => panic!("unexpected error {:?}", e),
        }
    }

} // mod tests
