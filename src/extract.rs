use error::{Error, Result};
use regex;
use regex::Regex;
use std::env;

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

pub fn extract_any_service_url(text: &str) -> Result<String> {
    for host in SERVICE_HOSTS.iter() {
        let r = extract_service_url(text, host);
        if r != Err(Error::TryNextHost) {
            return r;
        }
    }
    extract_project_url(text)
}
