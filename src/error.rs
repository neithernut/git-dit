// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use std::fmt;

use git2::Oid;

error_chain! {
    foreign_links {
        GitError(::git2::Error);
    }

    errors {
        CannotCreateMessage {
            description("Cannot create message")
            display("Cannot create a message")
        }

        CannotConstructRevwalk {
            description("Cannot construct revwalk")
            display("Cannot construct a revwalk for iterating over commits")
        }

        CannotGetCommit {
            description("Cannot get a commit from the repository")
            display("Cannot get a specific commit from repository")
        }

        CannotGetCommitForRev(rev: String) {
            description("Cannot get commit from rev")
            display("Cannot get commit from rev '{}'", rev)
        }

        ReferenceNameError {
            description("Error getting reference name")
            display("Error getting reference name")
        }

        CannotGetReferences(glob: String) {
            description("Cannot get references from repository")
            display("Cannot get references '{}' from repository", glob)
        }

        CannotGetReference {
            description("Cannot get a reference from repository")
            display("Cannot get a specific reference from repository")
        }

        CannotDeleteReference(reference: String) {
            description("Cannot delete a specific reference")
            display("Cannot delete the reference '{}'", reference)
        }

        CannotBuildTree {
            description("Cannot build Tree")
            display("Cannot build Tree")
        }

        CannotFindIssueHead(id: Oid) {
            description("Cannot find issue HEAD")
            display("Cannot find issue HEAD for {}", id)
        }

        CannotSetReference(refname: String) {
            description("Cannot set some reference")
            display("Cannot update or create reference '{}'", refname)
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

        TrailerFormatError(trailer: String) {
            description("Found malformed trailer")
            display("Malformed trailer: {}", trailer)
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


/// Kinds of errors which may be emitted by this library
#[derive(Clone, Debug)]
pub enum Kind<I: InnerError> {
    CannotCreateMessage,
    CannotConstructRevwalk,
    CannotGetCommit,
    CannotGetCommitForRev(String),
    ReferenceNameError,
    CannotGetReferences(String),
    CannotGetReference,
    CannotDeleteReference(I::Reference),
    CannotBuildTree,
    CannotFindIssueHead(I::Oid),
    CannotSetReference(I::Reference),
    NoTreeInitFound(I::Oid),
    OidFormatError(String),
    MalFormedHeadReference(I::Reference),
    TrailerFormatError(String),
    MalformedMessage,
    Other,
}

impl<I: InnerError> fmt::Display for Kind<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotCreateMessage       => write!(f, "cannot create a message"),
            Self::CannotConstructRevwalk    => write!(f, "cannot construct a revision walk"),
            Self::CannotGetCommit           => write!(f, "cannot get a specific commit from repository"),
            Self::CannotGetCommitForRev(r)  => write!(f, "cannot get commit from rev '{}'", r),
            Self::ReferenceNameError        => write!(f, "error getting reference name"),
            Self::CannotGetReferences(g)    => write!(f, "cannot get references '{}' from repository", g),
            Self::CannotGetReference        => write!(f, "cannot get a specific reference from repository"),
            Self::CannotDeleteReference(r)  => write!(f, "cannot delete the reference '{}'", r),
            Self::CannotBuildTree           => write!(f, "cannot build Tree"),
            Self::CannotFindIssueHead(i)    => write!(f, "cannot find issue HEAD for {}", i),
            Self::CannotSetReference(r)     => write!(f, "cannot update or create reference '{}'", r),
            Self::NoTreeInitFound(i)        => write!(f, "cannot find any tree init for {}", i),
            Self::OidFormatError(n)         => write!(f, "malformed OID: {}", n),
            Self::MalFormedHeadReference(n) => write!(f, "malformed head refernece: {}", n),
            Self::TrailerFormatError(t)     => write!(f, "malformed trailer: {}", t),
            Self::MalformedMessage          => write!(f, "malformed message"),
            Self::Other                     => write!(f, "other"),
        }
    }
}


/// [Error](std::error::Error) type specific to a git implementation
///
/// This trait is implemented for [Error](std::error::Error)s we wrap in our own
/// custom [Error]. The trait links that source error type to types we use for
/// representing certain entities with the specific git library in the context
/// of error reporting.
pub trait InnerError: std::error::Error {
    /// Type used for representing Object IDs
    type Oid: Clone + fmt::Debug + fmt::Display;

    /// Type used for representing refs
    type Reference: Clone + fmt::Debug + fmt::Display;
}

impl InnerError for git2::Error {
    type Oid = git2::Oid;
    type Reference = String;
}

