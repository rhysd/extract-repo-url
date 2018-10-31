use crate::error::{Error, Result};
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
                hosts.push(host.trim().to_string());
            }
        }
        hosts
    };
}

fn extract_service_url(text: &str, host: &str) -> Result<String> {
    let escaped = regex::escape(host);
    for pat in &[
        format!(r"\bgit@{}:([[:alnum:]_-]+/[[:alnum:]_.-]+)\b", escaped),
        format!(
            r"\b(?:https://){}/([[:alnum:]_-]+/[[:alnum:]_.-]+)\b",
            escaped
        ),
    ] {
        if let Some(caps) = Regex::new(pat)?.captures(text) {
            let mut slug = caps.get(1).unwrap().as_str();

            // Omit trailing .git file extension on clone URL
            if slug.ends_with(".git") {
                slug = &slug[..slug.len() - 4];
            }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::*;
    use std::io;
    use std::path::*;

    macro_rules! testcase {
        ($name:ident, $inputs:expr, $expected:expr,) => {
            #[test]
            fn $name() {
                for input in $inputs.iter() {
                    let result = extract_any_service_url(input);
                    assert_eq!(result, Ok($expected.to_string()));
                }
            }
        };
    }

    testcase!(
        github_in_text,
        [
            "https://github.com/foo/bar",
            "This is text https://github.com/foo/bar",
            "https://github.com/foo/bar is great",
            "oh, https://github.com/foo/bar?",
            "https://github.com/foo/bar/blob/master/tests/tests.rs"
        ],
        "https://github.com/foo/bar",
    );

    testcase!(
        dash_dot_underscore,
        [
            "https://github.com/dash-included/-some-awesome_repo.rs_",
            "This is text https://github.com/dash-included/-some-awesome_repo.rs_",
            "https://github.com/dash-included/-some-awesome_repo.rs_ is great",
            "oh, https://github.com/dash-included/-some-awesome_repo.rs_?",
        ],
        "https://github.com/dash-included/-some-awesome_repo.rs_",
    );

    testcase!(
        clone_urls,
        [
            "https://github.com/foo/bar.git",
            "git clone https://github.com/foo/bar.git",
            "git@github.com:foo/bar.git",
            "git clone git@github.com:foo/bar.git",
        ],
        "https://github.com/foo/bar",
    );

    testcase!(
        clone_url_edge_case,
        [
            "https://github.com/foo/bar.git.git",
            "git@github.com:foo/bar.git.git",
        ],
        "https://github.com/foo/bar.git",
    );

    testcase!(
        github_pages,
        [
            "https://foo-bar.github.io/proj-ect",
            "https://foo-bar.github.io/proj-ect/",
        ],
        "https://github.com/foo-bar/proj-ect",
    );

    testcase!(
        personal_github_pages,
        ["https://foo-bar.github.io", "https://foo-bar.github.io/"],
        "https://github.com/foo-bar/foo-bar.github.io",
    );

    testcase!(
        bitbucket_in_text,
        [
            "https://bitbucket.org/foo/bar",
            "This is text https://bitbucket.org/foo/bar",
            "https://bitbucket.org/foo/bar is great",
            "oh, https://bitbucket.org/foo/bar?",
        ],
        "https://bitbucket.org/foo/bar",
    );

    testcase!(
        gitlab_in_text,
        [
            "https://gitlab.com/foo/bar",
            "This is text https://gitlab.com/foo/bar",
            "https://gitlab.com/foo/bar is great",
            "oh, https://gitlab.com/foo/bar?",
        ],
        "https://gitlab.com/foo/bar",
    );

    #[test]
    fn additional_hosts() {
        struct EnvGuard(String);

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                env::set_var("EXTRACT_REPO_URL_SERVICE_HOSTS", self.0.clone())
            }
        }

        EnvGuard(env::var("EXTRACT_REPO_URL_SERVICE_HOSTS").unwrap_or("".to_string()));

        env::set_var(
            "EXTRACT_REPO_URL_SERVICE_HOSTS",
            "github.example.com, github2.example.com",
        );

        for text in &[
            "https://github.example.com/foo/bar",
            "https://github.example.com/foo/bar is awesome",
            "oh, https://github.example.com/foo/bar?",
            "git clone https://github.example.com/foo/bar.git",
            "git clone git@github.example.com:foo/bar.git",
        ] {
            assert_eq!(
                extract_any_service_url(text),
                Ok("https://github.example.com/foo/bar".to_string()),
                "{}",
                text
            );
        }

        for text in &[
            "https://github2.example.com/foo/bar",
            "https://github2.example.com/foo/bar is awesome",
            "oh, https://github2.example.com/foo/bar?",
            "git clone https://github2.example.com/foo/bar.git",
            "git clone git@github2.example.com:foo/bar.git",
        ] {
            assert_eq!(
                extract_any_service_url(text),
                Ok("https://github2.example.com/foo/bar".to_string()),
            );
        }

        // Check regex is escaped
        assert!(extract_any_service_url("https://githubxexampleycom/foo/bar").is_err());
    }

    #[test]
    fn error_cases() {
        for text in &[
            "",
            "hey",
            "https://github.com",
            "https://github.com/foo",
            "git@github.com",
            "git@github.com/foo/bar",
            "git@github.com:foo.git",
        ] {
            let ret = extract_any_service_url(text);
            assert!(ret.is_err(), "Unexpected success: {:?}", ret);
        }
    }

    #[test]
    fn real_world() {
        use crate::io::BufRead;

        if let Ok(var) = env::var("RUN_SHORT_TEST") {
            if var != "" {
                // Skip on short test
                return;
            }
        }

        let mut path = env::current_dir().unwrap();
        path.push(Path::new(file!()).parent().unwrap());
        path.push("testdata");
        path.push("top1000-repos.txt");

        for slug in io::BufReader::new(File::open(path).unwrap()).lines() {
            let slug = slug.unwrap();
            let url = format!("https://github.com/{}", slug);
            for text in &[
                url.clone(),
                format!("hello, {} world", url),
                format!("oh, {}!?", url),
                format!("{}/tree/master/tests/data", url),
            ] {
                assert_eq!(extract_any_service_url(text), Ok(url.clone()));
            }

            let url = format!("git@github.com:{}", slug);
            for text in &[
                url.clone(),
                format!("git clone {}", url),
                format!("oh, {}!?", url),
            ] {
                assert_eq!(
                    extract_any_service_url(text),
                    Ok(format!("https://github.com/{}", slug))
                );
            }
        }
    }
} // mod tests
