git-dit - the distributed issue tracker for git

This repository contains the "git-dit" project, a distributed issue tracking
system using git.

---

**WARNING: This is a proof of concept. This shows how we think distributed issue
tracking in git should work. Expect bugs and incompatibilities!**

---

git-dit is a distributed issue tracker in/for git, currently implemented as
proof-of-concept in Bash.
If you want to play with it, make sure you use current git versions.
It differs from things like bugseverywhere and fossil in that it is a
distributed issue tracker for git only, using git features to implement issue
tracking in a way so that merging of issues, attaching issues to commits,
creating PRs, etc is possible.
It does explicitely _not_ store any "structured data" like JSON, YAML or such,
but simply uses git commit messages for issue messages.
So, E-Mail workflows, github, gitlab and other hosting platforms and their issue
tracking schema can be adapted and mirrored into "git-dit", technically.
We are not there yet, though. We are planning to reimplement the current
featureset in a more robust language.

When playing with this, please keep in mind that this is a POC - there are bugs,
missing things and rough edges. Do not use on a production repository!

---

We developed this set of tools using the latest git version on Gentoo and NixOS,
which is, as of 2017-02-24, git version `2.11.1`.

---

Licensed under the terms of GNU GPLv2.
For more information, see the LICENSE file.

(c) Julian Ganz, Matthias Beyer
