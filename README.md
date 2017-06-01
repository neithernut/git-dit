git-dit - the distributed issue tracker for git

**WARNING: This is pre-1.0! Expect bugs and incompatibilities!**

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

# Installing

_TBD_

# Documentation

For a system overview and conceptual information, refer to the
[documentation](doc/README.md). For a more practical documentation, refer to the
[man page](git-dit.1.md).

# License

The [library module](./lib) is licensed under terms of [MPL-2.0](./lib/LICENSE).
The binary module (this directory) uses the library and provides a commandline
interface for it and is licensed under terms of [GNU GPLv2](./LICENSE).

(c) Julian Ganz, Matthias Beyer
