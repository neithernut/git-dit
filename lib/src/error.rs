// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use git2::Oid;

error_chain! {
    foreign_links {
        GitError(::git2::Error);
    }

    errors {
        WrappedGitError {
            description("TODO: Wrapped error")
            display("TODO: Wrapped error")
        }

        CannotFindIssueHead(id: Oid) {
            description("Cannot find issue HEAD")
            display("Cannot find issue HEAD for {}", id)
        }

        NoTreeInitFound(id: Oid) {
            description("Cannot find any tree init")
            display("Cannot find any tree init for {}", id)
        }

        OidFormatError(name: String) {
            description("Malformed HEAD OID")
            display("Malformed OID: {}", name)
        }

        MalFormedHeadReference(name: String) {
            description("Found malformed HEAD reference")
            display("Malformed head refernece: {}", name)
        }

        EmptyMessage {
            description("An empty message was supplied")
            display("The message is empty")
        }

        EmptySubject {
            description("The subject line of the message is empty")
            display("Empty subject line")
        }

        MalformedMessage {
            description("The message supplied is malformed")
            display("The message supplied is malformed")
        }
    }
}
