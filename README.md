git-dit - the distributed issue tracker for git

**WARNING: This is pre-1.0! Expect bugs and incompatibilities!**

However, we try to avoid breaking changes.

---

# git-dit features

* Distributed issue tracking, without checking files into the repository
* Convenient commandline interface (plumbing + porcelain)
* Implemented as `git` subcommand: `git dit`
* Each command has a `--help`
* No structured data 
  * An issue/comment is a commit
  * "Tags" are supported (see `man git-interpret-trailers`)
* Convenience `git dit push` and `git dit pull`
* No additional software needed on the server-side.

When playing with this, please keep in mind that this is alpha quality - there
are bugs, missing things and rough edges.

# Dependencies

The following crates are used:
* chrono 0.3
* error-chain 0.10
* git2 0.6
* is-match 0.1
* log 0.3

Additionally, for building the man page, `pandoc` is required.

# Installing

Cargo is used for building git-dit. Run

    cargo build

in this directory in order to build `git-dit`. Building the `git-dit` man page
is enabled through the "manpage" feature of the Cargo package. E.g. run

    cargo build --features manpage

instead.

We do not provide any installation scripts. If you intent using or testing
`git-dit`, make sure to have the binary in your `PATH`.

# Documentation

For a system overview and conceptual information, refer to the
[documentation](doc/README.md). For a more practical documentation, refer to the
[man page](git-dit.1.md).

# License

The [library module](./lib) is licensed under terms of [MPL-2.0](./lib/LICENSE).
The binary module (this directory) uses the library and provides a commandline
interface for it and is licensed under terms of [GNU GPLv2](./LICENSE).

(c) Julian Ganz, Matthias Beyer
