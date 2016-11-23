#!/usr/bin/env bash
#
#   git-dit - the distributed issue tracker for git
#   Copyright (C) 2016 Matthias Beyer <mail@beyermatthias.de>
#   Copyright (C) 2016 Julian Ganz <neither@nut.email>
#
#   This program is free software; you can redistribute it and/or modify
#   it under the terms of the GNU General Public License version 2 as
#   published by the Free Software Foundation.
#
# -----
#
# $(basename $0) <parent-hash> <tree-init-hash>
#
#   <parent-hash>       Create a comment on this comment
#   <tree-init-hash>    The hash of the initial commit in this issue tree
#
# If <parent-hash> and <tree-init-hash> are not provided, we create a new issue.
#
# If <parent-hash> is provided and <tree-init-hash> is not, we create a new
# issue which is attached to an existing commit. That can be used (for example)
# to attach a bug report to a commit.
#
# If both are provided, we create a reply to an existing issue discussion.
#
# Additional arguments are ignored
#
# Returns (prints) the hash of the new commit
#
# (c) 2016 Matthias Beyer, Julian Ganz

abort() {
    echo "$*" >&2
    exit 1
}

commit_exists_or_abort() {
    git rev-parse --quiet --verify $1^{commit} 2>/dev/null >/dev/null || \
        abort "Not a commit: $1"
}

PARENT="$1"
TREE_INIT_HASH="$2"

create_new_head() {
    git update-ref refs/dit/$1/head $1
    [[ ! $? ]] && abort "Failed to update reference: $1"
}

if [[ -z "$PARENT" ]]; then
    empty_tree=$(git hash-object -t tree /dev/null)
    [[ ! $? ]] && abort "Failed to get hash of empty tree"

    # Create new detached issue thread
    new_commit_hash=$(git commit-tree "$empty_tree")
    [[ ! $? ]] && abort "Failed to commit tree with message"

    create_new_head "$new_commit_hash"

else
    # Check whether $PARENT exists
    commit_exists_or_abort "$PARENT"

    new_commit_hash=$(git commit-tree -p "$PARENT" "$PARENT:")
    [[ ! $? ]] && abort "Failed to commit tree with message"

    if [[ -z "$TREE_INIT_HASH" ]];
    then
        # New attached issue thread
        create_new_head "$new_commit_hash"

    else
        # Reply
        git update-ref refs/dit/$TREE_INIT_HASH/leaves/$new_commit_hash $new_commit_hash
        [[ ! $? ]] && abort "Failed to update reference: $new_commit_hash"

    fi

fi

echo "$new_commit_hash"

