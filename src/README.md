# git-dit binary internals

The purpose of the binary is providing a command line interface for managing
git-dit issues, associated messages and metadata.
Naturally, this is done via types representing different aspects which are
grouped in modules.


## Significant modules

 * `display` provides formatting utilities for various items.
 * `filters` provides issue filtering facilities.
 * `gitext` provides some extensions to the `git2` library which are relevant
   (only) for this application.
 * `system` provides I/O utilities as well as utilities for spawning specific
   programs based on configuration and the logger.
 * `util` provides application specific utilities, e.g. retrieving specific
   command line arguments or configuration variables.
 * `error` provides error types.


## Dependencies

We impose a number of restrictions on inter-module dependencies.

 * `error` shall not depend on any other module of the library.

 * The `system` module shall not depend on `libgitdit`.

 * Generally, sub-modules shall not depend on siblings of its super-module.
   For example, `formatter::msgtree` shall not depend on `util`.
   However, any module or submodule may depend on `error`.


## Error handling

 * Functions of the root module as well as functions in `util` and `filter` may
   abort rather than return a `Result`. For the root and `util` modules, in
   fact, aborting should be preferred to returning a `Result`.

 * All other modules shall not abort or panic. The `system::abort` module is
   the only exception since it provides the aborting functionality.

