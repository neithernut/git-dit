// git-dit - the distributed issue tracker for git
// Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 2 as
// published by the Free Software Foundation.
//

use std::error::Error as EError;


error_chain! {
    foreign_links {
        GitError(::git2::Error);
        GitDitError(::libgitdit::error::Error);
    }

    errors {
        MalformedFilterSpec(spec: String) {
            description("Malformed filter spec")
            display("Malformed filter spec: {}", spec)
        }

        UnknownMetadataKey(key: String) {
            description("Unknown metadata key")
            display("Unknown metadata key: {}", key)
        }

        WrappedIOError {
            description("IO Error")
            display("IO Error")
        }

        ProgramError(program_name: String) {
            description("Could not find some configuration or ENV variable specifying a program")
            display("Could not find {} configuration or ENV variable", program_name)
        }

        ChildError {
            description("A child program was unsuccessful")
            display("A child program was unsuccessful")
        }
    }
}


/// Convenience trait for logging error types
///
/// Logs all layers of an error using the `error!` macro.
///
pub trait LoggableError {
    fn log(&self);
}

impl<E> LoggableError for E
    where E: EError
{
    fn log(&self) {
        let mut current = Some(self as &EError);
        while let Some(err) = current {
            error!("{}", err);
            current = err.cause();
        }
    }
}

