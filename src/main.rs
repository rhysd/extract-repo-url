extern crate clap;
extern crate clipboard;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod error;

use error::Error;
use error::Result;

mod extract;

mod open;

use clap::{App, Arg};
use clipboard::{ClipboardContext, ClipboardProvider};
use std::io;

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
                .help("Read text from stdin"),
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

fn main() -> Result<()> {
    let (text, action) = parse_argv()?;
    let url = extract::extract_any_service_url(text.as_str())?;
    match action {
        Action::Print => println!("{}", url),
        Action::Open => open::open_in_browser(url.as_str())?,
    }
    Ok(())
}
