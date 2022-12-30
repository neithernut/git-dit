// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use std::fmt;


/// Alias for wrapping git library specific [Error](std::error::Error)s
pub type Result<T, I> = std::result::Result<T, Error<I>>;


/// Extension trait for convenience functionality
pub trait ResultExt<T, I: InnerError> {
    /// Wrap a git library specific error with a specific [Kind]
    fn chain_err<F: FnOnce() -> Kind<I>>(self, kind: F) -> Result<T, I>;
}

impl<T, I: InnerError> ResultExt<T, I> for std::result::Result<T, I> {
    fn chain_err<F: FnOnce() -> Kind<I>>(self, kind: F) -> Result<T, I> {
        self.map_err(|e| Error::from(kind()).with_inner(e))
    }
}


/// Custom [Error](std::error::Error) type for this library
#[derive(Clone, Debug)]
pub struct Error<I: InnerError> {
    inner: Option<I>,
    kind: Kind<I>,
}

impl<I: InnerError> Error<I> {
    /// Set an inner error
    pub fn with_inner(self, inner: I) -> Self {
        Self {inner: Some(inner), ..self}
    }
}

impl<I: InnerError> From<Kind<I>> for Error<I> {
    fn from(kind: Kind<I>) -> Self {
        Self {inner: None, kind}
    }
}

impl<I: InnerError> From<I> for Error<I> {
    fn from(inner: I) -> Self {
        Self {inner: Some(inner), kind: Kind::Other}
    }
}

impl<I: InnerError + 'static> std::error::Error for Error<I> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.as_ref().map(|x| x as &(dyn std::error::Error + 'static))
    }
}

impl<I: InnerError> fmt::Display for Error<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
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

