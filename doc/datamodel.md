# Data model

Git-dit stores issues and associated data directly in commits rather than
in blobs within the tree. Similar to threads in a mailing list, issues and
comments are modeled as a tree of messages. Each message is stored in one
commit.


## Message tree

An issue is identified by the hash of its initial commit. For each issue, a
"head" references `refs/dit/<issue-hash>/head` is created. Other message refer
to the replied message via their first parent. Other parents may be used to
refer to other commits, e.g. in bug-reports or cross-references to messages in
other issues.

The message trees are spanned via "leaf" references which are maintained in
`refs/dit/<issue-hash>/leaves/` for each issue. Leaf references which are not
required any more for preventing garbage collection of messages may be removed.
Git-dit will feature a garbage collector for removing unnecessary references at
some point.

The aforementioned head reference of an issue may be used by maintainers to mark
an agreed accepted state of the discussion or the status of an issue (as the
status is also altered through commit messages). Metadata is collected for an
issue by iterating over the commits from an issue's head reference to its
initial message via the first parent of each commit and accumulating the
metadata from each message.

A maintainer may update the head reference to a specific point in the
discussion. However, she may also choose to maintain an independent sequence of
status changes, referring to messages in the discussion through its second
parent. At this point, we do not yet provide tools for managing the head
reference of an issue.

Since the initial message of an issue can be identified by the presence of an
associated head reference, it can safely refer to arbitrary commits as parents.
For example, a bug report may refer to a specific commit in which a bug was
observed. A patch set, on the other hand, clearly should be rooted in the
project's history. Git-dit tools shall hence never assume that an issue's
initial message has no parents.


## Message structure

As messages are stored as commit messages, the message has to adhere to the
usual rules for commit messages: it must consist of at least a subject line and
the second line of a message shall be empty. Issues should always have a message
body providing some details.

A message may contain metadata "tags" in the form of trailers. The following
tags are currently used:

 * Dit-status
 * Dit-type

Additional tags, as well as a more elaborate explanation of the tags, may be
provided in the future.

