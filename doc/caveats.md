# Caveats

Git-dit is, obviously, git specific and will hence not work with other VCS.
Import and export functionality for other issue tracking systems are, however,
not inconceivable. Functionality like referencing issues across repositories
will, however, most likely never be provided across different VCS.

Git-commits are immutable. Hence, a message can not be edited once it is
published. Commits/messages may, however, be "amended". This operation is
equivalent to creating a new commit with the edited content and discarding the
old one. Hence, if a reply already exists on the old commit, it will not be
removed and the reply will still refer to the old commit. However, we do not
consider this restriction sever, especially when compared to mailing-lists,
which inhibit the same restriction. Note that we currently still lack a tool
for convenient amending of messages.


## Issue notification/access control

Issues and related messages have to be pulled by the maintainer from public
repositories. Otherwise, issue reporters would require push access to the bug
repository. Originally, we planned (and still do plan) to provide a tool for
automated imports of issues and patches from mailing lists. A bug repository
may also be accessed via a web front-end at some point.

For another project, we also consider a notification mechanism for
cross-platform notification of events in git repositories (e.g. pushes).
Using such a mechanism, contributors could notify a project maintainer about
changes in their public repositories, including issues or messages, via a public
API. A maintainer could set up automated fetches of new issues, importing new
issues and messages. However, this is unrelated and still work in progress.
Additionally, it would still require reporters of issues to expose them via a
public repository.

It was also suggested that maintainers or service providers could provide
special public bug repositories with public push access. Those repositories
would have specially crafted hooks installed, implementing some sort of access
control, e.g. preventing issues from being deleted by unauthorized actors.

