# Libgitdit library internals

The purpose of the library is providing an interface for managing git-dit
issues, associated messages and metadata.
Naturally, this is done via types representing different aspects which are
grouped in modules.


## Significant modules

 * `repository` provides the `RepositoryExt` extension trait, which is the main
   facade through which users may create, find and access issues.

 * `issue` provides the `Issue` type which represents an issue and provides
   interfaces for creating and accessing the issue's messages.

 * `message` provides the `Message` trait as well as line- and block-oriented
   iterators for processing a single git-dit message.

 * `trailer` provides the `Trailer` type for representing trailers as well as
   interfaces for specifying, accumulating and matching trailers.

 * `gc` provides utilities which may be used for garbage collection in git-dit
   environment.

 * `iter` provides various iterators for stream-processing, most notably the
   `Messages` iterator.

 * `error` provides all error types for the library.

 * `utils` provides strictly library internal utilities, which will never be
   exported.

 * `test_utils` provides library internal testing utilities.


## Dependencies

We impose a number of restrictions on inter-module dependencies.

 * `utils`, `test_utils` and `error` shall not depend on any other module of the
   library.

 * Generally, sub-modules shall not depend on siblings of its super-module.
   For example, `trailers::accumulation` shall not depend on `repository`.
   The aforementioned `utils`, `test_utils` `error` modules are excepted from
   this rule. Any module or submodule may depend on them.

 * However, the `message` module and associated sub-modules may depend on
   `trailer` and its submodules.

