# git-dit ChangeLog

## Next

Changes:
  * Some traits in the library were refactored to not be generic over their
    functions return types, but rather use associated types for that.

## v0.4.0 (2017-09-15)

### Binary

Changes:
 * The message formatting for the "show" subcommand was improved.
 * In filter-specs for the "list" subcommand, a '=' is used for separating key
   and value when filtering for an exact value, rather than a ':'.

Added features:
 * The "create-message", "new", "reply" and "tag" now accept the "--author" and
   "--date" options. Analogue to the options of "git commit" with the same name,
   the new options alter the author and authoring date of the resulting message.
 * New "reporter-name" and "reporter-email" filter keys for issue filtering
   using the "list" subcommand. As the name suggests, the keys may be used for
   filtering issues based on the reporter's name or email.
 * Filter specs for the "list" subcommand can now be used for filtering by
   substring or presence of a piece of issue metadata.
 * Filter specs for the "list" subcommand can now be inverted.

Bug-fixes:
 * Git-dit did not handle pager and editor flags correctly. For example,
   configuring `less -R` as the pager (contrary to `less`) would cause git-dit
   search for a binary named "less -R", resulting in an error.

### Library

Changes:
 * `LineIteratorExt::stripped()` now returns an iterator which also strips
   trailing blank lines.
 * `Issue::new()` now returns a `Result`.
 * Multi-line trailers parsed by the `Blocks` iterator are represented as String
   `TrailerValue` with newlines separating the individual lines from the input.
 * `trailer` was promoted to a top-level module.
 * `message::accumulation` was moved to `trailer`
 * `message::metadata` was moved to `trailer` and renamed to `spec`.
 * `MetadataSpecification` was renames to `TrailerSpec`.
 * `PairsToTrailers` and `DitTrailers` iterator was moved to new
   `trailers::iter` module.
 * `Trailers` iterator was moved to `message::block` module.
 * `DitTrailers` is now a generic iterator adapter on top of an iterator over
   `Trailer`s.

Added features:
 * New `TrailingBlankTrimmer` for removing trailing blank lines from sequences
   of lines.
 * New `StrippingIter` for stripping messages of comments and unnecessary white
   space.
 * New `TrailerFilter` type for filtering based on trailers.
 * New `ValueMatcher` type for matching `TrailerValue`s.
 * New `MessagesExt` convenience trait for iterators over messages. The trait
   provides accumulation functionality.
 * New `MultiAccumulator` trait for map-based accumulation of multiple trailers.
 * Implemented `Default` for `TrailerValue`.
 * Implemented `Clone` for `ValueAccumulator`.
 * Implemented `From<Blocks>` for `Trailers`

### Documentation
 * Add documentation of internals as a `README.md` to the sources of both the
   binary and the library.


## v0.3.0 (2017-08-13)

### Binary

Changes:
 * "tag" now doesn't require a local head reference to be present in advance.
 * "check-message" does not require a repository to be present any more.

Added features:
 * New git-config option "dit.remote-prios" controlling how remote references
   are chosen for various actions.
 * New "mirror" subcommand for mirroring remote references as local ones.
 * New "gc" subcommand for collecting old references.
 * New "check-refname" subcommand for extracting information from reference
   names.
 * The "list" subcommand now supports a small set simple filters for filtering
   issues based on metadata.
 * "get-issue-metadata" can now be used for querying a single piece of metadata.
 * Example server-side update hook for a dit-only policy for repositories with
   global unauthenticated push access.

Bug-fixes:
 * "fetch" and "push" tried fetching/pushing issues multiple times under some
   circumstances.
 * "list" and "get-issue-tree-init-hashes" reported issues multiple times under
   some circumstances.

### Library

Changes:
 * `RepositoryExt::issues_with_prefix()` and `RepositoryExt::issues()` now
   return a unique set of issues.
 * `Issue::update_head()` now takes an additional parameter controlling whether
   an existing head reference is replaced.
 * `Issue::messages_revwalk()` was replaced by new `Issue::messages()` function.
 * `Issue::first_parent_revwalk()` was replaced by new
   `Issue::first_parent_messages()` function.
 * Function `Issue::find_local_head()` was renamed to `Issue::local_head()`.
 * `Issue::local_refs()` now has an additional parameter controlling the type of
   references returned.
 * `Issue::issue_leaves()` was removed.
 * `message::line` module was replaces by `message::block` module, including
   associated types.
 * `LineIteratorExt::categorized_lines()` was replaced by
   `LineIteratorExt::line_blocks()`.
 * `Message::categorized_body()` was replaced by `Message::body_blocks()`.
 * `TrailerValue::append()` now operates in-place, on a mutable reference.

Added features:
 * New type `Messages` for iterating over messages.
 * New function `Issue::terminated_messages()` for preparing a `Messages`
   instance which terminates at the initial message.
 * New function `Issue::messages_from()` for creating a `Messages` instance
   returning messages from one specific commit to the initial message.
 * New module `message::accumulation` for accumulating issue metadata.
 * New module `message::metadata` providing specification of a small set of
   predefined pieces of issue metadata
 * New type `IssueRefType` representing the type of a reference (e.g. "head" or
   "leaf").
 * New `PairsToTrailers` iterator for assembling trailers from key-value pairs.
 * New `Issue::remote_refs()` and `Issue::all_refs()` functions for retrieving
   references.
 * Implemented `PartialEq`, `Eq` and `Hash` for `Issue`.
 * Implemented `From<Messages>` for `IssueMessagesIter`.
 * Implemented `AsRef<String>` for `TrailerKey`

Bug-fixes:
 * Fixed lifetimes of return values of several `Issue` functions.

### Documentation
 * Building the manpage is now controlled via the `manpage` feature rather than
   an environment variable.
 * The subcommand descriptions are now longer set as code snippets.
 * Various small fixes and updates.


## v0.2.1 (2017-07-16)

Changes:
 * Add WORKFLOWS section to the man page
 * Clarify implementation of metadata "tags" in the documentation
 * Clarify membership of initial issue messages to branches in man page
 * Fix punktuation issues in the man page

Bug-fixes:
 * Fix bug in `Trailers` iterator which caused emission of trailers even if they
   are embedded in a block of text.


## v0.2 (2017-06-22)

Changes:
 * Rewrite in Rust (single binary)
 * Drop "prepare-metadata" script
 * "push" and "fetch" now only support dummy auth and ssh-agent
 * Updated README
 * Updated man page

Added features:
 * Documentation of data model, semantics and use-cases
 * Library crate "libgitdit"
 * Long options

Bug-fixes:
 * Read editor from git config, fall back to default
   (We previously relied on the `EDITOR` variable.)


## v0.1 (2017-02-01)

Initial prototype, written in bash.

