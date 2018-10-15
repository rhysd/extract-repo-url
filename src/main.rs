extern crate clap;
extern crate clipboard;
extern crate regex;
#[macro_use]
extern crate lazy_static;

use clap::{App, Arg};
use clipboard::{ClipboardContext, ClipboardProvider};
use regex::Regex;
use std::process::Command;
use std::{env, fmt, io};

type Result<T> = ::std::result::Result<T, Error>;

#[derive(PartialEq)]
enum Error {
    EmptyText,
    IoFailure(String),
    RepoUrlNotFound(String),
    ClipboardReadFailure(String),
    TryNextHost,
    InvalidRegex(regex::Error),
    OpenNotSupported,
    CannotOpenUrl(String),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Error::EmptyText => "Input text is empty".to_string(),
            Error::IoFailure(msg) => format!("IO failure: {}", msg),
            Error::RepoUrlNotFound(text) => format!("No repository URL was found in '{}'", text),
            Error::TryNextHost => unreachable!(),
            Error::ClipboardReadFailure(msg) => {
                format!("Could not read clipboard content: {}", msg)
            }
            Error::InvalidRegex(inner) => format!("{}", inner),
            Error::OpenNotSupported => "Cannot open a browser on this OS".to_string(),
            Error::CannotOpenUrl(u) => format!("Cannot open URL '{}'", u),
        };
        write!(f, "{}", msg)
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Error {
        Error::IoFailure(format!("{}", inner))
    }
}

impl From<Box<std::error::Error>> for Error {
    fn from(inner: Box<std::error::Error>) -> Error {
        Error::ClipboardReadFailure(format!("{:?}", inner))
    }
}

impl From<regex::Error> for Error {
    fn from(inner: regex::Error) -> Error {
        Error::InvalidRegex(inner)
    }
}

lazy_static! {
    static ref SERVICE_HOSTS: Vec<String> = {
        let mut hosts = vec![
            "github.com".to_string(),
            "bitbucket.org".to_string(),
            "gitlab.com".to_string(),
        ];
        if let Ok(var) = env::var("EXTRACT_REPO_URL_SERVICE_HOSTS") {
            for host in var.split(',') {
                hosts.push(host.to_string());
            }
        }
        hosts
    };
}

enum Action {
    Open,
    Print,
}

fn parse_argv() -> Result<(String, Action)> {
    let matches = App::new("extract-repo-url")
        .author("rhysd <https://rhysd.github.io>")
        .version("v0.1.0")
        .usage("extract-repo-url [<text>]")
        .about("Extract repository URL from text (from clipboard by default)")
        .arg(
            Arg::with_name("open")
                .long("open")
                .short("o")
                .help("Open URL in a browser"),
        ).arg(
            Arg::with_name("stdin")
                .long("stdin")
                .short("s")
                .help("Read text from STDIN"),
        ).arg(
            Arg::with_name("text")
                .value_name("TEXT")
                .help("Text extracting from"),
        ).get_matches();

    let text = if let Some(text) = matches.value_of("text") {
        text.to_string()
    } else if matches.is_present("stdin") {
        use io::Read;
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
        ctx.get_contents()?
    };

    let action = if matches.is_present("open") {
        Action::Open
    } else {
        Action::Print
    };

    if text.is_empty() {
        Err(Error::EmptyText)?
    } else {
        Ok((text, action))
    }
}

fn extract_service_url(text: &str, host: &str) -> Result<String> {
    let escaped = regex::escape(host);
    for pat in &[
        format!(
            r"\bgit@{}:([[:alnum:]_-]+/[[:alnum:]_.-]+)(?:\.git)?\b",
            escaped
        ),
        format!(
            r"\b(?:https://){}/([[:alnum:]_-]+/[[:alnum:]_.-]+)(?:\.git)?\b",
            escaped
        ),
    ] {
        if let Some(caps) = Regex::new(pat)?.captures(text) {
            let slug = caps.get(1).unwrap().as_str();
            return Ok(format!("https://{}/{}", host, slug));
        }
    }
    Err(Error::TryNextHost)
}

// TODO: Support bitbucket pages and gitlab pages
fn extract_project_url(text: &str) -> Result<String> {
    let pat = r"\bhttps://([[:alnum:]_-]+)\.github\.io(?:/([[:alnum:]_.-]+))?\b";
    match Regex::new(pat)?.captures(text) {
        None => Err(Error::RepoUrlNotFound(text.to_string())),
        Some(caps) => {
            let user = caps.get(1).unwrap().as_str();
            if let Some(proj) = caps.get(2) {
                Ok(format!("https://github.com/{}/{}", user, proj.as_str()))
            } else {
                Ok(format!("https://github.com/{u}/{u}.github.io", u = user))
            }
        }
    }
}

fn extract_any_service_url(text: &str) -> Result<String> {
    for host in SERVICE_HOSTS.iter() {
        let r = extract_service_url(text, host);
        if r != Err(Error::TryNextHost) {
            return r;
        }
    }
    extract_project_url(text)
}

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

fn open_in_browser(url: &str) -> Result<()> {
    if let Ok(cmd) = env::var("EXTRACT_REPO_URL_OPEN_CMD") {
        Command::new(cmd).arg(url).status()?;
        return Ok(());
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

fn main() -> Result<()> {
    let (text, action) = parse_argv()?;
    let url = extract_any_service_url(text.as_str())?;
    match action {
        Action::Print => println!("{}", url),
        Action::Open => open_in_browser(url.as_str())?,
    }
    Ok(())
}
