% GIT-DIT(1) User Manuals
% Matthias Beyer, Julian Ganz
% June 22, 2017

# NAME

git-dit - the distributed issue tracker for git


# SYNOPSIS

git [git options] dit \<command\> [\<args\>]


# DESCRIPTION

Git-dit is a distributed issue tracking system based on the git revision
control system.
Following some of the design principles of git, it offers both high level
commands for end users as well as a low level interface to internals for use in
scripts or third-party extensions.

For more information, refer to the DISCUSSION section of this manual.
Additionally, the WORKFLOWS section provides some examples of git-dit usage.


# OPTIONS

There are no git-dit specific global options, contrary to sub-commands of
git-dit.
The `-h` short option usually prints a short help message listing all options
and command line arguments accepted by a specific sub-command.


# GIT-DIT COMMANDS

Like git, the git-dit suite is divided into high level ("porcelain") commands
and low level ("plumbing") commands.


# HIGH LEVEL COMMANDS (PORCELAIN)

## git-dit-new
    Add a new issue.

## git-dit-reply
    Reply to an existing issue with a new message.

## git-dit-show
    Show the contents (messages) of an issue.

## git-dit-list
    List all issues known to git-dit in the current directory.

## git-dit-tag
    Show or modify meta-data of issues.

## git-dit-fetch
    Fetch issues from a remote repository.

## git-dit-push
    Push issues to a remote repository.


# LOW LEVEL COMMANDS (PLUMBING)

## git-dit-check-message
    Check whether the format of an issue message is valid.

## git-dit-create-message
    Create a bare message.

## git-dit-find-tree-init-hash
    Find the issue hash for a message's hash.

## git-dit-get-issue-tree-init-hashes
    List all known issue hashes.

## git-dit-get-issue-metadata
    Extract meta-data from a thread in an issue.


# DISCUSSION

Git-dit is modeled after classical mailing lists.
Issue messages, which are stored as commits in the repository, correspond to
Emails.
Each issue can be thought of as a discussion thread on a mailing list, whether
it be a bug-report, a feature request or a patch-set to be merged into some
branch.

The command `git-dit-new` creates a new issue, or discussion thread.
It can be either based on an existing commit, or as a new initial commit.
The latter one is most appropriate for feature requests and bug-reports, while
the former one is especially useful for patch-sets.
Discussion messages will be created using `git-dit-reply` as empty commits,
preserving the parent commit's tree.


# WORKFLOWS

Git-dit tries not to force a specific work-flow on its users.
Various workflows are supported either directly or indirectly.
This section discusses some possible workflows for managing issues using
git-dit.

## General setup considerations

Project hosting sites often provide exactly one logical issue tracker for each
project repository.
As with mailing lists and stand-alone issue tracking web-services, users of
git-dit are not forced to bind a set of issues to a specific source code
repository.
While Git-dit-issues do live in a git repository, it doesn't necessarily need to
be the same repository used for the source code.
For example, maintainers may choose to use separate repositories for code and
issues.

Consider, for example, a big project with several code repositories for
different components.
Now consider a user who wants to file a bug.
In many cases, the user will not be able to correctly assign the bug to a
specific component.
With git-dit, maintainers may provide a dedicated issue repository for filing
bug reports and feature requests.

Contributors and sub-maintainers may add the central issue repository as an
additional remote to their local clone.
Optionally, issues assigned to a component may be transferred or copied to a
component's code repository or an associated specialized issue repository
(though there is no convenient support for such functionality, yet).

It is also possible to have multiple issue repositories for a single code
repository.
Consider for example one of these closed source projects with many change
requests from difficult customers.
With git-dit, you can set up a dedicated issue repository for each customer in
order to accept and share change requests.
Developers can add each of those issue repositories as remotes, viewing and
interacting with all of the issues from different customers.

## Retrieving issues from a remote repository

Naturally, contributors as well as maintainers will want to retrieve issues from
a remote repository, be it a dedicated issue repository or a repository
containing both issues and source code.
Git-dit offers multiple ways for retrieving issues.

Issues may be fetched manually from a remote repository using the `fetch`
subcommand.

    git dit fetch issue-repo

will fetch all issues from a remote "issue-repo". The subcommand also supports
fetching only updates of issues explicitly specified or all issues which are
known to the current repository.

Alternatively, a developer may choose to subscribe to the issues present in a
remote repository.
Currently, this has to be done manually through a ref-spec.
The following refspec, for example, will cause the issues to be fetched from the
remote "origin" on each `git fetch`.

    refs/dit/*:refs/remotes/origin/dit/*

New issues and issue updates are pushed to a remote using git-dit's "push"
subcommand.

## Reporting an issue

Issues can be created in the local repository.

    git dit new

will spawn the editor configured for git.
The usual rules for commit messages also apply to issues: the first line will be
the subject line and should contain a summary or appropriate title; the second
line should be blank, followed by paragraphs of texts or blocks of trailers.
The trailers may be used for transporting metadata.
Once the message is written and the editor is closed, a new issue will be
created and git-dit will print the issue's id.

For others to see the issue, the issue has to be pushed to a public repository.
The command

    git dit push origin <id>

will push the issue with the id provided by the user to the remote "origin".
Alternatively, all local updates, including new issues, may be pushed to the
remote "origin" using the command

    git dit push origin

## Viewing issues

TBD

## Adding information and metadata to an issue

TBD

## Managing the state and other metadata of an issue

TBD


# SEE ALSO

