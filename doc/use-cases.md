# Use-cases

A vast variety of use-cases can be covered using git-dit.

As messages are stored in commit messages rather than in structured data, there
is no requirement for merging data. Instead, a maintainer adopts a status update
or stance by simply updating the public "upstream" head reference. Hence, issues
and replies can be fetched from arbitrary repositories without conflicts.

Contrary to regular branches, it is not necessary to duplicate most remote
references in the local repository the same way as with branches. Leaf
references, for example, are not advanced but new ones are generated as needed.
The head reference of an issue is an exception, as it is updated, e.g. by the
maintainer.


## Bug repositories

With git-dit, issues can spread across multiple repositories and be,
conceptually, easily transferred. It is hence possible to store issues in a
repository other than the source code repository and still have issues visible
via the remote in a developer's local repo.

For example, bugs encountered in some programs are manifestations of bugs in
libraries the program depends on. Currently, in such cases, a bug is often filed
by hand in the dependency's bug tracker. With git-dit, the issue can be simply
transferred or referenced in the dependency's (bug) repository.

Similarly, an organization or company, for example, may use a single repository
as a bug tracking database to which a certain set of customers (or even just a
single one) has access to. This repository may serve as an exchange point for
issues. Developers can use those locally, combining the issues from various
sources.


## Patch sets

The type of an issue can be changed arbitrarily by the maintainer. Also,
messages from other issues may be referenced in new issues. Hence, when
submitting a patch set resolving an issue, the relevant messages from the
original change request issue can be referenced in the new patch set issue. This
way, the reasons for a change are automatically documented via the commit
history, even after issue references are removed by a maintainer.

