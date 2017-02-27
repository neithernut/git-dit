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
which inhibit the same restriction. Note that we currently still lack an tool
for convenient amending of messages.

