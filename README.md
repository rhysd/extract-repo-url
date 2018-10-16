`extract-repo-url`
==================
[![Crates.io][crates-io badge]][repository]

## Installation

```
cargo install extract-repo-url
```

## Basic Usage

Outputs a repository URL extracted from clipboard text.

```
$ extract-repo-url # From clipboard by default
https://github.com/foo/bar

$ extract-repo-url 'Repository https://github.com/foo/bar is awesome' # Can receive text as 1st param
https://github.com/foo/bar
```

It can also detect

- Clone URL (e.g. `git@github.com:foo/bar.git` -> `https://github.com/foo/bar`)
- File page on github (e.g. `https://foo.github.io/bar/tree/master/tests/data` -> `https://github.com/foo/bar`)
- GitHub pages (e.g. `https://foo.github.io/bar/` -> `https://github.com/foo/bar`)
- Bitbucket (e.g. `foo bar https://bitbucket.org/foo/bar` -> `https://bitbucket.org/foo/bar`)
- GitLab (e.g. `foo bar https://gitlab.com/foo/bar` -> `https://gitlab.com/foo/bar`)

You can add more hosts with `$EXTRACT_REPO_URL_SERVICE_HOSTS` environment variable.

```sh
export EXTRACT_REPO_URL_SERVICE_HOSTS="github.your-site.com, gitlab.your-site.com"
```

Please try `--help` option to see more details.


## Use-cases

With Git:

```
$ git clone `extract-repo-url`
```

With `open` command (on macOS):

```
$ open `extract-repo-url`
```


## Development

On Linux, you need to install X11 library:

```
sudo apt-get install xorg-dev
```

How to run test:

```sh
$ RUST_TEST_THREADS=1 cargo test
```


## License

[MIT](./LICENSE.txt)


[repository]: https://github.com/rhysd/extract-repo-url
[crates-io badge]: https://img.shields.io/crates/v/extract-repo-url.svg
