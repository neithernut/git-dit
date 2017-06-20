// git-dit - the distributed issue tracker for git
// Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 2 as
// published by the Free Software Foundation.
//

error_chain! {
    foreign_links {
        GitError(::git2::Error);
        GitDitError(::libgitdit::error::Error);
    }

    errors {
        CannotOpenRepository {
            description("Cannot open repository")
            display("Cannot open repository")
        }

        CannotGetRepositoryConfig {
            description("Cannot get repository config")
            display("Cannot get repository config")
        }

        CannotGetSignature {
            description("Cannot get signature")
            display("Cannot get signature")
        }

        ParameterMissing(name: String) {
            description("Missing parameter")
            display("Missing parameter: {}", name)
        }

        WrappedIOError {
            description("IO Error")
            display("IO Error")
        }

        WrappedParseError {
            description("Parser Error")
            display("Error while parsing")
        }

        WrappedGitDitError {
            description("Error in libgitdit")
            display("Error in libgitdit")
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
